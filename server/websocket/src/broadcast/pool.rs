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

                // Create new broadcast group
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
        // Only remove groups that still have zero connections when we check
        // This prevents race conditions where a new connection was added
        // between marking for cleanup and actual cleanup
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
            let remaining = group.decrement_connections();
            if remaining == 0 {
                // Add a small delay before cleanup to reduce likelihood of race conditions
                // with new connections being established
                let pool = self.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    pool.cleanup_empty_groups().await;
                });
            }
        }
    }
}
