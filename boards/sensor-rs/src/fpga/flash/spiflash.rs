#![allow(unused)]

use core::result;

use defmt::{Debug2Format, debug, error, info};
use embassy_stm32::{
    gpio::{Output, OutputOpenDrain},
    mode::Async,
    pac::Interrupt::PVD_PVM,
    spi::{self, Spi, mode::Master},
};
use embassy_time::Timer;
use embedded_hal::digital::OutputPin;

mod command;
mod id;

use crate::fpga::flash::spiflash::{command::Command, id::Id};

pub mod size {
    // All sizes in bytes
    pub const PAGE: u32 = 0x100;
    pub const SUBSECTOR: u32 = 0x1000;
    pub const SECTOR: u32 = 0x1_0000;

    #[cfg(feature = "nucleo")] // st M25PE10
    pub const CHIP: u32 = 0x2_0000;

    #[cfg(not(feature = "nucleo"))] // winbond W25Q80
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
            error!("Unexpected ID");
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

    pub async fn page_write(&mut self, addr: u32, data: &[u8]) -> Result<(), Error> {
        check_bounds(addr, data.len())?;

        self.enable_writing().await?;

        {
            let cs = CsGuard::new(&mut self.cs_n);
            self.spi.write(&header(Command::PageWrite, addr)).await?;
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

    pub async fn page_erase(&mut self, addr: u32) -> Result<(), Error> {
        self.enable_writing().await?;
        self.send_header(Command::PageErase, addr).await?;
        self.wait_for_idle().await
    }

    pub async fn subsector_erase(&mut self, addr: u32) -> Result<(), Error> {
        self.enable_writing().await?;
        self.send_header(Command::SubsectorErase, addr).await?;
        self.wait_for_idle().await
    }

    pub async fn sector_erase(&mut self, addr: u32) -> Result<(), Error> {
        self.enable_writing().await?;
        self.send_header(Command::SectorErase, addr).await?;
        self.wait_for_idle().await
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
