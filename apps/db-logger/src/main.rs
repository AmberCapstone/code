use std::time::Duration;

use amber_connect::codec::PbReceiver;
use anyhow::anyhow;
use influx::{
    LogItem,
    policy::{LogPolicy, PolicyRouter},
};
use proto::sensor::Status;
use tokio::sync::mpsc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use zeromq::{Socket, SubSocket};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug,tower_http=debug,influx=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut status_socket = SubSocket::new();
    status_socket.connect(amber_connect::endpoint::STATUS).await?;
    status_socket.subscribe("").await?;

    let (item_tx, item_rx) = mpsc::channel(50);

    let db = influx::InfluxConfig {
        measurement: "sensor".to_string(),
        org: env!("INFLUX_ORG").to_string(),
        bucket: env!("INFLUX_BUCKET").to_string(),
        address: amber_connect::endpoint::INFLUX.to_string(),
        token: Some(env!("INFLUX_TOKEN").to_string()),
        tags: Vec::new(),
    };
    let logger = influx::Logger::new(db)
        .with_flush_interval(Duration::from_secs(1))
        .with_policies(
            PolicyRouter::new()
                .with_default(LogPolicy::after_interval(Duration::from_millis(100)))
                .rule("fpga.line.data", LogPolicy::Never)
                .rule("fpga.flash.readout_page.data", LogPolicy::Never)
                .rule(
                    "measurement.temperatureDegc",
                    LogPolicy::after_interval(Duration::from_millis(500)),
                )
                .rule("measurement.*Ua", LogPolicy::EveryMeasurement)
                .rule("*state*", LogPolicy::on_change(Duration::from_millis(500)))
                .rule("name", LogPolicy::on_change(Duration::from_secs(60)))
                .rule("sensor.nvm.parameters", LogPolicy::EveryMeasurement), // only present during NVM actions
        );

    let r = tokio::select! {
        r = feed_logger(&mut status_socket, item_tx) => r,
        () = logger.run(item_rx) => Ok(()),
        _ = tokio::signal::ctrl_c() => Err(anyhow!("Interrupted"))
    };

    status_socket.close().await;

    r
}

async fn feed_logger(status_socket: &mut SubSocket, item_tx: mpsc::Sender<LogItem<Status>>) -> anyhow::Result<()> {
    loop {
        match status_socket.recv_msg::<Status>().await {
            Ok(sts) => {
                let source = sts.name.clone().unwrap_or("unknown".to_string());
                let item = LogItem::new_now(sts, &source).unwrap();
                item_tx.send(item).await?;
            }
            Err(e) => return Err(anyhow!(e)),
        }
    }
}
