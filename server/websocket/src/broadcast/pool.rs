use crate::broadcast::group::{BroadcastConfig, BroadcastGroup};
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::RedisStore;
use crate::AwarenessRef;
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use rand;
use std::sync::Arc;
use uuid;
use yrs::sync::Awareness;
use yrs::{Doc, Transact};

#[derive(Clone, Debug)]
pub struct BroadcastPool {
    store: Arc<GcsStore>,
    redis_store: Option<Arc<RedisStore>>,
    groups: DashMap<String, Arc<BroadcastGroup>>,
    buffer_capacity: usize,
    groups_mutex: Arc<tokio::sync::Mutex<()>>,
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_store: Option<Arc<RedisStore>>) -> Self {
        Self {
            store,
            redis_store,
            groups: DashMap::new(),
            buffer_capacity: 1024,
            groups_mutex: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    pub fn with_buffer_capacity(
        store: Arc<GcsStore>,
        redis_store: Option<Arc<RedisStore>>,
        buffer_capacity: usize,
    ) -> Self {
        Self {
            store,
            redis_store,
            groups: DashMap::new(),
            buffer_capacity,
            groups_mutex: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    pub fn get_store(&self) -> Arc<GcsStore> {
        self.store.clone()
    }

    async fn apply_pending_updates_from_redis(
        &self,
        group: &Arc<BroadcastGroup>,
        doc_id: &str,
    ) -> Result<()> {
        let mut retry_count: u32 = 0;
        const MAX_RETRIES: u32 = 3;
        const BASE_RETRY_INTERVAL_MS: u64 = 10;

        while retry_count < MAX_RETRIES {
            if let Some(redis_store) = &self.redis_store {
                let updates_from_redis = redis_store.get_pending_updates(doc_id).await?;
                if !updates_from_redis.is_empty() {
                    let awareness_ref = group.awareness();
                    let awareness = awareness_ref.write().await;
                    let doc = awareness.doc();
                    let mut txn = doc.transact_mut();

                    for update in &updates_from_redis {
                        if let Ok(decoded) = yrs::updates::decoder::Decode::decode_v1(update) {
                            let _ = txn.apply_update(decoded);
                        }
                    }
                    return Ok(());
                }
            }

            retry_count += 1;
            if retry_count < MAX_RETRIES {
                let backoff = BASE_RETRY_INTERVAL_MS * (1u64 << retry_count.min(4));
                let jitter = (backoff as f64 * rand::random::<f64>() * 0.3) as u64;
                tokio::time::sleep(tokio::time::Duration::from_millis(backoff + jitter)).await;
            }
        }

        Ok(())
    }

    pub async fn get_or_create_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        if let Some(group) = self.groups.get(doc_id) {
            self.apply_pending_updates_from_redis(&group.clone(), doc_id)
                .await?;
            return Ok(group.clone());
        }

        let doc_lock_key = format!("lock:doc:{}", doc_id);
        let lock_value = uuid::Uuid::new_v4().to_string();
        let mut lock_acquired = false;

        {
            let _local_lock = self.groups_mutex.lock().await;

            if let Some(group) = self.groups.get(doc_id) {
                self.apply_pending_updates_from_redis(&group.clone(), doc_id)
                    .await?;
                return Ok(group.clone());
            }
        }

        if let Some(redis_store) = &self.redis_store {
            lock_acquired = redis_store
                .acquire_lock(&doc_lock_key, &lock_value, 3)
                .await?;

            if !lock_acquired {
                let mut retry_count = 0;
                const MAX_RETRIES: u32 = 10;
                const BASE_RETRY_INTERVAL_MS: u64 = 5;
                let mut found_main_workflow = false;

                while retry_count < MAX_RETRIES && !found_main_workflow {
                    let updates_from_redis = redis_store.get_pending_updates(doc_id).await?;

                    if !updates_from_redis.is_empty() {
                        found_main_workflow = true;
                    }

                    if !found_main_workflow {
                        let backoff = BASE_RETRY_INTERVAL_MS * (1u64 << retry_count.min(6));
                        let jitter = (backoff as f64 * rand::random::<f64>() * 0.5) as u64;
                        let wait_time = backoff + jitter;

                        tokio::time::sleep(tokio::time::Duration::from_millis(wait_time)).await;
                        retry_count += 1;
                    }
                }

                let _local_lock = self.groups_mutex.lock().await;

                if let Some(group) = self.groups.get(doc_id) {
                    self.apply_pending_updates_from_redis(&group.clone(), doc_id)
                        .await?;
                    return Ok(group.clone());
                }
            }
        }

        let doc_exists_key = format!("doc:exists:{}", doc_id);

        if let Some(redis_store) = &self.redis_store {
            let exists_in_redis = { redis_store.exists(&doc_exists_key).await? };

            if exists_in_redis {
                let _local_lock = self.groups_mutex.lock().await;
                if let Some(group) = self.groups.get(doc_id) {
                    self.apply_pending_updates_from_redis(&group.clone(), doc_id)
                        .await?;
                    if lock_acquired {
                        redis_store.release_lock(&doc_lock_key, &lock_value).await?;
                    }
                    return Ok(group.clone());
                }
            } else {
                let created = redis_store
                    .set_nx_with_expiry(&doc_exists_key, "creating", 1)
                    .await?;
                if !created {
                    let _local_lock = self.groups_mutex.lock().await;

                    if let Some(group) = self.groups.get(doc_id) {
                        self.apply_pending_updates_from_redis(&group.clone(), doc_id)
                            .await?;
                        if lock_acquired {
                            redis_store.release_lock(&doc_lock_key, &lock_value).await?;
                        }
                        return Ok(group.clone());
                    }
                }
            }
        }

        if self.groups.get(doc_id).is_some() && lock_acquired {
            if let Some(redis_store) = &self.redis_store {
                redis_store.release_lock(&doc_lock_key, &lock_value).await?;
            }
        }

        let awareness: AwarenessRef = {
            let doc = Doc::new();
            let mut updates_from_redis = Vec::new();

            if let Some(redis_store) = &self.redis_store {
                updates_from_redis = redis_store.get_pending_updates(doc_id).await?;
            }

            {
                let mut txn = doc.transact_mut();

                match self.store.load_doc(doc_id, &mut txn).await {
                    Ok(exists) => {
                        if !exists {
                            if let Some(redis_store) = &self.redis_store {
                                redis_store
                                    .set_with_expiry(&doc_exists_key, "created", 1)
                                    .await?;
                            }
                        }
                    }
                    Err(e) => {
                        if let Some(redis_store) = &self.redis_store {
                            redis_store.del(&doc_exists_key).await?;
                        }

                        if lock_acquired {
                            if let Some(redis_store) = &self.redis_store {
                                redis_store.release_lock(&doc_lock_key, &lock_value).await?;
                            }
                        }

                        return Err(anyhow!("Failed to load document: {}", e));
                    }
                }

                for update in &updates_from_redis {
                    if let Ok(decoded) = yrs::updates::decoder::Decode::decode_v1(update) {
                        let _ = txn.apply_update(decoded);
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
                self.redis_store.clone(),
                BroadcastConfig {
                    storage_enabled: true,
                    doc_name: Some(doc_id.to_string()),
                },
            )
            .await?,
        );

        self.groups.insert(doc_id.to_string(), group.clone());

        if lock_acquired {
            if let Some(redis_store) = &self.redis_store {
                redis_store.release_lock(&doc_lock_key, &lock_value).await?;
            }
        }

        Ok(group)
    }

    pub async fn cleanup_empty_groups(&self) {
        self.groups.retain(|_, group| {
            let count = group.connection_count();
            count > 0
        });
    }

    pub async fn remove_connection(&self, doc_id: &str) {
        if let Some(group) = self.groups.get(doc_id) {
            let group_clone = group.clone();
            let remaining = group.decrement_connections();

            if remaining == 0 {
                const CLEANUP_DELAY_MS: u64 = 250;
                tokio::time::sleep(tokio::time::Duration::from_millis(CLEANUP_DELAY_MS)).await;

                if group_clone.connection_count() == 0 {
                    self.groups.remove(doc_id);
                }
            }
        }
    }
}
