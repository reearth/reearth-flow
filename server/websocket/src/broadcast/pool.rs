use crate::broadcast::group::BroadcastGroup;
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::RedisStore;
use crate::AwarenessRef;
use anyhow::Result;
use bytes;
use dashmap::DashMap;
use rand;
use scopeguard;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, warn};
use yrs::sync::Awareness;
use yrs::updates::decoder::Decode;
use yrs::{updates, Doc, ReadTxn, StateVector, Transact, Update};

use super::types::BroadcastConfig;

const DEFAULT_DOC_ID: &str = "01jpjfpw0qtw17kbrcdbgefakg";

#[derive(Debug, Clone)]
pub struct BroadcastGroupManager {
    store: Arc<GcsStore>,
    redis_store: Arc<RedisStore>,
    buffer_capacity: usize,
    doc_to_id_map: Arc<DashMap<String, Arc<BroadcastGroup>>>,
}

impl BroadcastGroupManager {
    pub fn new(store: Arc<GcsStore>, redis_store: Arc<RedisStore>) -> Self {
        Self {
            store,
            redis_store,
            buffer_capacity: 512,
            doc_to_id_map: Arc::new(DashMap::new()),
        }
    }

    async fn create_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        match self.doc_to_id_map.entry(doc_id.to_string()) {
            dashmap::mapref::entry::Entry::Occupied(entry) => {
                let group_clone = entry.get().clone();
                drop(entry);

                let doc_name = group_clone.get_doc_name();
                let valid = self
                    .redis_store
                    .check_stream_exists(&doc_name)
                    .await
                    .unwrap_or(false);

                if !valid {
                    tokio::time::sleep(Duration::from_millis(500)).await;

                    let valid_recheck = self
                        .redis_store
                        .check_stream_exists(&doc_name)
                        .await
                        .unwrap_or(false);

                    if !valid_recheck {
                        self.doc_to_id_map.remove(doc_id);
                    } else {
                        return Ok(group_clone);
                    }
                } else {
                    return Ok(group_clone);
                }
            }
            dashmap::mapref::entry::Entry::Vacant(_) => {}
        }

        let mut need_initial_save = false;
        let awareness: AwarenessRef = match self.store.load_doc_v2(doc_id).await {
            Ok(direct_doc) => Arc::new(tokio::sync::RwLock::new(Awareness::new(direct_doc))),
            Err(_) => {
                let doc = Doc::new();
                {
                    let mut txn = doc.transact_mut();

                    let loaded = self.store.load_doc(doc_id, &mut txn).await.unwrap_or(false);

                    if !loaded
                        && !self
                            .store
                            .load_doc(DEFAULT_DOC_ID, &mut txn)
                            .await
                            .unwrap_or(false)
                    {
                        need_initial_save = true;
                    }
                }

                Arc::new(tokio::sync::RwLock::new(Awareness::new(doc)))
            }
        };

        let mut start_id = "0".to_string();
        let batch_size = 2048;
        let mut final_last_id = "0".to_string();

        let mut lock_value: Option<String> = None;

        let awareness_guard = awareness.write().await;
        let mut txn = awareness_guard.doc().transact_mut();

        loop {
            match self
                .redis_store
                .read_stream_data_in_batches(
                    doc_id,
                    batch_size,
                    &start_id,
                    start_id == "0",
                    false,
                    &mut lock_value,
                )
                .await
            {
                Ok((updates, last_id)) => {
                    if updates.is_empty() {
                        if start_id != "0" {
                            if let Err(e) = self
                                .redis_store
                                .read_stream_data_in_batches(
                                    doc_id,
                                    1,
                                    &last_id,
                                    false,
                                    true,
                                    &mut lock_value,
                                )
                                .await
                            {
                                warn!("Failed to release lock in final batch: {}", e);
                            }
                        }
                        break;
                    }

                    for update_data in &updates {
                        if let Ok(update) = Update::decode_v1(update_data) {
                            if let Err(e) = txn.apply_update(update) {
                                warn!("Failed to apply Redis update: {}", e);
                            }
                        }
                    }

                    final_last_id = last_id.clone();

                    if last_id == start_id {
                        if let Err(e) = self
                            .redis_store
                            .read_stream_data_in_batches(
                                doc_id,
                                1,
                                &last_id,
                                false,
                                true,
                                &mut lock_value,
                            )
                            .await
                        {
                            warn!("Failed to release lock in final batch: {}", e);
                        }
                        break;
                    }

                    start_id = last_id;
                }
                Err(e) => {
                    warn!(
                        "Failed to read updates from Redis stream for document '{}': {}",
                        doc_id, e
                    );
                    break;
                }
            }
        }

        drop(txn);
        drop(awareness_guard);

        if need_initial_save {
            let doc_id_clone = doc_id.to_string();
            let store_clone = Arc::clone(&self.store);
            let awareness_clone = Arc::clone(&awareness);
            let redis_store_clone = Arc::clone(&self.redis_store);
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(1)).await;

                let awareness_guard = awareness_clone.read().await;
                let doc = awareness_guard.doc();
                let txn = doc.transact();
                let update = txn.encode_diff_v1(&StateVector::default());
                let update_bytes = bytes::Bytes::from(update);

                if let Err(e) = store_clone
                    .push_update(&doc_id_clone, &update_bytes, &redis_store_clone)
                    .await
                {
                    error!("Failed to push initial update to Redis: {}", e);
                }
            });
        }

        let group = Arc::new(
            BroadcastGroup::new(
                awareness,
                self.buffer_capacity,
                Arc::clone(&self.redis_store),
                Arc::clone(&self.store),
                BroadcastConfig {
                    storage_enabled: true,
                    doc_name: Some(doc_id.to_string()),
                },
            )
            .await?,
        );

        if final_last_id != "0" {
            let last_read_id = group.get_last_read_id();
            let mut last_id_guard = last_read_id.lock().await;
            *last_id_guard = final_last_id;
        }

        match self.doc_to_id_map.entry(doc_id.to_string()) {
            dashmap::mapref::entry::Entry::Occupied(entry) => {
                let existing_group = entry.get().clone();
                Ok(existing_group)
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                let new_group = entry.insert(Arc::clone(&group)).clone();
                Ok(new_group)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct BroadcastPool {
    manager: BroadcastGroupManager,
    cleanup_locks: Arc<DashMap<String, bool>>,
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_store: Arc<RedisStore>) -> Self {
        let manager = BroadcastGroupManager::new(store, redis_store);
        Self {
            manager,
            cleanup_locks: Arc::new(DashMap::new()),
        }
    }

    pub fn get_store(&self) -> Arc<GcsStore> {
        self.manager.store.clone()
    }

    pub async fn get_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        if let Some(group) = self.manager.doc_to_id_map.get(doc_id) {
            tracing::info!("Found group for doc_id: {}", doc_id);
            return Ok(group.clone());
        }

        let group: Arc<BroadcastGroup> = self.manager.create_group(doc_id).await?;
        Ok(group)
    }

    pub async fn flush_to_gcs(&self, doc_id: &str) -> Result<()> {
        let broadcast_group = match self.manager.doc_to_id_map.get(doc_id) {
            Some(group) => group.clone(),
            None => {
                return Ok(());
            }
        };

        let lock_id = format!("gcs:lock:{}", doc_id);
        let instance_id = format!("sync-{}", rand::random::<u64>());

        let lock_acquired = self
            .manager
            .redis_store
            .acquire_doc_lock(&lock_id, &instance_id)
            .await?;

        if lock_acquired {
            let redis_store = self.manager.redis_store.clone();
            let awareness = broadcast_group.awareness().read().await;
            let awareness_doc = awareness.doc();

            let gcs_doc = Doc::new();
            let mut gcs_txn = gcs_doc.transact_mut();

            if let Err(e) = self.manager.store.load_doc(doc_id, &mut gcs_txn).await {
                warn!("Failed to load current state from GCS: {}", e);
            }

            let gcs_state = gcs_txn.state_vector();
            let awareness_txn = awareness_doc.transact();
            let update = awareness_txn.encode_diff_v1(&gcs_state);

            if !update.is_empty() {
                let update_bytes = bytes::Bytes::from(update);
                self.manager
                    .store
                    .push_update(doc_id, &update_bytes, &self.manager.redis_store)
                    .await?;

                self.manager
                    .store
                    .flush_doc_v2(doc_id, awareness_doc)
                    .await?;
            }

            if let Err(e) = redis_store.release_doc_lock(&lock_id, &instance_id).await {
                warn!("Failed to release GCS lock: {}", e);
            }
        }

        Ok(())
    }

    pub async fn save_snapshot(&self, doc_id: &str) -> Result<()> {
        let valid_recheck = self
            .manager
            .redis_store
            .check_stream_exists(doc_id)
            .await
            .unwrap_or(false);

        if !valid_recheck {
            return Err(anyhow::anyhow!("doc_id does not exist or no updates"));
        }

        let doc = self.manager.store.load_doc_v2(doc_id).await?;
        let mut txn = doc.transact_mut();
        let mut start_id = "0".to_string();
        let batch_size = 2048;

        let mut lock_value: Option<String> = None;

        loop {
            match self
                .manager
                .redis_store
                .read_stream_data_in_batches(
                    doc_id,
                    batch_size,
                    &start_id,
                    start_id == "0",
                    false,
                    &mut lock_value,
                )
                .await
            {
                Ok((updates, last_id)) => {
                    if updates.is_empty() {
                        if start_id != "0" {
                            if let Err(e) = self
                                .manager
                                .redis_store
                                .read_stream_data_in_batches(
                                    doc_id,
                                    1,
                                    &last_id,
                                    false,
                                    true,
                                    &mut lock_value,
                                )
                                .await
                            {
                                warn!("Failed to release lock in final batch: {}", e);
                            }
                        }
                        break;
                    }

                    for update_data in &updates {
                        if let Ok(update) = Update::decode_v1(update_data) {
                            if let Err(e) = txn.apply_update(update) {
                                warn!("Failed to apply Redis update: {}", e);
                            }
                        }
                    }

                    if last_id == start_id {
                        if let Err(e) = self
                            .manager
                            .redis_store
                            .read_stream_data_in_batches(
                                doc_id,
                                1,
                                &last_id,
                                false,
                                true,
                                &mut lock_value,
                            )
                            .await
                        {
                            warn!("Failed to release lock in final batch: {}", e);
                        }
                        break;
                    }

                    start_id = last_id;
                }
                Err(e) => {
                    warn!(
                        "Failed to read updates from Redis stream for document '{}': {}",
                        doc_id, e
                    );
                    break;
                }
            }
        }

        drop(txn);
        let gcs_doc = self.manager.store.load_doc_v2(doc_id).await?;
        let gcs_txn = gcs_doc.transact_mut();

        let update = gcs_txn.encode_diff_v1(&gcs_txn.state_vector());
        let update_bytes = bytes::Bytes::from(update);
        self.manager
            .store
            .push_update(doc_id, &update_bytes, &self.manager.redis_store)
            .await?;

        self.manager.store.flush_doc_v2(doc_id, &doc).await?;

        Ok(())
    }

    pub async fn cleanup_empty_group(&self, doc_id: &str) -> Result<()> {
        if let Some(group) = self.manager.doc_to_id_map.get(doc_id) {
            if group.connection_count() > 0 {
                return Ok(());
            }
        }
        match self.cleanup_locks.entry(doc_id.to_string()) {
            dashmap::mapref::entry::Entry::Occupied(_) => {
                return Ok(());
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                entry.insert(true);
            }
        }

        let _cleanup_guard = scopeguard::guard(doc_id.to_string(), |key| {
            self.cleanup_locks.remove(&key);
        });

        let group_to_shutdown: Option<Arc<BroadcastGroup>> = {
            match self.manager.doc_to_id_map.entry(doc_id.to_string()) {
                dashmap::mapref::entry::Entry::Occupied(entry) => {
                    if entry.get().connection_count() == 0 {
                        Some(entry.remove())
                    } else {
                        None
                    }
                }
                dashmap::mapref::entry::Entry::Vacant(_) => None,
            }
        };

        if let Some(group) = group_to_shutdown {
            if let Err(e) = group.shutdown().await {
                error!("Error shutting down group for doc_id {}: {}", doc_id, e);
            }
        }

        Ok(())
    }
}
