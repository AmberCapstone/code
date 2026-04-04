use core::cell::Cell;
use core::future::pending;

use defmt::{info, warn};
use embassy_futures::select::select;
use embassy_stm32::gpio::{Flex, Level, Output, OutputOpenDrain, Speed};
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::rcc::{Mco, McoConfig, McoPrescaler, McoSource};
use embassy_stm32::time::Hertz;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_time::Timer;

use crate::power::PowerSignal;
use crate::proto::sensor_::camera_::{State, Status};
use crate::resources::{Camera, CameraPower, Irqs};

mod sccb;

static STATE: Mutex<ThreadModeRawMutex, Cell<State>> = Mutex::new(Cell::new(State::Off));
static POWER_SIGNAL: PowerSignal<()> = PowerSignal::new();

#[embassy_executor::task]
pub async fn task(r_power: CameraPower, mut r: Camera) {
    let mut power_en = Output::new(r_power.en, Level::Low, Speed::Low);

    POWER_SIGNAL.turn_off();

    loop {
        // Ensure the MCO output is off (https://github.com/embassy-rs/embassy/issues/5737)
        Flex::new(r.xclk.reborrow()).set_as_analog();
        Flex::new(r.sda.reborrow()).set_as_analog();
        Flex::new(r.scl.reborrow()).set_as_analog();

        power_en.set_low();
        set_state(State::Off);

        POWER_SIGNAL.wait_for_on().await;

        power_en.set_high();
        select(run_fsm(&mut r), POWER_SIGNAL.wait_for_off()).await;
    }
}

pub async fn run_fsm(r: &mut Camera) {
    use sccb::Reg;

    let mut reset_n = OutputOpenDrain::new(r.reset_n.reborrow(), Level::High, Speed::Low);
    let _power_down = Output::new(r.pwrdn.reborrow(), Level::Low, Speed::Low);

    let _mco = Mco::new(r.mco.reborrow(), r.xclk.reborrow(), McoSource::HSI48, {
        let mut c = McoConfig::default();
        c.prescaler = McoPrescaler::DIV4; // 12 MHz
        c.speed = Speed::VeryHigh;
        c
    });

    let i2c = I2c::new(
        r.i2c.reborrow(),
        r.scl.reborrow(),
        r.sda.reborrow(),
        r.dma_tx.reborrow(),
        r.dma_rx.reborrow(),
        Irqs,
        {
            let mut config = i2c::Config::default();
            config.gpio_speed = Speed::Medium;
            config.frequency = Hertz::khz(100);
            config.scl_pullup = false;
            config.scl_pullup = false;
            config
        },
    );

    set_state(State::Booting);
    Timer::after_millis(20).await; // I2C gets stuck if we use it too fast

    let mut sccb = sccb::SccbInterface::new(i2c);

    loop {
        match sccb.read_register(sccb::Reg::MIDH).await {
            Ok(midh) if midh == sccb::Reg::MIDH.initial() => {
                info!("Connected to Camera over I2C");
                break;
            }
            Ok(midh) => warn!("Incorrect MIDH: {}", midh),
            Err(_) => info!("Camera not talking yet"),
        }

        Timer::after_millis(10).await;
    }

    // Hard reset
    reset_n.set_low();
    Timer::after_millis(2).await;
    reset_n.set_high();
    Timer::after_millis(2).await;

    set_state(State::Configuring);

    const SENSOR_HEIGHT: u16 = 480;
    const SENSOR_WIDTH: u16 = 640;
    const BLANK_COLS: u16 = 144;

    // From Adafruit_OV7670
    const VSTART: u16 = 10;
    const HSTART: u16 = 176;
    const EDGE_OFFSET: u16 = 0;
    const PCLK_DELAY: u16 = 2;
    const VSTOP: u16 = VSTART + SENSOR_HEIGHT;
    const HSTOP: u16 = (HSTART + SENSOR_WIDTH) % (SENSOR_WIDTH + BLANK_COLS);

    let r_hstart = (HSTART >> 3) as u8;
    let r_hstop = (HSTOP >> 3) as u8;
    let r_href = (EDGE_OFFSET << 6) as u8 | ((HSTOP & 0b111) << 3) as u8 | (HSTART & 0b111) as u8;
    let r_vtart = (VSTART >> 2) as u8;
    let r_vstop = (VSTOP >> 2) as u8;
    let r_vref = ((VSTOP & 0b11) << 2) as u8 | (VSTART & 0b11) as u8;

    let settings = [
        (Reg::CLKRC, 0x00), // input clock prescaler = 1
        (Reg::TSLB, 0x00),  // output sequence YUYV
        (Reg::COM10, 0x00), // run PCLK continuously to clock FPGA
        (Reg::COM7, 0x00),  // YUV
        (Reg::COM15, 0xc0), // full dynamic range
        (Reg::COM3, 0x04),
        (Reg::COM14, 0x19),
        (Reg::SCALING_XSC, 0x3a),
        (Reg::SCALING_YSC, 0x35),
        (Reg::SCALING_DCWCTR, 0x11),
        (Reg::SCALING_PCLK_DIV, 0xf1),
        (Reg::SCALING_PCLK_DELAY, 2),
        (Reg::HSTART, r_hstart),
        (Reg::HSTOP, r_hstop),
        (Reg::HREF, r_href),
        (Reg::VSTRT, r_vtart),
        (Reg::VSTOP, r_vstop),
        (Reg::VREF, r_vref),
    ];

    for (reg, val) in settings {
        let _ = sccb.write_register(reg, val).await;
        Timer::after_millis(2).await;
    }

    Timer::after_millis(300).await; // wait for settings to apply

    set_state(State::Running);

    pending::<()>().await;
}

fn set_state(state: State) {
    STATE.lock(|s| s.set(state));
}

pub fn get_state() -> State {
    STATE.lock(Cell::get)
}

pub fn get_status() -> Status {
    Status::default().init_state(get_state())
}

pub fn turn_on() {
    POWER_SIGNAL.turn_on(());
}

pub fn turn_off() {
    POWER_SIGNAL.turn_off();
}

pub fn is_off() -> bool {
    get_state() == State::Off
}
