mod flash;

use core::cell::Cell;

use embassy_futures::select::select;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_time::Timer;

use crate::power::PowerSignal;
use crate::proto::sensor_::fpga_::{Action, Command, State, Status};
use crate::resources::{Flash, Fpga, FpgaPower};

static STATE: Mutex<ThreadModeRawMutex, Cell<State>> = Mutex::new(Cell::new(State::Off));
static POWER_SIGNAL: PowerSignal = PowerSignal::new();

#[embassy_executor::task]
pub async fn task(r_power: FpgaPower, mut r_fpga: Fpga, r_flash: Flash) {
    let mut power_en = Output::new(r_power.en, Level::Low, Speed::Low);

    POWER_SIGNAL.turn_off();

    loop {
        power_en.set_low();
        set_state(State::Off);

        POWER_SIGNAL.wait_for_on().await;

        panic!("NOT READY YET - CHECK POWER");

        power_en.set_high();
        select(run_fsm(&mut r_fpga), POWER_SIGNAL.wait_for_off()).await;
    }
}

pub async fn run_fsm(r: &mut Fpga) {}

fn set_state(state: State) {
    STATE.lock(|s| s.set(state));
}

pub fn get_state() -> State {
    STATE.lock(Cell::get)
}

pub fn handle_command(mut command: Command) {
    if let Some(cmd) = command.take_flash() {
        flash::handle_command(cmd);
    }
}
