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

static NODE_STATUS_TOPIC: Lazy<String> = Lazy::new(|| {
    env::var("FLOW_WORKER_NODE_STATUS_TOPIC")
        .ok()
        .unwrap_or("flow-node-status-topic".to_string())
});

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum NodeStatus {
    Pending,
    Starting,
    Processing,
    Idle,
    Completed,
    Failed,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NodeStatusEvent {
    pub workflow_id: Uuid,
    pub job_id: Uuid,
    pub node_id: String,
    pub status: NodeStatus,
    pub feature_id: Option<Uuid>,
    pub timestamp: chrono::DateTime<Utc>,
}

impl NodeStatusEvent {
    pub fn new(
        workflow_id: Uuid,
        job_id: Uuid,
        node_id: String,
        status: NodeStatus,
        feature_id: Option<Uuid>,
    ) -> Self {
        Self {
            workflow_id,
            job_id,
            node_id,
            status,
            feature_id,
            timestamp: Utc::now(),
        }
    }
}

impl EncodableMessage for NodeStatusEvent {
    type Error = crate::errors::Error;

    fn topic(&self) -> Topic {
        Topic::new(NODE_STATUS_TOPIC.clone())
    }

    fn encode(&self) -> crate::errors::Result<ValidatedMessage<Bytes>> {
        serde_json::to_string(self)
            .map_err(crate::errors::Error::FailedToEncode)
            .map(|payload| {
                ValidatedMessage::new(uuid::Uuid::new_v4(), self.timestamp, Bytes::from(payload))
            })
    }
}
