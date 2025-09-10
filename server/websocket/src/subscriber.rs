use crate::api::{is_smaller_redis_id, Api};
use crate::storage::redis::RedisStore;
use anyhow::Result;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::error;

pub type SubHandler = Arc<dyn Fn(String, Vec<Bytes>) + Send + Sync>;

struct StreamSubscription {
    handlers: Vec<SubHandler>,
    id: String,
    next_id: Option<String>,
}

pub struct Subscriber {
    api: Arc<Api>,
    _redis_store: Arc<RedisStore>,
    subs: Arc<RwLock<HashMap<String, StreamSubscription>>>,
    running: Arc<Mutex<bool>>,
}

impl Subscriber {
    pub async fn new(redis_store: Arc<RedisStore>, api: Arc<Api>) -> Self {
        let subscriber = Self {
            api,
            _redis_store: redis_store,
            subs: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(Mutex::new(true)),
        };

        subscriber.start_runner().await;
        subscriber
    }

    async fn start_runner(&self) {
        let subs_clone = Arc::clone(&self.subs);
        let api_clone = Arc::clone(&self.api);
        let running_clone = Arc::clone(&self.running);

        tokio::spawn(async move {
            while *running_clone.lock().await {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                let subs_read = subs_clone.read().await;
                if subs_read.is_empty() {
                    continue;
                }

                let streams: Vec<(String, String)> = subs_read
                    .iter()
                    .map(|(stream, sub)| (stream.clone(), sub.id.clone()))
                    .collect();
                drop(subs_read);

                match Self::run_iteration(&api_clone, &subs_clone, streams).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Subscriber error: {}", e);
                    }
                }
            }
        });
    }

    async fn run_iteration(
        api: &Arc<Api>,
        subs: &Arc<RwLock<HashMap<String, StreamSubscription>>>,
        streams: Vec<(String, String)>,
    ) -> Result<()> {
        let timeout = tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            api.get_messages(streams),
        );

        let messages = match timeout.await {
            Ok(Ok(messages)) => messages,
            Ok(Err(e)) => {
                error!("Error getting messages from Redis: {}", e);
                return Err(e);
            }
            Err(_) => {
                error!("Timeout getting messages from Redis");
                return Ok(());
            }
        };

        for message_result in messages {
            let mut subs_write = subs.write().await;

            if let Some(sub) = subs_write.get_mut(&message_result.stream) {
                sub.id = message_result.last_id.clone();

                if let Some(next_id) = sub.next_id.take() {
                    sub.id = next_id;
                }

                let handlers = sub.handlers.clone();
                drop(subs_write);

                for handler in handlers {
                    for individual_message in &message_result.messages {
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            handler(
                                message_result.stream.clone(),
                                vec![individual_message.clone()],
                            );
                        }))
                        .unwrap_or_else(|_| {
                            error!("Handler panicked for stream: {}", message_result.stream);
                        });
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn subscribe<F>(&self, stream: String, handler: F) -> SubscriptionResult
    where
        F: Fn(String, Vec<Bytes>) + Send + Sync + 'static,
    {
        let handler_arc = Arc::new(handler);
        let mut subs_write = self.subs.write().await;

        let subscription = subs_write
            .entry(stream)
            .or_insert_with(|| StreamSubscription {
                handlers: Vec::new(),
                id: "0".to_string(),
                next_id: None,
            });

        let redis_id = subscription.id.clone();
        subscription.handlers.push(handler_arc);

        SubscriptionResult { redis_id }
    }

    /// Unsubscribe from a stream
    pub async fn unsubscribe<F>(&self, stream: &str, _handler: F)
    where
        F: Fn(String, Vec<Bytes>) + Send + Sync + 'static,
    {
        let mut subs_write = self.subs.write().await;

        if let Some(subscription) = subs_write.get_mut(stream) {
            subscription.handlers.clear();

            if subscription.handlers.is_empty() {
                subs_write.remove(stream);
            }
        }
    }

    pub async fn ensure_sub_id(&self, stream: &str, id: &str) {
        let mut subs_write = self.subs.write().await;

        if let Some(sub) = subs_write.get_mut(stream) {
            if is_smaller_redis_id(id, &sub.id) {
                sub.next_id = Some(id.to_string());
            }
        }
    }

    pub async fn destroy(&self) {
        let mut running = self.running.lock().await;
        *running = false;

        let mut subs_write = self.subs.write().await;
        subs_write.clear();
    }
}

#[derive(Debug)]
pub struct SubscriptionResult {
    pub redis_id: String,
}

pub async fn create_subscriber(redis_store: Arc<RedisStore>, api: Arc<Api>) -> Result<Subscriber> {
    Ok(Subscriber::new(redis_store, api).await)
}
