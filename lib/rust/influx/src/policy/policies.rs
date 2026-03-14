use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
pub enum LogPolicy {
    #[default]
    EveryMeasurement,
    AfterInterval(AfterInterval),
    OnChange(OnChange),
}

impl LogPolicy {
    pub fn after_interval(interval: Duration) -> Self {
        Self::AfterInterval(AfterInterval::new(interval))
    }

    pub fn on_change(timeout: Duration) -> Self {
        Self::OnChange(OnChange::new(timeout))
    }

    pub fn should_log(&mut self, value: &serde_json::Value) -> bool {
        match self {
            Self::EveryMeasurement => true,
            Self::AfterInterval(after_interval) => after_interval.should_log(),
            Self::OnChange(on_change) => on_change.should_log(value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AfterInterval {
    interval: Duration,
    last_time: Option<Instant>,
}

impl AfterInterval {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last_time: None,
        }
    }

    fn should_log(&mut self) -> bool {
        let out_of_date = self.last_time.is_none_or(|t| t.elapsed() >= self.interval);

        if out_of_date {
            self.last_time = Some(Instant::now());
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct OnChange {
    last_value: Option<serde_json::Value>,
    timeout: Duration,
    last_time: Option<Instant>,
}

impl OnChange {
    /// It is impossible for an external viewer to differentiate between a value that stopped
    /// changing and a value which stopped arriving.
    /// Specify a timeout to ensure the value is occasionally logged as long as it continues
    /// arriving.
    pub fn new(timeout: Duration) -> Self {
        Self {
            last_value: None,
            timeout,
            last_time: None,
        }
    }

    fn should_log(&mut self, value: &serde_json::Value) -> bool {
        let changed = Some(value) != self.last_value.as_ref();
        let timed_out = self.last_time.is_none_or(|t| t.elapsed() >= self.timeout);

        if changed || timed_out {
            self.last_value = Some(value.clone());
            self.last_time = Some(Instant::now());
            true
        } else {
            false
        }
    }
}
