mod codec;

use std::io;

use futures::{SinkExt, stream::StreamExt};
use tokio::sync::mpsc;
use tokio_serial::{SerialPortBuilderExt, SerialPortInfo, SerialPortType};
use tokio_util::{
    codec::{FramedRead, FramedWrite},
    sync::CancellationToken,
};

use codec::{RxCodec, TxCodec};

/// # Errors
///
/// If serial port cannot be opened
pub fn start<Tx, Rx>(
    port: &str,
    mut outgoing: mpsc::Receiver<Tx>,
    incoming: mpsc::Sender<Rx>,
    stop: CancellationToken,
) -> Result<(), tokio_serial::Error>
where
    Tx: prost::Message + Default + 'static,
    Rx: prost::Message + Default + 'static,
{
    let port = tokio_serial::new(port, 9600).open_native_async()?;

    let (port_read, port_write) = tokio::io::split(port);

    let mut reader = FramedRead::new(port_read, RxCodec::<Rx>::default());
    let mut writer = FramedWrite::new(port_write, TxCodec::<Tx>::default());

    let stop_read = stop.clone();
    let stop_write = stop;

    tokio::spawn(async move {
        loop {
            tokio::select! {
                result = reader.next() => {
                    match result {
                        Some(Ok(msg)) => if incoming.send(msg).await.is_err() {
                            stop_read.cancel();
                        }
                        Some(Err(e)) => eprintln!("read error {e}"),
                        None => break, // port closed
                    }
                }
                () = stop_read.cancelled() => { break; }
            }
        }
    });

    tokio::spawn(async move {
        loop {
            tokio::select! {
                command = outgoing.recv() => {
                    match command {
                        Some(command) => if writer.send(command).await.is_err() {
                            stop_write.cancel();
                        }
                        None => break, // port closed
                    }
                }
                () = stop_write.cancelled() => { break; }
            }
        }
    });

    Ok(())
}

/// # Errors
///
/// Fails to enumerature the system serial ports
pub fn get_sensor_ports() -> Result<Vec<SerialPortInfo>, io::Error> {
    Ok(tokio_serial::available_ports()?
        .into_iter()
        .filter(|p| {
            matches!(&p.port_type,
            SerialPortType::UsbPort(info) if
                info.manufacturer.as_deref() == Some("amber")
                    && info.product.as_deref() == Some("Sensor Board")
            )
        })
        .collect())
}
