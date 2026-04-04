use amber_connect::control;
use clap::Parser;
use futures::{FutureExt, TryFutureExt};
use proto::sensor::{Command, Status};
use std::time::Duration;
use tokio::{sync::mpsc, task::JoinSet};
use tokio_serial::UsbPortInfo;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod status;

const LEASE_DURATION: Duration = Duration::from_secs(60);

#[derive(Parser)]
struct Args {
    #[arg(short, action=clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
    let (status_tx, status_rx) = mpsc::channel::<Status>(100);

    let stop = CancellationToken::new();

    let mut services = JoinSet::<anyhow::Result<()>>::new();
    services.spawn(serial::run(command_rx, status_tx, is_sensor_board, stop.clone()).map(Ok));
    services.spawn(control::server::run(command_tx, LEASE_DURATION, stop.clone()).map_err(anyhow::Error::from));
    services.spawn(status::run(status_rx, stop.clone()).map_err(anyhow::Error::from));
    services.spawn(async move {
        tokio::signal::ctrl_c().await.expect("failed to install CTRL-C handler");
        info!("user killed the server");
        Ok(())
    });

    while let Some(exit_value) = services.join_next().await {
        if let Err(err) = exit_value {
            error!(err=?err, "task failed");
        }
        stop.cancel();
    }

    Ok(())
}

fn is_sensor_board(info: &UsbPortInfo) -> bool {
    info.manufacturer.as_deref() == Some("amber") && info.product.as_deref() == Some("Sensor Board")
}
