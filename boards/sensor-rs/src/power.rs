use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};

#[derive(PartialEq)]
enum Power {
    Off,
    On,
}

pub struct PowerSignal(Signal<ThreadModeRawMutex, Power>);

impl PowerSignal {
    pub const fn new() -> Self {
        Self(Signal::new())
    }

    pub fn turn_on(&self) {
        self.0.signal(Power::On)
    }

    pub fn turn_off(&self) {
        self.0.signal(Power::Off)
    }

    pub async fn wait_for_on(&self) {
        while self.0.wait().await != Power::On {}
    }

    pub async fn wait_for_off(&self) {
        while self.0.wait().await != Power::Off {}
    }
}
