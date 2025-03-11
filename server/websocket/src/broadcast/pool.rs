use crate::broadcast::group::{BroadcastConfig, BroadcastGroup, RedisConfig};
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::AwarenessRef;
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use redis::AsyncCommands;
use std::sync::Arc;
use yrs::sync::Awareness;
use yrs::{Doc, Transact};

#[derive(Clone, Debug)]
pub struct BroadcastPool {
    store: Arc<GcsStore>,
    redis_config: Option<RedisConfig>,
    groups: DashMap<String, Arc<BroadcastGroup>>,
    buffer_capacity: usize,
    groups_mutex: Arc<tokio::sync::Mutex<()>>,
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_config: Option<RedisConfig>) -> Self {
        Self {
            store,
            redis_config,
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
        Self {
            store,
            redis_config,
            groups: DashMap::new(),
            buffer_capacity,
            groups_mutex: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    pub fn get_store(&self) -> Arc<GcsStore> {
        self.store.clone()
    }

    pub async fn get_or_create_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        let _lock = self.groups_mutex.lock().await;

        if let Some(group) = self.groups.get(doc_id) {
            return Ok(group.clone());
        }

        let awareness: AwarenessRef = {
            let doc = Doc::new();
            let mut updates_from_redis = Vec::new();

            if let Some(redis_config) = &self.redis_config {
                let redis_key = format!("pending_updates:{}", doc_id);
                if let Ok(manager) = redis::Client::open(redis_config.url.clone()) {
                    if let Ok(mut conn) = manager.get_multiplexed_async_connection().await {
                        match conn.lrange::<_, Vec<Vec<u8>>>(&redis_key, 0, -1).await {
                            Ok(updates) => {
                                if !updates.is_empty() {
                                    tracing::debug!(
                                        "Found {} pending updates in Redis for document '{}'",
                                        updates.len(),
                                        doc_id
                                    );
                                    updates_from_redis = updates;
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to load pending updates from Redis for document '{}': {}",
                                    doc_id,
                                    e
                                );
                            }
                        }
                    }
                }
            }

            {
                let mut txn = doc.transact_mut();

                match self.store.load_doc(doc_id, &mut txn).await {
                    Ok(_) => {
                        tracing::debug!("Successfully loaded existing document: {}", doc_id);
                    }
                    Err(e) => {
                        if e.to_string().contains("not found") {
                            tracing::debug!("Creating new document: {}", doc_id);
                        } else {
                            tracing::error!("Failed to load document {}: {}", doc_id, e);
                            return Err(anyhow!("Failed to load document: {}", e));
                        }
                    }
                }

                for update in updates_from_redis {
                    if let Ok(decoded) = yrs::updates::decoder::Decode::decode_v1(&update) {
                        if let Err(e) = txn.apply_update(decoded) {
                            tracing::warn!(
                                "Failed to apply update from Redis for document '{}': {}",
                                doc_id,
                                e
                            );
                        }
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
        Ok(group)
    }

    pub async fn cleanup_empty_groups(&self) {
        self.groups.retain(|_, group| {
            let count = group.connection_count();
            if count == 0 {
                tracing::debug!("Removing empty broadcast group");
                false
            } else {
                true
            }
        });
    }

    pub async fn remove_connection(&self, doc_id: &str) {
        if let Some(group) = self.groups.get(doc_id) {
            let group_clone = group.clone();
            let remaining = group.decrement_connections();

            tracing::info!(
                "Connection disconnected for document '{}', updates will be flushed in decrement_connections",
                doc_id
            );

            if remaining == 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                if group_clone.connection_count() == 0 {
                    tracing::info!("Removing empty group for document '{}'", doc_id);
                    self.groups.remove(doc_id);
                }
            }
        }
    }
}
