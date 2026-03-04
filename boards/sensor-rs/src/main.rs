#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{self};
use embassy_stm32::spi;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

mod proto {
    #![allow(clippy::all)]
    #![allow(nonstandard_style, unused, irrefutable_let_patterns)]
    include!(concat!(env!("OUT_DIR"), "/generated_proto.rs"));
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut led = gpio::Flex::new(p.PA5);

    led.set_as_analog();

    let mut mosi = p.PC12;
    let mut miso = p.PC11;
    let mut sck = p.PC10;

    let flash_spi = spi::Spi::new_blocking(
        p.SPI3,
        sck.reborrow(),
        mosi.reborrow(),
        miso.reborrow(),
        spi::Config::default(),
    );
    drop(flash_spi);

    let mut mosi = gpio::Flex::new(mosi);
    mosi.set_as_analog();
    mosi.set_high();

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(700).await;

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;
    }
}
