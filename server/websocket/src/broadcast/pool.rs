use crate::broadcast::group::{BroadcastConfig, BroadcastGroup};
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::RedisStore;
use crate::AwarenessRef;
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use std::sync::Arc;
use uuid;
use yrs::sync::Awareness;
use yrs::{Doc, Transact};

#[derive(Clone, Debug)]
pub struct BroadcastPool {
    store: Arc<GcsStore>,
    redis_store: Option<Arc<RedisStore>>,
    groups: DashMap<String, Arc<BroadcastGroup>>,
    buffer_capacity: usize,
    groups_mutex: Arc<tokio::sync::Mutex<()>>,
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_store: Option<Arc<RedisStore>>) -> Self {
        Self {
            store,
            redis_store,
            groups: DashMap::new(),
            buffer_capacity: 1024,
            groups_mutex: Arc::new(tokio::sync::Mutex::new(())),
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
            groups_mutex: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    pub fn get_store(&self) -> Arc<GcsStore> {
        self.store.clone()
    }

    pub async fn get_or_create_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        if let Some(group) = self.groups.get(doc_id) {
            if let Some(redis_store) = &self.redis_store {
                if redis_store
                    .has_pending_updates(doc_id)
                    .await
                    .unwrap_or(false)
                {
                    if let Ok(updates) = redis_store.get_pending_updates(doc_id).await {
                        if !updates.is_empty() {
                            let _ = self.apply_updates_to_doc(&group, updates).await;
                        }
                    }
                }
            }
            return Ok(group.clone());
        }

        let doc_lock_key = format!("lock:doc:{}", doc_id);
        let lock_value = uuid::Uuid::new_v4().to_string();
        let mut lock_acquired = false;

        if let Some(redis_store) = &self.redis_store {
            lock_acquired = redis_store
                .acquire_lock(&doc_lock_key, &lock_value, 1)
                .await?;
        }

        let _local_lock = self.groups_mutex.lock().await;

        if let Some(group) = self.groups.get(doc_id) {
            if lock_acquired {
                if let Some(redis_store) = &self.redis_store {
                    let _ = redis_store.release_lock(&doc_lock_key, &lock_value).await;
                }
            }
            return Ok(group.clone());
        }

        let group: Arc<BroadcastGroup>;

        if let Some(redis_store) = &self.redis_store {
            let doc_exists_key = format!("doc:exists:{}", doc_id);
            let exists_in_redis = redis_store.exists(&doc_exists_key).await?;

            if exists_in_redis {
                let updates_from_redis = redis_store.get_pending_updates(doc_id).await?;

                let doc = Doc::new();
                if !updates_from_redis.is_empty() {
                    let mut txn = doc.transact_mut();
                    for update in &updates_from_redis {
                        if let Ok(decoded) = yrs::updates::decoder::Decode::decode_v1(update) {
                            let _ = txn.apply_update(decoded);
                        }
                    }
                }

                let awareness = Arc::new(tokio::sync::RwLock::new(Awareness::new(doc)));
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

                self.groups.insert(doc_id.to_string(), group.clone());

                if lock_acquired {
                    let _ = redis_store.release_lock(&doc_lock_key, &lock_value).await;
                }

                return Ok(group);
            }
        }

        let awareness: AwarenessRef = {
            let doc = Doc::new();
            let mut updates_from_redis = Vec::new();

            {
                let mut txn = doc.transact_mut();
                let load_result = self.store.load_doc(doc_id, &mut txn).await;

                if let Err(e) = load_result {
                    if lock_acquired {
                        if let Some(redis_store) = &self.redis_store {
                            redis_store.release_lock(&doc_lock_key, &lock_value).await?;
                        }
                    }
                    return Err(anyhow!("Failed to load document: {}", e));
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

        self.groups.insert(doc_id.to_string(), group.clone());

        if let Some(redis_store) = &self.redis_store {
            let doc_exists_key = format!("doc:exists:{}", doc_id);
            let _ = redis_store.set(&doc_exists_key, "created").await;

            if lock_acquired {
                let _ = redis_store.release_lock(&doc_lock_key, &lock_value).await;
            }
        }

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

    pub async fn remove_connection(&self, doc_id: &str) {
        if let Some(group) = self.groups.get(doc_id) {
            let group_clone = group.clone();
            let remaining = group.decrement_connections();

            if remaining == 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                if group_clone.connection_count() == 0 {
                    self.groups.remove(doc_id);
                }
            }
        }
    }
}
