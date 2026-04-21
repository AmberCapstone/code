use amber_connect::{
    codec::{PbReceiver, PbSocketError},
    control,
};
use anyhow::{Context, anyhow};
use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use proto::{
    sensor::{
        Action, Command, Status,
        fpga::{self, CaptureSource, DataRequest},
    },
    state::State,
};
use std::{path::Path, time::Duration};
use tokio::time::timeout;
use zeromq::{Socket, SubSocket};

const NUM_LINES: u32 = 240;
const BYTE_PER_LINE: u32 = 320;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    source: StrSource,
    data: Data,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut control = control::Client::try_acquire("capture")
        .await
        .context("Failed to acquire exclusive control")?;

    let mut status_socket = SubSocket::new();
    status_socket.connect(amber_connect::endpoint::STATUS).await?;
    status_socket.subscribe("").await?;

    let r = tokio::select! {
        r = capture(args, &mut control, &mut status_socket) => r,
        _ = tokio::signal::ctrl_c() => Err(anyhow!("Interrupted"))
    };

    println!("Exiting manual mode");
    let mut cmd = Command::default();
    cmd.set_action(Action::Monitor);
    control.send(cmd).await?;

    control.release().await;

    r
}

async fn capture(args: Args, control: &mut control::Client, status_socket: &mut SubSocket) -> anyhow::Result<()> {
    const TIMEOUT: Duration = Duration::from_secs(1);

    println!("Resetting");
    let mut cmd = Command::default();
    cmd.set_action(Action::Monitor);
    control.send(cmd).await?;
    timeout(TIMEOUT, wait_until(status_socket, |s| s.state() != State::Manual))
        .await
        .context("Timed out trying to reset the sensor")??;

    println!("Entering manual mode");
    let mut cmd = Command::default();
    cmd.set_action(Action::Manual);
    control.send(cmd).await?;
    timeout(TIMEOUT, wait_until(status_socket, |s| s.state() == State::Manual))
        .await
        .context("Timed out trying to enter manual mode")??;

    println!("Starting Capture");
    let cmd = Command {
        fpga: Some({
            let mut c = fpga::Command::default();
            c.set_action(fpga::Action::Capture);
            c.set_capture_source(args.source.into());
            c.set_data_request(args.data.clone().into());
            c
        }),
        ..Default::default()
    };
    control.send(cmd).await?;

    match args.data {
        Data::Image => {
            let mut img_buf: Vec<u8> = Vec::with_capacity((NUM_LINES * BYTE_PER_LINE) as usize);

            let sty =
                ProgressStyle::with_template("{msg:<11} {bar:40.cyan/blue} {bytes}/{total_bytes} [{elapsed_precise}]")
                    .unwrap()
                    .progress_chars("##-");
            let bar = ProgressBar::new((BYTE_PER_LINE * NUM_LINES).into())
                .with_style(sty.clone())
                .with_message("Reading image");

            for lineno in 0..NUM_LINES {
                'wait: loop {
                    let resp = timeout(TIMEOUT, status_socket.recv_msg::<Status>())
                        .await
                        .context("Timed out waiting for a new status")??;
                    if let Some(line) = resp.fpga.and_then(|fp| fp.line) {
                        if line.number != lineno {
                            println!("WRONG LINE");
                            continue;
                        }
                        img_buf.extend_from_slice(&line.data);

                        break 'wait;
                    }
                }
                bar.inc(BYTE_PER_LINE.into());
            }

            bar.finish();

            let path = Path::new("image.png");
            let res = image::save_buffer(path, &img_buf, BYTE_PER_LINE, NUM_LINES, image::ColorType::L8);

            match res {
                Ok(()) => println!("Saved to {}", path.display()),
                Err(e) => println!("Failed to save image {e}"),
            }
        }
        Data::Vessels => {
            let s = timeout(
                Duration::from_secs(5),
                wait_until(status_socket, |s| {
                    s.fpga
                        .as_ref()
                        .is_some_and(|fp| fp.state() == fpga::State::DataReady && fp.vessels.is_some())
                }),
            )
            .await
            .context("Timed out waiting for vessel metrics")??;

            let vessels = s.fpga.unwrap().vessels.unwrap();

            println!("{vessels:#?}");
        }
    }

    Ok(())
}

#[derive(ValueEnum, Clone, Debug)]
enum StrSource {
    Camera,
    FakeVga,
    FakeSram,
}

#[derive(ValueEnum, Clone, Debug)]
enum Data {
    Image,
    Vessels,
}

impl From<StrSource> for CaptureSource {
    fn from(val: StrSource) -> Self {
        match val {
            StrSource::Camera => Self::Camera,
            StrSource::FakeVga => Self::FakeVga,
            StrSource::FakeSram => Self::FakeSram,
        }
    }
}

impl From<Data> for DataRequest {
    fn from(val: Data) -> Self {
        match val {
            Data::Image => DataRequest::Image,
            Data::Vessels => DataRequest::Vessels,
        }
    }
}

/// Return the first status meeting `condition`. Drop all statuses until then.
async fn wait_until<S: PbReceiver>(
    socket: &mut S,
    condition: impl Fn(&Status) -> bool,
) -> Result<Status, PbSocketError> {
    loop {
        let msg = socket.recv_msg::<Status>().await?;

        if condition(&msg) {
            return Ok(msg);
        }
    }
}
