use core::cell::Cell;

use defmt::{debug, info, warn};
use embassy_futures::select::select;
use embassy_stm32::gpio::{Level, Output, OutputOpenDrain, Speed};
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::rcc::{Mco, McoConfig, McoPrescaler, McoSource};
use embassy_stm32::time::Hertz;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_time::Timer;

use crate::power::PowerSignal;
use crate::proto::sensor_::camera_::{Action, Command, State, Status};
use crate::resources::{Camera, CameraPower, Irqs};

mod sccb;

static STATE: Mutex<ThreadModeRawMutex, Cell<State>> = Mutex::new(Cell::new(State::Off));
static POWER_SIGNAL: PowerSignal<()> = PowerSignal::new();

#[embassy_executor::task]
pub async fn task(r_power: CameraPower, mut r: Camera) {
    let mut power_en = Output::new(r_power.en, Level::Low, Speed::Low);

    POWER_SIGNAL.turn_off();

    loop {
        power_en.set_low();
        set_state(State::Off);

        POWER_SIGNAL.wait_for_on().await;
        todo!("Check power sequencing with fpga");

        power_en.set_high();
        select(run_fsm(&mut r), POWER_SIGNAL.wait_for_off()).await;
    }
}

pub async fn run_fsm(r: &mut Camera) {
    let _reset_n = OutputOpenDrain::new(r.reset_n.reborrow(), Level::High, Speed::Low);
    let _power_down = Output::new(r.pwrdn.reborrow(), Level::Low, Speed::Low);

    todo!("Verify MCO turns off when xclk is dropped. Else this is unsafe when camera is powered off");

    let xclk = Mco::new(r.mco.reborrow(), r.xclk.reborrow(), McoSource::HSI48, {
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

    let sccb = sccb::SccbInterface::new(i2c);

    set_state(State::Booting);
    loop {
        match sccb.read_register(sccb::Reg::MIDH).await {
            Ok(midh) if midh == sccb::Reg::MIDH.initial() => break,
            Ok(midh) => warn!("Incorrect MIDH: {}", midh),
            Err(_) => debug!("Camera not talking yet"),
        }

        Timer::after_millis(10).await;
    }

    set_state(State::Configuring);

    let settings = [
        (sccb::Reg::CLKRC, 0x9f), // input clock prescaler = 2
        (sccb::Reg::TSLB, 0x00),  // output sequence YUYV
        // VGA Settings (from Table 2-2)
        (sccb::Reg::COM7, 0x00),  // VGA, YUV output (this is default)
        (sccb::Reg::COM3, 0x00),  // No scale, no tristate (this is default)
        (sccb::Reg::COM14, 0x00), // No manual scaling, PCLK divider = 1
        (sccb::Reg::COM10, 0x20),
        (sccb::Reg::SCALING_XSC, 0x3A),
        (sccb::Reg::SCALING_YSC, 0x35),
        (sccb::Reg::SCALING_DCWCTR, 0x11),
        (sccb::Reg::SCALING_PCLK_DIV, 0xF0),
        (sccb::Reg::SCALING_PCLK_DELAY, 0x02),
    ];

    for (reg, val) in settings {
        let _ = sccb.write_register(reg, val).await;
    }

    set_state(State::Running);
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
