#![allow(unused)]

use defmt::{Debug2Format, info};
use embassy_stm32::{
    gpio::{Output, OutputOpenDrain},
    mode::Async,
    pac::Interrupt::PVD_PVM,
    spi::{Spi, mode::Master},
};
use embassy_time::Timer;

// ==== Memory Layout ====
pub const PAGES_PER_SUBSECTOR: u32 = 16;
pub const SUBSECTORS_PER_SECTOR: u32 = 16;
pub const NUM_SECTORS: u32 = 2;

// All sizes in bytes
pub const PAGE_SIZE: u32 = 0x100;
pub const SUBSECTOR_SIZE: u32 = PAGE_SIZE * PAGES_PER_SUBSECTOR;
pub const SECTOR_SIZE: u32 = SUBSECTOR_SIZE * SUBSECTORS_PER_SECTOR;
pub const TOTAL_SIZE: u32 = SECTOR_SIZE * NUM_SECTORS;

struct StatusRegister(u8);
impl StatusRegister {
    fn busy(self) -> bool {
        (self.0 & 0b0000_0001) != 0
    }

    fn write_enable_latch(self) -> bool {
        (self.0 & 0b0000_0010) != 0
    }
}

#[derive(Debug)]
pub struct Id {
    manufacturer_id: u8,
    memory_type: u8,
    capacity: u8,
}

impl From<&[u8; 3]> for Id {
    fn from(value: &[u8; 3]) -> Self {
        Self {
            manufacturer_id: value[0],
            memory_type: value[1],
            capacity: value[2],
        }
    }
}

#[repr(u8)]
enum Command {
    WriteEnable = 0x06,
    WriteDisable = 0x04,
    ReadIdentification = 0x9f,
    ReadStatusRegister = 0x05,
    WriteStatusRegister = 0x01,
    WriteToLockRegister = 0xe5,
    ReadLockRegister = 0xe8,
    ReadDataBytes = 0x03,
    ReadDataBytesHighSpeed = 0x0b,
    PageWrite = 0x0a,
    PageProgram = 0x02,
    PageErase = 0xdb,
    SubsectorErase = 0x20,
    SectorErase = 0xd8,
    BulkErase = 0xc7,
    DeepPowerDown = 0xb9,
    ReleaseDeepPowerDown = 0xab,
}

const fn header(command: Command, addr: u32) -> [u8; 4] {
    // 8 bit command, 24 bit address
    let mut h = addr.to_be_bytes();
    h[0] = command as u8;
    h
}

pub struct SpiFlash<'a> {
    spi: Spi<'a, Async, Master>,
    cs_n: OutputOpenDrain<'a>,
}

impl<'a> SpiFlash<'a> {
    pub async fn init(spi: Spi<'a, Async, Master>, cs_n: OutputOpenDrain<'a>) -> Self {
        let mut s = Self { spi, cs_n };

        let id = s.read_id().await;
        info!("Connected to SPI Flash {:?}", Debug2Format(&id));

        s
    }

    async fn read_status_register(&mut self) -> StatusRegister {
        let mut rx = [0; 2];
        let tx = [Command::ReadStatusRegister as u8, 0];

        self.cs_n.set_low();
        self.spi.transfer(&mut rx, &tx).await.unwrap();
        self.cs_n.set_high();

        StatusRegister(rx[1])
    }

    pub async fn is_busy(&mut self) -> bool {
        self.read_status_register().await.busy()
    }

    pub async fn wait_for_idle(&mut self) {
        while self.is_busy().await {
            Timer::after_millis(1).await;
        }
    }

    async fn send_command(&mut self, cmd: Command) {
        self.cs_n.set_low();
        self.spi.write(&[cmd as u8]).await.unwrap();
        self.cs_n.set_high();
    }

    pub async fn enable_writing(&mut self) {
        self.send_command(Command::WriteEnable).await
    }

    pub async fn disable_writing(&mut self) {
        self.send_command(Command::WriteDisable).await
    }

    pub async fn power_down(&mut self) {
        self.send_command(Command::DeepPowerDown).await
    }

    pub async fn wake_up(&mut self) {
        self.send_command(Command::ReleaseDeepPowerDown).await
    }

    pub async fn page_program(&mut self, addr: u32, data: &[u8]) {
        self.cs_n.set_low();
        self.spi.write(&header(Command::PageProgram, addr)).await.unwrap();
        self.spi.write(data).await.unwrap();
        self.cs_n.set_high();
    }

    pub async fn page_write(&mut self, addr: u32, data: &[u8]) {
        self.cs_n.set_low();
        self.spi.write(&header(Command::PageWrite, addr)).await.unwrap();
        self.spi.write(data).await.unwrap();
        self.cs_n.set_high();
    }

    pub async fn read_data(&mut self, addr: u32, len: usize, out: &mut [u8]) {
        self.cs_n.set_low();
        self.spi.write(&header(Command::ReadDataBytes, addr)).await.unwrap();
        self.spi.read(out).await.unwrap();
        self.cs_n.set_high();
    }

    async fn send_header(&mut self, command: Command, addr: u32) {
        self.cs_n.set_low();
        self.spi.write(&header(command, addr)).await.unwrap();
        self.cs_n.set_high();
    }

    pub async fn page_erase(&mut self, addr: u32) {
        self.send_header(Command::PageErase, addr).await;
    }

    pub async fn subsector_erase(&mut self, addr: u32) {
        self.send_header(Command::SubsectorErase, addr).await;
    }

    pub async fn sector_erase(&mut self, addr: u32) {
        self.send_header(Command::SectorErase, addr).await;
    }

    pub async fn bulk_erase(&mut self, addr: u32) {
        self.send_header(Command::BulkErase, addr).await;
    }

    pub async fn read_id(&mut self) -> Id {
        let mut txrx: [u8; 4] = [Command::ReadIdentification as u8, 0, 0, 0];

        self.cs_n.set_low();
        self.spi.transfer_in_place(&mut txrx).await.unwrap();
        self.cs_n.set_high();

        info!("{}", txrx);

        Id::from(&txrx[1..4].try_into().unwrap())
    }
}
