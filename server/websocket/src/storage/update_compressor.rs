use anyhow::Result;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use yrs::updates::decoder::Decode;
use yrs::Update;

pub struct UpdateCompressor {
    min_updates_to_compress: usize,
    max_buffer_size: usize,
    pending_updates: Mutex<Vec<Vec<u8>>>,
    last_compression: Mutex<Instant>,
    compression_interval: Duration,
}

impl UpdateCompressor {
    pub fn new(
        min_updates_to_compress: usize,
        max_buffer_size: usize,
        compression_interval: Duration,
    ) -> Self {
        Self {
            min_updates_to_compress,
            max_buffer_size,
            pending_updates: Mutex::new(Vec::new()),
            last_compression: Mutex::new(Instant::now()),
            compression_interval,
        }
    }

    pub async fn add_update(&self, update: Vec<u8>) -> Result<Option<Vec<u8>>> {
        let mut updates = self.pending_updates.lock().await;
        updates.push(update);

        let should_compress = updates.len() >= self.min_updates_to_compress
            || updates.iter().map(|u| u.len()).sum::<usize>() >= self.max_buffer_size;

        if should_compress {
            let mut last_compression = self.last_compression.lock().await;
            if last_compression.elapsed() >= self.compression_interval {
                let to_compress = std::mem::take(&mut *updates);
                *last_compression = Instant::now();
                drop(last_compression);
                drop(updates);

                return Ok(Some(self.compress_updates(to_compress)?));
            }
        }

        Ok(None)
    }

    pub async fn force_compress(&self) -> Result<Option<Vec<u8>>> {
        let mut updates = self.pending_updates.lock().await;

        if updates.is_empty() {
            return Ok(None);
        }

        let to_compress = std::mem::take(&mut *updates);
        *self.last_compression.lock().await = Instant::now();

        Ok(Some(self.compress_updates(to_compress)?))
    }

    fn compress_updates(&self, updates: Vec<Vec<u8>>) -> Result<Vec<u8>> {
        if updates.len() <= 1 {
            return Ok(updates.into_iter().next().unwrap_or_default());
        }

        match yrs::merge_updates_v1(&updates) {
            Ok(merged) => Ok(merged),
            Err(e) => {
                anyhow::bail!("Failed to merge updates: {}", e)
            }
        }
    }
}
