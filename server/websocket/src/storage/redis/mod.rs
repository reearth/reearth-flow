use anyhow::Result;
use bb8_redis::{bb8, RedisConnectionManager};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::broadcast::{channel, Sender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use uuid::Uuid;

const REDIS_STREAM_PREFIX: &str = "yjs";

type RedisStreamEntry = Vec<(String, Vec<(String, Vec<u8>)>)>;
type RedisGroupReadResult = bb8_redis::redis::RedisResult<Vec<(String, RedisStreamEntry)>>;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub ttl: u64,
}

pub type RedisPool = bb8::Pool<RedisConnectionManager>;

pub struct RedisStore {
    pub pool: RedisPool,
    subscriber_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    update_sender: Arc<Mutex<Option<Sender<Vec<u8>>>>>,
    redis_url: String,
}

impl RedisStore {
    pub async fn new(config: RedisConfig) -> Result<Self> {
        let manager = RedisConnectionManager::new(config.url.clone())?;
        let pool = bb8::Pool::builder()
            .max_size(1024)
            .min_idle(5)
            .connection_timeout(std::time::Duration::from_secs(5))
            .idle_timeout(Some(std::time::Duration::from_secs(500)))
            .max_lifetime(Some(std::time::Duration::from_secs(7200)))
            .build(manager)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create Redis pool: {}", e))?;

        Ok(Self {
            pool,
            subscriber_task: Arc::new(Mutex::new(None)),
            update_sender: Arc::new(Mutex::new(None)),
            redis_url: config.url,
        })
    }

    pub fn compute_redis_stream_name(doc_name: &str) -> String {
        format!("{}:room:{}", REDIS_STREAM_PREFIX, doc_name)
    }

    pub async fn subscribe_to_updates(&self, doc_name: &str) -> Result<()> {
        let stream_name = Self::compute_redis_stream_name(doc_name);
        let redis_url = self.redis_url.clone();

        let (tx, _) = channel(100);
        *self.update_sender.lock().await = Some(tx.clone());

        let task = tokio::spawn(async move {
            let client = bb8_redis::redis::Client::open(redis_url).unwrap();
            let mut conn = client.get_multiplexed_async_connection().await.unwrap();

            let create_result: bb8_redis::redis::RedisResult<()> = bb8_redis::redis::cmd("XGROUP")
                .arg("CREATE")
                .arg(&stream_name)
                .arg("yjs:worker")
                .arg("$")
                .arg("MKSTREAM")
                .query_async(&mut conn)
                .await;

            if let Err(e) = create_result {
                if !e.to_string().contains("BUSYGROUP") {
                    tracing::error!("Failed to create consumer group: {}", e);
                    return;
                }
            }

            let consumer_name = format!("consumer-{}", Uuid::new_v4());

            loop {
                let read_result: RedisGroupReadResult = bb8_redis::redis::cmd("XREADGROUP")
                    .arg("GROUP")
                    .arg("yjs:worker")
                    .arg(&consumer_name)
                    .arg("COUNT")
                    .arg(10)
                    .arg("BLOCK")
                    .arg(1000)
                    .arg("STREAMS")
                    .arg(&stream_name)
                    .arg(">")
                    .query_async(&mut conn)
                    .await;

                match read_result {
                    Ok(messages) => {
                        for (_, stream_messages) in messages {
                            for (id, fields) in stream_messages {
                                for (field, value) in fields {
                                    if field == "message" {
                                        if let Err(e) = tx.send(value) {
                                            tracing::warn!(
                                                "Failed to send update to subscribers: {}",
                                                e
                                            );
                                        }
                                    }
                                }

                                let _: bb8_redis::redis::RedisResult<()> =
                                    bb8_redis::redis::cmd("XACK")
                                        .arg(&stream_name)
                                        .arg("yjs:worker")
                                        .arg(id)
                                        .query_async(&mut conn)
                                        .await;
                            }
                        }
                    }
                    Err(e) => {
                        if !e.to_string().contains("NOGROUP") {
                            tracing::error!("Failed to read from Redis stream: {}", e);
                        }
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });

        *self.subscriber_task.lock().await = Some(task);

        Ok(())
    }

    pub async fn push_update(&self, doc_name: &str, update: &[u8]) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let stream_name = Self::compute_redis_stream_name(doc_name);

        let _: () = bb8_redis::redis::cmd("XADD")
            .arg(&stream_name)
            .arg("*")
            .arg("message")
            .arg(update)
            .query_async(&mut *conn)
            .await?;

        if let Some(sender) = self.update_sender.lock().await.as_ref() {
            if let Err(e) = sender.send(update.to_vec()) {
                tracing::warn!("Failed to send update to subscribers: {}", e);
            }
        }

        Ok(())
    }

    pub async fn load_updates(&self, doc_name: &str) -> Result<Vec<Vec<u8>>> {
        let mut conn = self.pool.get().await?;
        let stream_name = Self::compute_redis_stream_name(doc_name);

        let entries: RedisStreamEntry = bb8_redis::redis::cmd("XRANGE")
            .arg(&stream_name)
            .arg("-")
            .arg("+")
            .query_async(&mut *conn)
            .await?;

        let updates = entries
            .into_iter()
            .flat_map(|(_, fields)| {
                fields.into_iter().filter_map(|(field, value)| {
                    if field == "message" {
                        Some(value)
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok(updates)
    }

    pub async fn clear_updates(&self, doc_name: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let stream_name = Self::compute_redis_stream_name(doc_name);

        let _: () = bb8_redis::redis::cmd("DEL")
            .arg(&stream_name)
            .query_async(&mut *conn)
            .await?;

        Ok(())
    }
}

impl Drop for RedisStore {
    fn drop(&mut self) {
        if let Ok(mut task_guard) = self.subscriber_task.try_lock() {
            if let Some(task) = task_guard.take() {
                task.abort();
            }
        }
    }
}
