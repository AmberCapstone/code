use core::cell::Cell;

use embassy_sync::blocking_mutex::{Mutex, raw::ThreadModeRawMutex};

pub struct StateLock<T: Copy> {
    inner: Mutex<ThreadModeRawMutex, Cell<T>>,
}

impl<T: Copy> StateLock<T> {
    pub const fn new(initial: T) -> Self {
        Self {
            inner: Mutex::new(Cell::new(initial)),
        }
    }

    pub fn set(&self, state: T) {
        self.inner.lock(|s| s.set(state));
    }

    pub fn get(&self) -> T {
        self.inner.lock(Cell::get)
    }
}

impl<T: Copy + PartialEq> StateLock<T> {
    pub fn is(&self, other: T) -> bool {
        self.get() == other
    }
}
