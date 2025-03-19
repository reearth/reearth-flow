use crate::broadcast::group::{BroadcastConfig, BroadcastGroup};
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::RedisStore;
use crate::AwarenessRef;
use anyhow::Result;
use dashmap::DashMap;
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
    last_cleanup: Arc<std::sync::Mutex<std::time::Instant>>,
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_store: Option<Arc<RedisStore>>) -> Self {
        Self {
            store,
            redis_store,
            groups: DashMap::new(),
            buffer_capacity: 128,
            last_cleanup: Arc::new(std::sync::Mutex::new(std::time::Instant::now())),
        }
    }

    pub fn get_store(&self) -> Arc<GcsStore> {
        Arc::clone(&self.store)
    }

    pub fn get_redis_store(&self) -> Option<Arc<RedisStore>> {
        self.redis_store.as_ref().map(Arc::clone)
    }

    pub async fn get_or_create_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        let doc_id_string = doc_id.to_string();

        match self.groups.entry(doc_id_string.clone()) {
            dashmap::mapref::entry::Entry::Occupied(entry) => {
                let group_clone = entry.get().clone();
                drop(entry);

                if let (Some(redis_store), Some(doc_name)) =
                    (group_clone.get_redis_store(), group_clone.get_doc_name())
                {
                    let valid = match redis_store.check_stream_exists(&doc_name).await {
                        Ok(exists) => exists,
                        Err(e) => {
                            tracing::warn!("Error checking Redis stream: {}", e);
                            false
                        }
                    };

                    if !valid {
                        tracing::warn!("Found cached broadcast group for '{}' but Redis stream does not exist, recreating", doc_id);

                        self.groups.remove(&doc_id_string);
                    } else {
                        return Ok(group_clone);
                    }
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

                if let Err(e) = store_clone.push_update(&doc_id_clone, &update).await {
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

        match self.groups.entry(doc_id_string) {
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
            let new_count = group.decrement_connections().await;
            tracing::info!("Document '{}' remaining connections: {}", doc_id, new_count);

            if new_count == 0 {
                tracing::info!("Removing broadcast group for document '{}'", doc_id);

                let group_clone = Arc::clone(&group);
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
                    let redis_store_clone = Arc::clone(redis_store);
                    let doc_id_clone = doc_id.to_string();
                    let instance_id = format!("instance-{}", rand::random::<u64>());

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
                }
            }
        }
    }
}
