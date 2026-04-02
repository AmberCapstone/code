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
    let _reset_n = OutputOpenDrain::new(r.reset_n.reborrow(), Level::High, Speed::Low);
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

    set_state(State::Configuring);

    let settings = [
        (sccb::Reg::CLKRC, 0x87), // input clock prescaler = 8
        (sccb::Reg::TSLB, 0x00),  // output sequence YUYV
        // QVGA Settings (from Table 2-2)
        (sccb::Reg::COM7, 0x00),
        (sccb::Reg::COM3, 0x04),  // No scale, no tristate (this is default)
        (sccb::Reg::COM14, 0x18), // No manual scaling, PCLK divider = 1
        (sccb::Reg::COM10, 0x20),
        (sccb::Reg::SCALING_XSC, 0x3A | 0b1000_0000), // 8-bar color bar test pattern
        (sccb::Reg::SCALING_YSC, 0x35 | 0b0000_0000), // 8-bar color bar test pattern
        (sccb::Reg::SCALING_DCWCTR, 0x11),
        (sccb::Reg::SCALING_PCLK_DIV, 0xF0),
        (sccb::Reg::SCALING_PCLK_DELAY, 0x02),
    ];

    for (reg, val) in settings {
        let _ = sccb.write_register(reg, val).await;
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
