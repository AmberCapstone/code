#![no_std]
#![no_main]

#[allow(clippy::wildcard_imports)]
use crate::resources::*;

use embassy_executor::Spawner;
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _};

mod debug_led;
mod flash;
mod rcc;
mod resources;
mod serial;
mod state_machine;

mod proto {
    #![allow(clippy::all, clippy::pedantic, nonstandard_style, unused, irrefutable_let_patterns)]
    include!(concat!(env!("OUT_DIR"), "/generated_proto.rs"));
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    rcc::configure(&mut config.rcc);

    let p = embassy_stm32::init(config);

    let r = split_resources!(p);

    spawner.spawn(state_machine::task()).unwrap();
    spawner.spawn(flash::task(r.flash)).unwrap();
    spawner.spawn(debug_led::led_task(r.leds)).unwrap();
    spawner.spawn(serial::serial_task(r.usb)).unwrap();
}
