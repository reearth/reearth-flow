use futures_util::StreamExt;
use tokio::task::JoinHandle;
use tracing::{debug, error, warn};
use yrs::updates::decoder::Decode;
use yrs::Transact;
use yrs::Update;

use crate::AwarenessRef;
use tokio::sync::mpsc::Sender;

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
            loop {
                match redis::Client::open(redis_url.clone()) {
                    Ok(client) => match client.get_async_pubsub().await {
                        Ok(mut pubsub) => {
                            let channel = format!("yjs:updates:{}", doc_id);
                            match pubsub.subscribe(&channel).await {
                                Ok(_) => {
                                    let mut stream = pubsub.on_message();

                                    while let Some(msg) = stream.next().await {
                                        match msg.get_payload::<Vec<u8>>() {
                                            Ok(payload) => {
                                                let awareness = awareness.write().await;
                                                let mut txn = awareness.doc().transact_mut();

                                                if let Ok(decoded) = Update::decode_v1(&payload) {
                                                    if let Err(e) = txn.apply_update(decoded) {
                                                        warn!(
                                                            "Failed to apply update from Redis: {}",
                                                            e
                                                        );
                                                    } else {
                                                        // Drop the transaction before sending the message
                                                        drop(txn);
                                                        drop(awareness);
                                                        let _ = sender.send(payload).await;
                                                    }
                                                } else {
                                                    warn!("Failed to decode update from Redis");
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
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
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
}

impl Drop for RedisPubSub {
    fn drop(&mut self) {
        self.stop();
    }
}
