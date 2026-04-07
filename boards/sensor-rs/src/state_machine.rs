use core::sync::atomic::{AtomicU32, Ordering};

use crate::{
    comms, debug_led,
    flow::{ChangeSignal, StateLock, poll},
    fpga::{self},
    nvm,
    proto::sensor_::{
        self, Action, State,
        fpga_::{self, DataRequest},
    },
    resources::{self, Irqs},
    sensors,
};

use defmt::info;
use embassy_futures::select::select;
use embassy_stm32::{exti::ExtiInput, gpio::Pull};
use embassy_time::{Duration, Instant, Timer};

const MIN_CAPTURE_VBAT_MV: u32 = 4800;
const MIN_CAPTURE_PERIOD: Duration = Duration::from_secs(10);

#[derive(Clone, Copy, PartialEq)]
enum NormalState {
    Monitor,
    Manual,
}

static NORMAL_STATE: ChangeSignal<NormalState> = ChangeSignal::new(NormalState::Monitor);
static STATE: StateLock<State> = StateLock::new(State::LowCharge);
static LAST_CAPTURE_INTERVAL: AtomicU32 = AtomicU32::new(0);

#[embassy_executor::task]
pub async fn task(r: resources::StateMachine) {
    info!("Starting STATE MACHINE task");
    let mut vbat_ok = ExtiInput::new(r.vbat_ok, r.vbat_exti, Pull::None, Irqs);

    loop {
        select(low_power_loop(), vbat_ok.wait_for_high()).await;
        select(normal_loop(), async {
            loop {
                vbat_ok.wait_for_low().await;
                Timer::after_millis(150).await; // debounce
                if vbat_ok.is_low() {
                    break;
                }
            }
        })
        .await;
    }
}

async fn low_power_loop() -> ! {
    STATE.set(State::LowCharge);
    LAST_CAPTURE_INTERVAL.store(0, Ordering::Relaxed);
    loop {
        info!("In low_power_loop() {} mV", sensors::get_vbat_mv());
        fpga::turn_off();

        debug_led::send(debug_led::Sequence::LowCharge);
        // comms::send(());
        // Send a message over serial
        Timer::after_millis(1000).await;
    }
}

async fn normal_loop() -> ! {
    info!("Entering normal_loop");
    loop {
        fpga::turn_off();
        match NORMAL_STATE.get() {
            NormalState::Manual => select(manual_loop(), NORMAL_STATE.wait()).await,
            // NormalState::Monitor => select(monitor(), NORMAL_STATE.wait()).await,
            NormalState::Monitor => select(comms_test(), NORMAL_STATE.wait()).await,
        };
    }
}

async fn manual_loop() -> ! {
    STATE.set(State::Manual);
    loop {
        info!("In manual_loop()");
        Timer::after_millis(2000).await;
    }
}

async fn comms_test() -> ! {
    let mut x: u8 = 1;
    loop {
        info!("Comms {}", sensors::get_vbat_mv());
        STATE.set(State::Charging);

        comms::send([x, 1, x * 2, 2]);
        x += 1;

        Timer::after_millis(1000).await; // avoid captures going too fast
    }
}

async fn monitor() -> ! {
    LAST_CAPTURE_INTERVAL.store(0, Ordering::Relaxed);
    loop {
        info!("Charging");
        STATE.set(State::Charging);
        let start = Instant::now();

        Timer::after_millis(5000).await; // avoid captures going too fast

        while sensors::get_vbat_mv() < MIN_CAPTURE_VBAT_MV {
            let measure = sensors::get_status();
            info!("VBAT: {} mV    ISENSE: {} uA", measure.vbat_mv, measure.isense_ua);
            Timer::after_millis(1000).await;
        }

        STATE.set(State::Capture);
        info!("Capturing");
        fpga::capture(DataRequest::Vessels);

        poll::until(fpga::is_done, Duration::from_millis(100)).await;
        info!("Done capture");
        LAST_CAPTURE_INTERVAL.store(start.elapsed().as_millis() as u32, Ordering::Relaxed);
    }
}

pub fn get_state() -> State {
    STATE.get()
}

pub fn get_last_charge_ms() -> u32 {
    LAST_CAPTURE_INTERVAL.load(Ordering::Acquire)
}

pub fn handle_command(mut command: sensor_::Command) {
    if let Some(action) = command.take_action() {
        match action {
            Action::Manual => NORMAL_STATE.set(NormalState::Manual),
            Action::Monitor => NORMAL_STATE.set(NormalState::Monitor),
            _ => (),
        }
    }

    if STATE.is(State::Manual) {
        if let Some(cmd) = command.take_fpga() {
            fpga::handle_command(cmd);
        }

        if let Some(cmd) = command.take_nvm() {
            nvm::handle_command(cmd);
        }
    }
}
