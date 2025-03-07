use anyhow::Result;
use bb8_redis::{bb8, RedisConnectionManager};
use futures_util::StreamExt;
use redis::AsyncCommands;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::broadcast::{channel, Sender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

const REDIS_STREAM_PREFIX: &str = "yjs";
const REDIS_WORKER_GROUP: &str = "yjs:worker";

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub ttl: u64,
}

pub type RedisPool = bb8::Pool<RedisConnectionManager>;

pub struct RedisStore {
    pub pool: RedisPool,
    ttl: Option<usize>,
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
            ttl: Some(config.ttl as usize),
            subscriber_task: Arc::new(Mutex::new(None)),
            update_sender: Arc::new(Mutex::new(None)),
            redis_url: config.url,
        })
    }

    pub fn compute_redis_stream_name(doc_name: &str) -> String {
        format!("{}:room:{}", REDIS_STREAM_PREFIX, doc_name)
    }

    pub async fn subscribe_to_updates(&self, doc_name: &str) -> Result<()> {
        let (sender, _) = channel(1024);
        let stream_name = Self::compute_redis_stream_name(doc_name);
        let mut redis_conn = self.pool.get().await?;
        let _: redis::RedisResult<String> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(&stream_name)
            .arg(REDIS_WORKER_GROUP)
            .arg("0")
            .arg("MKSTREAM")
            .query_async(&mut *redis_conn)
            .await;

        let sender_clone = sender.clone();
        let redis_url = self.redis_url.clone();
        let doc_name = doc_name.to_string();

        let subscriber_task = tokio::spawn(async move {
            if let Ok(client) = redis::Client::open(redis_url) {
                if let Ok(mut pubsub) = client.get_async_pubsub().await {
                    let channel = format!("yjs:updates:{}", doc_name);
                    if (pubsub.subscribe(&channel).await).is_ok() {
                        let mut stream = pubsub.on_message();
                        while let Some(msg) = stream.next().await {
                            if let Ok(payload) = msg.get_payload::<Vec<u8>>() {
                                let _ = sender_clone.send(payload);
                            }
                        }
                    }
                }
            }
        });

        let mut task_guard = self.subscriber_task.lock().await;
        *task_guard = Some(subscriber_task);
        drop(task_guard);

        let mut sender_guard = self.update_sender.lock().await;
        *sender_guard = Some(sender);
        drop(sender_guard);

        Ok(())
    }

    pub async fn push_update(&self, doc_name: &str, update: &[u8]) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let cache_key = format!("doc:{}", doc_name);
        let redis_key = format!("pending_updates:{}", doc_name);
        let channel = format!("yjs:updates:{}", doc_name);
        let stream_name = Self::compute_redis_stream_name(doc_name);

        let mut pipe = redis::pipe();
        pipe.atomic()
            .cmd("SET")
            .arg(&cache_key)
            .arg(update)
            .arg("EX")
            .arg(self.ttl.unwrap_or(3600))
            .cmd("LPUSH")
            .arg(&redis_key)
            .arg(update)
            .cmd("EXPIRE")
            .arg(&redis_key)
            .arg(self.ttl.unwrap_or(3600))
            .cmd("PUBLISH")
            .arg(&channel)
            .arg(update)
            .cmd("XADD")
            .arg(&stream_name)
            .arg("*")
            .arg("message")
            .arg(update);

        let _: () = pipe.query_async(&mut *conn).await?;
        Ok(())
    }

    pub async fn load_updates(&self, doc_name: &str) -> Result<Vec<Vec<u8>>> {
        let mut conn = self.pool.get().await?;
        let redis_key = format!("pending_updates:{}", doc_name);
        let updates: Vec<Vec<u8>> = conn.lrange(&redis_key, 0, -1).await?;
        Ok(updates)
    }

    pub async fn clear_updates(&self, doc_name: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let redis_key = format!("pending_updates:{}", doc_name);
        let _: () = conn.del(&redis_key).await?;
        Ok(())
    }

    pub async fn add_to_stream(&self, doc_name: &str, update: &[u8]) -> Result<String> {
        let mut conn = self.pool.get().await?;
        let stream_name = Self::compute_redis_stream_name(doc_name);

        let channel = format!("yjs:updates:{}", doc_name);
        match conn.lpush::<_, _, ()>(&channel, update).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Failed to store update to Redis: {}", e);
                return Err(anyhow::anyhow!("Failed to store update to Redis: {}", e));
            }
        }

        let cmd_result: redis::RedisResult<String> = redis::cmd("XADD")
            .arg(&stream_name)
            .arg("*")
            .arg("message")
            .arg(update)
            .query_async(&mut *conn)
            .await;

        match cmd_result {
            Ok(id) => {
                tracing::debug!("Successfully added update to Redis stream with ID: {}", id);
                Ok(id)
            }
            Err(e) => {
                tracing::error!("Failed to add update to Redis stream: {}", e);
                Err(anyhow::anyhow!(
                    "Failed to add update to Redis stream: {}",
                    e
                ))
            }
        }
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
