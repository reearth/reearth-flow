use crate::storage::redis::RedisStore;
use anyhow::Result;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

pub struct StreamTrimmer {
    redis_store: Arc<RedisStore>,
    trim_interval: Duration,
    max_message_age_ms: u64,
    max_stream_length: u64,
    shutdown_receiver: tokio::sync::oneshot::Receiver<()>,
}

impl StreamTrimmer {
    pub fn new(
        redis_store: Arc<RedisStore>,
        trim_interval_secs: u64,
        max_message_age_ms: u64,
        max_stream_length: u64,
    ) -> (Self, tokio::sync::oneshot::Sender<()>) {
        let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel();

        let trimmer = Self {
            redis_store,
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

        info!("Stream trimmer stopped");
    }

    async fn perform_trim_cycle(&self) -> Result<()> {
        let start_time = std::time::Instant::now();

        match self
            .redis_store
            .trim_streams_comprehensive(self.max_message_age_ms, self.max_stream_length)
            .await
        {
            Ok((streams_processed, total_trimmed)) => {
                let duration = start_time.elapsed();

                if streams_processed > 0 || total_trimmed > 0 {
                    info!(
                        "Stream trimming cycle completed in {:?}: processed {} streams, trimmed {} entries",
                        duration, streams_processed, total_trimmed
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
}

pub fn spawn_stream_trimmer(
    redis_store: Arc<RedisStore>,
    trim_interval_secs: u64,
    max_message_age_ms: u64,
    max_stream_length: u64,
) -> tokio::sync::oneshot::Sender<()> {
    let (trimmer, shutdown_sender) = StreamTrimmer::new(
        redis_store,
        trim_interval_secs,
        max_message_age_ms,
        max_stream_length,
    );

    tokio::spawn(async move {
        trimmer.run().await;
    });

    shutdown_sender
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::redis::RedisConfig;

    #[tokio::test]
    async fn test_stream_trimmer_creation() {
        let config = RedisConfig {
            url: "redis://localhost:6379".to_string(),
            ttl: 3600,
            stream_trim_interval: 600,
            stream_max_message_age: 3600000,
            stream_max_length: 10000,
        };

        let redis_store = Arc::new(RedisStore::new(config).await.unwrap());
        let (trimmer, _shutdown) = StreamTrimmer::new(redis_store, 60, 3600000, 1000);

        assert_eq!(trimmer.trim_interval, Duration::from_secs(60));
        assert_eq!(trimmer.max_message_age_ms, 3600000);
        assert_eq!(trimmer.max_stream_length, 1000);
    }
}
