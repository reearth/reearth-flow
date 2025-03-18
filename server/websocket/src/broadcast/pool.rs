use crate::broadcast::group::{BroadcastConfig, BroadcastGroup};
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::RedisStore;
use crate::AwarenessRef;
use anyhow::Result;
use dashmap::DashMap;
use dashmap::DashSet;
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
        let pool = Self {
            store,
            redis_store,
            groups: DashMap::new(),
            buffer_capacity: 256,
            docs_in_creation: DashSet::new(),
            last_cleanup: Arc::new(std::sync::Mutex::new(std::time::Instant::now())),
        };

        let pool_clone = pool.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                pool_clone.cleanup_empty_groups().await;
            }
        });

        pool
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
            return Ok(group_clone);
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
            let group_clone = group.clone();
            let remaining = group.decrement_connections();

            if remaining == 0 && group_clone.connection_count() == 0 {
                if let Err(e) = group_clone.shutdown().await {
                    tracing::warn!(
                        "Failed to shutdown broadcast group for document '{}': {}",
                        doc_id,
                        e
                    );
                }
                self.groups.remove(doc_id);

                if let Some(redis_store) = &self.redis_store {
                    let redis_store_clone = redis_store.clone();
                    let doc_id_clone = doc_id.to_string();

                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                        if let Err(e) = redis_store_clone.delete_stream(&doc_id_clone).await {
                            tracing::warn!(
                                "Failed to delete Redis stream for '{}': {}",
                                doc_id_clone,
                                e
                            );
                        } else {
                            tracing::info!(
                                "Successfully deleted Redis stream for '{}'",
                                doc_id_clone
                            );
                        }
                    });
                }
            }
        }
    }
}
