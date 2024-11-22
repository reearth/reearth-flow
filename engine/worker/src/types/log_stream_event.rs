use std::env;

use bytes::Bytes;
use chrono::Utc;
use once_cell::sync::Lazy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::pubsub::{
    message::{EncodableMessage, ValidatedMessage},
    topic::Topic,
};

static LOG_STREAM_TOPIC: Lazy<String> = Lazy::new(|| {
    env::var("FLOW_WORKER_LOG_STREAM_TOPIC")
        .ok()
        .unwrap_or("flow-log-stream-topic".to_string())
});

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LogStreamEvent {
    pub workflow_id: Uuid,
    pub job_id: Uuid,
    pub log_level: String,
    pub node_id: Option<String>,
    pub message: String,
    pub timestamp: chrono::DateTime<Utc>,
}

impl LogStreamEvent {
    pub fn new(
        workflow_id: Uuid,
        job_id: Uuid,
        log_level: String,
        node_id: Option<String>,
        message: String,
    ) -> Self {
        Self {
            workflow_id,
            job_id,
            log_level,
            node_id,
            message,
            timestamp: chrono::Utc::now(),
        }
    }
}

impl EncodableMessage for LogStreamEvent {
    type Error = crate::errors::Error;

    fn topic(&self) -> Topic {
        Topic::new(LOG_STREAM_TOPIC.clone())
    }

    /// Encode the message payload.
    fn encode(&self) -> crate::errors::Result<ValidatedMessage<Bytes>> {
        serde_json::to_string(self)
            .map_err(crate::errors::Error::FailedToEncode)
            .map(|payload| {
                ValidatedMessage::new(uuid::Uuid::new_v4(), self.timestamp, Bytes::from(payload))
            })
    }
}
