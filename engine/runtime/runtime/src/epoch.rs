use std::time::{SystemTime, UNIX_EPOCH};

use chrono::DateTime;

#[derive(Clone, Debug)]
pub struct EpochCommonInfo {
    pub id: u64,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SourceTime {
    millis_since_epoch: u64,
    accuracy: u64,
}

impl SourceTime {
    pub fn elapsed_millis(&self) -> Option<u64> {
        let now_duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is before 1970-01-01 00:00");
        let rounded_now_millis: u64 = now_duration
            .as_millis()
            .next_multiple_of(self.accuracy.into())
            .try_into()
            .unwrap();
        rounded_now_millis.checked_sub(self.millis_since_epoch)
    }

    pub fn from_chrono<Tz: chrono::TimeZone>(date: &DateTime<Tz>, accuracy: u64) -> Self {
        Self {
            millis_since_epoch: date
                .timestamp_millis()
                .try_into()
                .expect("Only source times after 1970-01-01 00:00 are supported"),
            accuracy,
        }
    }

    pub fn new(millis_since_epoch: u64, accuracy: u64) -> Self {
        Self {
            millis_since_epoch,
            accuracy,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Epoch {
    pub common_info: EpochCommonInfo,
    pub decision_instant: SystemTime,
    pub source_time: Option<SourceTime>,
}

impl Epoch {
    pub fn new(id: u64, decision_instant: SystemTime) -> Self {
        Self {
            common_info: EpochCommonInfo { id },
            decision_instant,
            source_time: None,
        }
    }

    pub fn with_source_time(mut self, source_time: SourceTime) -> Self {
        self.source_time = Some(source_time);
        self
    }
}

#[cfg(test)]
#[path = "epoch_test.rs"]
mod epoch_test;
