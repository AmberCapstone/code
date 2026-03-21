use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};

#[derive(PartialEq)]
enum Power<T> {
    Off,
    On(T),
}

pub struct PowerSignal<T>(Signal<ThreadModeRawMutex, Power<T>>);

impl<T> PowerSignal<T> {
    pub const fn new() -> Self {
        Self(Signal::new())
    }

    pub fn turn_on(&self, t: T) {
        self.0.signal(Power::On(t));
    }

    pub fn turn_off(&self) {
        self.0.signal(Power::Off);
    }

    pub async fn wait_for_on(&self) -> T {
        loop {
            if let Power::On(t) = self.0.wait().await {
                return t;
            }
        }
    }

    pub async fn wait_for_off(&self) {
        while !matches!(self.0.wait().await, Power::Off) {}
    }
}
