#![allow(unused)]

use core::{ptr::addr_eq, result};

use defmt::{Debug2Format, debug, error, info};
use embassy_stm32::{
    gpio::{Output, OutputOpenDrain},
    mode::Async,
    pac::Interrupt::PVD_PVM,
    spi::{self, Spi, mode::Master},
};
use embassy_time::Timer;
use embedded_hal::digital::OutputPin;
use embedded_storage_async::nor_flash::{self, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash};

mod command;
mod id;

use crate::fpga::flash::spiflash::{command::Command, id::Id};

pub mod size {
    // All sizes in bytes
    pub const PAGE: u32 = 0x100;
    pub const SUBSECTOR: u32 = 0x1000;
    pub const SECTOR: u32 = 0x1_0000;
    pub const CHIP: u32 = 0x10_0000;
}

struct StatusRegister(u8);
impl StatusRegister {
    fn busy(&self) -> bool {
        (self.0 & 0b0000_0001) != 0
    }

    fn write_enable_latch(&self) -> bool {
        (self.0 & 0b0000_0010) != 0
    }
}

const fn header(command: Command, addr: u32) -> [u8; 4] {
    // 8 bit command, 24 bit address
    let mut h = addr.to_be_bytes();
    h[0] = command as u8;
    h
}

#[derive(Debug, defmt::Format)]
pub enum Error {
    UnxpectedId,
    PeripheralFailure(spi::Error),
    OutOfBounds,
    NotAligned,
}

impl From<spi::Error> for Error {
    fn from(value: spi::Error) -> Self {
        Self::PeripheralFailure(value)
    }
}

fn check_bounds(addr: u32, len: usize) -> Result<(), Error> {
    #[allow(clippy::cast_possible_truncation, reason = "All lens are small")]
    if addr + (len as u32) > size::CHIP {
        Err(Error::OutOfBounds)
    } else {
        Ok(())
    }
}

pub struct SpiFlash<'a, P: OutputPin> {
    spi: Spi<'a, Async, Master>,
    cs_n: P,
}

/// Context manager to hold CS without forgetting to set it high
#[must_use = "hold until SPI is complete"]
struct CsGuard<'a, P: OutputPin>(&'a mut P);

impl<'a, P: OutputPin> CsGuard<'a, P> {
    fn new(pin: &'a mut P) -> Self {
        pin.set_low();
        Self(pin)
    }
}

impl<P: OutputPin> Drop for CsGuard<'_, P> {
    fn drop(&mut self) {
        self.0.set_high();
    }
}

impl<'a, P: OutputPin> SpiFlash<'a, P> {
    pub async fn init(spi: Spi<'a, Async, Master>, cs_n: P) -> Result<Self, Error> {
        let mut s = Self { spi, cs_n };

        s.send_command(Command::ReleaseDeepPowerDown).await?;

        let id = s.read_id().await?;
        debug!("SPI Flash ID {:?}", Debug2Format(&id));

        if id == Id::expected() {
            info!("Connected to SPI Flash");
            Ok(s)
        } else {
            error!("Unexpected ID {:?}", Debug2Format(&id));
            Err(Error::UnxpectedId)
        }
    }

    async fn read_status_register(&mut self) -> Result<StatusRegister, Error> {
        let mut txrx = [Command::ReadStatusRegister as u8, 0u8];

        let cs = CsGuard::new(&mut self.cs_n);
        self.spi.transfer_in_place(&mut txrx).await?;

        Ok(StatusRegister(txrx[1]))
    }

    pub async fn is_busy(&mut self) -> Result<bool, Error> {
        self.read_status_register().await.map(|s| s.busy())
    }

    async fn wait_for_idle(&mut self) -> Result<(), Error> {
        while self.is_busy().await? {
            Timer::after_millis(1).await;
        }

        Ok(())
    }

    async fn send_command(&mut self, cmd: Command) -> Result<(), Error> {
        let cs = CsGuard::new(&mut self.cs_n);
        self.spi.write(&[cmd as u8]).await?;
        Ok(())
    }

    async fn enable_writing(&mut self) -> Result<(), Error> {
        self.send_command(Command::WriteEnable).await
    }

    async fn disable_writing(&mut self) -> Result<(), Error> {
        self.send_command(Command::WriteDisable).await
    }

    pub async fn power_down(&mut self) -> Result<(), Error> {
        self.send_command(Command::DeepPowerDown).await
    }

    pub async fn wake_up(&mut self) -> Result<(), Error> {
        self.send_command(Command::ReleaseDeepPowerDown).await
    }

    pub async fn page_program(&mut self, addr: u32, data: &[u8]) -> Result<(), Error> {
        check_bounds(addr, data.len())?;

        self.enable_writing().await?;

        {
            let cs = CsGuard::new(&mut self.cs_n);
            self.spi.write(&header(Command::PageProgram, addr)).await?;
            self.spi.write(data).await?;
        }

        self.wait_for_idle().await
    }

    pub async fn read_data(&mut self, addr: u32, out: &mut [u8]) -> Result<(), Error> {
        check_bounds(addr, out.len())?;

        let cs = CsGuard::new(&mut self.cs_n);
        self.spi.write(&header(Command::ReadDataBytes, addr)).await?;
        self.spi.read(out).await?;

        Ok(())
    }

    async fn send_header(&mut self, command: Command, addr: u32) -> Result<(), Error> {
        let cs = CsGuard::new(&mut self.cs_n);
        self.spi.write(&header(command, addr)).await?;

        Ok(())
    }

    pub async fn subsector_erase(&mut self, addr: u32) -> Result<(), Error> {
        if addr.is_multiple_of(size::SUBSECTOR) {
            self.enable_writing().await?;
            self.send_header(Command::SubsectorErase, addr).await?;
            self.wait_for_idle().await
        } else {
            Err(Error::NotAligned)
        }
    }

    pub async fn sector_erase(&mut self, addr: u32) -> Result<(), Error> {
        if addr.is_multiple_of(size::SECTOR) {
            self.enable_writing().await?;
            self.send_header(Command::SectorErase, addr).await?;
            self.wait_for_idle().await
        } else {
            Err(Error::NotAligned)
        }
    }

    pub async fn chip_erase(&mut self) -> Result<(), Error> {
        self.enable_writing().await?;
        self.send_command(Command::ChipErase).await?;
        self.wait_for_idle().await
    }

    pub async fn read_id(&mut self) -> Result<Id, Error> {
        let mut txrx: [u8; 4] = [Command::ReadIdentification as u8, 0, 0, 0];

        let cs = CsGuard::new(&mut self.cs_n);
        self.spi.transfer_in_place(&mut txrx).await?;

        Ok(Id::from(&[txrx[1], txrx[2], txrx[3]]))
    }
}

impl<P: OutputPin> nor_flash::ErrorType for SpiFlash<'_, P> {
    type Error = Error;
}

impl NorFlashError for Error {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            Error::UnxpectedId => todo!(),
            Error::PeripheralFailure(error) => NorFlashErrorKind::Other,
            Error::OutOfBounds => NorFlashErrorKind::OutOfBounds,
            Error::NotAligned => NorFlashErrorKind::NotAligned,
        }
    }
}

impl<P: OutputPin> ReadNorFlash for SpiFlash<'_, P> {
    const READ_SIZE: usize = 1;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read_data(offset, bytes).await
    }

    fn capacity(&self) -> usize {
        size::CHIP as usize
    }
}

impl<P: OutputPin> NorFlash for SpiFlash<'_, P> {
    const WRITE_SIZE: usize = size::PAGE as usize;
    const ERASE_SIZE: usize = size::SUBSECTOR as usize;

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        // This could be more efficient by erasing larger sectors when `to>>from`
        for pg in (from..to).step_by(Self::ERASE_SIZE) {
            self.subsector_erase(pg).await?;
        }
        Ok(())
    }

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.page_program(offset, bytes).await
    }
}
