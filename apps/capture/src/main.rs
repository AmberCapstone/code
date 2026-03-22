use indicatif::{ProgressBar, ProgressStyle};
use proto::sensor::{self, Action, Command, Status, fpga};
use serialport::{SerialPortInfo, SerialPortType};
use std::{
    path::Path,
    sync::{atomic::Ordering, mpsc},
    time::Duration,
};

const NUM_LINES: u32 = 240;
const BYTE_PER_LINE: u32 = 320;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    println!("Exiting manual mode");
    outgoing
        .send({
            let mut cmd = Command::default();
            cmd.set_action(Action::Monitor);
            cmd
        })
        .unwrap();

    wait_until(&incoming, |s| s.state() != sensor::State::Manual);

    // Put into manual mode
    println!("Entering manual mode");
    outgoing
        .send({
            let mut cmd = Command::default();
            cmd.set_action(Action::Manual);
            cmd
        })
        .unwrap();

    wait_until(&incoming, |s| s.state() == sensor::State::Manual);

    println!("Starting Capture");
    outgoing
        .send(Command {
            fpga: Some({
                let mut c = fpga::Command::default();
                c.set_action(fpga::Action::Capture);
                c
            }),
            ..Default::default()
        })
        .unwrap();

    let mut img_buf: Vec<u8> = Vec::with_capacity((NUM_LINES * BYTE_PER_LINE) as usize);

    let sty = ProgressStyle::with_template("{msg:<11} {bar:40.cyan/blue} {bytes}/{total_bytes} [{elapsed_precise}]")
        .unwrap()
        .progress_chars("##-");

    let bar = ProgressBar::new((BYTE_PER_LINE * NUM_LINES).into())
        .with_style(sty.clone())
        .with_message("Reading image");

    for lineno in 0..NUM_LINES {
        'wait: loop {
            if let Some(line) = incoming.try_recv().ok().and_then(|s| s.fpga).and_then(|fp| fp.line) {
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

    println!("Exiting manual mode");
    outgoing
        .send({
            let mut cmd = Command::default();
            cmd.set_action(Action::Monitor);
            cmd
        })
        .unwrap();

    wait_until(&incoming, |s| s.state() != sensor::State::Manual);

    stop_signal.store(true, Ordering::Relaxed);

    j1.join().unwrap();
    j2.join().unwrap();

    let path = Path::new("image.png");
    let res = image::save_buffer(path, &img_buf, BYTE_PER_LINE, NUM_LINES, image::ColorType::L8);

    match res {
        Ok(()) => println!("Saved to {}", path.display()),
        Err(e) => println!("Failed to save image {e}"),
    }

    Ok(())
}

fn wait_until(incoming: &mpsc::Receiver<Status>, done: impl Fn(Status) -> bool) {
    loop {
        let sts = incoming.recv().unwrap();
        if done(sts) {
            break;
        }
    }
}
