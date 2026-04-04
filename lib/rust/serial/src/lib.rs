mod codec;

use std::time::Duration;

use futures::{SinkExt, stream::StreamExt};
use tokio::{sync::mpsc, time::sleep};
use tokio_serial::{SerialPortBuilderExt, SerialPortInfo, SerialPortType, SerialStream, UsbPortInfo};
use tokio_util::{
    codec::{FramedRead, FramedWrite},
    sync::CancellationToken,
};

use codec::{RxCodec, TxCodec};

pub async fn run<Tx, Rx, F: Fn(&UsbPortInfo) -> bool + Clone>(
    mut outgoing: mpsc::Receiver<Tx>,
    incoming: mpsc::Sender<Rx>,
    port_condition: F,
    stop: CancellationToken,
) where
    Tx: prost::Message + Default + 'static,
    Rx: prost::Message + Default + 'static,
{
    stop.run_until_cancelled(async {
        loop {
            tracing::info!("looking for serial ports");
            let port_info = find_connection(&port_condition, Duration::from_secs(1)).await;

            tracing::info!("opening serial port {}", port_info.port_name);
            match tokio_serial::new(port_info.port_name.clone(), 9600).open_native_async() {
                Ok(port) => {
                    tracing::info!("connected to {}", port_info.port_name);
                    match run_connection(port, &mut outgoing, &incoming).await {
                        RunConnectionError::ChannelClosed => break,
                        RunConnectionError::SerialClosed => (),
                    }
                }
                Err(e) => {
                    tracing::error!(err=?e, "failed to connect to port. Trying again in 1 second.");
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    })
    .await;
}

/// Look for a USB connection satisfying `port`
///
/// Polls until one is found.
///
/// # Panics
///
/// Panics if port enumeration fails
async fn find_connection<F: Fn(&UsbPortInfo) -> bool>(port_condition: F, poll_interval: Duration) -> SerialPortInfo {
    loop {
        let mut desired_ports: Vec<SerialPortInfo> = tokio_serial::available_ports()
            .expect("port enumeration to succeed")
            .into_iter()
            .filter_map(|p| match p.port_type {
                SerialPortType::UsbPort(ref info) if port_condition(info) => Some(p),
                _ => None,
            })
            .collect();

        if let Some(p) = desired_ports.pop() {
            tracing::debug!("Found a matching port {}", p.port_name);
            if !desired_ports.is_empty() {
                tracing::warn!(others=?desired_ports, "Other matching ports were found");
            }
            return p;
        }

        tracing::debug!(
            "No matching ports found. Trying again in {} seconds",
            poll_interval.as_secs()
        );
        sleep(poll_interval).await;
    }
}

/// # Errors
///
/// If serial port cannot be opened or if one of the channels closes.
async fn run_connection<Tx, Rx>(
    port: SerialStream,
    outgoing: &mut mpsc::Receiver<Tx>,
    incoming: &mpsc::Sender<Rx>,
) -> RunConnectionError
where
    Tx: prost::Message + Default + 'static,
    Rx: prost::Message + Default + 'static,
{
    let (port_read, port_write) = tokio::io::split(port);

    let mut reader = FramedRead::new(port_read, RxCodec::<Rx>::default());
    let mut writer = FramedWrite::new(port_write, TxCodec::<Tx>::default());

    loop {
        tokio::select! {
            rx = reader.next() => match rx {
                Some(Ok(msg)) => {
                    tracing::trace!("received a valid message from serial");
                    if incoming.send(msg).await.is_err() {
                        tracing::error!("incoming channel closed");
                        return RunConnectionError::ChannelClosed;
                    }
                }
                Some(Err(e)) => tracing::warn!("invalid rx message ({e})"),
                None => {
                    tracing::info!("serial port closed");
                    return RunConnectionError::SerialClosed;
                },
            },
            tx = outgoing.recv() => if let Some(command) = tx {
                tracing::trace!("sending a tx message over serial");
                if writer.send(command).await.is_err() {
                    tracing::error!("failed to send tx message over serial");
                    return RunConnectionError::SerialClosed;
                }
            } else {
                tracing::error!("outgoing channel closed");
                return RunConnectionError::ChannelClosed;
            }
        }
    }
}

enum RunConnectionError {
    ChannelClosed,
    SerialClosed,
}
