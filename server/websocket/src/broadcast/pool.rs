use crate::broadcast::group::{BroadcastConfig, BroadcastGroup, RedisConfig};
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::AwarenessRef;
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::Mutex;
use yrs::sync::Awareness;
use yrs::{Any, Doc, Map, ReadTxn, Transact, WriteTxn};

#[derive(Clone, Debug)]
pub struct BroadcastPool {
    store: Arc<GcsStore>,
    redis_config: Option<RedisConfig>,
    groups: DashMap<String, Arc<BroadcastGroup>>,
    buffer_capacity: usize,
    doc_locks: Arc<DashMap<String, Arc<Mutex<()>>>>,
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_config: Option<RedisConfig>) -> Self {
        Self {
            store,
            redis_config,
            groups: DashMap::new(),
            buffer_capacity: 1024,
            doc_locks: Arc::new(DashMap::new()),
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
            doc_locks: Arc::new(DashMap::new()),
        }
    }

    pub fn get_store(&self) -> Arc<GcsStore> {
        self.store.clone()
    }

    async fn handle_doc_initialization(
        &self,
        conn: &mut redis::aio::MultiplexedConnection,
        lock_key: &str,
    ) -> Result<(bool, Option<String>)> {
        let acquired: bool = conn.set_nx(lock_key, "true").await?;

        if acquired {
            let _: () = conn.expire(lock_key, 300).await?;
            Ok((true, Some(lock_key.to_string())))
        } else {
            Ok((false, None))
        }
    }

    async fn release_lock(
        &self,
        conn: &mut redis::aio::MultiplexedConnection,
        lock_key: &str,
    ) -> Result<()> {
        let _: () = conn.del(lock_key).await?;
        tracing::info!("Released lock for {}", lock_key);
        Ok(())
    }

    pub async fn get_or_create_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        let doc_lock = self
            .doc_locks
            .entry(doc_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();

        let _guard = doc_lock.lock().await;

        let entry = self.groups.entry(doc_id.to_string());

        match entry {
            dashmap::mapref::entry::Entry::Occupied(entry) => {
                let group = entry.get().clone();
                {
                    let awareness = group.awareness().read().await;
                    let doc = awareness.doc();
                    let txn = doc.transact();

                    let init_map = txn.get_map("workflow_initialized");
                    if init_map.is_none() {
                        let mut txn = doc.transact_mut();
                        let init_map = txn.get_or_insert_map("workflow_initialized");
                        init_map.insert(&mut txn, "initialized", Any::Bool(true));
                    }
                }
                Ok(group)
            }

            dashmap::mapref::entry::Entry::Vacant(entry) => {
                let awareness: AwarenessRef = {
                    let doc = Doc::new();
                    let mut updates_from_redis = Vec::new();
                    let mut needs_initialization = false;

                    if let Some(redis_config) = &self.redis_config {
                        let redis_key = format!("pending_updates:{}", doc_id);
                        let init_key = format!("workflow_initialized:{}", doc_id);

                        if let Ok(manager) = redis::Client::open(redis_config.url.clone()) {
                            if let Ok(mut conn) = manager.get_multiplexed_async_connection().await {
                                let (acquired_lock, lock_key) =
                                    self.handle_doc_initialization(&mut conn, &init_key).await?;
                                needs_initialization = acquired_lock;

                                tracing::info!("redis_key: {}", redis_key);

                                match conn.lrange::<_, Vec<Vec<u8>>>(&redis_key, 0, -1).await {
                                    Ok(updates) => {
                                        if !updates.is_empty() {
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

                                if let Some(lock_key) = lock_key {
                                    if let Err(e) = self.release_lock(&mut conn, &lock_key).await {
                                        tracing::error!(
                                            "Failed to release lock for document '{}': {}",
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
                            Ok(loaded) => {
                                if loaded {
                                    tracing::info!(
                                        "Successfully loaded existing document: {}",
                                        doc_id
                                    );
                                    let init_map = txn.get_or_insert_map("workflow_initialized");
                                    init_map.insert(&mut txn, "initialized", Any::Bool(true));
                                } else if !needs_initialization {
                                    let init_map = txn.get_or_insert_map("workflow_initialized");
                                    init_map.insert(&mut txn, "initialized", Any::Bool(true));
                                } else {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                    let init_map = txn.get_or_insert_map("workflow_initialized");
                                    init_map.insert(&mut txn, "initialized", Any::Bool(false));
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to load document {}: {}", doc_id, e);
                                return Err(anyhow!("Failed to load document: {}", e));
                            }
                        }

                        for update in updates_from_redis {
                            if let Ok(decoded) = yrs::updates::decoder::Decode::decode_v1(&update) {
                                if let Err(e) = txn.apply_update(decoded) {
                                    tracing::warn!("Failed to apply update from Redis: {}", e);
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

                tracing::info!("inserting group");

                Ok(entry.insert(group.clone()).clone())
            }
        }
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
