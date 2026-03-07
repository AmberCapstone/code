use cobs::CobsDecoder;
use defmt::{debug, info, warn};
use embassy_futures::join::join3;
use embassy_stm32::usb::{Driver, Instance};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::Timer;
use embassy_usb::{
    Builder,
    class::cdc_acm::{self, CdcAcmClass, Receiver, Sender},
    driver::EndpointError,
};
use micropb::{MessageDecode, MessageEncode, PbDecoder, PbEncoder};

use crate::{flash, proto, resources, state_machine};

const PACKET_SIZE: u16 = 64;

#[derive(Clone)]
struct State {
    rx_counter: u32,
    tx_counter: u32,
}

static STATE: Mutex<ThreadModeRawMutex, State> = Mutex::new(State {
    rx_counter: 0,
    tx_counter: 0,
});

#[embassy_executor::task]
pub async fn serial_task(u: resources::Usb) {
    // Config largely copied from embassy/examples/stm32u/src/bin/usb_serial.rs
    let driver = Driver::new(u.usb, resources::Irqs, u.dp, u.dm);

    let mut config = embassy_usb::Config::new(0xbf00, 0xc0de);
    config.manufacturer = Some("amber");
    config.product = Some("Sensor Board");

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 256];

    let mut state = cdc_acm::State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // No msos descriptors
        &mut control_buf,
    );

    let (mut sender, mut receiver) = CdcAcmClass::new(&mut builder, &mut state, PACKET_SIZE).split();

    let mut usb = builder.build();
    let usb_fut = usb.run(); // can be suspended

    let send_fut = async {
        loop {
            sender.wait_connection().await;
            info!("TX Connected");
            let _ = send_loop(&mut sender).await;
            info!("TX Disconnected");
        }
    };
    let recv_fut = async {
        loop {
            receiver.wait_connection().await;
            info!("RX Connected");
            let _ = receive_loop(&mut receiver).await;
            info!("TX Disconnected");
        }
    };

    join3(usb_fut, send_fut, recv_fut).await; // never returns
}

async fn send_loop<'d, T: Instance + 'd>(sender: &mut Sender<'d, Driver<'d, T>>) -> Result<(), Disconnected> {
    const MAX_SIZE: usize = micropb::size::max_encoded_size::<proto::sensor_::Status>();
    const COBS_MAX_SIZE: usize = cobs::max_encoding_length(MAX_SIZE) + 1;

    let mut cobs_buf = [0u8; COBS_MAX_SIZE];

    loop {
        let state = STATE.lock().await.clone();
        let status = proto::sensor_::Status::default()
            .init_rx_counter(state.rx_counter)
            .init_tx_counter(state.tx_counter)
            .init_state(state_machine::get_state())
            .init_flash_status(flash::get_status());

        // Encode with protobuf then COBS
        let mut encoder = PbEncoder::new(heapless::Vec::<u8, MAX_SIZE>::new());
        status.encode(&mut encoder).unwrap();

        let mut len = cobs::encode(&encoder.into_writer(), &mut cobs_buf);
        cobs_buf[len] = 0; // must be manually terminated
        len += 1;

        // Split and send the buffer in packets
        for chunk in cobs_buf[..len].chunks(PACKET_SIZE.into()) {
            sender.write_packet(chunk).await?;
        }
        sender.write_packet(&[]).await?; // send zero length packet to ensure host processes the last chunk

        debug!("Sent {}", state.tx_counter);
        STATE.lock().await.tx_counter += 1;

        Timer::after_millis(10).await;
    }
}

async fn receive_loop<'d, T: Instance + 'd>(receiver: &mut Receiver<'d, Driver<'d, T>>) -> Result<(), Disconnected> {
    const MAX_SIZE: usize = micropb::size::max_encoded_size::<proto::sensor_::Command>();
    const COBS_MAX_SIZE: usize = cobs::max_encoding_length(MAX_SIZE) + 1;

    let mut packet_buf = [0u8; PACKET_SIZE as usize];

    let mut cobs_buf = [0u8; COBS_MAX_SIZE]; // this could overflow on bad data
    let mut decoder = CobsDecoder::new(&mut cobs_buf);

    loop {
        let len = receiver.read_packet(&mut packet_buf).await?;

        // Feed the decoder with the new packet
        match decoder.push(&packet_buf[..len]) {
            Ok(Some(n)) => {
                // Decode with proto
                let mut pb_dec = PbDecoder::new(&decoder.dest()[..n.frame_size()]);
                let mut command = proto::sensor_::Command::default();
                if command.decode(&mut pb_dec, n.frame_size()).is_ok() {
                    process_command(command).await;
                }
            }
            Ok(None) => debug!("I'M HUNGRY"),
            Err(_) => warn!("Invalid COBS"),
        }
    }
}

async fn process_command(mut command: proto::sensor_::Command) {
    STATE.lock().await.rx_counter += 1;

    if let Some(action) = command.take_action() {
        state_machine::handle_action(action);
    }

    if let Some(page) = command.take_page() {
        flash::accept_page(page);
    }

    if let Some(host_pg_req) = command.take_host_page_request() {
        flash::set_readout_req_number(host_pg_req);
    }
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Self {},
        }
    }
}
