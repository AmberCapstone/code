use core::future::pending;

use defmt::info;
use embassy_futures::select::{Either, select};
use embassy_stm32::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::Timer;

use crate::resources;

static NEW_SEQUENCE: Signal<ThreadModeRawMutex, Sequence> = Signal::new();

#[allow(unused)]
#[derive(Clone, Copy)]
pub enum Sequence {
    Toggle,
    Error,
    LowCharge,
    Backscattering,
    Charging,
    On,
    Off,
}

async fn pulse_ms(led: &mut Output<'_>, ms: u64) {
    led.set_high();
    Timer::after_millis(ms).await;
    led.set_low();
}

impl Sequence {
    async fn run(&self, led: &mut Output<'_>) {
        match self {
            Sequence::Toggle => loop {
                led.set_high();
                Timer::after_millis(500).await;
                led.set_low();
                Timer::after_millis(500).await;
            },
            Sequence::Error => loop {
                pulse_ms(led, 100).await;
                Timer::after_millis(400).await;
            },
            Sequence::LowCharge => loop {
                pulse_ms(led, 2).await;
                pending::<()>().await;
            },
            Sequence::Backscattering => {
                pulse_ms(led, 10).await;
                Timer::after_millis(50).await;
                pulse_ms(led, 10).await;
                pending::<()>().await;
            }
            Sequence::On => {
                led.set_high();
                pending::<()>().await;
            }
            Sequence::Off => {
                led.set_low();
                pending::<()>().await;
            }
            Sequence::Charging => {
                pulse_ms(led, 50).await;
                Timer::after_millis(50).await;
                pulse_ms(led, 50).await;
                pending::<()>().await;
            }
        }
    }
}

#[embassy_executor::task]
pub async fn led_task(r: resources::Leds) {
    let mut led = Output::new(r.debug_led, Level::High, embassy_stm32::gpio::Speed::Low);

    let mut seq = Sequence::Off;

    info!("Starting LED task");
    loop {
        if let Either::First(new_seq) = select(NEW_SEQUENCE.wait(), seq.run(&mut led)).await {
            seq = new_seq;
        }
    }
}

pub fn send(s: Sequence) {
    // NEW_SEQUENCE.signal(s);
}
