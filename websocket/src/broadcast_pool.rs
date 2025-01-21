use crate::broadcast::{BroadcastConfig, BroadcastGroup, RedisConfig};
use crate::storage::kv::DocOps;
//use crate::storage::sqlite::SqliteStore;
use crate::storage::gcs::GcsStore;
use crate::AwarenessRef;
use dashmap::DashMap;
use std::sync::Arc;
use yrs::sync::Awareness;
use yrs::{Doc, Transact};

pub struct BroadcastPool {
    store: Arc<GcsStore>,
    redis_config: RedisConfig,
    groups: DashMap<String, Arc<BroadcastGroup>>,
    buffer_capacity: usize,
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_config: RedisConfig) -> Self {
        Self {
            store,
            redis_config,
            groups: DashMap::new(),
            buffer_capacity: 1024,
        }
    }

    pub fn with_buffer_capacity(
        store: Arc<GcsStore>,
        redis_config: RedisConfig,
        buffer_capacity: usize,
    ) -> Self {
        Self {
            store,
            redis_config,
            groups: DashMap::new(),
            buffer_capacity,
        }
    }

    pub async fn get_or_create_group(&self, doc_id: &str) -> Arc<BroadcastGroup> {
        if let Some(group) = self.groups.get(doc_id) {
            return group.clone();
        }

        // Create new document and broadcast group
        let awareness: AwarenessRef = {
            let doc = Doc::new();

            // Load document state
            {
                let mut txn = doc.transact_mut();
                match self.store.load_doc(doc_id, &mut txn).await {
                    Ok(_) => {
                        tracing::debug!("Successfully loaded existing document: {}", doc_id);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "No existing document found or failed to load {}: {}",
                            doc_id,
                            e
                        );
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
                    redis_config: Some(self.redis_config.clone()),
                },
            )
            .await,
        );

        self.groups.insert(doc_id.to_string(), group.clone());
        group
    }

    pub async fn cleanup_empty_groups(&self) {
        self.groups.retain(|_, group| group.connection_count() > 0);
    }

    pub async fn remove_connection(&self, doc_id: &str) {
        if let Some(group) = self.groups.get(doc_id) {
            let remaining = group.decrement_connections();
            if remaining == 0 {
                self.cleanup_empty_groups().await;
            }
        }
    }
}
