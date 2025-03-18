use crate::broadcast::group::{BroadcastConfig, BroadcastGroup};
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::RedisStore;
use crate::AwarenessRef;
use anyhow::Result;
use dashmap::DashMap;
use dashmap::DashSet;
use rand;
use std::sync::Arc;
use yrs::sync::Awareness;
use yrs::{Doc, ReadTxn, StateVector, Transact};

const DEFAULT_DOC_ID: &str = "01jpjfpw0qtw17kbrcdbgefakg";
#[derive(Clone, Debug)]
pub struct BroadcastPool {
    store: Arc<GcsStore>,
    redis_store: Option<Arc<RedisStore>>,
    groups: DashMap<String, Arc<BroadcastGroup>>,
    buffer_capacity: usize,
    docs_in_creation: DashSet<String>,
    last_cleanup: Arc<std::sync::Mutex<std::time::Instant>>,
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_store: Option<Arc<RedisStore>>) -> Self {
        Self {
            store,
            redis_store,
            groups: DashMap::new(),
            buffer_capacity: 256,
            docs_in_creation: DashSet::new(),
            last_cleanup: Arc::new(std::sync::Mutex::new(std::time::Instant::now())),
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

            if let (Some(redis_store), Some(group_name), Some(doc_name)) = (
                group_clone.get_redis_store(),
                group_clone.get_redis_group_name(),
                group_clone.get_doc_name(),
            ) {
                let stream_key = format!("yjs:stream:{}", doc_name);

                let mut valid = false;
                if let Ok(mut conn) = redis_store.get_pool().get().await {
                    let exists: bool = redis::cmd("EXISTS")
                        .arg(&stream_key)
                        .query_async(&mut *conn)
                        .await
                        .unwrap_or(false);

                    if exists {
                        let result: Result<Vec<String>, redis::RedisError> = redis::cmd("XINFO")
                            .arg("GROUPS")
                            .arg(&stream_key)
                            .query_async(&mut *conn)
                            .await;

                        if let Ok(groups) = result {
                            for i in (0..groups.len()).step_by(8) {
                                if i + 1 < groups.len()
                                    && groups[i] == "name"
                                    && groups[i + 1] == group_name
                                {
                                    valid = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if valid {
                    tracing::debug!("Retrieved existing valid broadcast group for '{}'", doc_id);
                    return Ok(group_clone);
                } else {
                    tracing::warn!("Found cached broadcast group for '{}' but Redis resources are invalid, recreating", doc_id);
                    self.groups.remove(doc_id);
                }
            } else {
                return Ok(group_clone);
            }
        }

        if !self.docs_in_creation.insert(doc_id.to_string()) {
            for i in 0..6 {
                let delay_ms = 5 * (1 << i);
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

        if let Some(group) = self.groups.get(doc_id) {
            let group_clone = group.clone();
            drop(group);
            return Ok(group_clone);
        }

        let group: Arc<BroadcastGroup>;
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

        if let Some(existing_group) = self.groups.get(doc_id) {
            return Ok(existing_group.clone());
        }

        self.groups.insert(doc_id.to_string(), group.clone());

        Ok(group)
    }

    pub async fn cleanup_empty_groups(&self) {
        {
            let mut last_cleanup = self.last_cleanup.lock().unwrap();
            let now = std::time::Instant::now();
            if now.duration_since(*last_cleanup).as_secs() < 60 {
                return;
            }
            *last_cleanup = now;
        }

        let mut groups_to_remove = Vec::new();

        for entry in self.groups.iter() {
            let count = entry.value().connection_count();
            if count == 0 {
                groups_to_remove.push(entry.key().clone());
            }
        }

        for doc_id in groups_to_remove {
            if let Some((_, group)) = self.groups.remove(&doc_id) {
                if let Err(e) = group.shutdown().await {
                    tracing::warn!("Error shutting down empty group for '{}': {}", doc_id, e);
                }
            }
        }
    }

    pub async fn remove_connection(&self, doc_id: &str) {
        if let Some(group) = self.groups.get(doc_id) {
            let remaining = group.decrement_connections();
            tracing::info!("Remaining connections: {}", remaining);
            tracing::info!("Group connection count: {}", group.connection_count());

            if remaining == 1 && group.connection_count() == 0 {
                tracing::info!("Removing broadcast group for document '{}'", doc_id);

                let group_clone = group.clone();
                drop(group);

                tracing::info!("Shutting down broadcast group");
                if let Err(e) = group_clone.shutdown().await {
                    tracing::warn!("Failed to shutdown: {}", e);
                }

                if let Some(task) = &group_clone.redis_subscriber_task {
                    task.abort();
                }
                group_clone.awareness_updater.abort();

                if let Some((_, _)) = self.groups.remove(doc_id) {
                    tracing::info!("Group removed for document '{}'", doc_id);
                }

                if let Some(redis_store) = &self.redis_store {
                    let redis_store_clone = redis_store.clone();
                    let doc_id_clone = doc_id.to_string();
                    let instance_id = format!("instance-{}", rand::random::<u64>());

                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                        match redis_store_clone
                            .safe_delete_stream(&doc_id_clone, &instance_id)
                            .await
                        {
                            Ok(deleted) => {
                                if deleted {
                                    tracing::info!(
                                        "Successfully deleted Redis stream for '{}'",
                                        doc_id_clone
                                    );
                                } else {
                                    tracing::info!("Did not delete Redis stream for '{}' as it may still be in use", doc_id_clone);
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Error during safe Redis stream deletion for '{}': {}",
                                    doc_id_clone,
                                    e
                                );
                            }
                        }
                    });
                }
            }
        }
    }
}
