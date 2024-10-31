use bytes::Bytes;
use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::pubsub::{
    message::{EncodableMessage, ValidatedMessage},
    topic::Topic,
};

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum EventStatus {
    InProgress,
    Completed,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EdgePassThroughEvent {
    pub workflow_id: Uuid,
    pub job_id: Uuid,
    pub status: EventStatus,
    pub timestamp: chrono::DateTime<Utc>,
    pub updated_edges: Vec<UpdatedEdge>,
}

impl EncodableMessage for EdgePassThroughEvent {
    type Error = crate::errors::Error;

    fn topic(&self) -> Topic {
        Topic::new("flow-edge-pass-through-topic")
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

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedEdge {
    pub id: String,
    pub status: EventStatus,
    pub feature_id: Option<Uuid>,
}
