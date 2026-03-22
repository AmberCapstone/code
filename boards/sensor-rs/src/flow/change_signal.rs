use core::cell::Cell;

use embassy_sync::{blocking_mutex::Mutex, blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

pub struct ChangeSignal<T: PartialEq + Copy> {
    signal: Signal<CriticalSectionRawMutex, T>,
    current: Mutex<CriticalSectionRawMutex, Cell<T>>,
}

impl<T: PartialEq + Copy> ChangeSignal<T> {
    pub const fn new(initial: T) -> Self {
        Self {
            signal: Signal::new(),
            current: Mutex::new(Cell::new(initial)),
        }
    }

    pub fn set(&self, new: T) {
        self.current.lock(|c| {
            if new != c.get() {
                c.set(new);
                self.signal.signal(new);
            }
        });
    }

    pub fn get(&self) -> T {
        self.current.lock(Cell::get)
    }

    pub async fn wait(&self) -> T {
        self.signal.wait().await
    }
}
