use std::{collections::HashMap, sync::Arc};

use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::{client::Client, publisher::Publisher};

use crate::pubsub::{message::EncodableMessage, topic::Topic};

#[derive(thiserror::Error, Debug)]
pub enum CloudPubSubError {
    #[error("Failed to encode : {0}")]
    Encode(String),
    #[error("Failed to publish message: {0}")]
    Publish(String),
}

impl CloudPubSubError {
    pub(crate) fn encode<T: ToString>(message: T) -> Self {
        Self::Encode(message.to_string())
    }

    pub(crate) fn publish<T: ToString>(message: T) -> Self {
        Self::Publish(message.to_string())
    }
}

#[derive(Clone)]
pub struct CloudPubSub {
    pub(crate) client: Client,
    pub(crate) publishers: Arc<parking_lot::RwLock<HashMap<Topic, Arc<Publisher>>>>,
}

impl CloudPubSub {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            publishers: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl crate::pubsub::publisher::Publisher for CloudPubSub {
    type Error = CloudPubSubError;

    async fn publish<M: EncodableMessage>(&self, message: M) -> Result<(), CloudPubSubError> {
        let topic = message.topic();
        let publisher = {
            let mut publishers = self.publishers.write();
            if let Some(publisher) = publishers.get(&topic) {
                publisher.clone()
            } else {
                let pubsub_topic = self.client.topic(topic.as_ref());
                let publisher = Arc::new(pubsub_topic.new_publisher(None));
                publishers.insert(topic.clone(), publisher.clone());
                publisher
            }
        };
        let validated_data = message.encode().map_err(CloudPubSubError::encode)?;

        let message_id = validated_data.id.to_string();
        let timestamp = validated_data.timestamp;
        let data_bytes = validated_data.data;

        let pubsub_msg = PubsubMessage {
            message_id,
            publish_time: Some(std::time::SystemTime::from(timestamp).into()),
            data: data_bytes.to_vec(),
            attributes: HashMap::new(),
            ordering_key: String::new(),
        };
        let awaiter = publisher.publish(pubsub_msg).await;
        awaiter.get().await.map_err(CloudPubSubError::publish)?;
        Ok(())
    }

    async fn shutdown(&self) {
        tracing::debug!("CloudPubSub::shutdown called. Clearing publishers map.");
        let mut guard = self.publishers.write();
        if !guard.is_empty() {
            tracing::info!("Clearing {} cached publishers.", guard.len());
            guard.clear();
        } else {
            tracing::debug!("Publishers map already empty.");
        }
        tracing::info!("CloudPubSub shutdown logic finished (map cleared).");
    }
}
