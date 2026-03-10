use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};

use rand::RngExt;
use serde::Serialize;
use std::net::SocketAddr;
use tokio::time::{Duration, sleep};

#[derive(Serialize)]
struct SensorData {
    battery: f32,
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    println!("UPGRADING");
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let mut battery: f32 = 0.5;
    println!("Connected");

    loop {
        battery += rand::rng().random_range(-0.1..0.1);
        let data = SensorData { battery };

        let msg = serde_json::to_string(&data).unwrap();
        if socket.send(Message::text(msg)).await.is_err() {
            break;
        }

        sleep(Duration::from_millis(500)).await;
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ws", get(ws_handler));

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("server running on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
