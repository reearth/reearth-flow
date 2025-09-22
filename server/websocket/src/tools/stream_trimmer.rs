use crate::application::kv::DocOps;
use crate::broadcast::pool::BroadcastPool;
use crate::infrastructure::gcs::GcsStore;
use crate::storage::redis::RedisStore;
use anyhow::Result;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};
use yrs::Transact;

pub struct StreamTrimmer {
    broadcast_pool: Arc<BroadcastPool>,
    redis_store: Arc<RedisStore>,
    gcs_store: Arc<GcsStore>,
    trim_interval: Duration,
    max_message_age_ms: u64,
    max_stream_length: u64,
    shutdown_receiver: tokio::sync::oneshot::Receiver<()>,
}

impl StreamTrimmer {
    pub fn new(
        broadcast_pool: Arc<BroadcastPool>,
        redis_store: Arc<RedisStore>,
        gcs_store: Arc<GcsStore>,
        trim_interval_secs: u64,
        max_message_age_ms: u64,
        max_stream_length: u64,
    ) -> (Self, tokio::sync::oneshot::Sender<()>) {
        let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel();

        let trimmer = Self {
            broadcast_pool,
            redis_store,
            gcs_store,
            trim_interval: Duration::from_secs(trim_interval_secs),
            max_message_age_ms,
            max_stream_length,
            shutdown_receiver,
        };

        (trimmer, shutdown_sender)
    }

    pub async fn run(mut self) {
        info!(
            "Starting Redis stream trimmer with interval: {}s, max age: {}ms, max length: {}",
            self.trim_interval.as_secs(),
            self.max_message_age_ms,
            self.max_stream_length
        );

        let mut interval_timer = interval(self.trim_interval);

        interval_timer.tick().await;

        loop {
            tokio::select! {
                _ = interval_timer.tick() => {
                    if let Err(e) = self.perform_trim_cycle().await {
                        error!("Stream trimming cycle failed: {}", e);
                    }
                }
                _ = &mut self.shutdown_receiver => {
                    info!("Stream trimmer received shutdown signal");
                    break;
                }
            }
        }
    }

    async fn perform_trim_cycle(&self) -> Result<()> {
        let start_time = std::time::Instant::now();

        match self.trim_streams_with_flush().await {
            Ok((streams_processed, total_trimmed, flushed_count)) => {
                let duration = start_time.elapsed();

                if streams_processed > 0 || total_trimmed > 0 {
                    info!(
                        "Stream trimming cycle completed in {:?}: processed {} streams, flushed {} docs, trimmed {} entries",
                        duration, streams_processed, flushed_count, total_trimmed
                    );
                } else {
                    tracing::debug!(
                        "Stream trimming cycle completed in {:?}: no streams needed trimming",
                        duration
                    );
                }
            }
            Err(e) => {
                warn!("Stream trimming cycle encountered errors: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    async fn flush_doc_before_trim(&self, doc_id: &str) -> Result<bool> {
        let stream_length = match self.redis_store.get_stream_length(doc_id).await {
            Ok(length) => length,
            Err(_) => {
                return Ok(false);
            }
        };

        if stream_length == 0 {
            return Ok(false);
        }

        let group = match self.broadcast_pool.get_group(doc_id).await {
            Ok(group) => group,
            Err(e) => {
                warn!("Failed to get BroadcastGroup for doc '{}': {}", doc_id, e);
                return Ok(false);
            }
        };

        let awareness_ref = group.awareness();
        let awareness_guard = awareness_ref.read().await;
        let awareness_doc = awareness_guard.doc();
        let awareness_txn = awareness_doc.transact();

        self.gcs_store.flush_doc_v2(doc_id, &awareness_txn).await?;
        info!("Flushed doc '{}' to GCS before trimming", doc_id);

        Ok(true)
    }

    async fn trim_streams_with_flush(&self) -> Result<(u64, u64, u64)> {
        let streams = self.redis_store.list_all_streams().await?;
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let cutoff_time = current_time.saturating_sub(self.max_message_age_ms);
        let min_id = format!("{cutoff_time}-0");

        let mut streams_processed = 0u64;
        let mut total_trimmed = 0u64;
        let mut flushed_count = 0u64;

        for stream_key in streams {
            if let Some(doc_id) = stream_key.strip_prefix("yjs:stream:") {
                match self.flush_doc_before_trim(doc_id).await {
                    Ok(true) => {
                        flushed_count += 1;
                        tracing::debug!("Flushed doc '{}' to GCS before trimming", doc_id);
                    }
                    Ok(false) => {}
                    Err(e) => {
                        warn!("Failed to flush doc '{}' before trimming: {}", doc_id, e);
                    }
                }

                match self
                    .redis_store
                    .trim_stream_by_min_id(doc_id, &min_id)
                    .await
                {
                    Ok(trimmed) => {
                        total_trimmed += trimmed;
                        if trimmed > 0 {
                            tracing::debug!(
                                "Trimmed {} old entries from stream '{}'",
                                trimmed,
                                doc_id
                            );
                        }
                    }
                    Err(e) => {
                        error!("Failed to trim stream '{}' by age: {}", doc_id, e);
                        continue;
                    }
                }

                match self.redis_store.get_stream_length(doc_id).await {
                    Ok(length) if length > self.max_stream_length => {
                        match self
                            .redis_store
                            .trim_stream_by_length(doc_id, self.max_stream_length)
                            .await
                        {
                            Ok(trimmed) => {
                                total_trimmed += trimmed;
                                tracing::debug!(
                                    "Trimmed {} entries from stream '{}' by length",
                                    trimmed,
                                    doc_id
                                );
                            }
                            Err(e) => {
                                error!("Failed to trim stream '{}' by length: {}", doc_id, e);
                                continue;
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        error!("Failed to get length for stream '{}': {}", doc_id, e);
                        continue;
                    }
                }

                streams_processed += 1;
            }
        }

        if streams_processed > 0 {
            info!(
                "Stream trimming with flush completed: processed {} streams, flushed {} docs, trimmed {} total entries",
                streams_processed, flushed_count, total_trimmed
            );
        }

        Ok((streams_processed, total_trimmed, flushed_count))
    }
}

pub fn spawn_stream_trimmer(
    broadcast_pool: Arc<BroadcastPool>,
    redis_store: Arc<RedisStore>,
    gcs_store: Arc<GcsStore>,
    trim_interval_secs: u64,
    max_message_age_ms: u64,
    max_stream_length: u64,
) -> tokio::sync::oneshot::Sender<()> {
    let (trimmer, shutdown_sender) = StreamTrimmer::new(
        broadcast_pool,
        redis_store,
        gcs_store,
        trim_interval_secs,
        max_message_age_ms,
        max_stream_length,
    );

    tokio::spawn(async move {
        trimmer.run().await;
    });

    shutdown_sender
}
