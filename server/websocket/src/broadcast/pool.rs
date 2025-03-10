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
use yrs::{Any, Array, Doc, Map, ReadTxn, Transact, WriteTxn};

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

    async fn acquire_redis_lock(
        &self,
        conn: &mut redis::aio::MultiplexedConnection,
        lock_key: &str,
    ) -> Result<bool> {
        let set_options = redis::Script::new(
            r#"return redis.call('SET', KEYS[1], ARGV[1], 'EX', ARGV[2], 'NX')"#,
        );
        let lock_result: Option<String> = set_options
            .key(lock_key)
            .arg("1")
            .arg("30")
            .invoke_async(conn)
            .await?;
        Ok(lock_result.is_some())
    }

    async fn handle_doc_initialization(
        &self,
        conn: &mut redis::aio::MultiplexedConnection,
        lock_key: &str,
    ) -> Result<bool> {
        let mut should_initialize = true;

        // First check if the key exists
        let exists: bool = conn.exists(lock_key).await?;
        tracing::info!("exists: {}", exists);
        should_initialize = !exists;

        tracing::info!("should_initialize: {}", should_initialize);

        if should_initialize {
            tracing::info!("setting lock");
            let _: () = conn.set_ex(lock_key, "true", 300).await?;
            Ok(true)
        } else {
            tracing::info!("lock already exists");
            Ok(false)
        }
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
                        let lock_key = format!("workflow_init_lock:{}", doc_id);
                        let init_key = format!("workflow_initialized:{}", doc_id);

                        if let Ok(manager) = redis::Client::open(redis_config.url.clone()) {
                            if let Ok(mut conn) = manager.get_multiplexed_async_connection().await {
                                let lock_acquired =
                                    self.acquire_redis_lock(&mut conn, &lock_key).await?;

                                tracing::info!("lock_acquired: {}", lock_acquired);

                                if lock_acquired {
                                    // If we acquired the lock, we should initialize
                                    needs_initialization = true;
                                    let _: () = conn.del(&lock_key).await?;
                                } else {
                                    // If we didn't acquire the lock, check if it exists
                                    needs_initialization = !self
                                        .handle_doc_initialization(&mut conn, &init_key)
                                        .await?;
                                }

                                tracing::info!("redis_key: {}", redis_key);

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
                        tracing::info!("loading doc");
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
                                } else {
                                    tracing::info!("Creating new document: {}", doc_id);
                                    tracing::info!(
                                        "needs_initialization: {}",
                                        needs_initialization
                                    );
                                    if !needs_initialization {
                                        println!("needs_initialization: {}", needs_initialization);
                                        let init_map =
                                            txn.get_or_insert_map("workflow_initialized");
                                        init_map.insert(&mut txn, "initialized", Any::Bool(true));
                                    } else {
                                        let init_map =
                                            txn.get_or_insert_map("workflow_initialized");
                                        init_map.insert(&mut txn, "initialized", Any::Bool(false));
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to load document {}: {}", doc_id, e);
                                return Err(anyhow!("Failed to load document: {}", e));
                            }
                        }

                        tracing::info!("applying updates");

                        for update in updates_from_redis {
                            if let Ok(decoded) = yrs::updates::decoder::Decode::decode_v1(&update) {
                                if let Err(e) = txn.apply_update(decoded) {
                                    tracing::warn!("Failed to apply update from Redis: {}", e);
                                }
                            }
                        }
                    }

                    tracing::info!("creating awareness");

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
