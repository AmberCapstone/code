use defmt::info;
use embassy_stm32::{
    gpio::{Flex, Level, Output, Speed},
    rcc::{Mco, McoConfig, McoSource},
    usart::{self, Uart},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::Timer;

use crate::{debug_led, resources};

static MESSAGE: Signal<ThreadModeRawMutex, ()> = Signal::new();

#[embassy_executor::task]
pub async fn task(mut r: resources::Comms) {
    loop {
        let msg = MESSAGE.wait().await;
        backscatter(&mut r, msg).await;
    }
}

pub fn send(msg: ()) {
    MESSAGE.signal(msg);
}

async fn backscatter(r: &mut resources::Comms, _msg: ()) {
    let mco = Mco::new(r.mco.reborrow(), r.carrier.reborrow(), McoSource::HSI48, {
        let mut config = McoConfig::default();
        config.prescaler = embassy_stm32::rcc::McoPrescaler::DIV8; // 6 MHz
        config.speed = Speed::VeryHigh;
        config
    });

    let mut gate = Output::new(r.tx.reborrow(), Level::High, Speed::Low);

    info!("REFLECTING");
    // debug_led::send(debug_led::Sequence::On);
    gate.set_high();
    Timer::after_millis(50).await;

    info!("ABSORBING");
    gate.set_low();

    // Timer::after_millis(100).await;

    // let mut uart = Uart::new_blocking(r.uart.reborrow(), r.rx.reborrow(), r.tx.reborrow(), {
    //     let mut config = usart::Config::default();
    //     config.baudrate = 96;
    //     config.data_bits = usart::DataBits::DataBits8;
    //     config.stop_bits = usart::StopBits::STOP1;
    //     config.parity = usart::Parity::ParityNone;
    //     config.invert_tx = true;
    //     config
    // })
    // .unwrap();

    // Timer::after_millis(100).await;

    // let buffer: [u8; 6] = [b'A', b'M', b'B', b'E', b'R', count];
    // let _ = uart.blocking_write(&[0xaa]);

    // // info!("backshot {}", buffer);

    // Timer::after_millis(100).await;

    // Flex::new(r.carrier.reborrow()).set_as_analog();
    // Timer::after_millis(1000).await;

    Flex::new(r.carrier.reborrow()).set_as_analog();
}
