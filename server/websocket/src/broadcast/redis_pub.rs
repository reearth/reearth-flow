use crate::storage::redis::RedisStore;
use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tracing::error;

const DEFAULT_CAPACITY: usize = 8;
const DEFAULT_FLUSH_INTERVAL_MS: u64 = 1;

pub struct UpdateCache {
    stream_key: String,
    redis_store: Arc<RedisStore>,
    buffer: Arc<Mutex<Vec<Bytes>>>,
    capacity: usize,
    flush_interval: Duration,
    last_flush: Arc<Mutex<Instant>>,
    flush_task: Option<JoinHandle<()>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl UpdateCache {
    pub fn new(
        doc_name: &str,
        redis_store: Arc<RedisStore>,
        capacity: Option<usize>,
        flush_interval_ms: Option<u64>,
    ) -> Self {
        let stream_key = format!("yjs:stream:{}", doc_name);
        let capacity = capacity.unwrap_or(DEFAULT_CAPACITY);
        let flush_interval =
            Duration::from_millis(flush_interval_ms.unwrap_or(DEFAULT_FLUSH_INTERVAL_MS));

        Self {
            stream_key,
            redis_store,
            buffer: Arc::new(Mutex::new(Vec::with_capacity(capacity))),
            capacity,
            flush_interval,
            last_flush: Arc::new(Mutex::new(Instant::now())),
            flush_task: None,
            shutdown_tx: None,
        }
    }

    pub fn start(&mut self) {
        let stream_key = self.stream_key.clone();
        let redis_store = self.redis_store.clone();
        let buffer = self.buffer.clone();
        let flush_interval = self.flush_interval;
        let last_flush = self.last_flush.clone();

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let task = tokio::spawn(async move {
            let mut timer = interval(flush_interval);

            loop {
                tokio::select! {
                    _ = timer.tick() => {
                        let mut last_flush_guard = last_flush.lock().await;
                        let elapsed = last_flush_guard.elapsed();

                        if elapsed >= flush_interval {
                            let mut buffer_guard = buffer.lock().await;
                            if !buffer_guard.is_empty() {
                                if let Err(e) = Self::flush_updates_internal(
                                    &stream_key,
                                    &redis_store,
                                    &mut buffer_guard,
                                )
                                .await
                                {
                                    error!("Failed to flush updates: {}", e);
                                }
                                *last_flush_guard = Instant::now();
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        let mut buffer_guard = buffer.lock().await;
                        if !buffer_guard.is_empty() {
                            if let Err(e) = Self::flush_updates_internal(
                                &stream_key,
                                &redis_store,
                                &mut buffer_guard,
                            )
                            .await
                            {
                                error!("Failed to flush updates during shutdown: {}", e);
                            }
                        }
                        break;
                    }
                }
            }
        });

        self.flush_task = Some(task);
    }

    pub async fn add_update(&self, update: Bytes) -> Result<()> {
        let mut buffer = self.buffer.lock().await;
        buffer.push(update);

        if buffer.len() >= self.capacity {
            Self::flush_updates_internal(&self.stream_key, &self.redis_store, &mut buffer).await?;

            let mut last_flush = self.last_flush.lock().await;
            *last_flush = Instant::now();
        }

        Ok(())
    }

    pub async fn flush(&self) -> Result<()> {
        let mut buffer = self.buffer.lock().await;
        if !buffer.is_empty() {
            Self::flush_updates_internal(&self.stream_key, &self.redis_store, &mut buffer).await?;

            let mut last_flush = self.last_flush.lock().await;
            *last_flush = Instant::now();
        }
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        if let Some(shutdown_tx) = &self.shutdown_tx {
            let _ = shutdown_tx.send(()).await;
        }

        self.flush().await?;

        Ok(())
    }

    async fn flush_updates_internal(
        stream_key: &str,
        redis_store: &Arc<RedisStore>,
        buffer: &mut Vec<Bytes>,
    ) -> Result<()> {
        if buffer.is_empty() {
            return Ok(());
        }

        let mut conn = redis_store.get_pool().get().await?;

        let updates: Vec<Vec<u8>> = buffer.iter().map(|b| b.to_vec()).collect();

        redis_store
            .publish_batch_updates(stream_key, &updates, &mut conn)
            .await?;

        buffer.clear();

        Ok(())
    }
}

pub async fn create_update_cache(
    doc_name: &str,
    redis_store: Arc<RedisStore>,
    capacity: Option<usize>,
    flush_interval_ms: Option<u64>,
) -> Arc<UpdateCache> {
    let mut cache = UpdateCache::new(doc_name, redis_store, capacity, flush_interval_ms);

    cache.start();
    Arc::new(cache)
}

impl Drop for UpdateCache {
    fn drop(&mut self) {
        if let Some(task) = &self.flush_task {
            task.abort();
        }
    }
}
