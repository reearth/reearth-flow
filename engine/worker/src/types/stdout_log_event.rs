use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::env;
use uuid::Uuid;

use crate::pubsub::{
    message::{EncodableMessage, ValidatedMessage},
    topic::Topic,
};

#[derive(Serialize, Debug, Clone)]
pub struct StdoutLogEvent {
    #[serde(rename = "workflowId")]
    pub workflow_id: Uuid,
    #[serde(rename = "jobId")]
    pub job_id: Uuid,
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "logLevel")]
    pub level: String,
    pub message: String,
    pub target: String,
}

impl EncodableMessage for StdoutLogEvent {
    type Error = String;

    fn topic(&self) -> Topic {
        let topic_name = env::var("FLOW_WORKER_STDOUT_LOG_TOPIC")
            .unwrap_or_else(|_| "flow-worker-stdout-log-topic".to_string());
        Topic::new(topic_name)
    }

    fn encode(&self) -> Result<ValidatedMessage<Bytes>, Self::Error> {
        let data = serde_json::to_vec(self).map_err(|e| {
            eprintln!("Failed to serialize StdoutLogEvent: {e}");
            e.to_string()
        })?;
        Ok(ValidatedMessage::new(
            Uuid::new_v4(),
            self.timestamp,
            Bytes::from(data),
        ))
    }
}
