use crate::broadcast::group::BroadcastGroup;
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::RedisStore;
use crate::AwarenessRef;
use anyhow::Result;
use bytes;
use dashmap::DashMap;
use rand;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, warn};
use yrs::sync::Awareness;
use yrs::updates::decoder::Decode;
use yrs::{Doc, ReadTxn, StateVector, Transact, Update};

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
            buffer_capacity: 128,
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
                    self.doc_to_id_map.remove(doc_id);
                } else {
                    return Ok(group_clone);
                }
            }
            dashmap::mapref::entry::Entry::Vacant(_) => {}
        }

        let mut need_initial_save = false;
        let awareness: AwarenessRef = match self.store.load_doc_direct(doc_id).await {
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

        if let Ok(updates) = self.redis_store.read_all_stream_data(doc_id).await {
            // tracing::info!("updates length: {:?}", updates.len());
            // tracing::info!("--------------------------------read_all_stream_data--------------------------------");
            if !updates.is_empty() {
                let awareness_guard = awareness.write().await;
                let mut txn = awareness_guard.doc().transact_mut();

                for update_data in &updates {
                    if let Ok(update) = Update::decode_v1(update_data) {
                        let _ = txn.apply_update(update);
                    }
                }
            }
        }

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

                let _ = store_clone
                    .push_update(&doc_id_clone, &update_bytes, &redis_store_clone)
                    .await;
            });
        }

        let group = Arc::new(
            BroadcastGroup::with_storage(
                awareness,
                self.buffer_capacity,
                Arc::clone(&self.store),
                self.redis_store.clone(),
                BroadcastConfig {
                    storage_enabled: true,
                    doc_name: Some(doc_id.to_string()),
                },
            )
            .await?,
        );

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
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_store: Arc<RedisStore>) -> Self {
        let manager = BroadcastGroupManager::new(store, redis_store);

        let doc_to_id_map = manager.doc_to_id_map.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30000));
            loop {
                interval.tick().await;

                let mut empty_groups = vec![];
                for entry in doc_to_id_map.iter() {
                    let doc_id = entry.key().clone();
                    let group = entry.value().clone();

                    if group.connection_count() == 0 {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        if group.connection_count() == 0 {
                            empty_groups.push((doc_id, group));
                        }
                    }
                }

                for (doc_id, group) in empty_groups {
                    doc_to_id_map.remove(&doc_id);
                    // tracing::info!("Shutting down empty group for doc_id: {}", doc_id);

                    if let Err(e) = group.shutdown().await {
                        warn!("Error shutting down empty group for '{}': {}", doc_id, e);
                    }
                }
            }
        });

        Self { manager }
    }

    pub fn get_store(&self) -> Arc<GcsStore> {
        self.manager.store.clone()
    }

    pub async fn get_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        if let Some(group) = self.manager.doc_to_id_map.get(doc_id) {
            // tracing::info!("Using cached broadcast group for doc_id: {}", doc_id);
            return Ok(group.clone());
        }

        // tracing::info!("Creating new broadcast group for doc_id: {}", doc_id);
        let group: Arc<BroadcastGroup> = self.manager.create_group(doc_id).await?;
        Ok(group)
    }

    pub async fn flush_to_gcs(&self, doc_id: &str) -> Result<()> {
        let broadcast_group = match self.manager.doc_to_id_map.get(doc_id) {
            Some(group) => Some(group.clone()),
            None => {
                return Ok(());
            }
        };

        if let Some(group) = broadcast_group {
            let store = self.get_store();
            let doc_name = group.get_doc_name();

            let active_connections = match self
                .manager
                .redis_store
                .get_active_instances(&doc_name, 60)
                .await
            {
                Ok(count) => count,
                Err(e) => {
                    warn!("Failed to get active instances for '{}': {}", doc_id, e);
                    0
                }
            };

            if active_connections > 0 {
                let temp_doc = Doc::new();
                let mut temp_txn = temp_doc.transact_mut();

                if let Err(e) = store.load_doc(&doc_name, &mut temp_txn).await {
                    warn!("Failed to load current GCS state for '{}': {}", doc_id, e);
                }

                let gcs_state = temp_txn.state_vector();
                drop(temp_txn);

                match self
                    .manager
                    .redis_store
                    .read_all_stream_data(&doc_name)
                    .await
                {
                    Ok(updates) if !updates.is_empty() => {
                        let awareness = group.awareness().write().await;
                        let mut txn = awareness.doc().transact_mut();

                        for update_data in &updates {
                            match Update::decode_v1(update_data) {
                                Ok(update) => {
                                    if let Err(e) = txn.apply_update(update) {
                                        warn!("Failed to apply Redis update: {}", e);
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to decode Redis update: {}", e);
                                }
                            }
                        }
                        drop(txn);
                        drop(awareness);
                    }
                    Ok(_) => {
                        debug!("No Redis updates found for document '{}'", doc_id);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to read updates from Redis stream for document '{}': {}",
                            doc_id, e
                        );
                    }
                }

                let lock_id = format!("gcs:lock:{}", doc_name);
                let instance_id = format!("sync-{}", rand::random::<u64>());

                let lock_acquired = match self
                    .manager
                    .redis_store
                    .acquire_doc_lock(&lock_id, &instance_id)
                    .await
                {
                    Ok(true) => {
                        debug!("Acquired lock for GCS flush operation on {}", doc_name);
                        Some((self.manager.redis_store.clone(), lock_id, instance_id))
                    }
                    Ok(false) => {
                        warn!("Could not acquire lock for GCS flush operation");
                        None
                    }
                    Err(e) => {
                        warn!("Error acquiring lock for GCS flush operation: {}", e);
                        None
                    }
                };

                if lock_acquired.is_some() {
                    let awareness = group.awareness().read().await;
                    let awareness_doc = awareness.doc();
                    let awareness_txn = awareness_doc.transact();
                    let redis_store_clone = Arc::clone(&self.manager.redis_store);

                    let update = awareness_txn.encode_diff_v1(&gcs_state);

                    if !update.is_empty() {
                        let update_bytes = bytes::Bytes::from(update);
                        if let Err(e) = store
                            .push_update(&doc_name, &update_bytes, &redis_store_clone)
                            .await
                        {
                            error!(
                                "Failed to flush websocket changes to GCS for '{}': {}",
                                doc_id, e
                            );
                            return Err(anyhow::anyhow!("Failed to flush changes to GCS: {}", e));
                        }
                    }

                    if let Some((redis, lock_id, instance_id)) = lock_acquired {
                        redis.release_doc_lock(&lock_id, &instance_id).await?;
                    }
                }
            }
        }

        Ok(())
    }
}
