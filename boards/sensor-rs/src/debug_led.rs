use defmt::info;
use embassy_stm32::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use heapless::Vec;

use crate::resources;

static NEW_SEQUENCE: Signal<ThreadModeRawMutex, Sequence> = Signal::new();

#[derive(Clone, Copy)]
pub enum Sequence {
    Normal,
    Error,
}

impl Sequence {
    fn get_pattern(self) -> Pattern {
        match self {
            Self::Normal => Pattern::from_array([Blink::from_millis(500, 500)]),
            Self::Error => Pattern::from_array([Blink::from_millis(125, 125), Blink::from_millis(125, 125 + 500)]),
        }
    }
}

#[embassy_executor::task]
pub async fn led_task(r: resources::Leds) {
    let mut led = Output::new(r.debug_led, Level::High, embassy_stm32::gpio::Speed::Low);

    let mut seq = Sequence::Normal;

    info!("Starting LED task");
    loop {
        if let Some(new_sig) = NEW_SEQUENCE.try_take() {
            seq = new_sig;
        }

        for blink in seq.get_pattern() {
            led.set_high();
            Timer::after(blink.on).await;
            led.set_low();
            Timer::after(blink.off).await;
        }
    }
}

pub fn send(s: Sequence) {
    NEW_SEQUENCE.signal(s);
}

#[derive(Clone)]
struct Blink {
    on: Duration,
    off: Duration,
}

impl Blink {
    const fn from_millis(on: u64, off: u64) -> Self {
        Self {
            on: Duration::from_millis(on),
            off: Duration::from_millis(off),
        }
    }
}

type Pattern = Vec<Blink, 3>;
