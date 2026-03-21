use core::cell::Cell;

use crate::{
    flow::ChangeSignal,
    fpga::{self, flash},
    proto::sensor_::{self, Action, State, fpga_},
    resources::{self, Irqs},
    sensors,
};

use defmt::{Debug2Format, info};
use embassy_futures::select::select;
use embassy_stm32::{exti::ExtiInput, gpio::Pull};
use embassy_sync::{blocking_mutex::Mutex, blocking_mutex::raw::ThreadModeRawMutex};
use embassy_time::Timer;

const MIN_CAPTURE_VBAT_MV: u32 = 4800;

#[derive(Clone, Copy, PartialEq)]
enum NormalState {
    Monitor,
    Manual,
}

static NORMAL_STATE: ChangeSignal<NormalState> = ChangeSignal::new(NormalState::Monitor);
static STATE: Mutex<ThreadModeRawMutex, Cell<State>> = Mutex::new(Cell::new(State::LowCharge));

// struct CameraControl {}
// struct FpgaControl {}

#[embassy_executor::task]
pub async fn task(r: resources::StateMachine) {
    let mut vbat_ok = ExtiInput::new(r.vbat_ok, r.vbat_exti, Pull::Down, Irqs);

    // let camera_control = CameraControl {};
    // let fpga_control = FpgaControl {};

    loop {
        select(low_power_loop(), vbat_ok.wait_for_high()).await;
        select(normal_loop(), vbat_ok.wait_for_low()).await;
    }
}

async fn low_power_loop() -> ! {
    set_state(State::LowCharge);
    fpga::handle_command(fpga_::Command::default().init_action(fpga_::Action::Off));
    loop {
        info!("In low_power_loop()");
        Timer::after_millis(2000).await;
        // Send a message over serial
    }
}

async fn normal_loop() -> ! {
    loop {
        fpga::handle_command(fpga_::Command::default().init_action(fpga_::Action::Off));
        match NORMAL_STATE.get() {
            NormalState::Manual => select(manual_loop(), NORMAL_STATE.wait()).await,
            NormalState::Monitor => select(monitor(), NORMAL_STATE.wait()).await,
        };
    }
}

async fn manual_loop() -> ! {
    set_state(State::Manual);
    loop {
        info!(
            "MANUAL, FPGA = {:?}, FLASH = {:?}",
            Debug2Format(&fpga::get_state()),
            Debug2Format(&flash::get_state())
        );
        Timer::after_millis(2000).await;
    }
}

async fn monitor() -> ! {
    loop {
        info!(
            "MONITOR, FPGA = {:?}, FLASH = {:?}",
            Debug2Format(&fpga::get_state()),
            Debug2Format(&flash::get_state())
        );
        set_state(State::Charging);

        // while sensors::get_vbat_mv() < MIN_CAPTURE_VBAT_MV {
        //     Timer::after_millis(100).await;
        // }

        // set_state(State::Capture);

        Timer::after_millis(1000).await;
        // add actual code
    }
}

fn set_state(state: State) {
    STATE.lock(|s| s.set(state));
}

pub fn get_state() -> State {
    STATE.lock(Cell::get)
}

pub fn handle_command(mut command: sensor_::Command) {
    if let Some(action) = command.take_action() {
        handle_action(action);
    }

    if let Some(cmd) = command.take_fpga()
        && matches!(get_state(), State::Manual)
    {
        fpga::handle_command(cmd);
    }
}

fn handle_action(action: Action) {
    match action {
        Action::Manual => NORMAL_STATE.set(NormalState::Manual),
        Action::Monitor => NORMAL_STATE.set(NormalState::Monitor),
        _ => (),
    }
}
