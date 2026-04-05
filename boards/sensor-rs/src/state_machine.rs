use crate::{
    camera, comms, debug_led,
    flow::{ChangeSignal, StateLock},
    fpga::{self, flash},
    nvm,
    proto::sensor_::{self, Action, State, fpga_},
    resources::{self, Irqs},
    sensors,
};

use defmt::info;
use embassy_futures::select::select;
use embassy_stm32::{exti::ExtiInput, gpio::Pull};
use embassy_time::Timer;

const MIN_CAPTURE_VBAT_MV: u32 = 4800;

#[derive(Clone, Copy, PartialEq)]
enum NormalState {
    Monitor,
    Manual,
}

static NORMAL_STATE: ChangeSignal<NormalState> = ChangeSignal::new(NormalState::Monitor);
static STATE: StateLock<State> = StateLock::new(State::LowCharge);

#[embassy_executor::task]
pub async fn task(r: resources::StateMachine) {
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
    fpga::handle_command(fpga_::Command::default().init_action(fpga_::Action::Off));
    loop {
        info!("In low_power_loop()");
        debug_led::send(debug_led::Sequence::LowCharge);
        comms::send(());
        // Send a message over serial
        Timer::after_millis(1000).await;
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

pub fn get_state() -> State {
    STATE.get()
}

async fn manual_loop() -> ! {
    STATE.set(State::Manual);
    loop {
        info!("In manual_loop()");
        Timer::after_millis(2000).await;
    }
}

async fn monitor() -> ! {
    loop {
        STATE.set(State::Charging);

        // while sensors::get_vbat_mv() < MIN_CAPTURE_VBAT_MV {
        //     Timer::after_millis(100).await;
        // }

        // set_state(State::Capture);

        Timer::after_millis(1000).await;
        // add actual code
    }
}

pub fn handle_command(mut command: sensor_::Command) {
    if let Some(action) = command.take_action() {
        handle_action(action);
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

fn handle_action(action: Action) {
    match action {
        Action::Manual => NORMAL_STATE.set(NormalState::Manual),
        Action::Monitor => NORMAL_STATE.set(NormalState::Monitor),
        _ => (),
    }
}
