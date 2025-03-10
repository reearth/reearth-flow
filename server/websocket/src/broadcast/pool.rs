use crate::broadcast::group::{BroadcastConfig, BroadcastGroup, RedisConfig};
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::AwarenessRef;
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use redis::AsyncCommands;
use std::sync::Arc;
use yrs::sync::Awareness;
use yrs::{updates::decoder::Decode, Update};
use yrs::{Any, Array, Doc, Map, ReadTxn, Transact, WriteTxn};

#[derive(Clone, Debug)]
pub struct BroadcastPool {
    store: Arc<GcsStore>,
    redis_config: Option<RedisConfig>,
    groups: DashMap<String, Arc<BroadcastGroup>>,
    buffer_capacity: usize,
    doc_creation_lock: Arc<tokio::sync::Mutex<()>>,
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_config: Option<RedisConfig>) -> Self {
        Self {
            store,
            redis_config,
            groups: DashMap::new(),
            buffer_capacity: 1024,
            doc_creation_lock: Arc::new(tokio::sync::Mutex::new(())),
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
            doc_creation_lock: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    pub fn get_store(&self) -> Arc<GcsStore> {
        self.store.clone()
    }

    pub async fn get_or_create_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        if let Some(group) = self.groups.get(doc_id) {
            return Ok(group.clone());
        }

        let _guard = self.doc_creation_lock.lock().await;

        if let Some(group) = self.groups.get(doc_id) {
            return Ok(group.clone());
        }

        let doc = Doc::new();
        let mut redis_conn = None;
        let mut pending_updates = Vec::new();

        if let Some(redis_config) = &self.redis_config {
            let pending_updates_key = format!("pending_updates:{}", doc_id);
            let init_key = format!("doc_initialized:{}", doc_id);

            if let Ok(manager) = redis::Client::open(redis_config.url.clone()) {
                if let Ok(mut conn) = manager.get_multiplexed_async_connection().await {
                    redis_conn = Some(conn.clone());

                    let init_lock: bool = conn.set_nx(&init_key, "1").await?;
                    if init_lock {
                        let _: () = conn.expire(&init_key, 60).await?;

                        let mut txn = doc.transact_mut();
                        match self.store.load_doc(doc_id, &mut txn).await {
                            Ok(true) => {
                                tracing::debug!("Document {} already exists", doc_id);
                            }
                            Ok(false) | Err(_) => {
                                tracing::debug!(
                                    "Document {} will be initialized by client",
                                    doc_id
                                );
                            }
                        }

                        let _: () = conn.del(&init_key).await?;
                    } else {
                        let mut txn = doc.transact_mut();
                        let _ = self.store.load_doc(doc_id, &mut txn).await;
                        tracing::debug!("Could not acquire init lock for {}, another server may be initializing", doc_id);
                    }

                    match conn
                        .lrange::<_, Vec<Vec<u8>>>(&pending_updates_key, 0, -1)
                        .await
                    {
                        Ok(updates) => {
                            if !updates.is_empty() {
                                let mut txn = doc.transact_mut();
                                for update in &updates {
                                    if let Ok(decoded) = Update::decode_v1(update) {
                                        let _ = txn.apply_update(decoded);
                                    }
                                }
                                pending_updates = updates;
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load pending updates from Redis: {}", e);
                        }
                    }
                }
            }
        } else {
            let mut txn = doc.transact_mut();
            let _ = self.store.load_doc(doc_id, &mut txn).await;
        }

        let awareness = Arc::new(tokio::sync::RwLock::new(Awareness::new(doc)));

        let buffer_capacity = self.buffer_capacity;
        let group = if let Some(redis_config) = &self.redis_config {
            match BroadcastGroup::with_storage(
                awareness.clone(),
                buffer_capacity,
                self.store.clone(),
                BroadcastConfig {
                    storage_enabled: true,
                    doc_name: Some(doc_id.to_string()),
                    redis_config: Some(redis_config.clone()),
                },
            )
            .await
            {
                Ok(mut group) => {
                    if !pending_updates.is_empty() {
                        let awareness_ref = group.awareness();
                        let awareness = awareness_ref.write().await;
                        let mut txn = awareness.doc().transact_mut();
                        for update in &pending_updates {
                            if let Ok(decoded) = Update::decode_v1(update) {
                                if let Err(e) = txn.apply_update(decoded) {
                                    tracing::warn!("Failed to apply update: {}", e);
                                }
                            }
                        }
                    }
                    group
                }
                Err(e) => return Err(e),
            }
        } else {
            match BroadcastGroup::new(awareness.clone(), buffer_capacity).await {
                Ok(group) => group,
                Err(e) => return Err(e),
            }
        };

        let group = Arc::new(group);
        let result = self
            .groups
            .entry(doc_id.to_string())
            .or_insert(group.clone());
        Ok(result.clone())
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
