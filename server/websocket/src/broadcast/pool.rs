use crate::broadcast::group::{BroadcastConfig, BroadcastGroup};
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::{RedisConfig, RedisStore};
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
    redis_config: Option<RedisConfig>,
    redis_store: Option<Arc<RedisStore>>,
    groups: DashMap<String, Arc<BroadcastGroup>>,
    buffer_capacity: usize,
    groups_mutex: Arc<tokio::sync::Mutex<()>>,
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_config: Option<RedisConfig>) -> Self {
        let redis_store = redis_config.as_ref().map(|config| {
            let store = RedisStore::new(Some(config.clone()));
            Arc::new(store)
        });

        Self {
            store,
            redis_config,
            redis_store,
            groups: DashMap::new(),
            buffer_capacity: 1024,
            groups_mutex: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    pub fn with_buffer_capacity(
        store: Arc<GcsStore>,
        redis_config: Option<RedisConfig>,
        buffer_capacity: usize,
    ) -> Self {
        let redis_store = redis_config.as_ref().map(|config| {
            let store = RedisStore::new(Some(config.clone()));
            Arc::new(store)
        });

        Self {
            store,
            redis_config,
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
        let _local_lock = self.groups_mutex.lock().await;

        if let Some(group) = self.groups.get(doc_id) {
            let mut has_pending_updates = false;

            if let Some(redis_store) = &self.redis_store {
                has_pending_updates = redis_store.has_pending_updates(doc_id).await?;
            }

            if has_pending_updates {
                return Ok(group.clone());
            }
        }

        let doc_lock_key = format!("lock:doc:{}", doc_id);
        let lock_value = uuid::Uuid::new_v4().to_string();
        let mut lock_acquired = false;

        if let Some(redis_store) = &self.redis_store {
            lock_acquired = redis_store
                .acquire_lock(&doc_lock_key, &lock_value, 10)
                .await?;

            if !lock_acquired {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                if let Some(group) = self.groups.get(doc_id) {
                    return Ok(group.clone());
                }
            }
        }

        let doc_exists_key = format!("doc:exists:{}", doc_id);
        let mut doc_already_exists = false;

        if let Some(redis_store) = &self.redis_store {
            doc_already_exists = redis_store.exists(&doc_exists_key).await?;

            if doc_already_exists {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                if let Some(group) = self.groups.get(doc_id) {
                    if lock_acquired {
                        redis_store.release_lock(&doc_lock_key, &lock_value).await?;
                    }
                    return Ok(group.clone());
                }
            } else {
                let created = redis_store.set_nx(&doc_exists_key, "creating").await?;
                if created {
                    redis_store.expire(&doc_exists_key, 30).await?;
                } else {
                    doc_already_exists = true;
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                    if let Some(group) = self.groups.get(doc_id) {
                        if lock_acquired {
                            redis_store.release_lock(&doc_lock_key, &lock_value).await?;
                        }
                        return Ok(group.clone());
                    }
                }
            }
        }

        if let Some(group) = self.groups.get(doc_id) {
            if lock_acquired {
                if let Some(redis_store) = &self.redis_store {
                    redis_store.release_lock(&doc_lock_key, &lock_value).await?;
                }
            }
            return Ok(group.clone());
        }

        let awareness: AwarenessRef = {
            let doc = Doc::new();
            let mut updates_from_redis = Vec::new();

            if let Some(redis_store) = &self.redis_store {
                updates_from_redis = redis_store.get_pending_updates(doc_id).await?;
            }

            {
                let mut txn = doc.transact_mut();

                match self.store.load_doc(doc_id, &mut txn).await {
                    Ok(_) => {}
                    Err(e) => {
                        if e.to_string().contains("not found") {
                            if let Some(redis_store) = &self.redis_store {
                                redis_store.set(&doc_exists_key, "created").await?;
                            }
                        } else {
                            if let Some(redis_store) = &self.redis_store {
                                redis_store.del(&doc_exists_key).await?;
                            }

                            if lock_acquired {
                                if let Some(redis_store) = &self.redis_store {
                                    redis_store.release_lock(&doc_lock_key, &lock_value).await?;
                                }
                            }

                            return Err(anyhow!("Failed to load document: {}", e));
                        }
                    }
                }

                for update in &updates_from_redis {
                    if let Ok(decoded) = yrs::updates::decoder::Decode::decode_v1(update) {
                        let _ = txn.apply_update(decoded);
                    }
                }
            }

            Arc::new(tokio::sync::RwLock::new(Awareness::new(doc)))
        };

        let group = Arc::new(
            BroadcastGroup::with_storage(
                awareness,
                self.buffer_capacity,
                self.store.clone(),
                BroadcastConfig {
                    storage_enabled: true,
                    doc_name: Some(doc_id.to_string()),
                    redis_config: self.redis_config.clone(),
                },
            )
            .await?,
        );

        self.groups.insert(doc_id.to_string(), group.clone());

        if lock_acquired {
            if let Some(redis_store) = &self.redis_store {
                redis_store.release_lock(&doc_lock_key, &lock_value).await?;
            }
        }

        Ok(group)
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
