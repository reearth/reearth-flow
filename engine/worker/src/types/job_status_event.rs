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

static JOB_STATUS_TOPIC: Lazy<String> = Lazy::new(|| {
    env::var("FLOW_WORKER_JOB_STATUS_TOPIC")
        .ok()
        .unwrap_or("flow-job-status-topic".to_string())
});

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobStatus {
    Pending,
    Starting,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct JobStatusEvent {
    pub workflow_id: Uuid,
    pub job_id: Uuid,
    pub status: JobStatus,
    pub message: Option<String>,
    pub failed_nodes: Option<Vec<String>>,
    pub timestamp: chrono::DateTime<Utc>,
}

impl JobStatusEvent {
    pub fn new(
        workflow_id: Uuid,
        job_id: Uuid,
        status: JobStatus,
        message: Option<String>,
        failed_nodes: Option<Vec<String>>,
    ) -> Self {
        Self {
            workflow_id,
            job_id,
            status,
            message,
            failed_nodes,
            timestamp: Utc::now(),
        }
    }

    pub fn starting(workflow_id: Uuid, job_id: Uuid) -> Self {
        Self::new(workflow_id, job_id, JobStatus::Starting, None, None)
    }

    pub fn running(workflow_id: Uuid, job_id: Uuid) -> Self {
        Self::new(workflow_id, job_id, JobStatus::Running, None, None)
    }

    pub fn completed(workflow_id: Uuid, job_id: Uuid) -> Self {
        Self::new(workflow_id, job_id, JobStatus::Completed, None, None)
    }

    pub fn failed(
        workflow_id: Uuid,
        job_id: Uuid,
        message: Option<String>,
        failed_nodes: Option<Vec<String>>,
    ) -> Self {
        Self::new(
            workflow_id,
            job_id,
            JobStatus::Failed,
            message,
            failed_nodes,
        )
    }

    pub fn cancelled(workflow_id: Uuid, job_id: Uuid, message: Option<String>) -> Self {
        Self::new(workflow_id, job_id, JobStatus::Cancelled, message, None)
    }
}

impl EncodableMessage for JobStatusEvent {
    type Error = crate::errors::Error;

    fn topic(&self) -> Topic {
        Topic::new(JOB_STATUS_TOPIC.clone())
    }

    fn encode(&self) -> crate::errors::Result<ValidatedMessage<Bytes>> {
        serde_json::to_string(self)
            .map_err(crate::errors::Error::FailedToEncode)
            .map(|payload| {
                ValidatedMessage::new(uuid::Uuid::new_v4(), self.timestamp, Bytes::from(payload))
            })
    }
}
