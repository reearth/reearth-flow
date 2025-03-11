use futures_util::StreamExt;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use yrs::updates::decoder::Decode;
use yrs::Transact;
use yrs::Update;

use crate::AwarenessRef;
use tokio::sync::broadcast::Sender;

pub struct RedisPubSub {
    redis_url: String,
    awareness: AwarenessRef,
    sender: Sender<Vec<u8>>,
    doc_id: String,
    task: Option<JoinHandle<()>>,
}

impl RedisPubSub {
    pub fn new(
        redis_url: String,
        awareness: AwarenessRef,
        sender: Sender<Vec<u8>>,
        doc_id: String,
    ) -> Self {
        Self {
            redis_url,
            awareness,
            sender,
            doc_id,
            task: None,
        }
    }

    pub fn start(&mut self) {
        let redis_url = self.redis_url.clone();
        let awareness = self.awareness.clone();
        let sender = self.sender.clone();
        let doc_id = self.doc_id.clone();

        let task = tokio::spawn(async move {
            let mut reconnect_delay = 1;
            const MAX_RECONNECT_DELAY: u64 = 30;

            loop {
                info!("Connecting to Redis PubSub for document: {}", doc_id);

                match redis::Client::open(redis_url.clone()) {
                    Ok(client) => match client.get_async_pubsub().await {
                        Ok(mut pubsub) => {
                            let channel = format!("yjs:updates:{}", doc_id);
                            match pubsub.subscribe(&channel).await {
                                Ok(_) => {
                                    info!("Successfully subscribed to Redis channel: {}", channel);
                                    reconnect_delay = 1;

                                    let mut stream = pubsub.on_message();

                                    while let Some(msg) = stream.next().await {
                                        match msg.get_payload::<Vec<u8>>() {
                                            Ok(payload) => {
                                                info!(
                                                    "Received update from Redis for doc {}",
                                                    doc_id
                                                );
                                                let payload_clone = payload.clone();
                                                {
                                                    let awareness = awareness.write().await;
                                                    let mut txn = awareness.doc().transact_mut();

                                                    if let Ok(decoded) = Update::decode_v1(&payload)
                                                    {
                                                        if let Err(e) = txn.apply_update(decoded) {
                                                            warn!(
                                                                "Failed to apply update from Redis: {}",
                                                                e
                                                            );
                                                        } else {
                                                            debug!("Successfully applied Redis update to document");
                                                        }
                                                    } else {
                                                        warn!("Failed to decode update from Redis");
                                                    }
                                                }

                                                if let Err(e) = sender.send(payload_clone) {
                                                    warn!(
                                                        "Failed to broadcast Redis update: {}",
                                                        e
                                                    );
                                                } else {
                                                    debug!("Successfully broadcast Redis update");
                                                }
                                            }
                                            Err(e) => {
                                                error!(
                                                    "Failed to get payload from Redis message: {}",
                                                    e
                                                );
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to subscribe to Redis channel: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to get async connection to Redis: {}", e);
                        }
                    },
                    Err(e) => {
                        error!("Failed to open Redis client: {}", e);
                    }
                }

                let delay = std::cmp::min(reconnect_delay, MAX_RECONNECT_DELAY);
                warn!(
                    "Redis PubSub connection lost, reconnecting in {} seconds...",
                    delay
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                reconnect_delay = std::cmp::min(reconnect_delay * 2, MAX_RECONNECT_DELAY);
            }
        });

        self.task = Some(task);
    }

    pub fn stop(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
            debug!(
                "Stopped Redis PubSub subscription for document: {}",
                self.doc_id
            );
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.task.is_none() || self.task.as_ref().unwrap().is_finished()
    }
}

impl Drop for RedisPubSub {
    fn drop(&mut self) {
        self.stop();
    }
}
