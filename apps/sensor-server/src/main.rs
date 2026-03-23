use amber_connect::{codec::ZmqMsgSender, control};
use proto::sensor::{Command, Status};

use std::{error::Error, time::Duration};
use tokio::{select, sync::mpsc};
use tokio_util::sync::CancellationToken;
use zeromq::{PubSocket, Socket};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let amber_ports = serial::get_sensor_ports().unwrap();
    let Some(port) = amber_ports.first() else {
        return Err("No amber Sensor Boards detected".into());
    };

    let (command_tx, command_rx) = mpsc::channel::<Command>(100);
    let (status_tx, mut status_rx) = mpsc::channel::<Status>(100);

    let stop = CancellationToken::new();

    let mut status_socket = PubSocket::new();
    status_socket.bind(amber_connect::endpoint::STATUS).await?;

    let stop_serial = stop.clone();
    let j1 = tokio::spawn(serial::run(port.port_name.clone(), command_rx, status_tx, stop_serial));

    let stop_control = stop.clone();
    let j2 = tokio::spawn(async move {
        let _r = control::server::run(command_tx, Duration::from_secs(10), stop_control.clone()).await;
        stop_control.cancel();
    });

    let stop_status = stop.clone();
    let j3 = tokio::spawn(async move {
        loop {
            select! {
                status = status_rx.recv() => {
                    match status {
                        Some(status) => status_socket.send_msg(&status).await.unwrap(),
                        None => break
                    }
                }
                () = stop_status.cancelled() => { break; }
            }
        }
        status_socket.close().await;
    });

    tokio::signal::ctrl_c().await.unwrap();
    println!("Stopping");
    stop.cancel();

    let _ = tokio::join!(j1, j2, j3);

    Ok(())
}
