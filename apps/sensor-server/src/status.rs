use amber_connect::codec::PbSender;
use proto::sensor::Status;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use zeromq::{PubSocket, Socket, ZmqResult};

pub async fn run(mut status_rx: mpsc::Receiver<Status>, stop: CancellationToken) -> ZmqResult<()> {
    info!("starting status server");

    let mut socket = PubSocket::new();
    socket.bind(amber_connect::endpoint::STATUS).await?;

    stop.run_until_cancelled(async {
        while let Some(status) = status_rx.recv().await {
            let r = socket.send_msg(&status).await;
            if let Err(err) = r {
                warn!(err=?err, "failed to send a message to the status socket");
            }
        }
        error!("status_rx channel closed");
    })
    .await;

    for err in socket.close().await {
        error!(err=?err, "error while closing socket");
    }

    Ok(())
}
