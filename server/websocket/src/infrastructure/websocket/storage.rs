use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use rand;
use tracing::{info, warn};
use yrs::updates::decoder::Decode;
use yrs::{Doc, ReadTxn, StateVector, Transact, Update};

use crate::application::usecases::kv::DocOps;
use crate::infrastructure::gcs::GcsStore;
use crate::infrastructure::redis::RedisStore;

#[derive(Debug)]
pub struct CollaborativeStorage {
    store: Arc<GcsStore>,
    redis_store: Arc<RedisStore>,
}

impl CollaborativeStorage {
    pub fn new(store: Arc<GcsStore>, redis_store: Arc<RedisStore>) -> Self {
        Self { store, redis_store }
    }

    pub fn store(&self) -> Arc<GcsStore> {
        Arc::clone(&self.store)
    }

    pub fn redis_store(&self) -> Arc<RedisStore> {
        Arc::clone(&self.redis_store)
    }

    pub async fn flush_to_gcs(&self, doc_id: &str) -> Result<()> {
        let lock_id = format!("gcs:lock:{doc_id}");
        let instance_id = format!("sync-{}", rand::random::<u64>());

        let lock_acquired = self
            .redis_store
            .acquire_doc_lock(&lock_id, &instance_id)
            .await?;

        if lock_acquired {
            let redis_store = self.redis_store();

            let temp_doc = Doc::new();
            let mut temp_txn = temp_doc.transact_mut();

            if let Err(e) = self.store.load_doc(doc_id, &mut temp_txn).await {
                warn!("Failed to load current state from GCS: {}", e);
            }
            match redis_store.read_all_stream_data(doc_id).await {
                Ok((updates, _last_id)) => {
                    for update_data in updates {
                        if let Ok(update) = Update::decode_v1(&update_data) {
                            if let Err(e) = temp_txn.apply_update(update) {
                                warn!("Failed to apply Redis update: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read updates from Redis: {}", e);
                }
            }

            let gcs_doc = Doc::new();
            let mut gcs_txn = gcs_doc.transact_mut();
            if let Err(e) = self.store.load_doc(doc_id, &mut gcs_txn).await {
                warn!("Failed to load current state from GCS: {}", e);
            }
            let gcs_state = gcs_txn.state_vector();
            let temp_txn_read = temp_doc.transact();
            let update = temp_txn_read.encode_diff_v1(&gcs_state);

            if !update.is_empty() {
                let update_bytes = Bytes::from(update);
                self.store
                    .push_update(doc_id, &update_bytes, &self.redis_store)
                    .await?;

                self.store.flush_doc_v2(doc_id, &temp_txn_read).await?;
            }

            if let Err(e) = self
                .redis_store
                .release_doc_lock(&lock_id, &instance_id)
                .await
            {
                warn!("Failed to release GCS lock: {}", e);
            }
        }

        Ok(())
    }

    pub async fn save_snapshot(&self, doc_id: &str) -> Result<()> {
        let valid_recheck = self
            .redis_store
            .check_stream_exists(doc_id)
            .await
            .unwrap_or(false);

        if !valid_recheck {
            return Err(anyhow::anyhow!("doc_id does not exist or no updates"));
        }

        let doc = Doc::new();
        let mut txn = doc.transact_mut();

        let gcs_doc = Doc::new();
        let mut gcs_txn = gcs_doc.transact_mut();
        self.store.load_doc_v2(doc_id, &mut gcs_txn).await?;
        let mut gcs_txn = gcs_doc.transact_mut();

        info!(
            "Loaded document {} from GCS, now applying Redis stream updates",
            doc_id
        );

        match self.redis_store.read_all_stream_data(doc_id).await {
            Ok((updates, _last_id)) => {
                for update_data in &updates {
                    if let Ok(update) = Update::decode_v1(update_data) {
                        if let Err(e) = txn.apply_update(update) {
                            warn!("Failed to apply Redis update: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Failed to read updates from Redis stream for document '{}': {}",
                    doc_id, e
                );
            }
        }

        let update = txn.encode_diff_v1(&StateVector::default());
        drop(txn);
        let update_bytes = Bytes::from(update);
        self.store
            .push_update(doc_id, &update_bytes, &self.redis_store)
            .await?;

        let update = Update::decode_v1(&update_bytes)?;
        gcs_txn.apply_update(update)?;
        drop(gcs_txn);
        let gcs_txn = gcs_doc.transact();
        self.store.flush_doc_v2(doc_id, &gcs_txn).await?;
        Ok(())
    }
}
