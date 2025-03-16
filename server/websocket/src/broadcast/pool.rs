use crate::broadcast::group::{BroadcastConfig, BroadcastGroup};
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::RedisStore;
use crate::AwarenessRef;
use anyhow::Result;
use dashmap::DashMap;
use dashmap::DashSet;
use std::sync::Arc;
use yrs::sync::Awareness;
use yrs::{Doc, ReadTxn, StateVector, Transact};

#[derive(Clone, Debug)]
pub struct BroadcastPool {
    store: Arc<GcsStore>,
    redis_store: Option<Arc<RedisStore>>,
    groups: DashMap<String, Arc<BroadcastGroup>>,
    buffer_capacity: usize,
    docs_in_creation: DashSet<String>,
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_store: Option<Arc<RedisStore>>) -> Self {
        Self {
            store,
            redis_store,
            groups: DashMap::new(),
            buffer_capacity: 256,
            docs_in_creation: DashSet::new(),
        }
    }

    pub fn with_buffer_capacity(
        store: Arc<GcsStore>,
        redis_store: Option<Arc<RedisStore>>,
        buffer_capacity: usize,
    ) -> Self {
        Self {
            store,
            redis_store,
            groups: DashMap::new(),
            buffer_capacity,
            docs_in_creation: DashSet::new(),
        }
    }

    pub fn get_store(&self) -> Arc<GcsStore> {
        self.store.clone()
    }

    pub fn get_redis_store(&self) -> Option<Arc<RedisStore>> {
        self.redis_store.clone()
    }

    pub async fn get_or_create_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        if let Some(group) = self.groups.get(doc_id) {
            let group_clone = group.clone();
            drop(group);

            if let Some(redis_store) = &self.redis_store {
                if let Ok(has_updates) = redis_store.has_pending_updates(doc_id).await {
                    if has_updates {
                        if let Ok(updates) = redis_store.get_pending_updates(doc_id).await {
                            if !updates.is_empty() {
                                let _ = self.apply_updates_to_doc(&group_clone, updates).await;
                            }
                        }
                    }
                }
            }

            return Ok(group_clone);
        }
        if !self.docs_in_creation.insert(doc_id.to_string()) {
            for delay_ms in [1, 2, 5, 10, 20, 50, 100] {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

                if let Some(group) = self.groups.get(doc_id) {
                    let group_clone = group.clone();
                    drop(group);
                    return Ok(group_clone);
                }

                if self.docs_in_creation.insert(doc_id.to_string()) {
                    break;
                }
            }
        }

        struct CreationGuard<'a> {
            pool: &'a BroadcastPool,
            doc_id: String,
        }

        impl Drop for CreationGuard<'_> {
            fn drop(&mut self) {
                self.pool.docs_in_creation.remove(&self.doc_id);
            }
        }

        let _creation_guard = CreationGuard {
            pool: self,
            doc_id: doc_id.to_string(),
        };

        let group: Arc<BroadcastGroup>;
        let mut need_initial_save = false;

        let awareness: AwarenessRef = {
            let doc = Doc::new();
            let mut updates_from_redis = Vec::new();

            {
                let mut txn = doc.transact_mut();
                match self.store.load_doc(doc_id, &mut txn).await {
                    Ok(loaded) => {
                        if !loaded {
                            need_initial_save = true;
                        }
                    }

                    Err(e) => {
                        tracing::error!("Failed to load document '{}': {}", doc_id, e);
                        return Err(e);
                    }
                }
            }

            if let Some(redis_store) = &self.redis_store {
                updates_from_redis = redis_store.get_pending_updates(doc_id).await?;
            }

            if !updates_from_redis.is_empty() {
                let mut txn = doc.transact_mut();
                for update in &updates_from_redis {
                    if let Ok(decoded) = yrs::updates::decoder::Decode::decode_v1(update) {
                        let _ = txn.apply_update(decoded);
                    }
                }
            }

            Arc::new(tokio::sync::RwLock::new(Awareness::new(doc)))
        };

        if need_initial_save {
            let doc_id_clone = doc_id.to_string();
            let store_clone = self.store.clone();
            let awareness_clone = awareness.clone();

            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                let awareness_guard = awareness_clone.read().await;
                let doc = awareness_guard.doc();
                let txn = doc.transact();
                let update = txn.encode_state_as_update_v1(&StateVector::default());

                if let Err(e) = store_clone.push_update(&doc_id_clone, &update).await {
                    tracing::error!(
                        "Failed to save initial awareness state for document '{}' after 2s: {}",
                        doc_id_clone,
                        e
                    );
                }
            });
        }

        group = Arc::new(
            BroadcastGroup::with_storage(
                awareness,
                self.buffer_capacity,
                self.store.clone(),
                self.redis_store.clone(),
                BroadcastConfig {
                    storage_enabled: true,
                    doc_name: Some(doc_id.to_string()),
                },
            )
            .await?,
        );

        if let Some(group) = self.groups.get(doc_id) {
            return Ok(group.clone());
        }

        self.groups.insert(doc_id.to_string(), group.clone());

        Ok(group)
    }

    async fn apply_updates_to_doc(
        &self,
        group: &Arc<BroadcastGroup>,
        updates: Vec<Vec<u8>>,
    ) -> Result<()> {
        if updates.is_empty() {
            return Ok(());
        }

        let awareness = group.awareness();
        let awareness_lock = awareness.read().await;
        let doc = awareness_lock.doc();
        let mut txn = doc.transact_mut();

        for update in &updates {
            if let Ok(decoded) = yrs::updates::decoder::Decode::decode_v1(update) {
                let _ = txn.apply_update(decoded);
            }
        }

        Ok(())
    }

    pub async fn cleanup_empty_groups(&self) {
        self.groups.retain(|_, group| {
            let count = group.connection_count();
            count > 0
        });
    }

    pub async fn remove_connection(&self, doc_id: &str, instance_id: &str) {
        if let Some(group) = self.groups.get(doc_id) {
            let group_clone = group.clone();
            let remaining = group.decrement_connections();

            if remaining == 0 && group_clone.connection_count() == 0 {
                if let Some(redis_store) = &self.redis_store {
                    let mut success = false;
                    for retry in 0..3 {
                        match redis_store.release_doc_instance(doc_id, instance_id).await {
                            Ok(_) => {
                                tracing::info!(
                                    "Released document instance registration for '{}' (attempt {})",
                                    doc_id,
                                    retry + 1
                                );
                                success = true;
                                break;
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to release document instance registration for '{}' (attempt {}): {}",
                                    doc_id,
                                    retry + 1,
                                    e
                                );
                                if retry < 2 {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                            }
                        }
                    }

                    if !success {
                        let key = format!("doc:instance:{}", doc_id);
                        if let Err(e) = redis_store.expire(&key, 1).await {
                            tracing::warn!(
                                "Failed to set short TTL for document '{}': {}",
                                doc_id,
                                e
                            );
                        }
                    }
                }

                if let Err(e) = group_clone.shutdown().await {
                    tracing::warn!(
                        "Failed to shutdown broadcast group for document '{}': {}",
                        doc_id,
                        e
                    );
                }
                self.groups.remove(doc_id);
            }
        }
    }
}
