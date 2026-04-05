use core::future::pending;
use defmt::{info, warn};
use embassy_futures::select::select;
use embassy_stm32::{
    gpio::{Flex, Level, Output, OutputOpenDrain, Speed},
    i2c::{self, I2c},
    rcc::{Mco, McoConfig, McoPrescaler, McoSource},
    time::Hertz,
};
use embassy_time::Timer;
use sccb::{self, Reg};

use crate::power::PowerSignal;
use crate::proto::sensor_::camera_::{State, Status};
use crate::resources::{Camera, CameraPower, Irqs};
use crate::{flow::StateLock, nvm};
use interface::CameraInterface;

mod interface;

static STATE: StateLock<State> = StateLock::new(State::Off);
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
        STATE.set(State::Off);

        POWER_SIGNAL.wait_for_on().await;

        power_en.set_high();
        select(run_fsm(&mut r), POWER_SIGNAL.wait_for_off()).await;
    }
}

pub async fn run_fsm(r: &mut Camera) {
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

    STATE.set(State::Booting);
    Timer::after_millis(20).await; // I2C gets stuck if we use it too fast

    let mut cam = CameraInterface::new(i2c);

    loop {
        match cam.read_register(sccb::Reg::MIDH).await {
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

    STATE.set(State::Configuring);

    let settings = nvm::get_camera_settings();

    for (reg, val) in settings {
        let _ = cam.write_register(reg, val).await;
        Timer::after_millis(2).await;
    }

    Timer::after_millis(300).await; // wait for settings to apply

    STATE.set(State::Running);

    pending::<()>().await;
}

pub fn get_status() -> Status {
    Status::default().init_state(STATE.get())
}

pub fn turn_on() {
    POWER_SIGNAL.turn_on(());
}

pub fn turn_off() {
    POWER_SIGNAL.turn_off();
}

pub fn is_off() -> bool {
    STATE.is(State::Off)
}

pub fn is_running() -> bool {
    STATE.is(State::Running)
}

#[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
pub fn get_default_settings() -> heapless::Vec<(Reg, u8), { sccb::NUM_REGISTERS }> {
    // From Adafruit_OV7670
    const VSTART: u16 = 10;
    const HSTART: u16 = 176;
    const EDGE_OFFSET: u16 = 0;
    const PCLK_DELAY: u8 = 2;
    const VSTOP: u16 = VSTART + sccb::SENSOR_HEIGHT;
    const HSTOP: u16 = (HSTART + sccb::SENSOR_WIDTH) % (sccb::SENSOR_WIDTH + sccb::BLANK_COLUMNS);

    [
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
        (Reg::SCALING_PCLK_DELAY, PCLK_DELAY),
        (Reg::HSTART, (HSTART >> 3) as u8),
        (Reg::HSTOP, (HSTOP >> 3) as u8),
        (
            Reg::HREF,
            (EDGE_OFFSET << 6) as u8 | ((HSTOP & 0b111) << 3) as u8 | (HSTART & 0b111) as u8,
        ),
        (Reg::VSTRT, (VSTART >> 2) as u8),
        (Reg::VSTOP, (VSTOP >> 2) as u8),
        (Reg::VREF, ((VSTOP & 0b11) << 2) as u8 | (VSTART & 0b11) as u8),
    ]
    .into()
}
