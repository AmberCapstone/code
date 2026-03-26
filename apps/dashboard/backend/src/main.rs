use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use influx::{
    LogItem,
    policy::{LogPolicy, PolicyRouter},
};
use proto::sensor::{self, Status};
use serde::Serialize;
use std::{net::SocketAddr, time::Instant};
use tokio::{
    sync::mpsc,
    time::{Duration, interval},
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize, Debug)]
struct MockSensorData {
    battery: f32,
    state: FwState,
    power_mw: PowerMw,
}

#[derive(Serialize, Debug)]
struct PowerMw {
    solar: f32,
    fpga: f32,
    camera: f32,
    mcu: f32,
    antenna: f32,
}

impl PowerMw {
    fn net(&self) -> f32 {
        self.solar - self.out()
    }
    fn out(&self) -> f32 {
        self.fpga + self.camera + self.mcu + self.antenna
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
    fn power(self) -> PowerMw {
        PowerMw {
            solar: 9.0,
            fpga: match self {
                FwState::Capture => 5.0,
                FwState::Process => 15.0,
                _ => 0.0,
            },
            camera: if self == Self::Capture { 60.0 } else { 0.0 },
            mcu: if self == Self::Transmit { 7.0 } else { 2.0 },
            antenna: if self == Self::Transmit { 5.0 } else { 0.0 },
        }
    }
}

#[derive(Clone)]
struct AppState {
    tx: mpsc::Sender<LogItem<Status>>,
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
        let data = MockSensorData {
            battery,
            state: fw_state,
            power_mw: power,
        };
        tracing::debug!(data=?data);

        let msg = serde_json::to_string(&data).unwrap();
        if socket.send(Message::text(msg)).await.is_err() {
            break;
        }

        tracing::trace!("Sending measurement to Logger");
        let log_item = LogItem::new_now(data.into(), "mock").unwrap();
        app_state.tx.send(log_item).await.expect("send to work");
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

    let (item_tx, item_rx) = mpsc::channel(50);

    let app_state = AppState { tx: item_tx };
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    let ((), r) = tokio::join!(logger.run(item_rx), axum::serve(listener, app));
    r.unwrap();
}

impl From<MockSensorData> for Status {
    fn from(data: MockSensorData) -> Self {
        let mut status = Status::default();
        status.set_state(data.state.into());
        status.fpga.get_or_insert_default().set_state(match data.state {
            FwState::Capture => sensor::fpga::State::Booting,
            FwState::Process => sensor::fpga::State::Running,
            FwState::Charging | FwState::Transmit => sensor::fpga::State::Off,
        });
        status
            .camera
            .get_or_insert_default()
            .set_state(if data.state == FwState::Capture {
                sensor::camera::State::Running
            } else {
                sensor::camera::State::Off
            });
        status.measurement = Some(data.into());

        status
    }
}

impl From<FwState> for sensor::State {
    fn from(value: FwState) -> Self {
        match value {
            FwState::Charging => Self::Charging,
            FwState::Capture | FwState::Process | FwState::Transmit => Self::Capture,
        }
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "this is only for mocking"
)]
impl From<MockSensorData> for sensor::Measurement {
    fn from(value: MockSensorData) -> Self {
        const VDD: f32 = 3.3;
        const UA_PER_MW: f32 = 1000.0 / VDD;
        Self {
            temperature_degc: 22,
            vdd_mv: (VDD * 1000.0) as u32,
            vbat_mv: (value.battery * 1000.0) as u32,
            isense_ua: (value.power_mw.out() * UA_PER_MW) as u32,
            fpga_isense_ua: (value.power_mw.fpga * UA_PER_MW) as u32,
        }
    }
}
