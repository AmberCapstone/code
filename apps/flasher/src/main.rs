use amber_connect::{
    codec::{PbReceiver, PbSocketError},
    control,
};
use anyhow::Context;
use anyhow::anyhow;
use clap::{Parser, ValueEnum};
use file::FlashFile;
use flash_layout::PAGE_SIZE;
use indicatif::{ProgressBar, ProgressStyle};
use proto::sensor::{
    self, Action, Command, Status,
    fpga::{
        self,
        flash::{self, Segment},
    },
};
use std::{
    path::PathBuf,
    thread,
    time::{Duration, SystemTime},
};
use tokio::time::{sleep, timeout};
use zeromq::{Socket, SubSocket};

mod file;

const PAD_BYTE: u8 = 0x00;
const RESEND_TIMEOUT: Duration = Duration::from_millis(1000);
const INTERVAL: Duration = Duration::from_millis(20);

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    segment: StrSegment,
    file: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut control = control::Client::try_acquire("flasher")
        .await
        .context("Failed to acquire exclusive sensor control")?;

    let mut status_socket = SubSocket::new();
    status_socket.connect(amber_connect::endpoint::STATUS).await?;
    status_socket.subscribe("").await?;

    let file = FlashFile::new(&args.file, args.segment.into(), PAD_BYTE)?;

    let r = tokio::select! {
        r = flash(file, &mut control, &mut status_socket) => r,
        _ = tokio::signal::ctrl_c() => Err(anyhow!("Interrupted"))
    };

    let mut reset = Command::default();
    reset.set_action(Action::Monitor);

    let _ = control.send(reset).await;

    control.release().await;
    r
}

async fn flash(file: FlashFile, control: &mut control::Client, status_socket: &mut SubSocket) -> anyhow::Result<()> {
    const TIMEOUT: Duration = Duration::from_secs(1);

    println!("Resetting");
    let mut cmd = Command::default();
    cmd.set_action(Action::Monitor);
    timeout(
        Duration::from_secs(1),
        command_until(
            cmd,
            |s| s.state() != sensor::State::Manual,
            control,
            status_socket,
            TIMEOUT,
        ),
    )
    .await
    .context("Timed out trying to reset the sensor")??;

    println!("Entering manual mode");
    let mut cmd = Command::default();
    cmd.set_action(Action::Manual);
    timeout(
        TIMEOUT,
        command_until(
            cmd,
            |s| s.state() == sensor::State::Manual,
            control,
            status_socket,
            TIMEOUT,
        ),
    )
    .await
    .context("Timed out trying to enter manual mode")??;

    println!("Activating SPI Flash circuit");
    let mut fpga_cmd = fpga::Command::default();
    fpga_cmd.set_action(fpga::Action::SpiFlash);
    fpga_cmd
        .flash
        .get_or_insert_default()
        .set_action(flash::Action::Program);
    let cmd = Command {
        fpga: Some(fpga_cmd),
        ..Default::default()
    };
    command_until(
        cmd,
        |s| {
            s.fpga.as_ref().is_some_and(|fp| {
                fp.state() == fpga::State::SpiFlash
                    && fp.flash.as_ref().is_some_and(|fs| fs.state() == flash::State::Erasing)
            })
        },
        control,
        status_socket,
        TIMEOUT,
    )
    .await?;

    let bar = ProgressBar::new_spinner().with_message("Erasing");
    bar.enable_steady_tick(Duration::from_millis(100));
    wait_until(status_socket, |s| {
        s.fpga
            .as_ref()
            .and_then(|fp| fp.flash.as_ref())
            .is_some_and(|fs| fs.state() == flash::State::Programming)
    })
    .await?;
    bar.finish_and_clear();
    println!("Erased");

    let sty = ProgressStyle::with_template("{msg:<11} {bar:40.cyan/blue} {bytes}/{total_bytes} [{elapsed_precise}]")
        .unwrap()
        .progress_chars("##-");
    let bar = ProgressBar::new(file.size() as u64)
        .with_style(sty.clone())
        .with_message("Programming");

    for page in file.pages() {
        'retry: loop {
            let mut cmd = Command::default();
            cmd.fpga.get_or_insert_default().flash.get_or_insert_default().page = Some(page.clone());
            control.send(cmd).await?;

            let send_time = SystemTime::now();

            while send_time.elapsed().unwrap() < RESEND_TIMEOUT {
                if let Some(flash_status) = status_socket.recv_msg::<Status>().await?.fpga.and_then(|fp| fp.flash) {
                    if flash_status.stm_page_request.is_some_and(|rn| rn > page.page_number()) {
                        break 'retry;
                    }
                    if flash_status.state() != flash::State::Programming {
                        let is_last_page = (page.page_number() + 1) == file.num_pages() as u32;
                        if is_last_page {
                            break 'retry;
                        } else {
                            eprintln!("Sensor aborted flashing early");
                        }
                    }
                }
                thread::sleep(Duration::from_millis(2));
            }
        }

        bar.inc(PAGE_SIZE as u64);
    }

    bar.finish();

    let mut cmd = Command::default();
    cmd.fpga
        .get_or_insert_default()
        .flash
        .get_or_insert_default()
        .set_action(flash::Action::Readout);
    command_until(
        cmd,
        |s| {
            s.fpga
                .as_ref()
                .and_then(|fp| fp.flash.as_ref())
                .is_some_and(|fs| fs.state() == flash::State::Readout)
        },
        control,
        status_socket,
        TIMEOUT,
    )
    .await?;

    let bar = ProgressBar::new(file.size() as u64)
        .with_style(sty)
        .with_message("Verifying");

    let mut errors: Vec<(u32, flash::Page)> = Vec::new();
    let mut readout_data: Vec<u8> = Vec::new();

    for tx_page in file.pages() {
        let mut cmd = Command::default();
        cmd.fpga
            .get_or_insert_default()
            .flash
            .get_or_insert_default()
            .host_page_request = Some(tx_page.page_number());
        control.send(cmd).await?;

        'rx: loop {
            while let Some(rx_page) = status_socket
                .recv_msg::<Status>()
                .await?
                .fpga
                .and_then(|f| f.flash)
                .and_then(|f| f.readout_page)
            {
                if rx_page.page_number() == tx_page.page_number() {
                    readout_data.extend(rx_page.data());

                    if rx_page.crc != tx_page.crc {
                        errors.push((tx_page.crc(), rx_page));
                    }
                    bar.inc(PAGE_SIZE as u64);
                    break 'rx;
                }
            }
            thread::sleep(Duration::from_millis(2));
        }
    }

    bar.finish();

    println!("Exiting manual mode");
    let mut cmd = Command::default();
    cmd.set_action(Action::Monitor);
    command_until(
        cmd,
        |s| s.state() != sensor::State::Manual,
        control,
        status_socket,
        TIMEOUT,
    )
    .await?;

    if errors.is_empty() {
        Ok(())
    } else {
        println!("Errors detected: {}", errors.len());
        println!(" Page |   CRC    | Expected ");
        println!("------+----------+----------");
        for (tx_crc, pg) in errors {
            println!(" {:>4} | {:08x} | {:08x} ", pg.page_number(), pg.crc(), tx_crc);
        }
        Err(anyhow!("Faulty write!"))
    }
}

async fn command_until(
    cmd: Command,
    condition: impl Fn(&Status) -> bool,
    control: &mut control::Client,
    status_socket: &mut SubSocket,
    timeout: Duration,
) -> anyhow::Result<()> {
    tokio::select! {
        r = control.send_continuous(cmd, INTERVAL) => r?,
        r = wait_until(status_socket, condition) => r.map(|_| ())?,
        _ = sleep(timeout) => {return Err(anyhow!("timed out"));}
    };
    Ok(())
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

#[derive(ValueEnum, Clone, Debug)]
enum StrSegment {
    Fpga,
    Qvga0,
    Qvga1,
    Qvga2,
    Qvga3,
    Qvga4,
    User,
}

impl From<StrSegment> for Segment {
    fn from(val: StrSegment) -> Self {
        match val {
            StrSegment::Fpga => Segment::Fpga,
            StrSegment::Qvga0 => Segment::Qvga0,
            StrSegment::Qvga1 => Segment::Qvga1,
            StrSegment::Qvga2 => Segment::Qvga2,
            StrSegment::Qvga3 => Segment::Qvga3,
            StrSegment::Qvga4 => Segment::Qvga4,
            StrSegment::User => Segment::User,
        }
    }
}
