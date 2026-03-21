#![no_std]
#![no_main]

#[allow(clippy::wildcard_imports)]
use crate::resources::*;

use embassy_executor::Spawner;
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _};

mod camera;
mod clock;
mod debug_led;
mod flow;
mod fpga;
mod power;
mod resources;
mod sensors;
mod serial;
mod state_machine;

mod proto {
    #![allow(clippy::all, clippy::pedantic, nonstandard_style, unused, irrefutable_let_patterns)]
    include!(concat!(env!("OUT_DIR"), "/generated_proto.rs"));
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc = clock::get_config();

    let p = embassy_stm32::init(config);

    let r = split_resources!(p);

    spawner.spawn(state_machine::task(r.state_machine)).unwrap();
    spawner.spawn(serial::task(r.usb)).unwrap();
    spawner.spawn(camera::task(r.camera_power, r.camera)).unwrap();
    spawner.spawn(fpga::task(r.fpga_power, r.fpga)).unwrap();
    spawner.spawn(fpga::flash::task(r.flash)).unwrap();
    spawner.spawn(debug_led::led_task(r.leds)).unwrap();
    spawner.spawn(sensors::task(r.sensors)).unwrap();
}
