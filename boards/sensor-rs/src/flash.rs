use core::cell::Cell;

use defmt::{Debug2Format, info};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{self, Config, Spi};
use embassy_stm32::time::Hertz;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::Timer;

use crate::proto::sensor_::flash_;
use crate::resources::Flash;

mod spiflash;

enum Trigger {
    Flash,
    Readout,
}

static TRIGGER: Signal<ThreadModeRawMutex, Trigger> = Signal::new();
static STATE: Mutex<ThreadModeRawMutex, Cell<flash_::State>> = Mutex::new(Cell::new(flash_::State::Idle));

#[embassy_executor::task]
pub async fn task(r: Flash) {
    let mut reset_n = Output::new(r.reset_n, Level::Low, Speed::Low);
    let mut cs_n = Output::new(r.cs_n, Level::High, Speed::Low);

    let mut spi_config = Config::default();
    spi_config.bit_order = spi::BitOrder::MsbFirst;
    spi_config.mode = spi::MODE_0;
    spi_config.frequency = Hertz(2_000_000);
    spi_config.gpio_speed = Speed::VeryHigh; // can this be reduced for power?

    let mut spi = Spi::new(r.spi, r.sck, r.mosi, r.miso, r.dma_tx, r.dma_rx, spi_config);

    let mut w_flash = spiflash::SpiFlash::init(spi, cs_n).await;

    loop {
        let id = w_flash.read_id().await;
        info!("ID is still {:?}", Debug2Format(&id));
        Timer::after_millis(100).await;
        // match TRIGGER.wait().await {
        //     Trigger::Flash => {
        //         set_state(flash_::State::Erasing);
        //     }
        //     Trigger::Readout => info!("Readout!"),
        // }
    }
}

fn set_state(state: flash_::State) {
    STATE.lock(|s| s.set(state));
}

pub fn start() {
    TRIGGER.signal(Trigger::Flash);
}

pub fn start_readout() {
    TRIGGER.signal(Trigger::Readout);
}
