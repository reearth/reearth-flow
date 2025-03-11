use crate::broadcast::group::{BroadcastConfig, BroadcastGroup, RedisConfig};
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::AwarenessRef;
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use redis::AsyncCommands;
use std::sync::Arc;
use uuid;
use yrs::sync::Awareness;
use yrs::{Doc, Transact};

#[derive(Clone, Debug)]
pub struct BroadcastPool {
    store: Arc<GcsStore>,
    redis_config: Option<RedisConfig>,
    groups: DashMap<String, Arc<BroadcastGroup>>,
    buffer_capacity: usize,
    groups_mutex: Arc<tokio::sync::Mutex<()>>,
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_config: Option<RedisConfig>) -> Self {
        Self {
            store,
            redis_config,
            groups: DashMap::new(),
            buffer_capacity: 1024,
            groups_mutex: Arc::new(tokio::sync::Mutex::new(())),
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
            groups_mutex: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    pub fn get_store(&self) -> Arc<GcsStore> {
        self.store.clone()
    }

    pub async fn get_or_create_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        let _local_lock = self.groups_mutex.lock().await;

        if let Some(group) = self.groups.get(doc_id) {
            tracing::debug!(
                "Document '{}' found in local cache after acquiring local lock",
                doc_id
            );

            let mut has_pending_updates = false;

            if let Some(redis_config) = &self.redis_config {
                let redis_key = format!("pending_updates:{}", doc_id);
                let mut retry_count = 0;
                const MAX_RETRIES: usize = 5;

                while retry_count < MAX_RETRIES && !has_pending_updates {
                    if let Ok(manager) = redis::Client::open(redis_config.url.clone()) {
                        if let Ok(mut conn) = manager.get_multiplexed_async_connection().await {
                            match conn.llen::<_, i64>(&redis_key).await {
                                Ok(len) if len > 0 => {
                                    has_pending_updates = true;
                                    tracing::debug!(
                                        "Found {} pending updates for document '{}' in local cache check (attempt {}/{})",
                                        len, doc_id, retry_count + 1, MAX_RETRIES
                                    );
                                    break;
                                }
                                Ok(_) => {
                                    tracing::debug!(
                                        "No pending updates found for document '{}' in local cache check (attempt {}/{})",
                                        doc_id, retry_count + 1, MAX_RETRIES
                                    );
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to check pending updates for document '{}' in local cache check: {} (attempt {}/{})",
                                        doc_id, e, retry_count + 1, MAX_RETRIES
                                    );
                                }
                            }
                        }
                    }

                    retry_count += 1;
                    if retry_count < MAX_RETRIES && !has_pending_updates {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }

                if has_pending_updates || retry_count >= MAX_RETRIES {
                    tracing::debug!(
                        "Document '{}' found in local cache, returning after pending updates check",
                        doc_id
                    );
                    return Ok(group.clone());
                } else {
                    tracing::debug!("Document '{}' found in local cache but no pending updates, continuing to check", doc_id);
                }
            } else {
                tracing::debug!(
                    "Document '{}' found in local cache (no Redis config)",
                    doc_id
                );
                return Ok(group.clone());
            }
        }

        let doc_lock_key = format!("lock:doc:{}", doc_id);
        let lock_value = uuid::Uuid::new_v4().to_string();
        let mut lock_acquired = false;

        if let Some(redis_config) = &self.redis_config {
            if let Ok(manager) = redis::Client::open(redis_config.url.clone()) {
                if let Ok(mut conn) = manager.get_multiplexed_async_connection().await {
                    match redis::cmd("SET")
                        .arg(&doc_lock_key)
                        .arg(&lock_value)
                        .arg("NX")
                        .arg("EX")
                        .arg(10)
                        .query_async::<Option<String>>(&mut conn)
                        .await
                    {
                        Ok(Some(_)) => {
                            lock_acquired = true;
                            tracing::debug!("Acquired Redis lock for document '{}'", doc_id);

                            let redis_key = format!("pending_updates:{}", doc_id);
                            let mut has_pending_updates = false;

                            for attempt in 1..=5 {
                                if let Ok(manager) = redis::Client::open(redis_config.url.clone()) {
                                    if let Ok(mut conn) =
                                        manager.get_multiplexed_async_connection().await
                                    {
                                        match conn.llen::<_, i64>(&redis_key).await {
                                            Ok(len) if len > 0 => {
                                                has_pending_updates = true;
                                                tracing::debug!(
                                                    "Found {} pending updates for document '{}' after {} attempt(s)",
                                                    len, doc_id, attempt
                                                );
                                                break;
                                            }
                                            Ok(_) => {
                                                tracing::debug!(
                                                    "No pending updates found for document '{}', attempt {}/5",
                                                    doc_id, attempt
                                                );
                                            }
                                            Err(e) => {
                                                tracing::warn!(
                                                    "Failed to check pending updates for document '{}': {}",
                                                    doc_id, e
                                                );
                                            }
                                        }
                                    }
                                }

                                if !has_pending_updates && attempt < 5 {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                            }

                            if !has_pending_updates {
                                tracing::debug!(
                                    "No pending updates found for document '{}' after 5 attempts",
                                    doc_id
                                );
                            }
                        }
                        _ => {
                            tracing::debug!(
                                "Failed to acquire Redis lock for document '{}', waiting...",
                                doc_id
                            );
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                            if let Some(group) = self.groups.get(doc_id) {
                                tracing::debug!(
                                    "Document '{}' found in local cache after waiting for lock",
                                    doc_id
                                );
                                return Ok(group.clone());
                            }
                        }
                    }
                }
            }
        }

        let doc_exists_key = format!("doc:exists:{}", doc_id);
        let mut doc_already_exists = false;

        if let Some(redis_config) = &self.redis_config {
            if let Ok(manager) = redis::Client::open(redis_config.url.clone()) {
                if let Ok(mut conn) = manager.get_multiplexed_async_connection().await {
                    match redis::cmd("EXISTS")
                        .arg(&doc_exists_key)
                        .query_async(&mut conn)
                        .await
                    {
                        Ok(exists) => {
                            if exists {
                                doc_already_exists = true;
                                tracing::debug!("Document '{}' already exists in Redis", doc_id);

                                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                                if let Some(group) = self.groups.get(doc_id) {
                                    if lock_acquired {
                                        self.release_redis_lock(&doc_lock_key, &lock_value).await;
                                    }
                                    tracing::debug!(
                                        "Document '{}' found in local cache after waiting",
                                        doc_id
                                    );
                                    return Ok(group.clone());
                                }
                            } else {
                                match redis::cmd("SETNX")
                                    .arg(&doc_exists_key)
                                    .arg("creating")
                                    .query_async(&mut conn)
                                    .await
                                {
                                    Ok(true) => {
                                        tracing::debug!(
                                            "Marked document '{}' as creating in Redis",
                                            doc_id
                                        );
                                        let _: Result<(), _> = redis::cmd("EXPIRE")
                                            .arg(&doc_exists_key)
                                            .arg(30) // 30秒过期时间
                                            .query_async(&mut conn)
                                            .await;
                                    }
                                    Ok(false) => {
                                        doc_already_exists = true;
                                        tracing::debug!(
                                            "Document '{}' is being created by another server",
                                            doc_id
                                        );

                                        tokio::time::sleep(tokio::time::Duration::from_millis(500))
                                            .await;

                                        if let Some(group) = self.groups.get(doc_id) {
                                            if lock_acquired {
                                                self.release_redis_lock(&doc_lock_key, &lock_value)
                                                    .await;
                                            }
                                            tracing::debug!(
                                                "Document '{}' found in local cache after waiting",
                                                doc_id
                                            );
                                            return Ok(group.clone());
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "Failed to mark document as creating in Redis: {}",
                                            e
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to check document existence in Redis: {}", e);
                        }
                    }
                }
            }
        }

        if let Some(group) = self.groups.get(doc_id) {
            if lock_acquired {
                self.release_redis_lock(&doc_lock_key, &lock_value).await;
            }

            tracing::debug!(
                "Document '{}' found in local cache after final check",
                doc_id
            );
            return Ok(group.clone());
        }

        if doc_already_exists {
            tracing::debug!(
                "Attempting to load existing document '{}' from storage",
                doc_id
            );
        } else {
            tracing::debug!("Creating new document '{}'", doc_id);
        }

        let awareness: AwarenessRef = {
            let doc = Doc::new();
            let mut updates_from_redis = Vec::new();

            if let Some(redis_config) = &self.redis_config {
                let redis_key = format!("pending_updates:{}", doc_id);
                let mut retry_count = 0;
                const MAX_RETRIES: usize = 5;

                while retry_count < MAX_RETRIES {
                    if let Ok(manager) = redis::Client::open(redis_config.url.clone()) {
                        if let Ok(mut conn) = manager.get_multiplexed_async_connection().await {
                            match conn.lrange::<_, Vec<Vec<u8>>>(&redis_key, 0, -1).await {
                                Ok(updates) => {
                                    if !updates.is_empty() {
                                        tracing::debug!(
                                            "Found {} pending updates in Redis for document '{}' (attempt {}/{})",
                                            updates.len(),
                                            doc_id,
                                            retry_count + 1,
                                            MAX_RETRIES
                                        );
                                        updates_from_redis = updates;
                                        break;
                                    } else {
                                        tracing::debug!(
                                            "No pending updates found for document '{}' (attempt {}/{})",
                                            doc_id,
                                            retry_count + 1,
                                            MAX_RETRIES
                                        );
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to load pending updates from Redis for document '{}': {} (attempt {}/{})",
                                        doc_id,
                                        e,
                                        retry_count + 1,
                                        MAX_RETRIES
                                    );
                                }
                            }
                        }
                    }

                    retry_count += 1;
                    if retry_count < MAX_RETRIES {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }

                if updates_from_redis.is_empty() {
                    tracing::debug!(
                        "No pending updates found for document '{}' after {} attempts",
                        doc_id,
                        MAX_RETRIES
                    );
                }
            }

            {
                let mut txn = doc.transact_mut();

                match self.store.load_doc(doc_id, &mut txn).await {
                    Ok(_) => {
                        tracing::debug!("Successfully loaded existing document: {}", doc_id);
                    }
                    Err(e) => {
                        if e.to_string().contains("not found") {
                            tracing::debug!("Creating new document: {}", doc_id);

                            if let Some(redis_config) = &self.redis_config {
                                if let Ok(manager) = redis::Client::open(redis_config.url.clone()) {
                                    if let Ok(mut conn) =
                                        manager.get_multiplexed_async_connection().await
                                    {
                                        let _: Result<(), _> = redis::cmd("SET")
                                            .arg(&doc_exists_key)
                                            .arg("created")
                                            .query_async(&mut conn)
                                            .await;
                                    }
                                }
                            }
                        } else {
                            tracing::error!("Failed to load document {}: {}", doc_id, e);

                            if let Some(redis_config) = &self.redis_config {
                                if let Ok(manager) = redis::Client::open(redis_config.url.clone()) {
                                    if let Ok(mut conn) =
                                        manager.get_multiplexed_async_connection().await
                                    {
                                        let _: Result<(), _> = redis::cmd("DEL")
                                            .arg(&doc_exists_key)
                                            .query_async(&mut conn)
                                            .await;
                                    }
                                }
                            }

                            if lock_acquired {
                                self.release_redis_lock(&doc_lock_key, &lock_value).await;
                            }

                            return Err(anyhow!("Failed to load document: {}", e));
                        }
                    }
                }

                if !updates_from_redis.is_empty() {
                    tracing::debug!(
                        "Applying {} pending updates to document '{}'",
                        updates_from_redis.len(),
                        doc_id
                    );
                    for (i, update) in updates_from_redis.iter().enumerate() {
                        if let Ok(decoded) = yrs::updates::decoder::Decode::decode_v1(update) {
                            if let Err(e) = txn.apply_update(decoded) {
                                tracing::warn!(
                                    "Failed to apply update {}/{} from Redis for document '{}': {}",
                                    i + 1,
                                    updates_from_redis.len(),
                                    doc_id,
                                    e
                                );
                            } else {
                                tracing::debug!(
                                    "Successfully applied update {}/{} to document '{}'",
                                    i + 1,
                                    updates_from_redis.len(),
                                    doc_id
                                );
                            }
                        } else {
                            tracing::warn!(
                                "Failed to decode update {}/{} for document '{}'",
                                i + 1,
                                updates_from_redis.len(),
                                doc_id
                            );
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

        self.groups.insert(doc_id.to_string(), group.clone());
        tracing::info!(
            "Successfully created and cached document group for '{}'",
            doc_id
        );

        if lock_acquired {
            self.release_redis_lock(&doc_lock_key, &lock_value).await;
        }

        Ok(group)
    }

    async fn release_redis_lock(&self, lock_key: &str, lock_value: &str) {
        if let Some(redis_config) = &self.redis_config {
            if let Ok(manager) = redis::Client::open(redis_config.url.clone()) {
                if let Ok(mut conn) = manager.get_multiplexed_async_connection().await {
                    let script = redis::Script::new(
                        r"
                        if redis.call('get', KEYS[1]) == ARGV[1] then
                            return redis.call('del', KEYS[1])
                        else
                            return 0
                        end
                    ",
                    );

                    match script
                        .key(lock_key)
                        .arg(lock_value)
                        .invoke_async::<()>(&mut conn)
                        .await
                    {
                        Ok(_) => {
                            tracing::debug!("Released Redis lock for key '{}'", lock_key);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to release Redis lock for key '{}': {}",
                                lock_key,
                                e
                            );
                        }
                    }
                }
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
                "Connection disconnected for document '{}', updates will be flushed in decrement_connections",
                doc_id
            );

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
