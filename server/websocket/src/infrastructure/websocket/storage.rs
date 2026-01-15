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
        let overall = std::time::Instant::now();
        info!("save_snapshot: start for doc_id: {}", doc_id);

        let t = std::time::Instant::now();
        let valid_recheck = self
            .redis_store
            .check_stream_exists(doc_id)
            .await
            .unwrap_or(false);
        info!(
            "save_snapshot: check_stream_exists done for doc_id: {}, ok: {}, elapsed_ms: {}",
            doc_id,
            valid_recheck,
            t.elapsed().as_millis()
        );

        if !valid_recheck {
            return Err(anyhow::anyhow!("doc_id does not exist or no updates"));
        }

        let doc = Doc::new();
        let mut txn = doc.transact_mut();

        let gcs_doc = Doc::new();
        let mut gcs_txn = gcs_doc.transact_mut();

        let t = std::time::Instant::now();
        self.store.load_doc_v2(doc_id, &mut gcs_txn).await?;
        info!(
            "save_snapshot: load_doc_v2 done for doc_id: {}, elapsed_ms: {}",
            doc_id,
            t.elapsed().as_millis()
        );

        let mut gcs_txn = gcs_doc.transact_mut();

        info!(
            "save_snapshot: document loaded from GCS, starting Redis stream updates for doc_id: {}",
            doc_id
        );

        let t = std::time::Instant::now();
        match self.redis_store.read_all_stream_data(doc_id).await {
            Ok((updates, last_id)) => {
                let updates_count = updates.len();
                let updates_total_bytes: usize = updates.iter().map(|u| u.len()).sum();

                info!(
                    "save_snapshot: read_all_stream_data done for doc_id: {}, elapsed_ms: {}, updates_count: {}, updates_total_bytes: {}, last_stream_id: {}",
                    doc_id,
                    t.elapsed().as_millis(),
                    updates_count,
                    updates_total_bytes,
                    last_id.as_deref().unwrap_or("<none>"),
                );

                let t_apply = std::time::Instant::now();
                let mut applied = 0usize;
                let mut decode_failed = 0usize;
                let mut apply_failed = 0usize;

                for update_data in &updates {
                    match Update::decode_v1(update_data) {
                        Ok(update) => match txn.apply_update(update) {
                            Ok(_) => applied += 1,
                            Err(e) => {
                                apply_failed += 1;
                                warn!(
                                    "save_snapshot: failed to apply Redis update for doc_id: {}, err: {}",
                                    doc_id,
                                    e
                                );
                            }
                        },
                        Err(e) => {
                            decode_failed += 1;
                            warn!(
                                "save_snapshot: failed to decode Redis update for doc_id: {}, err: {}",
                                doc_id,
                                e
                            );
                        }
                    }
                }

                info!(
                    "save_snapshot: apply updates done for doc_id: {}, elapsed_ms: {}, applied: {}, decode_failed: {}, apply_failed: {}",
                    doc_id,
                    t_apply.elapsed().as_millis(),
                    applied,
                    decode_failed,
                    apply_failed
                );
            }
            Err(e) => {
                warn!(
                    "save_snapshot: Failed to read updates from Redis stream for doc_id: {}, elapsed_ms: {}, err: {}",
                    doc_id,
                    t.elapsed().as_millis(),
                    e
                );
            }
        }

        let t = std::time::Instant::now();
        let update = txn.encode_diff_v1(&StateVector::default());
        let update_len = update.len();
        info!(
            "save_snapshot: encode_diff_v1 done for doc_id: {}, elapsed_ms: {}, update_bytes: {}",
            doc_id,
            t.elapsed().as_millis(),
            update_len
        );

        drop(txn);
        let update_bytes = Bytes::from(update);

        let t = std::time::Instant::now();
        self.store
            .push_update(doc_id, &update_bytes, &self.redis_store)
            .await?;
        info!(
            "save_snapshot: push_update done for doc_id: {}, elapsed_ms: {}, pushed_bytes: {}",
            doc_id,
            t.elapsed().as_millis(),
            update_len
        );

        let t = std::time::Instant::now();
        let update = Update::decode_v1(&update_bytes)?;
        gcs_txn.apply_update(update)?;
        drop(gcs_txn);
        let gcs_txn = gcs_doc.transact();
        self.store.flush_doc_v2(doc_id, &gcs_txn).await?;
        info!(
            "save_snapshot: flush_doc_v2 done for doc_id: {}, elapsed_ms: {}",
            doc_id,
            t.elapsed().as_millis()
        );

        info!(
            "save_snapshot: finished for doc_id: {}, elapsed_ms: {}",
            doc_id,
            overall.elapsed().as_millis()
        );

        Ok(())
    }
}
