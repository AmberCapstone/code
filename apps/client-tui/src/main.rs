use amber_connect::{self, codec::PbReceiver};
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
                print!("\x1B[2J\x1B[H"); // Move cursor to top

                // println!("{}", serde_json::to_string_pretty(&status).unwrap()); // json
                println!("{status:#?}"); // rust debug
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
