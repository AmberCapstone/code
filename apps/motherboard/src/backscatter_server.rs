use amber_connect::codec::JsonSender;
use proto::base_station::Status;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use zeromq::{PubSocket, Socket, ZmqResult};

pub async fn run(mut backscatter_rx: mpsc::Receiver<Status>, stop: CancellationToken) -> ZmqResult<()> {
    let mut socket = PubSocket::new();
    socket.bind(amber_connect::endpoint::STATUS).await?;

    stop.run_until_cancelled(async {
        while let Some(status) = backscatter_rx.recv().await {
            if let Some(bs) = status.backscatter {
                let _ = socket.send_json(&bs).await;
            }
        }
    })
    .await;

    Ok(())
}
