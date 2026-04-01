use core::{cell::Cell, future::pending};

use defmt::{Debug2Format, debug, info};
use embassy_futures::select::select;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Level, Output, OutputOpenDrain, Pull, Speed},
    spi::{self, Spi},
    time::Hertz,
};
use embassy_sync::{
    blocking_mutex::{Mutex, raw::ThreadModeRawMutex},
    channel::Channel,
};
use embassy_time::{Duration, Timer};

use crate::{
    camera,
    flow::poll,
    power::PowerSignal,
    proto::sensor_::{
        camera_,
        fpga_::{Action, CaptureSource, Command, State, Status, image_},
    },
    resources::{Fpga, FpgaPower, Irqs},
};

pub mod flash;
mod spi_cmd;

enum RunMode {
    Capture(CaptureSource),
    SpiFlash,
    RunConstant,
}

const NUM_LINES: u32 = 240; // for QVGA
const LINE_LEN: u32 = 320;
const BYTE_PER_ADDR: u32 = 2; // FPGA is 16-bit addressed

static STATE: Mutex<ThreadModeRawMutex, Cell<State>> = Mutex::new(Cell::new(State::Off));
static POWER_SIGNAL: PowerSignal<RunMode> = PowerSignal::new();
static LINES_TO_SEND: Channel<ThreadModeRawMutex, image_::Line, 2> = Channel::new();

#[embassy_executor::task]
pub async fn task(r_power: FpgaPower, mut r_fpga: Fpga) {
    let mut power_en = Output::new(r_power.en, Level::Low, Speed::Low);

    POWER_SIGNAL.turn_off();

    loop {
        flash::turn_off();
        camera::turn_off();

        poll::until(flash::is_off, Duration::from_millis(50)).await;
        poll::until(camera::is_off, Duration::from_millis(50)).await;

        power_en.set_low();
        set_state(State::Off);

        let mode = POWER_SIGNAL.wait_for_on().await;

        power_en.set_high();

        select(POWER_SIGNAL.wait_for_off(), async {
            match mode {
                RunMode::Capture(src) => run_capture(&mut r_fpga, src).await,
                RunMode::SpiFlash => run_spiflash(&mut r_fpga).await,
                RunMode::RunConstant => run_constant(&mut r_fpga).await,
            }
        })
        .await;
    }
}

pub async fn run_constant(r: &mut Fpga) {
    set_state(State::Booting);
    let mut _reset_n = OutputOpenDrain::new(r.creset_n.reborrow(), Level::High, Speed::Low);
    let mut cdone = ExtiInput::new(r.cdone.reborrow(), r.cdone_exti.reborrow(), Pull::None, Irqs);

    info!("Waiting for CDONE");
    #[cfg(not(feature = "nucleo"))]
    cdone.wait_for_high().await;
    info!("CDONE complete");

    set_state(State::Running);

    loop {
        info!("FPGA RunConstant");
        Timer::after_millis(2000).await;
    }
}

pub async fn run_spiflash(r: &mut Fpga) {
    set_state(State::Booting);
    let _reset_n = OutputOpenDrain::new(r.creset_n.reborrow(), Level::Low, Speed::Low);

    set_state(State::SpiFlash);

    flash::turn_on();

    pending::<()>().await; // wait for a command to turn it off
}

pub async fn run_capture(r: &mut Fpga, src: CaptureSource) {
    set_state(State::Booting);
    let _creset_n = OutputOpenDrain::new(r.creset_n.reborrow(), Level::High, Speed::Low);
    let mut cdone = ExtiInput::new(r.cdone.reborrow(), r.cdone_exti.reborrow(), Pull::None, Irqs);

    cdone.wait_for_high().await;
    set_state(State::Running);

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
    let _ = spi.write(&[cmd]).await;
    cs_n.set_high();

    if src == CaptureSource::Camera {
        // reduce instantaneous current by staggering FPGA and Camera boots
        Timer::after_millis(200).await;

        info!("Turning on the camera");
        camera::turn_on();
        poll::until(
            || camera::get_state() == camera_::State::Running,
            Duration::from_millis(50),
        )
        .await;
    }
    Timer::after_millis(100).await;

    info!("Setting capture high");
    start_capture.set_high();
    Timer::after_micros(100).await;
    info!("Setting capture low");
    start_capture.set_low();

    info!("Waiting for drdy");
    drdy.wait_for_high().await;
    info!("Seen DRDY");

    if src == CaptureSource::Camera {
        info!("Turning on the camera");
        camera::turn_off();
    }

    for line_no in 0..NUM_LINES {
        info!("Line no {}", line_no);
        let mut new_line = image_::Line::default()
            .init_number(line_no)
            .init_data([0u8; LINE_LEN as usize].into());

        let address = line_no * LINE_LEN / BYTE_PER_ADDR;
        let [_, _, addr_hi, addr_lo] = address.to_be_bytes();

        cs_n.set_low();
        let _res = spi.write(&[spi_cmd::Command::Read as u8, addr_hi, addr_lo]).await;
        let _res = spi.read(new_line.mut_data()).await;
        cs_n.set_high();

        LINES_TO_SEND.send(new_line).await; // blocks until channel has capacity
    }
}

fn set_state(state: State) {
    STATE.lock(|s| s.set(state));
}

pub fn get_state() -> State {
    STATE.lock(Cell::get)
}

pub fn handle_command(mut command: Command) {
    if let Some(action) = command.take_action() {
        debug!("FPGA received Action={:?}", Debug2Format(&action));
        match action {
            Action::Capture => {
                if get_state() == State::Off
                    && let Some(src) = command.take_capture_source()
                    && matches!(
                        src,
                        CaptureSource::Camera | CaptureSource::FakeVga | CaptureSource::FakeSram
                    )
                {
                    POWER_SIGNAL.turn_on(RunMode::Capture(src));
                }
            }
            Action::SpiFlash => {
                if get_state() == State::Off {
                    POWER_SIGNAL.turn_on(RunMode::SpiFlash);
                }
            }
            Action::RunConstant => {
                if get_state() == State::Off {
                    POWER_SIGNAL.turn_on(RunMode::RunConstant);
                }
            }
            Action::Off => {
                if get_state() != State::Off {
                    POWER_SIGNAL.turn_off();
                }
            }
            #[allow(clippy::wildcard_in_or_patterns)]
            Action::None | _ => (),
        }
    }

    if let Some(flash_cmd) = command.take_flash()
        && matches!(get_state(), State::SpiFlash)
    {
        flash::handle_command(flash_cmd);
    }
}

pub fn get_status() -> Status {
    let mut s = Status::default()
        .init_state(get_state())
        .init_flash(flash::get_status());

    if let Ok(line) = LINES_TO_SEND.try_receive() {
        s.set_line(line);
    }

    s
}
