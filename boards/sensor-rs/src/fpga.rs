use core::future::pending;

use defmt::{Debug2Format, debug, info};
use embassy_futures::select::select;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Level, Output, OutputOpenDrain, Pull, Speed},
    spi::{self, Spi},
    time::Hertz,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel, signal::Signal};
use embassy_time::{Duration, Timer};
use heapless::Vec;

use crate::{
    camera, comms,
    flow::{StateLock, poll},
    power::PowerSignal,
    proto::{
        backscatter_,
        sensor_::fpga_::{Action, CaptureSource, Centroid, Command, DataRequest, State, Status, Vessels, image_},
    },
    resources::{Fpga, FpgaPower, Irqs},
    serial,
};

pub mod flash;
mod spi_cmd;

enum RunMode {
    Capture(CaptureSource, DataRequest),
    SpiFlash,
}

const NUM_LINES: u32 = 240; // for QVGA
const LINE_LEN: u32 = 320;
const BYTE_PER_ADDR: u32 = 2; // FPGA is 16-bit addressed
const SPI_PROTO_MAX_BYTES: usize = 64; // match fpga.toml

static STATE: StateLock<State> = StateLock::new(State::Off);
static POWER_SIGNAL: PowerSignal<RunMode> = PowerSignal::new();
static LINES_TO_SEND: Channel<ThreadModeRawMutex, image_::Line, 2> = Channel::new();
static VESSELS_TO_SEND: Signal<ThreadModeRawMutex, Vessels> = Signal::new();
static SPI_TX_TO_SEND: Channel<ThreadModeRawMutex, Vec<u8, SPI_PROTO_MAX_BYTES>, 8> = Channel::new();
static SPI_RX_TO_SEND: Channel<ThreadModeRawMutex, Vec<u8, SPI_PROTO_MAX_BYTES>, 8> = Channel::new();

#[embassy_executor::task]
pub async fn task(r_power: FpgaPower, mut r_fpga: Fpga) {
    info!("Starting FPGA task");
    let mut power_en = Output::new(r_power.en, Level::Low, Speed::Low);

    POWER_SIGNAL.turn_off();

    loop {
        flash::turn_off();
        camera::turn_off();

        poll::until(flash::is_off, Duration::from_millis(50)).await;
        poll::until(camera::is_off, Duration::from_millis(50)).await;

        power_en.set_low();
        STATE.set(State::Off);

        let mode = POWER_SIGNAL.wait_for_on().await;

        power_en.set_high();

        select(POWER_SIGNAL.wait_for_off(), async {
            match mode {
                RunMode::Capture(src, data_request) => run_capture(&mut r_fpga, src, data_request).await,
                RunMode::SpiFlash => run_spiflash(&mut r_fpga).await,
            }
        })
        .await;
    }
}

pub async fn run_spiflash(r: &mut Fpga) {
    STATE.set(State::Booting);
    let _reset_n = OutputOpenDrain::new(r.creset_n.reborrow(), Level::Low, Speed::Low);

    STATE.set(State::SpiFlash);

    flash::turn_on();

    pending::<()>().await; // wait for a command to turn it off
}

pub async fn run_capture(r: &mut Fpga, src: CaptureSource, data_request: DataRequest) {
    STATE.set(State::Booting);

    let _creset_n = OutputOpenDrain::new(r.creset_n.reborrow(), Level::High, Speed::Low);
    let mut cdone = ExtiInput::new(r.cdone.reborrow(), r.cdone_exti.reborrow(), Pull::None, Irqs);

    cdone.wait_for_high().await;

    STATE.set(State::Running);

    let mut start_capture = OutputOpenDrain::new(r.gpio1.reborrow(), Level::Low, Speed::Low);
    let mut drdy = ExtiInput::new(r.drdy.reborrow(), r.drdy_exti.reborrow(), Pull::Up, Irqs);
    let mut cs_n = OutputOpenDrain::new(r.cs_n.reborrow(), Level::High, Speed::Low);

    // Reset FPGA internals
    let mut pwrdn_n = OutputOpenDrain::new(r.pwrdn_n.reborrow(), Level::Low, Speed::Low);
    Timer::after_micros(100).await;
    pwrdn_n.set_high();
    Timer::after_micros(100).await;

    let mut spi = Spi::new(
        r.spi.reborrow(),
        r.sck.reborrow(),
        r.mosi.reborrow(),
        r.miso.reborrow(),
        r.dma_tx.reborrow(),
        r.dma_rx.reborrow(),
        Irqs,
        {
            let mut c = spi::Config::default();
            c.bit_order = spi::BitOrder::LsbFirst;
            c.mode = spi::MODE_0;
            c.frequency = Hertz(1_000_000);
            c.gpio_speed = Speed::VeryHigh;
            c
        },
    );

    let cmd = match src {
        CaptureSource::Camera => spi_cmd::Command::RealCapture,
        CaptureSource::FakeVga => spi_cmd::Command::FakeCaptureVga,
        CaptureSource::FakeSram => spi_cmd::Command::FakeCaptureWrite,
        _ => panic!("src={:?} should not have passed through handle_process", src),
    } as u8;

    info!("Setting capture source over SPI (cmd={:x})", cmd);
    cs_n.set_low();
    let cmd = [cmd];
    let _ = SPI_TX_TO_SEND.try_send(cmd.into());
    let _ = spi.write(&cmd).await;
    cs_n.set_high();

    if src == CaptureSource::Camera {
        // reduce instantaneous current by staggering FPGA and Camera boots
        Timer::after_millis(200).await;

        info!("Turning on the camera");
        camera::turn_on();
        poll::until(camera::is_running, Duration::from_millis(50)).await;
    }
    Timer::after_millis(100).await;

    info!("Setting capture high");
    start_capture.set_high();
    Timer::after_micros(100).await;
    info!("Setting capture low");
    start_capture.set_low();

    info!("Waiting for drdy");
    drdy.wait_for_high().await;
    STATE.set(State::DataReady);
    info!("Seen DRDY");

    if src == CaptureSource::Camera {
        info!("Turning on the camera");
        camera::turn_off();
    }

    match data_request {
        DataRequest::Image => {
            info!("Reading Image");
            for line_no in 0..NUM_LINES {
                let line = read_line(&mut spi, &mut cs_n, line_no).await;

                if serial::is_running() {
                    LINES_TO_SEND.send(line).await; // blocks until channel has capacity
                }
            }
        }
        DataRequest::Vessels => {
            let vessels = read_vessels(spi, cs_n).await;

            let backscatter_msg = backscatter_::Status::default()
                .init_x(vessels.centroids.first().map_or(0, |c| c.x))
                .init_y(vessels.centroids.first().map_or(0, |c| c.y));
            comms::send(backscatter_msg);

            if serial::is_running() {
                VESSELS_TO_SEND.signal(vessels);
            }
            Timer::after_millis(100).await; // hacky way to hold data ready state
        }
        _ => (),
    }
}

async fn read_line(
    spi: &mut Spi<'_, embassy_stm32::mode::Async, spi::mode::Master>,
    cs_n: &mut OutputOpenDrain<'_>,
    line_no: u32,
) -> image_::Line {
    info!("Line no {}", line_no);
    let mut new_line = image_::Line::default()
        .init_number(line_no)
        .init_data([0u8; LINE_LEN as usize].into());

    let address = line_no * LINE_LEN / BYTE_PER_ADDR;
    let [_, _, addr_hi, addr_lo] = address.to_be_bytes();

    cs_n.set_low();
    let to_tx = [spi_cmd::Command::ReadData as u8, addr_hi, addr_lo];
    let _ = SPI_TX_TO_SEND.try_send(to_tx.into());
    let _res = spi.write(&to_tx).await;
    let _res = spi.read(new_line.mut_data()).await;

    // Only send the first 64 bytes of image data
    let _ = SPI_RX_TO_SEND.try_send(Vec::from_slice(&new_line.data()[0..SPI_PROTO_MAX_BYTES]).unwrap());
    cs_n.set_high();

    new_line
}

async fn read_vessels(
    mut spi: Spi<'_, embassy_stm32::mode::Async, spi::mode::Master>,
    mut cs_n: OutputOpenDrain<'_>,
) -> Vessels {
    info!("Reading Vessels");
    cs_n.set_low();

    let to_tx = [spi_cmd::Command::GetVessels as u8];
    let _ = SPI_TX_TO_SEND.try_send(to_tx.into());
    let _ = spi.write(&to_tx).await;

    let mut buffer = [0u8; 5];
    let _ = spi.read(&mut buffer).await;
    let _ = SPI_RX_TO_SEND.try_send(buffer.into());
    cs_n.set_high();

    let count = u32::from(buffer[0]);
    let x = u32::from(u16::from_le_bytes(buffer[1..3].try_into().unwrap()));
    let y = u32::from(u16::from_le_bytes(buffer[3..5].try_into().unwrap()));
    // FPGA currently only returns 1 (x,y) pair

    Vessels {
        count,
        centroids: [Centroid { x, y }].into(),
    }
}

pub fn capture(data: DataRequest) {
    if STATE.is(State::Off) {
        POWER_SIGNAL.turn_on(RunMode::Capture(CaptureSource::Camera, data));
    }
}

pub fn turn_off() {
    POWER_SIGNAL.turn_off();
}

pub fn is_off() -> bool {
    STATE.is(State::Off)
}

pub fn is_done() -> bool {
    STATE.is(State::DataReady)
}

pub fn handle_command(mut command: Command) {
    if let Some(action) = command.take_action() {
        debug!("FPGA received Action={:?}", Debug2Format(&action));
        match action {
            Action::Capture => {
                if STATE.is(State::Off)
                    && let Some(src) = command.take_capture_source()
                    && matches!(
                        src,
                        CaptureSource::Camera | CaptureSource::FakeVga | CaptureSource::FakeSram
                    )
                {
                    let data_request = command.take_data_request().unwrap_or(DataRequest::Vessels);
                    POWER_SIGNAL.turn_on(RunMode::Capture(src, data_request));
                }
            }
            Action::SpiFlash => {
                if STATE.is(State::Off) {
                    POWER_SIGNAL.turn_on(RunMode::SpiFlash);
                }
            }
            Action::Off => {
                if !STATE.is(State::Off) {
                    POWER_SIGNAL.turn_off();
                }
            }
            _ => (),
        }
    }

    if let Some(flash_cmd) = command.take_flash()
        && STATE.is(State::SpiFlash)
    {
        flash::handle_command(flash_cmd);
    }
}

pub fn get_status() -> Status {
    let mut s = Status::default()
        .init_state(STATE.get())
        .init_flash(flash::get_status());

    if let Ok(line) = LINES_TO_SEND.try_receive() {
        s.set_line(line);
    }

    if let Ok(tx) = SPI_TX_TO_SEND.try_receive() {
        s.set_spi_tx(tx);
    }

    if let Ok(rx) = SPI_RX_TO_SEND.try_receive() {
        s.set_spi_rx(rx);
    }

    if let Some(vessels) = VESSELS_TO_SEND.try_take() {
        s.set_vessels(vessels);
    }

    s
}
