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
        let entry = self.groups.entry(doc_id.to_string());

        match entry {
            dashmap::mapref::entry::Entry::Occupied(entry) => Ok(entry.get().clone()),
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                let awareness: AwarenessRef = {
                    let doc = Doc::new();

                    {
                        let mut txn = doc.transact_mut();
                        match self.store.load_doc(doc_id, &mut txn).await {
                            Ok(_) => {
                                tracing::debug!(
                                    "Successfully loaded existing document: {}",
                                    doc_id
                                );
                            }
                            Err(e) => {
                                if e.to_string().contains("not found") {
                                    tracing::info!("Creating new document: {}", doc_id);
                                } else {
                                    tracing::error!("Failed to load document {}: {}", doc_id, e);
                                    return Err(anyhow!("Failed to load document: {}", e));
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

            tracing::info!(
                "Connection disconnected for document '{}', flushing updates",
                doc_id
            );
            if let Err(e) = group_clone.flush_updates().await {
                tracing::error!(
                    "Failed to flush updates for group '{}' on disconnect: {}",
                    doc_id,
                    e
                );
            } else {
                tracing::info!(
                    "Successfully flushed updates for group '{}' on disconnect",
                    doc_id
                );
            }

            if remaining == 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                if group_clone.connection_count() == 0 {
                    tracing::info!("Removing empty group for document '{}'", doc_id);
                    self.groups.remove(doc_id);
                }
            }
        }
    }

    pub async fn flush_all_updates(&self) -> Result<(), anyhow::Error> {
        tracing::info!("Flushing updates for all groups");
        let mut errors = Vec::new();

        for entry in self.groups.iter() {
            let doc_id = entry.key().clone();
            let group = entry.value().clone();

            tracing::info!("Flushing updates for group '{}'", doc_id);
            if let Err(e) = group.flush_updates().await {
                tracing::error!("Failed to flush updates for group '{}': {}", doc_id, e);
                errors.push((doc_id, e));
            }
        }

        if errors.is_empty() {
            tracing::info!("Successfully flushed updates for all groups");
            Ok(())
        } else {
            let error_msg = errors
                .iter()
                .map(|(id, e)| format!("{}: {}", id, e))
                .collect::<Vec<_>>()
                .join(", ");
            Err(anyhow!(
                "Failed to flush updates for some groups: {}",
                error_msg
            ))
        }
    }
}
