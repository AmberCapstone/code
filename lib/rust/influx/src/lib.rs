use reqwest::Client;
use std::time::SystemTime;
use std::{path::Path, time::Duration};
use tokio::sync::mpsc;

pub mod policy;
mod protocol;
mod registry;

use crate::{
    policy::{LogPolicy, PolicyRouter},
    protocol::line_protocol,
    registry::ValueRegistry,
};

pub struct InfluxConfig {
    pub org: String,
    pub bucket: String,
    pub address: String,
    pub token: Option<String>,
    pub measurement: String,
    pub tags: Vec<String>,
}

pub enum Error {}

pub struct Logger<'a, T: serde::Serialize> {
    db: InfluxConfig,
    rx: mpsc::Receiver<(T, SystemTime)>,
    tx: mpsc::Sender<(T, SystemTime)>,

    backup_file: Option<&'a Path>,
    flush_interval: Duration,
    policies: PolicyRouter,
}

impl<'a, T: serde::Serialize> Logger<'a, T> {
    #[must_use]
    pub fn new(db: InfluxConfig) -> Self {
        let (tx, rx) = mpsc::channel(50);
        Self {
            db,
            rx,
            tx,

            backup_file: None,
            flush_interval: Duration::from_secs(1),
            policies: PolicyRouter::new(),
        }
    }

    #[must_use]
    pub fn with_backup_file(mut self, backup_file: &'a Path) -> Self {
        self.backup_file = Some(backup_file);
        self
    }

    #[must_use]
    pub fn with_flush_interval(mut self, flush_interval: Duration) -> Self {
        self.flush_interval = flush_interval;
        self
    }

    #[must_use]
    pub fn with_policies(mut self, policies: PolicyRouter) -> Self {
        self.policies = policies;
        self
    }

    #[must_use]
    pub fn sender(&self) -> mpsc::Sender<(T, SystemTime)> {
        self.tx.clone()
    }

    pub fn run(mut self) -> impl Future<Output = ()> {
        tracing::info!("Running Logger");
        let (tx_line, mut rx_line) = mpsc::channel::<String>(500);

        let parser = async move {
            tracing::info!("Starting Logger Parser");
            let mut registry = ValueRegistry::new(self.policies);

            loop {
                tracing::trace!("Waiting for a measurement");
                let (measurement, time) = self.rx.recv().await.unwrap();
                let fields_to_write = registry.update(measurement);

                let line = line_protocol(&self.db.measurement, &self.db.tags, &fields_to_write, time);
                tracing::trace!(line = line, "Sending a line to the db writer");
                tx_line.send(line).await.unwrap();
                tracing::trace!("Sent a line to the db writer");
            }
        };

        let db_writer = async move {
            const MAX_BATCH_SIZE: usize = 100;

            tracing::info!("Starting Logger Writer");

            loop {
                let mut batch = Vec::<String>::new();
                let deadline = tokio::time::sleep(self.flush_interval);
                tokio::pin!(deadline);

                tracing::trace!("Batching lines");
                loop {
                    tokio::select! {
                        Some(line) = rx_line.recv() => {
                            batch.push(line);
                            if batch.len() >= MAX_BATCH_SIZE {
                                tracing::trace!("Reached max batch size");
                                break;
                            }
                        }
                        () = &mut deadline => {
                            tracing::trace!("Batching timeout");
                            break;
                        }
                    }
                }

                if !batch.is_empty() {
                    let client = Client::new();
                    let mut request = client
                        .post(format!(
                            "{}/api/v2/write?org={}&bucket={}&precision=ns",
                            self.db.address, self.db.org, self.db.bucket
                        ))
                        .header("Content-Type", "text/plain; charset=utf-8")
                        .header("Accept", "application/json");

                    if let Some(token) = &self.db.token {
                        request = request.header("Authorization", format!("Token {token}"));
                    }

                    request = request.body(batch.join("\n"));

                    tracing::debug!("Making Influx request");
                    let response = request.send().await;

                    match response {
                        Ok(r) => tracing::debug!(response=?r, "Received Influx response"),
                        Err(e) => tracing::error!(err = ?e, "Failed to write to DB"),
                    }

                    batch.clear();
                }
            }
        };

        async move {
            tokio::join!(parser, db_writer);
        }
    }
}
