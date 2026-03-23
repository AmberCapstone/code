use amber_connect::codec::ZmqMsgSender;
use proto::sensor::{Command, Status};
use std::error::Error;
use tokio::{select, sync::mpsc};
use tokio_util::sync::CancellationToken;
use zeromq::{PubSocket, Socket};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let amber_ports = serial::get_sensor_ports().unwrap();
    let Some(port) = amber_ports.first() else {
        return Err("No amber Sensor Boards detected".into());
    };

    let (_outgoing, outgoing_rx) = mpsc::channel::<Command>(100);
    let (incoming_tx, mut incoming) = mpsc::channel::<Status>(100);

    let stop = CancellationToken::new();
    serial::start(&port.port_name, outgoing_rx, incoming_tx, stop.clone())?;

    let mut socket = PubSocket::new();
    socket.bind(amber_connect::endpoint::STATUS).await?;

    loop {
        select! {
            status = incoming.recv() => {
                match status {
                    Some(status) => socket.send_msg(&status).await?,
                    None => break
                }
            }
            ctrl = tokio::signal::ctrl_c() => {
                if ctrl.is_err() {
                    eprintln!("Failed to listen for CTRL-C");
                }
                break;
            }
        }
    }
    stop.cancel();

    socket.close().await;

    Ok(())
}
