use crate::broadcast::group::{BroadcastConfig, BroadcastGroup, RedisConfig};
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::AwarenessRef;
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use std::sync::Arc;
use yrs::sync::Awareness;
use yrs::{Doc, Transact};

#[derive(Clone, Debug)]
pub struct BroadcastPool {
    store: Arc<GcsStore>,
    redis_config: Option<RedisConfig>,
    groups: DashMap<String, Arc<BroadcastGroup>>,
    buffer_capacity: usize,
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_config: Option<RedisConfig>) -> Self {
        tracing::info!(
            "[POOL] Creating new BroadcastPool with buffer capacity: {}",
            1024
        );
        tracing::info!("[POOL] Storage enabled: {}", redis_config.is_some());
        Self {
            store,
            redis_config,
            groups: DashMap::new(),
            buffer_capacity: 1024,
        }
    }

    pub fn with_buffer_capacity(
        store: Arc<GcsStore>,
        redis_config: Option<RedisConfig>,
        buffer_capacity: usize,
    ) -> Self {
        tracing::info!(
            "[POOL] Creating new BroadcastPool with buffer capacity: {}",
            buffer_capacity
        );
        tracing::info!("[POOL] Storage enabled: {}", redis_config.is_some());
        Self {
            store,
            redis_config,
            groups: DashMap::new(),
            buffer_capacity,
        }
    }

    pub fn get_store(&self) -> Arc<GcsStore> {
        self.store.clone()
    }

    pub async fn get_or_create_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        tracing::info!(
            "[POOL] Getting or creating broadcast group for doc_id: {}",
            doc_id
        );
        let start_time = std::time::Instant::now();

        let entry = self.groups.entry(doc_id.to_string());

        match entry {
            dashmap::mapref::entry::Entry::Occupied(entry) => {
                tracing::info!(
                    "[POOL] Found existing broadcast group for doc_id: {} in {:?}",
                    doc_id,
                    start_time.elapsed()
                );
                Ok(entry.get().clone())
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                tracing::info!(
                    "[POOL] No existing group found, creating new group for doc_id: {}",
                    doc_id
                );

                let awareness: AwarenessRef = {
                    let doc = Doc::new();
                    tracing::info!("[POOL] Created new Doc for doc_id: {}", doc_id);

                    {
                        tracing::info!("[POOL] Starting transaction to load document: {}", doc_id);
                        let load_start = std::time::Instant::now();
                        let mut txn = doc.transact_mut();
                        match self.store.load_doc(doc_id, &mut txn).await {
                            Ok(_) => {
                                tracing::info!(
                                    "[POOL] Successfully loaded existing document: {} in {:?}",
                                    doc_id,
                                    load_start.elapsed()
                                );
                            }
                            Err(e) => {
                                if e.to_string().contains("not found") {
                                    tracing::info!(
                                        "[POOL] Creating new document: {} (load attempt took {:?})",
                                        doc_id,
                                        load_start.elapsed()
                                    );
                                } else {
                                    tracing::error!(
                                        "[POOL] Failed to load document {}: {} (after {:?})",
                                        doc_id,
                                        e,
                                        load_start.elapsed()
                                    );
                                    return Err(anyhow!("Failed to load document: {}", e));
                                }
                            }
                        }
                        tracing::info!("[POOL] Completed transaction for document: {}", doc_id);
                    }

                    tracing::info!("[POOL] Creating new Awareness for doc_id: {}", doc_id);
                    Arc::new(tokio::sync::RwLock::new(Awareness::new(doc)))
                };

                tracing::info!(
                    "[POOL] Setting up BroadcastGroup with storage for doc_id: {}",
                    doc_id
                );
                let group_start = std::time::Instant::now();
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
                tracing::info!(
                    "[POOL] BroadcastGroup created for doc_id: {} in {:?}",
                    doc_id,
                    group_start.elapsed()
                );

                tracing::info!(
                    "[POOL] Inserting new broadcast group for doc_id: {}",
                    doc_id
                );
                let result = entry.insert(group.clone()).clone();
                tracing::info!(
                    "[POOL] Group creation completed for doc_id: {} in total time: {:?}",
                    doc_id,
                    start_time.elapsed()
                );
                Ok(result)
            }
        }
    }

    pub async fn cleanup_empty_groups(&self) {
        tracing::info!("[POOL] Starting cleanup of empty broadcast groups");
        let before_count = self.groups.len();
        self.groups.retain(|doc_id, group| {
            let count = group.connection_count();
            if count == 0 {
                tracing::info!(
                    "[POOL] Removing empty broadcast group for doc_id: {}",
                    doc_id
                );
                false
            } else {
                true
            }
        });
        let after_count = self.groups.len();
        tracing::info!(
            "[POOL] Cleanup complete. Groups before: {}, after: {}",
            before_count,
            after_count
        );
    }

    pub async fn remove_connection(&self, doc_id: &str) {
        tracing::info!("[POOL] Removing connection for doc_id: {}", doc_id);
        if let Some(group) = self.groups.get(doc_id) {
            let group_clone = group.clone();
            let remaining = group.decrement_connections();
            tracing::info!(
                "[POOL] Remaining connections for doc_id {}: {}",
                doc_id,
                remaining
            );

            tracing::info!(
                "[POOL] Flushing updates for group '{}' on disconnect",
                doc_id
            );
            if let Err(e) = group_clone.flush_updates().await {
                tracing::error!(
                    "[POOL] Failed to flush updates for group '{}' on disconnect: {}",
                    doc_id,
                    e
                );
            } else {
                tracing::info!(
                    "[POOL] Successfully flushed updates for group '{}' on disconnect",
                    doc_id
                );
            }

            if remaining == 0 {
                tracing::info!(
                    "[POOL] No connections left for doc_id: {}, scheduling cleanup",
                    doc_id
                );
                let pool = self.clone();
                let doc_id_clone = doc_id.to_string();
                tokio::spawn(async move {
                    tracing::info!("[POOL] Waiting before cleanup for doc_id: {}", doc_id_clone);
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    tracing::info!(
                        "[POOL] Starting delayed cleanup for doc_id: {}",
                        doc_id_clone
                    );
                    pool.cleanup_empty_groups().await;
                });
            }
        } else {
            tracing::warn!(
                "[POOL] Attempted to remove connection for non-existent doc_id: {}",
                doc_id
            );
        }
    }

    pub async fn flush_all_updates(&self) -> Result<(), anyhow::Error> {
        tracing::info!("[POOL] Flushing updates for all groups");
        let mut errors = Vec::new();
        let total_groups = self.groups.len();
        tracing::info!("[POOL] Total groups to flush: {}", total_groups);

        for entry in self.groups.iter() {
            let doc_id = entry.key().clone();
            let group = entry.value().clone();

            tracing::info!("[POOL] Flushing updates for group '{}'", doc_id);
            if let Err(e) = group.flush_updates().await {
                tracing::error!(
                    "[POOL] Failed to flush updates for group '{}': {}",
                    doc_id,
                    e
                );
                errors.push((doc_id, e));
            } else {
                tracing::info!("[POOL] Successfully flushed updates for group '{}'", doc_id);
            }
        }

        if errors.is_empty() {
            tracing::info!(
                "[POOL] Successfully flushed updates for all {} groups",
                total_groups
            );
            Ok(())
        } else {
            let error_count = errors.len();
            let error_msg = errors
                .iter()
                .map(|(id, e)| format!("{}: {}", id, e))
                .collect::<Vec<_>>()
                .join(", ");
            tracing::error!(
                "[POOL] Failed to flush updates for {}/{} groups",
                error_count,
                total_groups
            );
            Err(anyhow!(
                "Failed to flush updates for some groups: {}",
                error_msg
            ))
        }
    }
}
