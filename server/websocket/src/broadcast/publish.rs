use crate::storage::redis::RedisStore;
use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;
use std::time::{Duration, Instant};
use yrs::{updates::decoder::Decode, Doc, ReadTxn, StateVector, Transact, Update};

pub struct Publish {
    redis_store: Arc<RedisStore>,
    stream_key: String,
    count: u32,
    doc: Doc,
    last_flush: Option<Instant>,
}

impl Publish {
    pub fn new(redis_store: Arc<RedisStore>, stream_key: String) -> Self {
        Self {
            redis_store,
            stream_key,
            count: 0,
            doc: Doc::new(),
            last_flush: None,
        }
    }

    pub async fn insert(&mut self, bytes: Bytes) -> Result<()> {
        let update = Update::decode_v1(&bytes)?;

        self.doc.transact_mut().apply_update(update)?;
        self.count += 1;
        let time_since_last_flush = self
            .last_flush
            .map_or(Duration::from_secs(0), |t| t.elapsed());

        if time_since_last_flush > Duration::from_millis(20) || self.count >= 4 {
            self.flush().await?;
        }
        Ok(())
    }

    pub async fn flush(&mut self) -> Result<()> {
        let update = {
            let txn = self.doc.transact_mut();
            txn.encode_state_as_update_v1(&StateVector::default())
        };
        self.redis_store
            .publish_update(&self.stream_key, &update)
            .await?;
        self.doc = Doc::new();
        self.last_flush = Some(Instant::now());
        self.count = 0;
        Ok(())
    }
}
