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

static JOB_COMPLETE_TOPIC: Lazy<String> = Lazy::new(|| {
    env::var("FLOW_WORKER_JOB_COMPLETE_TOPIC")
        .ok()
        .unwrap_or("flow-job-complete-topic".to_string())
});

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum JobResult {
    Success,
    Failed,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct JobCompleteEvent {
    pub workflow_id: Uuid,
    pub job_id: Uuid,
    pub result: JobResult,
    pub timestamp: chrono::DateTime<Utc>,
}

impl JobCompleteEvent {
    pub fn new(workflow_id: Uuid, job_id: Uuid, result: JobResult) -> Self {
        Self {
            workflow_id,
            job_id,
            result,
            timestamp: chrono::Utc::now(),
        }
    }
}

impl EncodableMessage for JobCompleteEvent {
    type Error = crate::errors::Error;

    fn topic(&self) -> Topic {
        Topic::new(JOB_COMPLETE_TOPIC.clone())
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
