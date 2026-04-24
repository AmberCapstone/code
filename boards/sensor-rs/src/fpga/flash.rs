use core::sync::atomic::{AtomicU32, Ordering};

use defmt::{debug, error, info};
use embassy_futures::select::select;
use embassy_stm32::{
    crc::{self, Crc},
    gpio::{Level, OutputOpenDrain, Speed},
    spi::{self, Spi},
    time::Hertz,
};
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex},
    channel::Channel,
    signal::Signal,
    watch::Watch,
};
use embassy_time::{Duration, TimeoutError, WithTimeout};
use embedded_hal::digital::OutputPin;
use embedded_storage_async::nor_flash::{NorFlash, ReadNorFlash};

use crate::{
    flow::StateLock,
    fpga::flash::spiflash::SpiFlash,
    power::PowerSignal,
    proto::sensor_::fpga_::flash_::{Action, Command, Page, Segment, State, Status},
    resources::{Flash, Irqs},
};

mod layout;
mod spiflash;

use spiflash::size::PAGE as PAGE_SIZE;

const PAGE_MAX_ATTEMPTS: u32 = 5;
const ARQ_N: usize = 1;

enum Trigger {
    Flash(Segment),
    Readout(Segment),
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

static POWER_SIGNAL: PowerSignal<()> = PowerSignal::new();
static OPERATION: Signal<CriticalSectionRawMutex, Trigger> = Signal::new();
static STATE: StateLock<State> = StateLock::new(State::Off);

static RN: AtomicU32 = AtomicU32::new(0);
static PAGE_RX: Channel<ThreadModeRawMutex, Page, ARQ_N> = Channel::new();

static DONE: Signal<ThreadModeRawMutex, ()> = Signal::new();
static READOUT_RN: Signal<ThreadModeRawMutex, u32> = Signal::new();
static READOUT_PAGE: Watch<ThreadModeRawMutex, Page, 1> = Watch::new();

#[embassy_executor::task]
pub async fn task(mut r: Flash) {
    info!("Starting FLASH task");
    POWER_SIGNAL.turn_off();

    #[allow(clippy::never_loop)]
    loop {
        STATE.set(State::Off);
        POWER_SIGNAL.wait_for_on().await;

        select(run(&mut r), POWER_SIGNAL.wait_for_off()).await;
    }
}

async fn run(r: &mut Flash) {
    loop {
        STATE.set(State::Idle);
        OPERATION.reset();
        let trigger = OPERATION.wait().await;
        DONE.reset();

        // Activate SPI
        let mut _reset_n = OutputOpenDrain::new(r.reset_n.reborrow(), Level::High, Speed::Low);
        let cs_n = OutputOpenDrain::new(r.cs_n.reborrow(), Level::High, Speed::Low);
        let spi = Spi::new(
            r.spi.reborrow(),
            r.sck.reborrow(),
            r.mosi.reborrow(),
            r.miso.reborrow(),
            r.dma_tx.reborrow(),
            r.dma_rx.reborrow(),
            Irqs,
            {
                let mut c = spi::Config::default();
                c.bit_order = spi::BitOrder::MsbFirst;
                c.mode = spi::MODE_3;
                c.frequency = Hertz(12_000_000);
                c.gpio_speed = Speed::VeryHigh;
                c
            },
        );

        let Ok(mut flash) = spiflash::SpiFlash::init(spi, cs_n).await else {
            error!("Failed to configure SPI. Ignoring action");
            continue;
        };

        let mut crc = Crc::new(
            r.crc.reborrow(),
            crc::Config::new(
                crc::InputReverseConfig::Byte,
                true,
                crc::PolySize::Width32,
                0xffff_ffff,
                0x04c1_1db7,
            )
            .unwrap(),
        );

        match trigger {
            Trigger::Flash(segment) => match flash_file(&mut flash, &mut crc, segment).await {
                Ok(()) => info!("Wrote file to flash!"),
                Err(e) => error!("Failed to flash the file {:?}", e),
            },
            Trigger::Readout(segment) => match readout_file(&mut flash, &mut crc, segment).await {
                Ok(()) => info!("Readout complete!"),
                Err(e) => error!("Readout failed {:?}", e),
            },
        }
        DONE.signal(());

        // Deactivate SPI
        drop(flash); // dropping SPI calls set_as_disconnected() on all pins
    }
}

async fn flash_file<P: OutputPin>(
    flash: &mut SpiFlash<'_, P>,
    crc: &mut Crc<'_>,
    segment: Segment,
) -> Result<(), Error> {
    STATE.set(State::Erasing);

    let bounds = layout::get_bounds(segment);

    info!("Erasing Flash Segment {}", bounds);
    flash.erase(bounds.origin, bounds.end()).await?;

    STATE.set(State::Programming);
    info!("Programming Flash Segment {}", bounds);

    for pg_num in 0..bounds.num_pages() {
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

            flash.write(bounds.origin + pg_num * PAGE_SIZE, data).await?;
            break;
        }
    }

    Ok(())
}

async fn readout_file<P: OutputPin>(
    flash: &mut SpiFlash<'_, P>,
    crc: &mut Crc<'_>,
    segment: Segment,
) -> Result<(), Error> {
    STATE.set(State::Readout);

    let bounds = layout::get_bounds(segment);
    info!("Reading Flash Segment {}", bounds);

    let mut last_page_loaded: Option<u32> = None;
    READOUT_RN.reset();

    loop {
        let req_num = READOUT_RN.wait().await;
        if req_num >= bounds.num_pages() {
            break;
        }

        if last_page_loaded.is_none_or(|pg| pg != req_num) {
            let mut page = Page::default()
                .init_page_number(req_num)
                .init_data(heapless::Vec::from_array([0; PAGE_SIZE as usize]));

            debug!("Reading page {}", req_num);
            flash
                .read(
                    bounds.origin + req_num * PAGE_SIZE,
                    page.mut_data().expect("data buffer exists"),
                )
                .await?;

            page.set_crc(compute_crc(req_num, page.data().unwrap(), crc));

            READOUT_PAGE.sender().send(page);

            last_page_loaded = Some(req_num);
        }
    }

    READOUT_PAGE.sender().clear();

    Ok(())
}

fn compute_crc(page_number: u32, data: &[u8], crc: &mut Crc<'_>) -> u32 {
    crc.reset();
    crc.feed_bytes(&page_number.to_le_bytes());
    crc.feed_bytes(data);
    crc.read() ^ 0xffff_ffff
}

pub fn get_status() -> Status {
    let mut s = Status::default();

    let state = STATE.get();
    s.set_state(state);

    match state {
        State::Programming => {
            s.set_stm_page_request(RN.load(Ordering::Acquire));
        }
        State::Readout => {
            if let Some(pg) = READOUT_PAGE.try_get() {
                s.set_readout_page(pg);
            }
        }
        _ => (),
    }

    s
}

pub fn handle_command(mut command: Command) {
    if let Some(action) = command.take_action()
        && let Some(segment) = command.take_segment()
    {
        match action {
            Action::Program => OPERATION.signal(Trigger::Flash(segment)),
            Action::Readout => OPERATION.signal(Trigger::Readout(segment)),
            _ => (),
        }
    }
    if let Some(page) = command.take_page() {
        let _ = PAGE_RX.try_send(page);
    }

    if let Some(host_pg_req) = command.take_host_page_request() {
        READOUT_RN.signal(host_pg_req);
    }
}

pub(super) fn turn_on() {
    POWER_SIGNAL.turn_on(());
}

pub(super) fn turn_off() {
    POWER_SIGNAL.turn_off();
}

pub(super) fn is_off() -> bool {
    STATE.is(State::Off)
}
