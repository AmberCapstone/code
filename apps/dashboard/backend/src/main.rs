use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use influx::policy::{LogPolicy, PolicyRouter};
use serde::Serialize;
use std::{
    net::SocketAddr,
    time::{Instant, SystemTime},
};
use tokio::{
    sync::mpsc,
    time::{Duration, interval},
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize, Debug)]
struct SensorData {
    battery: f32,
    state: FwState,
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

impl Power {
    fn net(&self) -> f32 {
        self.solar - (self.fpga + self.camera + self.mcu + self.antenna)
    }
}

#[derive(Debug, PartialEq, Serialize, Copy, Clone)]
enum FwState {
    Charging,
    Capture,
    Process,
    Transmit,
}

impl FwState {
    #[allow(clippy::unused_self)]
    fn solar(self) -> f32 {
        9.0
    }

    fn fpga(self) -> f32 {
        match self {
            FwState::Capture => 5.0,
            FwState::Process => 15.0,
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

#[derive(Clone)]
struct AppState {
    tx: mpsc::Sender<(SensorData, SystemTime)>,
}

async fn ws_handler(ws: WebSocketUpgrade, State(app_state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, app_state))
}

async fn handle_socket(mut socket: WebSocket, app_state: AppState) {
    const PERIOD: Duration = Duration::from_millis(50);

    tracing::info!("Client socket connected");

    let mut battery: f32 = 0.5;
    let mut fw_state = FwState::Charging;

    let mut started = Instant::now();

    let mut ticker = interval(PERIOD);

    loop {
        #[allow(clippy::enum_glob_use)]
        use FwState::*;

        ticker.tick().await;

        match fw_state {
            Charging => {
                if battery > 0.99 {
                    fw_state = Capture;
                    started = Instant::now();
                }
            }
            Capture => {
                if started.elapsed() > Duration::from_secs(1) {
                    fw_state = Process;
                    started = Instant::now();
                }
            }
            Process => {
                if started.elapsed() > Duration::from_secs(2) {
                    fw_state = Transmit;
                    started = Instant::now();
                }
            }
            Transmit => {
                if started.elapsed() > Duration::from_secs(2) {
                    fw_state = Charging;
                    started = Instant::now();
                }
            }
        }

        let power = fw_state.power();
        battery += power.net() * PERIOD.as_secs_f32() * 0.01;
        let data = SensorData {
            battery,
            state: fw_state,
            power,
        };
        tracing::debug!(data=?data);

        let msg = serde_json::to_string(&data).unwrap();
        if socket.send(Message::text(msg)).await.is_err() {
            break;
        }

        tracing::trace!("Sending measurement to Logger");
        app_state
            .tx
            .send((data, SystemTime::now()))
            .await
            .expect("send to work");
        tracing::trace!("Sent measurement to Logger");
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug,tower_http=debug,influx=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = influx::InfluxConfig {
        measurement: "mock".to_string(),
        org: "amber".to_string(),
        bucket: "capstone".to_string(),
        address: "http://localhost:8086".to_string(),
        token: Some(std::env::var("INFLUX_TOKEN").unwrap()),
        tags: Vec::new(),
    };
    let logger = influx::Logger::new(db)
        .with_flush_interval(Duration::from_secs(1))
        .with_policies(
            PolicyRouter::new()
                .rule("power.*", LogPolicy::on_change(Duration::from_millis(500)))
                .rule("state", LogPolicy::on_change(Duration::from_millis(500))),
        );

    let app_state = AppState { tx: logger.sender() };
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    let ((), r) = tokio::join!(logger.run(), axum::serve(listener, app));
    r.unwrap();
}
