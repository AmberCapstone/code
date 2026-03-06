use core::cell::Cell;

use crate::flash;
use crate::proto::{sensor_::Action, sensor_::State};

use defmt::{Debug2Format, error, info, trace, warn};
use embassy_sync::signal::Signal;
use embassy_sync::{blocking_mutex::Mutex, blocking_mutex::raw::ThreadModeRawMutex};
use embassy_time::{Duration, Ticker};

static ACTION: Signal<ThreadModeRawMutex, Action> = Signal::new();

static STATE: Mutex<ThreadModeRawMutex, Cell<State>> = Mutex::new(Cell::new(State::Idle));

#[embassy_executor::task]
pub async fn task() {
    let mut ticker = Ticker::every(Duration::from_hz(1000));

    let mut transition: Option<State> = None;

    loop {
        let mut on_enter = false;
        if let Some(new_state) = transition.take().or_else(|| {
            ACTION
                .try_take()
                .and_then(|action| action_transition(action, get_state()))
        }) {
            info!("Transitioning into {:?}", Debug2Format(&new_state));
            STATE.lock(|s| s.set(new_state));
            on_enter = true;
        }

        match get_state() {
            State::Idle => (),
            State::Flashing => {
                if on_enter {
                    flash::start();
                }

                // if flash::IsDone() {
                transition = Some(State::Idle);
                // }
            }
            State::Readout => {
                if on_enter {
                    flash::start_readout();
                }

                // if flash::IsDone() {
                transition = Some(State::Idle);
                // }
            }
            _ => {
                error!("Unknown state");
                transition = Some(State::Idle);
            }
        }

        ticker.next().await;
    }
}

pub fn get_state() -> State {
    STATE.lock(|s| s.get())
}

pub fn handle_action(action: Action) {
    trace!("Received action {:?}", Debug2Format(&action));
    ACTION.signal(action);
}

fn action_transition(action: Action, state: State) -> Option<State> {
    match action {
        Action::None => None,
        Action::Reset => (state != State::Idle).then_some(State::Idle),
        Action::Flash => (state == State::Idle).then_some(State::Flashing),
        Action::Readout => (state == State::Idle).then_some(State::Readout),
        _ => {
            warn!("Unknown action {:?}", Debug2Format(&action));
            None
        }
    }
}
