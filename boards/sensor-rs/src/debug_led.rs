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
    LowCharge,
    Monitor,
    Manual,
    On,
    Off,
}

async fn pulse_ms(led: &mut Output<'_>, ms: u64) {
    led.set_high();
    Timer::after_millis(ms).await;
    led.set_low();
}

async fn pulses(led: &mut Output<'_>, count: u32) {
    pulse_ms(led, 1).await; // No delay before first pulse
    for _ in 1..count {
        Timer::after_millis(150).await;
        pulse_ms(led, 1).await;
    }
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
            Sequence::LowCharge => loop {
                pulses(led, 1).await;
                Timer::after_millis(2000).await;
            },
            Sequence::Monitor => loop {
                pulses(led, 2).await;
                Timer::after_millis(900).await;
            },
            Sequence::Manual => loop {
                pulses(led, 2).await;
                Timer::after_millis(500).await;
                pulses(led, 1).await;
                Timer::after_millis(1500).await;
            },
            Sequence::On => {
                led.set_high();
                pending::<()>().await;
            }
            Sequence::Off => {
                led.set_low();
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
    NEW_SEQUENCE.signal(s);
}
