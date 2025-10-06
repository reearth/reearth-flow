use crate::application::kv::DocOps;
use crate::broadcast::group::BroadcastGroup;
use crate::infrastructure::gcs::GcsStore;
use crate::infrastructure::redis::RedisStore;
use crate::AwarenessRef;
use anyhow::Result;
use bytes;
use dashmap::{mapref::entry::Entry, DashMap};
use rand;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};
use yrs::sync::Awareness;
use yrs::updates::decoder::Decode;
use yrs::{Doc, ReadTxn, StateVector, Transact, Update};

use super::types::BroadcastConfig;

#[derive(Debug, Clone)]
pub struct BroadcastGroupManager {
    store: Arc<GcsStore>,
    redis_store: Arc<RedisStore>,
    buffer_capacity: usize,
}

impl BroadcastGroupManager {
    pub fn new(store: Arc<GcsStore>, redis_store: Arc<RedisStore>) -> Self {
        Self {
            store,
            redis_store,
            buffer_capacity: 512,
        }
    }

    async fn create_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        let doc = Doc::new();
        let mut txn = doc.transact_mut();
        self.store.load_doc_v2(doc_id, &mut txn).await?;
        drop(txn);

        let awareness: AwarenessRef = Arc::new(tokio::sync::RwLock::new(Awareness::new(doc)));

        let mut final_last_id = "0".to_string();

        let awareness_guard = awareness.write().await;
        let mut txn = awareness_guard.doc().transact_mut();

        match self.redis_store.read_all_stream_data(doc_id).await {
            Ok((updates, last_id)) => {
                for update_data in &updates {
                    if let Ok(update) = Update::decode_v1(update_data) {
                        if let Err(e) = txn.apply_update(update) {
                            warn!("Failed to apply Redis update: {}", e);
                        }
                    }
                }

                if let Some(last_id) = last_id {
                    final_last_id = last_id;
                }
            }
            Err(e) => {
                warn!(
                    "Failed to read updates from Redis stream for document '{}': {}",
                    doc_id, e
                );
            }
        }

        drop(txn);
        drop(awareness_guard);

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

        Ok(group)
    }
}

#[derive(Clone, Debug)]
pub struct BroadcastPool {
    manager: BroadcastGroupManager,
    groups: Arc<DashMap<String, Arc<BroadcastGroup>>>,
    locks: Arc<DashMap<String, Arc<Mutex<()>>>>,
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_store: Arc<RedisStore>) -> Self {
        let manager = BroadcastGroupManager::new(store, redis_store);
        Self {
            manager,
            groups: Arc::new(DashMap::new()),
            locks: Arc::new(DashMap::new()),
        }
    }

    pub fn get_store(&self) -> Arc<GcsStore> {
        self.manager.store.clone()
    }

    fn get_or_create_lock(&self, doc_id: &str) -> Arc<Mutex<()>> {
        match self.locks.entry(doc_id.to_string()) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let lock = Arc::new(Mutex::new(()));
                entry.insert(lock.clone());
                lock
            }
        }
    }

    pub async fn get_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        let lock = self.get_or_create_lock(doc_id);
        let _guard = lock.lock_owned().await;

        if let Some(group) = self.groups.get(doc_id).map(|entry| entry.clone()) {
            info!("Reusing existing BroadcastGroup for doc_id: {}", doc_id);
            return Ok(group);
        }

        info!("Creating new BroadcastGroup for doc_id: {}", doc_id);
        let group = self.manager.create_group(doc_id).await?;

        self.groups.insert(doc_id.to_string(), group.clone());
        Ok(group)
    }

    pub fn get_existing_group(&self, doc_id: &str) -> Option<Arc<BroadcastGroup>> {
        self.groups.get(doc_id).map(|group| group.clone())
    }

    pub async fn cleanup_group(&self, doc_id: &str) {
        let lock = self.get_or_create_lock(doc_id);
        let guard = lock.lock_owned().await;
        let mut should_remove_lock = false;

        if let Some(group) = self.groups.get(doc_id).map(|entry| entry.clone()) {
            let active_connections = group.get_connections_count().await;
            if active_connections == 0 {
                if let Some((_, group)) = self.groups.remove(doc_id) {
                    let _ = group.shutdown().await;
                    info!("Shutdown BroadcastGroup for doc_id: {}", doc_id);
                }
                should_remove_lock = true;
            } else {
                info!(
                    "Skipping cleanup for doc_id: {} ({} active connections)",
                    doc_id, active_connections
                );
            }
        } else {
            should_remove_lock = true;
        }

        drop(guard);

        if should_remove_lock {
            self.locks.remove(doc_id);
        }
    }

    pub fn get_cached_groups_count(&self) -> usize {
        self.groups.len()
    }

    pub async fn flush_to_gcs(&self, doc_id: &str) -> Result<()> {
        let lock_id = format!("gcs:lock:{doc_id}");
        let instance_id = format!("sync-{}", rand::random::<u64>());

        let lock_acquired = self
            .manager
            .redis_store
            .acquire_doc_lock(&lock_id, &instance_id)
            .await?;

        if lock_acquired {
            let redis_store = self.manager.redis_store.clone();

            let temp_doc = Doc::new();
            let mut temp_txn = temp_doc.transact_mut();

            if let Err(e) = self.manager.store.load_doc(doc_id, &mut temp_txn).await {
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
            if let Err(e) = self.manager.store.load_doc(doc_id, &mut gcs_txn).await {
                warn!("Failed to load current state from GCS: {}", e);
            }
            let gcs_state = gcs_txn.state_vector();
            let temp_txn_read = temp_doc.transact();
            let update = temp_txn_read.encode_diff_v1(&gcs_state);

            if !update.is_empty() {
                let update_bytes = bytes::Bytes::from(update);
                self.manager
                    .store
                    .push_update(doc_id, &update_bytes, &self.manager.redis_store)
                    .await?;

                self.manager
                    .store
                    .flush_doc_v2(doc_id, &temp_txn_read)
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

        let doc = Doc::new();
        let mut txn = doc.transact_mut();

        let gcs_doc = Doc::new();
        let mut gcs_txn = gcs_doc.transact_mut();
        self.manager.store.load_doc_v2(doc_id, &mut gcs_txn).await?;
        let mut gcs_txn = gcs_doc.transact_mut();

        info!(
            "Loaded document {} from GCS, now applying Redis stream updates",
            doc_id
        );

        match self.manager.redis_store.read_all_stream_data(doc_id).await {
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
        let update_bytes = bytes::Bytes::from(update);
        self.manager
            .store
            .push_update(doc_id, &update_bytes, &self.manager.redis_store)
            .await?;

        let update = Update::decode_v1(&update_bytes)?;
        gcs_txn.apply_update(update)?;
        drop(gcs_txn);
        let gcs_txn = gcs_doc.transact();
        self.manager.store.flush_doc_v2(doc_id, &gcs_txn).await?;
        Ok(())
    }
}