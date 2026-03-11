use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};

use serde::Serialize;
use std::{net::SocketAddr, time::Instant};
use tokio::time::{Duration, sleep};
use tower_http::trace::TraceLayer;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize, Debug)]
struct SensorData {
    battery: f32,
    state: State,
    power: Power,
}

#[derive(Serialize, Debug)]
struct Power {
    solar: f32,
    fpga: f32,
    camera: f32,
    mcu: f32,
    antenna: f32,
}

// #[derive(Serialize, Debug)]
// enum Heading {
//     Up,
//     Down,
//     Left,
//     Right,
// }

// #[derive(Serialize, Debug)]
// struct Boat {
//     heading: Heading,
// }

// #[derive(Serialize, Debug)]
// struct Boat {}

impl Power {
    fn net(&self) -> f32 {
        self.solar - (self.fpga + self.camera + self.mcu + self.antenna)
    }
}

#[derive(Debug, PartialEq, Serialize, Copy, Clone)]
enum State {
    Charging,
    Capture,
    Process,
    Transmit,
}

impl State {
    fn solar(self) -> f32 {
        9.0
    }

    fn fpga(self) -> f32 {
        match self {
            State::Capture => 5.0,
            State::Process => 15.0,
            _ => 0.0,
        }
    }

    fn camera(self) -> f32 {
        if self == Self::Capture { 60.0 } else { 0.0 }
    }

    fn mcu(self) -> f32 {
        if self == Self::Transmit { 7.0 } else { 2.0 }
    }

    fn antenna(self) -> f32 {
        if self == Self::Transmit { 5.0 } else { 0.0 }
    }

    fn power(self) -> Power {
        Power {
            solar: self.solar(),
            fpga: self.fpga(),
            camera: self.camera(),
            mcu: self.mcu(),
            antenna: self.antenna(),
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    const PERIOD: Duration = Duration::from_millis(50);

    tracing::info!("Client socket connected");

    let mut battery: f32 = 0.5;
    let mut state = State::Charging;

    let mut started = Instant::now();

    loop {
        use State::*;
        match state {
            Charging => {
                if battery > 0.99 {
                    state = Capture;
                    started = Instant::now();
                }
            }
            Capture => {
                if started.elapsed() > Duration::from_secs(1) {
                    state = Process;
                    started = Instant::now();
                }
            }
            Process => {
                if started.elapsed() > Duration::from_secs(2) {
                    state = Transmit;
                    started = Instant::now();
                }
            }
            Transmit => {
                if started.elapsed() > Duration::from_secs(2) {
                    state = Charging;
                    started = Instant::now();
                }
            }
        }

        let power = state.power();
        battery += power.net() * PERIOD.as_secs_f32() * 0.01;
        let data = SensorData { battery, state, power };
        tracing::debug!(data=?data);

        let msg = serde_json::to_string(&data).unwrap();
        if socket.send(Message::text(msg)).await.is_err() {
            break;
        }

        sleep(PERIOD).await;
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
