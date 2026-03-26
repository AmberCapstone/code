use reqwest::Client;
use serde::Serialize;
use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};
use thiserror::Error;
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

pub struct Logger {
    db: InfluxConfig,

    flush_interval: Duration,
    policies: PolicyRouter,
}

pub struct LogItem<T: Serialize> {
    data: T,
    at_time: SystemTime,
    source: String,
}

#[derive(Debug, Error)]
pub enum LogItemError {
    #[error("source string must be alphanumeric to be a valid tag")]
    InvalidSource,
}

impl<T: Serialize> LogItem<T> {
    /// # Errors
    ///
    /// `source` is not alphanumeric.
    pub fn new(data: T, source: &str, at_time: SystemTime) -> Result<Self, LogItemError> {
        if source.chars().all(char::is_alphanumeric) {
            Ok(Self {
                data,
                at_time,
                source: source.to_string(),
            })
        } else {
            Err(LogItemError::InvalidSource)
        }
    }

    /// # Errors
    ///
    /// See `LogItem::new()`
    pub fn new_now(data: T, source: &str) -> Result<Self, LogItemError> {
        Self::new(data, source, SystemTime::now())
    }
}

impl Logger {
    #[must_use]
    pub fn new(db: InfluxConfig) -> Self {
        Self {
            db,

            flush_interval: Duration::from_secs(1),
            policies: PolicyRouter::new(),
        }
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

    /// # Panics
    ///
    /// Panics if the item channel closes
    pub fn run<T: Serialize>(self, mut item_rx: mpsc::Receiver<LogItem<T>>) -> impl Future<Output = ()> {
        tracing::info!("Running Logger");
        let (tx_line, mut rx_line) = mpsc::channel::<String>(500);

        let parser = async move {
            tracing::info!("Starting Logger Parser");
            let mut registries = HashMap::<String, ValueRegistry>::new(); // Separate registry per source

            loop {
                tracing::trace!("Waiting for a measurement");
                let item = item_rx.recv().await.unwrap();
                let registry = registries
                    .entry(item.source.clone())
                    .or_insert_with(|| ValueRegistry::new(self.policies.clone()));

                let fields_to_write = registry.update(item.data);

                let mut tags = self.db.tags.clone();
                tags.push(format!("source={}", item.source));

                let line = line_protocol(&self.db.measurement, &tags, &fields_to_write, item.at_time);
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
