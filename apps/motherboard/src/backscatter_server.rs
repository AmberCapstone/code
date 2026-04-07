use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use proto::{backscatter, base_station::Status};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<backscatter::Status>,
}

pub async fn run(mut backscatter_rx: mpsc::Receiver<Status>, stop: CancellationToken) -> anyhow::Result<()> {
    let (tx, _rx) = broadcast::channel::<backscatter::Status>(100);

    let tx2 = tx.clone();
    let adapter = tokio::spawn(async move {
        while let Some(s) = backscatter_rx.recv().await {
            if let Some(bs) = s.backscatter
                && tx2.send(bs).is_err()
            {
                break;
            }
        }
    });

    let app_state = Arc::new(AppState { tx });
    let app = Router::new().route("/ws", get(websocket_handler)).with_state(app_state);

    let listener = tokio::net::TcpListener::bind(amber_connect::endpoint::BASESTATION)
        .await
        .unwrap();

    tokio::select! {
        _ = axum::serve(listener, app) => {},
        _ = adapter => {},
        () = stop.cancelled() => {}
    };

    Ok(())
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(mut stream: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();

    while let Ok(bs) = rx.recv().await {
        let json = serde_json::to_string(&bs).unwrap();
        if stream.send(Message::Text(json.into())).await.is_err() {
            break;
        }
    }
}
