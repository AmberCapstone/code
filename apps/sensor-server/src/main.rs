use amber_connect::{codec::PbSender, control};
use clap::Parser;
use proto::sensor::{Command, Status};
use tokio_serial::UsbPortInfo;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::{error::Error, time::Duration};
use tokio::{select, sync::mpsc};
use tokio_util::sync::CancellationToken;
use zeromq::{PubSocket, Socket};

const LEASE_DURATION: Duration = Duration::from_secs(60);

#[derive(Parser)]
struct Args {
    #[arg(short, action=clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let filter = match args.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(filter))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (command_tx, command_rx) = mpsc::channel::<Command>(100);
    let (status_tx, mut status_rx) = mpsc::channel::<Status>(100);

    let stop = CancellationToken::new();

    let mut status_socket = PubSocket::new();
    status_socket.bind(amber_connect::endpoint::STATUS).await?;

    let stop_serial = stop.clone();
    let j1 = tokio::spawn(serial::run(is_sensor_board, command_rx, status_tx, stop_serial));

    let stop_control = stop.clone();
    let j2 = tokio::spawn(async move {
        let _r = control::server::run(command_tx, LEASE_DURATION, stop_control.clone()).await;
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
    tracing::info!("Stopping server");
    stop.cancel();

    let _ = tokio::join!(j1, j2, j3);

    Ok(())
}

fn is_sensor_board(info: &UsbPortInfo) -> bool {
    info.manufacturer.as_deref() == Some("amber") && info.product.as_deref() == Some("Sensor Board")
}
