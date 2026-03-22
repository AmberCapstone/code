use serialport::SerialPortBuilder;
use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub struct Connection<Tx: prost::Message + Default, Rx: prost::Message + Default> {
    incoming: mpsc::Sender<Rx>,
    outgoing: mpsc::Receiver<Tx>,
    tx_interval: Duration,

    stop: Arc<AtomicBool>,
}

impl<Tx, Rx> Connection<Tx, Rx>
where
    Tx: prost::Message + Default + 'static,
    Rx: prost::Message + Default + 'static,
{
    #[must_use]
    pub fn new(out_rx: mpsc::Receiver<Tx>, in_tx: mpsc::Sender<Rx>) -> Self {
        Self {
            incoming: in_tx,
            outgoing: out_rx,
            tx_interval: Duration::from_millis(100),
            stop: AtomicBool::new(false).into(),
        }
    }

    #[must_use]
    pub fn with_tx_interval(mut self, tx_interval: Duration) -> Self {
        self.tx_interval = tx_interval;
        self
    }

    #[must_use]
    pub fn get_stop_signal(&self) -> Arc<AtomicBool> {
        self.stop.clone()
    }

    pub fn start(self, ser: SerialPortBuilder) -> Result<(JoinHandle<()>, JoinHandle<()>), serialport::Error> {
        let port = ser.open()?;
        let port_tx = port.try_clone()?;
        let stop_tx = self.get_stop_signal();

        let mut reader = BufReader::new(port);
        let mut writer = BufWriter::new(port_tx);

        // Reader thread
        let j1 = thread::spawn(move || {
            while !self.stop.load(Ordering::Acquire) {
                let mut buf = Vec::new();

                if let Err(e) = reader.read_until(0x00, &mut buf) {
                    eprintln!("Read error {e:?}");
                    break;
                }

                let Ok(len) = cobs::decode_in_place(&mut buf) else {
                    continue;
                };

                if let Ok(rx) = Rx::decode(&buf[..len]) {
                    self.incoming.send(rx).unwrap();
                }
            }

            self.stop.store(true, Ordering::Relaxed);
        });

        let j2 = thread::spawn(move || {
            let mut current_msg: Vec<u8> = Vec::new();

            while !stop_tx.load(Ordering::Acquire) {
                if let Ok(v) = self.outgoing.try_recv() {
                    let proto_buf = v.encode_to_vec();
                    current_msg = cobs::encode_vec(&proto_buf);
                    current_msg.push(0x00); // must manually terminate
                }

                if let Err(e) = writer.write(&current_msg) {
                    eprintln!("Write error {e:?}");
                    break;
                }
                if let Err(e) = writer.flush() {
                    eprintln!("Flush error {e:?}");
                    break;
                }

                thread::sleep(self.tx_interval);
            }

            stop_tx.store(true, Ordering::Relaxed);
        });

        Ok((j1, j2))
    }
}
