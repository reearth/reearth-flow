use crate::broadcast::group::BroadcastGroup;
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::RedisStore;
use crate::AwarenessRef;
use anyhow::{Error, Result};
use bytes;
use dashmap::DashMap;
use deadpool::managed::{self, Manager, Metrics, RecycleResult};
use rand;
use std::sync::Arc;
use std::time::Duration;
use yrs::sync::Awareness;
use yrs::updates::decoder::Decode;
use yrs::{Doc, ReadTxn, StateVector, Transact, Update};

use super::types::{BroadcastConfig, BroadcastGroupContext};

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
        let doc_id_string = doc_id.to_string();

        match self.doc_to_id_map.entry(doc_id_string.clone()) {
            dashmap::mapref::entry::Entry::Occupied(entry) => {
                let group_clone = entry.get().clone();
                drop(entry);

                let doc_name = group_clone.get_doc_name();
                let redis_store = group_clone.get_redis_store();
                let valid = match redis_store.check_stream_exists(&doc_name).await {
                    Ok(exists) => exists,
                    Err(e) => {
                        tracing::warn!("Error checking Redis stream: {}", e);
                        false
                    }
                };

                if !valid {
                    tracing::warn!("Found cached broadcast group for '{}' but Redis stream does not exist, recreating", doc_id);
                    self.doc_to_id_map.remove(&doc_id_string);
                } else {
                    return Ok(group_clone);
                }
            }
            dashmap::mapref::entry::Entry::Vacant(_) => {}
        }

        let mut need_initial_save = false;
        let awareness: AwarenessRef = {
            let doc = Doc::new();

            {
                let mut txn = doc.transact_mut();
                let mut loaded = false;

                match self.store.load_doc(doc_id, &mut txn).await {
                    Ok(true) => {
                        loaded = true;
                    }
                    Ok(false) => match self.store.load_doc(DEFAULT_DOC_ID, &mut txn).await {
                        Ok(true) => {
                            tracing::debug!("Loaded default document '{}'", DEFAULT_DOC_ID);
                            loaded = true;
                        }
                        Ok(false) => {
                            need_initial_save = true;
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to load default document '{}': {}",
                                DEFAULT_DOC_ID,
                                e
                            );
                            need_initial_save = true;
                        }
                    },
                    Err(e) => {
                        tracing::error!("Failed to load document '{}': {}", doc_id, e);
                        return Err(e);
                    }
                }

                if !loaded {
                    need_initial_save = true;
                }
            }

            Arc::new(tokio::sync::RwLock::new(Awareness::new(doc)))
        };

        if let Ok(updates) = self.redis_store.read_all_stream_data(doc_id).await {
            if !updates.is_empty() {
                let awareness_guard = awareness.write().await;
                let mut txn = awareness_guard.doc().transact_mut();

                for update_data in &updates {
                    if let Ok(update) = Update::decode_v1(update_data) {
                        if let Err(e) = txn.apply_update(update) {
                            tracing::warn!("Failed to apply update from Redis: {}", e);
                        }
                    }
                }
            }
        }

        if need_initial_save {
            let doc_id_clone = doc_id_string.clone();
            let store_clone = Arc::clone(&self.store);
            let awareness_clone = Arc::clone(&awareness);

            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                let awareness_guard = awareness_clone.read().await;
                let doc = awareness_guard.doc();
                let txn = doc.transact();
                let update = txn.encode_diff_v1(&StateVector::default());
                let update_bytes = bytes::Bytes::from(update);

                if let Err(e) = store_clone.push_update(&doc_id_clone, &update_bytes).await {
                    tracing::error!(
                        "Failed to save initial awareness state for document '{}' after 2s: {}",
                        doc_id_clone,
                        e
                    );
                }
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
                    doc_name: Some(doc_id_string.clone()),
                },
            )
            .await?,
        );

        match self.doc_to_id_map.entry(doc_id_string) {
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

impl Manager for BroadcastGroupManager {
    type Type = BroadcastGroupContext;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let group = self.create_group(DEFAULT_DOC_ID).await?;

        Ok(BroadcastGroupContext { group })
    }

    fn recycle(
        &self,
        obj: &mut Self::Type,
        _metrics: &Metrics,
    ) -> impl std::future::Future<Output = RecycleResult<Self::Error>> + Send {
        let doc_to_id_map = self.doc_to_id_map.clone();
        let group = obj.group.clone();

        async move {
            if group.connection_count() == 0 {
                let doc_id = group.get_doc_name();
                tracing::info!("Recycling empty broadcast group for document '{}'", doc_id);

                doc_to_id_map.remove(&doc_id);

                if let Err(e) = group.shutdown().await {
                    tracing::warn!("Error shutting down empty group for '{}': {}", doc_id, e);
                    return Err(managed::RecycleError::Message(
                        format!("Failed to shutdown: {}", e).into(),
                    ));
                }

                return Err(managed::RecycleError::Message(
                    "Group has no connections".into(),
                ));
            }
            let doc_id = group.get_doc_name();
            tracing::info!("Recycling broadcast group for document '{}'", doc_id);
            Ok(())
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
            let mut interval = tokio::time::interval(Duration::from_secs(3));
            loop {
                interval.tick().await;

                let mut empty_groups = vec![];
                for entry in doc_to_id_map.iter() {
                    let doc_id = entry.key().clone();
                    let group = entry.value().clone();

                    if group.connection_count() == 0 {
                        empty_groups.push((doc_id, group));
                    }
                }

                for (doc_id, group) in empty_groups {
                    tracing::info!(
                        "Cleaning up empty broadcast group for document '{}'",
                        doc_id
                    );
                    doc_to_id_map.remove(&doc_id);

                    if let Err(e) = group.shutdown().await {
                        tracing::warn!("Error shutting down empty group for '{}': {}", doc_id, e);
                    }
                }
            }
        });

        Self { manager }
    }

    pub fn get_store(&self) -> Arc<GcsStore> {
        self.manager.store.clone()
    }

    pub fn get_redis_store(&self) -> Arc<RedisStore> {
        self.manager.redis_store.clone()
    }

    pub async fn get_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        if let Some(group) = self.manager.doc_to_id_map.get(doc_id) {
            return Ok(group.clone());
        }

        let group = self.manager.create_group(doc_id).await?;
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

            let redis_store = group.get_redis_store();
            let active_connections = match redis_store.get_active_instances(&doc_name, 60).await {
                Ok(count) => count,
                Err(e) => {
                    tracing::warn!("Failed to get active instances for '{}': {}", doc_id, e);
                    0
                }
            };

            if active_connections > 0 {
                let temp_doc = Doc::new();
                let mut temp_txn = temp_doc.transact_mut();

                if let Err(e) = store.load_doc(&doc_name, &mut temp_txn).await {
                    tracing::warn!("Failed to load current GCS state for '{}': {}", doc_id, e);
                }

                let gcs_state = temp_txn.state_vector();
                drop(temp_txn);

                match redis_store.read_all_stream_data(&doc_name).await {
                    Ok(updates) if !updates.is_empty() => {
                        tracing::info!(
                            "Found {} updates in Redis stream for '{}', applying before GCS flush",
                            updates.len(),
                            doc_id
                        );
                        let awareness = group.awareness().write().await;
                        let mut txn = awareness.doc().transact_mut();

                        for update_data in &updates {
                            match Update::decode_v1(update_data) {
                                Ok(update) => {
                                    if let Err(e) = txn.apply_update(update) {
                                        tracing::warn!("Failed to apply Redis update: {}", e);
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to decode Redis update: {}", e);
                                }
                            }
                        }
                        drop(txn);
                        drop(awareness);
                    }
                    Ok(_) => {
                        tracing::debug!("No Redis updates found for document '{}'", doc_id);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to read updates from Redis stream for document '{}': {}",
                            doc_id,
                            e
                        );
                    }
                }

                let lock_id = format!("gcs:lock:{}", doc_name);
                let instance_id = format!("sync-{}", rand::random::<u64>());

                let lock_acquired = match redis_store.acquire_doc_lock(&lock_id, &instance_id).await
                {
                    Ok(true) => {
                        tracing::debug!("Acquired lock for GCS flush operation on {}", doc_name);
                        Some((redis_store.clone(), lock_id, instance_id))
                    }
                    Ok(false) => {
                        tracing::warn!("Could not acquire lock for GCS flush operation");
                        None
                    }
                    Err(e) => {
                        tracing::warn!("Error acquiring lock for GCS flush operation: {}", e);
                        None
                    }
                };

                if lock_acquired.is_some() {
                    let awareness = group.awareness().read().await;
                    let awareness_doc = awareness.doc();
                    let awareness_txn = awareness_doc.transact();

                    let update = awareness_txn.encode_diff_v1(&gcs_state);

                    if !update.is_empty() {
                        let update_bytes = bytes::Bytes::from(update);
                        if let Err(e) = store.push_update(&doc_name, &update_bytes).await {
                            tracing::error!(
                                "Failed to flush websocket changes to GCS for '{}': {}",
                                doc_id,
                                e
                            );
                            return Err(anyhow::anyhow!("Failed to flush changes to GCS: {}", e));
                        }
                    }

                    if let Some((redis, lock_id, instance_id)) = lock_acquired {
                        if let Err(e) = redis.release_doc_lock(&lock_id, &instance_id).await {
                            tracing::warn!("Failed to release GCS lock: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
