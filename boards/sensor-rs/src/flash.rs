use core::cell::Cell;

use defmt::{Debug2Format, info};
use embassy_stm32::gpio::{Level, Output, OutputOpenDrain, Speed};
use embassy_stm32::spi::{self, Config, Spi};
use embassy_stm32::time::Hertz;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};

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
pub async fn task(mut r: Flash) {
    let mut spi_config = Config::default();
    spi_config.bit_order = spi::BitOrder::MsbFirst;
    spi_config.mode = spi::MODE_0;
    spi_config.frequency = Hertz(12_000_000);
    spi_config.gpio_speed = Speed::VeryHigh; // can this be reduced for power?

    loop {
        match TRIGGER.wait().await {
            Trigger::Flash => {
                let mut reset_n = Output::new(r.reset_n.reborrow(), Level::High, Speed::Low);
                let mut cs_n = OutputOpenDrain::new(r.cs_n.reborrow(), Level::High, Speed::Low);
                let mut w_flash = spiflash::SpiFlash::init(
                    Spi::new(
                        r.spi.reborrow(),
                        r.sck.reborrow(),
                        r.mosi.reborrow(),
                        r.miso.reborrow(),
                        r.dma_tx.reborrow(),
                        r.dma_rx.reborrow(),
                        spi_config,
                    ),
                    cs_n,
                )
                .await;
                set_state(flash_::State::Erasing);

                drop(w_flash); // dropping SPI calls set_as_disconnected() on all pins
            }
            Trigger::Readout => info!("Readout!"),
        }
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
