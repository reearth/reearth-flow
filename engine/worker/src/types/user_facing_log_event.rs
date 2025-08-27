use bytes::Bytes;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::env;
use uuid::Uuid;

use crate::pubsub::{
    message::{EncodableMessage, ValidatedMessage},
    topic::Topic,
};

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum UserFacingLogLevel {
    Info,
    Success,
    Error,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserFacingLogEvent {
    pub workflow_id: Uuid,
    pub job_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub level: UserFacingLogLevel,
    pub node_name: Option<String>,
    pub node_id: Option<String>,
    pub display_message: String,
}

// Cache the topic name to avoid repeated environment variable lookups
static USER_FACING_LOG_TOPIC: Lazy<String> = Lazy::new(|| {
    env::var("FLOW_WORKER_USER_FACING_LOG_TOPIC")
        .unwrap_or_else(|_| "flow-worker-user-facing-log-topic".to_string())
});

impl EncodableMessage for UserFacingLogEvent {
    type Error = String;

    fn topic(&self) -> Topic {
        Topic::new(USER_FACING_LOG_TOPIC.clone())
    }

    fn encode(&self) -> Result<ValidatedMessage<Bytes>, Self::Error> {
        let data = serde_json::to_vec(self).map_err(|e| {
            tracing::error!("Failed to serialize UserFacingLogEvent: {}", e);
            e.to_string()
        })?;
        Ok(ValidatedMessage::new(
            Uuid::new_v4(),
            self.timestamp,
            Bytes::from(data),
        ))
    }
}
