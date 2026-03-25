use std::time::Duration;

use proto::sensor::Command;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use zeromq::{RepSocket, Socket, ZmqResult};

use super::{
    lease::{Lease, WrongToken},
    messages::{AcquireResponse, Request, Response},
};
use crate::{
    codec::{JsonReceiver, JsonSender, JsonSocketError},
    endpoint,
};

/// # Errors
///
/// Fails to bind socket
///
/// # Panics
///
/// Serde failure. Channel closes
pub async fn run(
    commands_tx: mpsc::Sender<Command>,
    lease_lifetime: Duration,
    stop: CancellationToken,
) -> ZmqResult<()> {
    let mut lease = Lease::new(lease_lifetime);

    let mut socket = RepSocket::new();
    socket.bind(endpoint::COMMAND).await?;

    loop {
        tokio::select! {
            request = socket.recv_json::<Request<Command>>() => {
                let response: Response = match request {
                    Ok(Request::Acquire) => {
                        Response::Acquire(lease.acquire().map(|(token, expiry)| AcquireResponse { token, expiry }))
                    }
                    Ok(Request::Release { token }) => Response::Release(lease.withdraw(token)),
                    Ok(Request::Renew{ token }) => Response::Renew(lease.renew(token)),
                    Ok(Request::Send { token, item }) => {
                        if lease.is_owned_by(token) {
                            commands_tx.send(item).await.expect("channel is alive");
                            Response::Send(Ok(()))
                        } else {
                            Response::Send(Err(WrongToken))
                        }
                    },
                    Err(JsonSocketError::EmptyFrame | JsonSocketError::Serde(_)) => Response::InvalidRequest,
                    Err(JsonSocketError::Socket(z)) => {
                        eprintln!("ZMQ error: {z:?}");
                        Response::InvalidRequest
                    }
                };

                socket.send_json(&response).await.unwrap();
            }
            () = stop.cancelled() => { break; }
        };
    }

    Ok(())
}
