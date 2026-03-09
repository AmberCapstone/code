use clap::{Parser, ValueEnum};
use file::FlashFile;
use flash_layout::PAGE_SIZE;
use indicatif::{ProgressBar, ProgressStyle};
use proto::sensor::{
    self, Action, Command, Status,
    flash::{self, Segment},
};
use serialport::{SerialPortInfo, SerialPortType};
use std::{
    path::PathBuf,
    sync::{atomic::Ordering, mpsc},
    thread,
    time::{Duration, SystemTime},
};

mod file;

const PAD_BYTE: u8 = 0x00;
const RESEND_TIMEOUT: Duration = Duration::from_millis(1000);

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    segment: StrSegment,
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let file = FlashFile::new(&args.file, args.segment.into(), PAD_BYTE)?;

    let amber_ports: Vec<SerialPortInfo> = serialport::available_ports()?
        .into_iter()
        .filter(|p| {
            matches!(&p.port_type,
            SerialPortType::UsbPort(info) if
                info.manufacturer.as_deref() == Some("amber")
                    && info.product.as_deref() == Some( "Sensor Board")
            )
        })
        .collect();

    let Some(port_info) = amber_ports.first() else {
        return Err("No amber Sensor Boards detected".into());
    };

    let port = serialport::new(port_info.port_name.clone(), 9600).timeout(Duration::from_millis(1000));

    let (outgoing, outgoing_rx) = mpsc::channel::<Command>();
    let (incoming_tx, incoming) = mpsc::channel::<Status>();

    let ser = serial::Connection::new(outgoing_rx, incoming_tx).with_tx_interval(Duration::from_millis(10));
    let stop_signal = ser.get_stop_signal();

    let (j1, j2) = ser.start(port)?;

    fn wait_until(incoming: &mpsc::Receiver<Status>, done: impl Fn(Status) -> bool) {
        loop {
            let sts = incoming.recv().unwrap();
            if done(sts) {
                break;
            }
        }
    }

    {
        let mut cmd = Command::default();
        cmd.set_action(Action::Reset);
        outgoing.send(cmd).unwrap();

        wait_until(&incoming, |s| {
            s.state() == sensor::State::Idle && s.flash_status.is_some_and(|f| f.state() == flash::State::Idle)
        });
    }

    {
        let mut cmd = Command::default();
        cmd.set_action(Action::Flash);
        outgoing.send(cmd).unwrap();

        let bar = ProgressBar::new_spinner().with_message("Erasing");
        bar.enable_steady_tick(Duration::from_millis(100));

        wait_until(&incoming, |s| {
            s.state() == sensor::State::Flashing && s.flash_status.map(|f| f.state()) == Some(flash::State::Erasing)
        });

        wait_until(&incoming, |s| {
            s.flash_status.map(|f| f.state()) == Some(flash::State::Programming)
        });

        bar.finish_and_clear();
        println!("Erased")
    }

    let sty = ProgressStyle::with_template("{msg:<11} {bar:40.cyan/blue} {bytes}/{total_bytes} [{elapsed_precise}]")
        .unwrap()
        .progress_chars("##-");
    {
        let bar = ProgressBar::new(file.size() as u64)
            .with_style(sty.clone())
            .with_message("Programming");

        for page in file.pages() {
            'retry: loop {
                outgoing
                    .send(Command {
                        page: Some(page.clone()),
                        ..Default::default()
                    })
                    .unwrap();

                let send_time = SystemTime::now();

                while send_time.elapsed().unwrap() < RESEND_TIMEOUT {
                    if let Ok(Some(flash_status)) = incoming.try_recv().map(|m| m.flash_status) {
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
    }

    {
        let mut cmd = Command::default();
        cmd.set_action(Action::Readout);
        outgoing.send(cmd).unwrap();

        wait_until(&incoming, |s| s.state() == sensor::State::Readout);
    }

    let bar = ProgressBar::new(file.size() as u64)
        .with_style(sty)
        .with_message("Verifying");

    let mut errors: Vec<(u32, flash::Page)> = Vec::new();
    let mut readout_data: Vec<u8> = Vec::new();

    for tx_page in file.pages() {
        outgoing
            .send(Command {
                host_page_request: Some(tx_page.page_number()),
                ..Default::default()
            })
            .unwrap();
        'rx: loop {
            while let Some(rx_page) = incoming
                .try_recv()
                .ok()
                .and_then(|s| s.flash_status)
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

    stop_signal.store(true, Ordering::Relaxed);

    j1.join().unwrap();
    j2.join().unwrap();

    if errors.is_empty() {
        Ok(())
    } else {
        println!("Errors detected: {}", errors.len());
        println!(" Page |   CRC    | Expected ");
        println!("------+----------+----------");
        for (tx_crc, pg) in errors {
            println!(" {:>4} | {:08x} | {:08x} ", pg.page_number(), pg.crc(), tx_crc);
        }
        Err("Errors".into())
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
