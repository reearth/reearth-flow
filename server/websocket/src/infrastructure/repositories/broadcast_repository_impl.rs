use crate::domain::repositories::BroadcastRepository;
use crate::infrastructure::broadcast::pool::BroadcastPool;
use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::warn;

#[derive(Debug, Clone)]
pub struct BroadcastRepositoryImpl {
    pool: Arc<BroadcastPool>,
    subscribers: Arc<Mutex<HashMap<String, Vec<mpsc::Sender<Bytes>>>>>,
}

impl BroadcastRepositoryImpl {
    pub fn new(pool: Arc<BroadcastPool>) -> Self {
        Self {
            pool,
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl BroadcastRepository for BroadcastRepositoryImpl {
    async fn publish(&self, channel: &str, message: Bytes) -> Result<()> {
        let _group = self.pool.get_group(channel).await?;

        let subscribers = self.subscribers.lock().await;
        if let Some(channel_subscribers) = subscribers.get(channel) {
            let mut failed_senders = Vec::new();

            for (index, sender) in channel_subscribers.iter().enumerate() {
                if (sender.send(message.clone()).await).is_err() {
                    failed_senders.push(index);
                }
            }

            if !failed_senders.is_empty() {
                drop(subscribers);
                let mut subscribers = self.subscribers.lock().await;
                if let Some(channel_subscribers) = subscribers.get_mut(channel) {
                    for &index in failed_senders.iter().rev() {
                        if index < channel_subscribers.len() {
                            channel_subscribers.remove(index);
                        }
                    }

                    if channel_subscribers.is_empty() {
                        subscribers.remove(channel);
                    }
                }
            }
        }

        Ok(())
    }

    async fn subscribe(&self, channel: &str) -> Result<mpsc::Receiver<Bytes>> {
        let _group = self.pool.get_group(channel).await?;

        let (tx, rx) = mpsc::channel::<Bytes>(1000);

        let mut subscribers = self.subscribers.lock().await;
        subscribers
            .entry(channel.to_string())
            .or_insert_with(Vec::new)
            .push(tx);

        Ok(rx)
    }

    async fn unsubscribe(&self, channel: &str) -> Result<()> {
        let mut subscribers = self.subscribers.lock().await;
        subscribers.remove(channel);

        if let Err(e) = self.pool.cleanup_empty_group(channel).await {
            warn!(
                "Failed to cleanup empty group for channel {}: {}",
                channel, e
            );
        }

        Ok(())
    }

    async fn subscriber_count(&self, channel: &str) -> Result<usize> {
        let subscribers = self.subscribers.lock().await;
        let count = subscribers.get(channel).map(|subs| subs.len()).unwrap_or(0);

        Ok(count)
    }
}
