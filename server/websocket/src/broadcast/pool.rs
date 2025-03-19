use crate::broadcast::group::{BroadcastConfig, BroadcastGroup};
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::RedisStore;
use crate::AwarenessRef;
use anyhow::{Error, Result};
use dashmap::DashMap;
use deadpool::managed::{self, Manager, Metrics, Pool, RecycleResult};
use std::sync::Arc;
use std::time::Duration;
use yrs::sync::Awareness;
use yrs::{Doc, ReadTxn, StateVector, Transact};

const DEFAULT_DOC_ID: &str = "01jpjfpw0qtw17kbrcdbgefakg";

#[derive(Debug)]
pub struct BroadcastGroupContext {
    doc_id: String,
    group: Arc<BroadcastGroup>,
}

#[derive(Debug)]
pub struct BroadcastGroupManager {
    store: Arc<GcsStore>,
    redis_store: Arc<RedisStore>,
    buffer_capacity: usize,
    doc_to_id_map: Arc<DashMap<String, Arc<BroadcastGroup>>>,
}

impl BroadcastGroupManager {
    pub fn new(store: Arc<GcsStore>, redis_store: Arc<RedisStore>) -> Self {
        Self {
            store,
            redis_store,
            buffer_capacity: 128,
            doc_to_id_map: Arc::new(DashMap::new()),
        }
    }

    async fn create_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        let doc_id_string = doc_id.to_string();

        if let Some(group) = self.doc_to_id_map.get(&doc_id_string) {
            if let (Some(redis_store), Some(doc_name)) =
                (group.get_redis_store(), group.get_doc_name())
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
                    self.doc_to_id_map.remove(&doc_id_string);
                } else {
                    return Ok(group.clone());
                }
            } else {
                return Ok(group.clone());
            }
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

        self.doc_to_id_map.insert(doc_id_string, Arc::clone(&group));

        Ok(group)
    }
}

impl Manager for BroadcastGroupManager {
    type Type = BroadcastGroupContext;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let group = self.create_group(DEFAULT_DOC_ID).await?;

        Ok(BroadcastGroupContext {
            doc_id: DEFAULT_DOC_ID.to_string(),
            group,
        })
    }

    fn recycle(
        &self,
        obj: &mut Self::Type,
        _metrics: &Metrics,
    ) -> impl std::future::Future<Output = RecycleResult<Self::Error>> + Send {
        let doc_to_id_map = self.doc_to_id_map.clone();
        let group = obj.group.clone();
        let doc_id = obj.doc_id.clone();

        async move {
            if group.connection_count() == 0 {
                tracing::info!("Recycling empty broadcast group for document '{}'", doc_id);

                doc_to_id_map.remove(&doc_id);

                if let Err(e) = group.shutdown().await {
                    tracing::warn!("Error shutting down empty group for '{}': {}", doc_id, e);
                    return Err(managed::RecycleError::Message(
                        format!("Failed to shutdown: {}", e).into(),
                    ));
                }

                return Err(managed::RecycleError::Message(
                    "Group has no connections".into(),
                ));
            }
            tracing::info!("Recycling broadcast group for document '{}'", doc_id);
            Ok(())
        }
    }
}

#[derive(Clone, Debug)]
pub struct BroadcastPool {
    pool: Pool<BroadcastGroupManager>,
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_store: Arc<RedisStore>) -> Self {
        let manager = BroadcastGroupManager::new(store.clone(), redis_store.clone());
        let pool = Pool::builder(manager)
            .max_size(100)
            .wait_timeout(Some(Duration::from_secs(5)))
            .runtime(deadpool::Runtime::Tokio1)
            .recycle_timeout(Some(Duration::from_secs(5)))
            .build()
            .unwrap();

        let doc_to_id_map = pool.manager().doc_to_id_map.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;

                let mut empty_groups = vec![];
                for entry in doc_to_id_map.iter() {
                    let doc_id = entry.key().clone();
                    let group = entry.value().clone();

                    if group.connection_count() == 0 {
                        empty_groups.push((doc_id, group));
                    }
                }

                for (doc_id, group) in empty_groups {
                    tracing::info!(
                        "Cleaning up empty broadcast group for document '{}'",
                        doc_id
                    );
                    doc_to_id_map.remove(&doc_id);

                    if let Err(e) = group.shutdown().await {
                        tracing::warn!("Error shutting down empty group for '{}': {}", doc_id, e);
                    }
                }
            }
        });

        Self { pool }
    }

    pub fn get_store(&self) -> Arc<GcsStore> {
        self.pool.manager().store.clone()
    }

    pub fn get_redis_store(&self) -> Arc<RedisStore> {
        self.pool.manager().redis_store.clone()
    }

    pub async fn get_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        if let Some(group) = self.pool.manager().doc_to_id_map.get(doc_id) {
            return Ok(group.clone());
        }

        let group = self.pool.manager().create_group(doc_id).await?;
        Ok(group)
    }
}
