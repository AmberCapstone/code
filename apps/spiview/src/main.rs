use amber_connect::{self, codec::PbReceiver};
use chrono::Local;
use proto::sensor::Status;
use std::error::Error;
use tokio::select;
use zeromq::{Socket, SubSocket};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut status_sock = SubSocket::new();
    status_sock.connect(amber_connect::endpoint::STATUS).await?;
    status_sock.subscribe("").await?;

    loop {
        select! {
            Ok(status) = status_sock.recv_msg::<Status>() => {
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");

                if let Some(spi_tx) = status.fpga.as_ref().and_then(|fp| fp.spi_tx.clone()) {
                    let bytes = spi_tx.into_iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ");
                    println!("{timestamp} >>> {bytes}");
                }

                if let Some(spi_rx) = status.fpga.as_ref().and_then(|fp| fp.spi_rx.clone()) {
                    let bytes = spi_rx.into_iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ");
                    println!("{timestamp} <<< {bytes}");
                }
            }
            ctrl = tokio::signal::ctrl_c() => {
                if ctrl.is_err() {
                    eprintln!("Failed to listen for CTRL-C");
                }
                break;
            }
            else => break
        };
    }

    status_sock.close().await;

    Ok(())
}
