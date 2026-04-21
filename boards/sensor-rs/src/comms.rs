use core::sync::atomic::{AtomicU32, Ordering};

use crate::{proto::backscatter_, sensors, state_machine};
use defmt::{error, info};
use embassy_stm32::{
    gpio::{Flex, Speed},
    rcc::{Mco, McoConfig},
    usart::{self, Uart},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::Timer;
use micropb::{MessageEncode, PbEncoder};

use crate::{
    clock::MCO_CLOCKS,
    resources::{self, Irqs},
};

type Msg = backscatter_::Status;
const MAX_SIZE: usize = micropb::size::max_encoded_size::<backscatter_::Status>();
const COBS_MAX_SIZE: usize = cobs::max_encoding_length(MAX_SIZE) + 1;
static MESSAGE: Signal<ThreadModeRawMutex, Msg> = Signal::new();

static BACKSCATTER_TX_COUNT: AtomicU32 = AtomicU32::new(0);

#[embassy_executor::task]
pub async fn task(mut r: resources::Comms) {
    info!("Starting COMMS task");
    info!("Backscatter COBS_MAX_SIZE={}", COBS_MAX_SIZE);
    loop {
        let msg = MESSAGE.wait().await;
        backscatter(&mut r, msg).await;
    }
}

pub fn send(msg: Msg) {
    MESSAGE.signal(msg);
}

async fn backscatter(r: &mut resources::Comms, mut msg: Msg) {
    let _mco = Mco::new(r.mco.reborrow(), r.carrier.reborrow(), MCO_CLOCKS.source, {
        let mut config = McoConfig::default();
        config.prescaler = MCO_CLOCKS.carrier_div;
        config.speed = Speed::VeryHigh;
        config
    });

    let count = BACKSCATTER_TX_COUNT.load(Ordering::Acquire);
    msg.set_backscatter_tx_count(count);
    BACKSCATTER_TX_COUNT.store(count + 1, Ordering::Relaxed);
    msg.set_vbat_mv(sensors::get_vbat_mv());
    msg.set_isense_ua(sensors::get_isense_ua());
    msg.set_state(state_machine::get_state());

    Timer::after_millis(10).await;

    let mut uart = Uart::new(
        r.uart.reborrow(),
        r.rx.reborrow(),
        r.tx.reborrow(),
        r.rx_dma.reborrow(),
        r.tx_dma.reborrow(),
        Irqs,
        {
            let mut config = usart::Config::default();
            config.baudrate = 5000;
            config.data_bits = usart::DataBits::DataBits8;
            config.stop_bits = usart::StopBits::STOP2;
            config.parity = usart::Parity::ParityNone;
            config.invert_tx = true;
            config
        },
    )
    .unwrap();

    // Encode with protobuf then COBS
    let mut encoder = PbEncoder::new(heapless::Vec::<u8, MAX_SIZE>::new());
    msg.encode(&mut encoder).unwrap();

    let mut cobs_buf = [0u8; COBS_MAX_SIZE];

    let mut len = cobs::encode(&encoder.into_writer(), &mut cobs_buf);
    cobs_buf[len] = 0; // buffer must be manually terminated
    len += 1;

    info!("Backscatter {}", cobs_buf[..len]);
    if let Err(e) = uart.write(&cobs_buf[..len]).await {
        error!("UART write failed: {}", e);
    }

    Timer::after_millis(100).await; // await doesn't actually wait
    Flex::new(r.carrier.reborrow()).set_as_analog(); // Dropping MCO doesn't disable pin
}
