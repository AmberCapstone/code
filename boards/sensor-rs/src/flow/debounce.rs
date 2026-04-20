use embassy_stm32::exti::ExtiInput;
use embassy_stm32::mode::Async;
use embassy_time::{Duration, Instant};

pub struct DebouncedExtiInput<'d> {
    pin: ExtiInput<'d, Async>,
    timeout: Duration,
    last_rise: Instant,
    last_fall: Instant,
}

impl<'d> DebouncedExtiInput<'d> {
    pub fn new(pin: ExtiInput<'d, Async>, timeout: Duration) -> Self {
        Self {
            pin,
            timeout,
            last_rise: Instant::MIN,
            last_fall: Instant::MIN,
        }
    }

    pub async fn wait_for_rising_edge(&mut self) {
        loop {
            self.pin.wait_for_rising_edge().await;
            if self.last_rise.elapsed() > self.timeout {
                self.last_rise = Instant::now();
                break;
            }
        }
    }

    pub async fn wait_for_falling_edge(&mut self) {
        loop {
            self.pin.wait_for_falling_edge().await;
            if self.last_fall.elapsed() > self.timeout {
                self.last_fall = Instant::now();
                break;
            }
        }
    }
}
