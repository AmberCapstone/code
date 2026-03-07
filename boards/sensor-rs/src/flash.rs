use core::{
    cell::Cell,
    sync::atomic::{AtomicU32, Ordering},
};

use defmt::{debug, error, info};
use embassy_stm32::{
    crc::{self, Crc},
    gpio::{Level, Output, OutputOpenDrain, Speed},
    spi::{self, Spi},
    time::Hertz,
};
use embassy_sync::{
    blocking_mutex::{Mutex, raw::ThreadModeRawMutex},
    channel::Channel,
    signal::Signal,
    watch::Watch,
};
use embassy_time::{Duration, TimeoutError, WithTimeout};
use embedded_hal::digital::OutputPin;

use crate::{
    flash::spiflash::SpiFlash,
    proto::sensor_::flash_::{self, State},
    resources::Flash,
};

mod spiflash;

use spiflash::size::PAGE as PAGE_SIZE;

const PAGE_MAX_ATTEMPTS: u32 = 5;
const ARQ_N: usize = 1;
const PAGE_PER_FILE: u32 = 512;

enum Trigger {
    Flash,
    Readout,
}

#[derive(Debug, defmt::Format)]
enum Error {
    Spi(spiflash::Error),
    PageTimeout,
    PageRetriesExceeded,
}

impl From<spiflash::Error> for Error {
    fn from(value: spiflash::Error) -> Self {
        Self::Spi(value)
    }
}

impl From<TimeoutError> for Error {
    fn from(_: TimeoutError) -> Self {
        Self::PageTimeout
    }
}

static TRIGGER: Signal<ThreadModeRawMutex, Trigger> = Signal::new();
static STATE: Mutex<ThreadModeRawMutex, Cell<flash_::State>> = Mutex::new(Cell::new(flash_::State::Idle));

static RN: AtomicU32 = AtomicU32::new(0);
static PAGE_RX: Channel<ThreadModeRawMutex, flash_::Page, ARQ_N> = Channel::new();

static READOUT_RN: Signal<ThreadModeRawMutex, u32> = Signal::new();
static READOUT_PAGE: Watch<ThreadModeRawMutex, flash_::Page, 1> = Watch::new();

#[embassy_executor::task]
pub async fn task(mut r: Flash) {
    let mut spi_config = spi::Config::default();
    spi_config.bit_order = spi::BitOrder::MsbFirst;
    spi_config.mode = spi::MODE_0;
    spi_config.frequency = Hertz(12_000_000);
    spi_config.gpio_speed = Speed::VeryHigh;

    let crc_config = crc::Config::new(
        crc::InputReverseConfig::Byte,
        true,
        crc::PolySize::Width32,
        0xffff_ffff,
        0x04c1_1db7,
    )
    .unwrap();
    let mut crc = Crc::new(r.crc, crc_config);

    loop {
        let trigger = TRIGGER.wait().await;

        // Activate SPI
        let mut _reset_n = Output::new(r.reset_n.reborrow(), Level::High, Speed::Low);
        let cs_n = OutputOpenDrain::new(r.cs_n.reborrow(), Level::High, Speed::Low);
        let flash = spiflash::SpiFlash::init(
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

        let Ok(mut flash) = flash else {
            error!("Failed to configure SPI. Ignoring action");
            continue;
        };

        match trigger {
            Trigger::Flash => match flash_file(&mut flash, &mut crc).await {
                Ok(()) => info!("Wrote file to flash!"),
                Err(e) => error!("Failed to flash the file {:?}", e),
            },
            Trigger::Readout => match readout_file(&mut flash, &mut crc).await {
                Ok(()) => info!("Readout complete!"),
                Err(e) => error!("Readout failed {:?}", e),
            },
        }

        // Deactivate SPI
        drop(flash); // dropping SPI calls set_as_disconnected() on all pins
    }
}

async fn flash_file<P: OutputPin>(flash: &mut SpiFlash<'_, P>, crc: &mut Crc<'_>) -> Result<(), Error> {
    set_state(State::Erasing);
    info!("Erasing Flash");
    flash.chip_erase().await?;

    set_state(State::Programming);
    info!("Programming Flash");

    for pg_num in 0..PAGE_PER_FILE {
        RN.store(pg_num, Ordering::Release);

        let mut attempt = 0;
        loop {
            attempt += 1;
            if attempt > PAGE_MAX_ATTEMPTS {
                return Err(Error::PageRetriesExceeded);
            }

            let new_page = PAGE_RX
                .receive()
                .with_timeout(Duration::from_millis(if pg_num == 0 { 5000 } else { 500 }))
                .await?;

            // Validate the page
            let Some(num) = new_page.page_number() else { continue };
            if *num != pg_num {
                continue;
            }
            let Some(data) = new_page.data() else { continue };
            let Some(exp_crc) = new_page.crc() else { continue };

            if compute_crc(*num, data, crc) != *exp_crc {
                continue;
            }

            debug!("Programming page {}", pg_num);
            flash.page_program(pg_num * PAGE_SIZE, data).await?;
            break;
        }
    }

    set_state(State::Done);
    Ok(())
}

async fn readout_file<P: OutputPin>(flash: &mut SpiFlash<'_, P>, crc: &mut Crc<'_>) -> Result<(), Error> {
    set_state(State::Readout);
    info!("Starting readout");

    let mut last_page_loaded: Option<u32> = None;

    loop {
        let req_num = READOUT_RN.wait().await;
        if req_num >= PAGE_PER_FILE {
            break;
        }

        if last_page_loaded.is_none_or(|pg| pg != req_num) {
            let mut page = flash_::Page::default()
                .init_page_number(req_num)
                .init_data(heapless::Vec::from_array([0; PAGE_SIZE as usize]));

            debug!("Reading page {}", req_num);
            flash
                .read_data(req_num * PAGE_SIZE, page.mut_data().expect("Data exists"))
                .await?;

            page.set_crc(compute_crc(req_num, page.data().unwrap(), crc));

            READOUT_PAGE.sender().send(page);

            last_page_loaded = Some(req_num);
        }
    }

    READOUT_PAGE.sender().clear();

    set_state(State::Done);
    Ok(())
}

fn compute_crc(page_number: u32, data: &[u8], crc: &mut Crc<'_>) -> u32 {
    crc.reset();
    let _ = crc.feed_bytes(&page_number.to_le_bytes());
    crc.feed_bytes(data) ^ 0xffff_ffff
}

pub fn get_status() -> flash_::Status {
    let mut s = flash_::Status::default();

    let state = get_state();
    s.set_state(state);

    match state {
        flash_::State::Programming => {
            s.set_stm_page_request(RN.load(Ordering::Acquire));
        }
        flash_::State::Readout => {
            if let Some(pg) = READOUT_PAGE.try_get() {
                s.set_readout_page(pg);
            }
        }
        _ => (),
    }

    s
}

pub fn accept_page(page: flash_::Page) {
    let _ = PAGE_RX.try_send(page);
}

pub fn set_readout_req_number(host_pg_req: u32) {
    READOUT_RN.signal(host_pg_req);
}

fn get_state() -> flash_::State {
    STATE.lock(Cell::get)
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

pub fn is_done() -> bool {
    get_state() == flash_::State::Done
}
