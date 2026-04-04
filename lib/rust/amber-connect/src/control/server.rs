use std::time::Duration;

use proto::sensor::Command;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use zeromq::{RepSocket, Socket, ZmqResult};

use super::{
    lease::{Lease, WrongToken},
    messages::{Request, Response},
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
    info!("starting control server");
    let mut lease = Lease::new(lease_lifetime);

    let mut socket = RepSocket::new();
    socket.bind(endpoint::COMMAND).await?;

    stop.run_until_cancelled(async {
        loop {
            let request = socket.recv_json::<Request<Command>>().await;

            let response: Response = match request {
                Ok(Request::Acquire { name }) => Response::Acquire(lease.acquire(&name).map(Into::into)),
                Ok(Request::Release { token }) => Response::Release(lease.withdraw(token)),
                Ok(Request::Renew { token }) => Response::Renew(lease.renew(token).map(|h| h.expiry)),
                Ok(Request::Send { token, item }) => {
                    if lease.is_owned_by(token) {
                        commands_tx.send(item).await.expect("channel is alive");
                        Response::Send(Ok(()))
                    } else {
                        Response::Send(Err(WrongToken))
                    }
                }
                Err(JsonSocketError::EmptyFrame | JsonSocketError::Serde(_)) => Response::InvalidRequest,
                Err(JsonSocketError::Socket(z)) => {
                    eprintln!("ZMQ error: {z:?}");
                    Response::InvalidRequest
                }
            };

            if let Err(e) = socket.send_json(&response).await {
                warn!(err=?e, "failed to send json response");
            }
        }
    })
    .await;

    for err in socket.close().await {
        error!(err=?err, "error while closing socket");
    }

    Ok(())
}
