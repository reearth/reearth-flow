use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::Duration;

type RedisField = (String, Vec<u8>);
type RedisFields = Vec<RedisField>;
type RedisStreamMessage = (String, RedisFields);
type RedisStreamMessages = Vec<RedisStreamMessage>;
type RedisStreamResult = (String, RedisStreamMessages);
type RedisStreamResults = Vec<RedisStreamResult>;

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub ttl: u64,
}

pub type RedisPool = Pool<RedisConnectionManager>;

#[derive(Debug, Clone)]
pub struct RedisStore {
    pool: Option<Arc<RedisPool>>,
    config: Option<RedisConfig>,
}

impl RedisStore {
    pub fn new(config: Option<RedisConfig>) -> Self {
        Self { pool: None, config }
    }

    pub async fn init(&mut self) -> Result<(), redis::RedisError> {
        if let Some(config) = &self.config {
            let pool = Self::init_redis_connection(&config.url).await?;
            self.pool = Some(pool);
            Ok(())
        } else {
            Err(redis::RedisError::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Redis configuration is missing",
            )))
        }
    }

    pub fn get_pool(&self) -> Option<Arc<RedisPool>> {
        self.pool.clone()
    }

    pub fn get_config(&self) -> Option<RedisConfig> {
        self.config.clone()
    }

    pub async fn init_redis_connection(url: &str) -> Result<Arc<RedisPool>, redis::RedisError> {
        let manager = RedisConnectionManager::new(url)?;
        let pool = Pool::builder()
            .max_size(1024)
            .min_idle(5)
            .connection_timeout(Duration::from_secs(5))
            .idle_timeout(Some(Duration::from_secs(500)))
            .max_lifetime(Some(Duration::from_secs(7200)))
            .build(manager)
            .await
            .map_err(|e| redis::RedisError::from(std::io::Error::other(e)))?;
        Ok(Arc::new(pool))
    }

    pub async fn publish_update(&self, doc_id: &str, update: &[u8]) -> Result<(), anyhow::Error> {
        if let Some(pool) = &self.pool {
            let stream_key = format!("yjs:stream:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
                let fields = &[("update", update)];
                let _: String = redis::cmd("XADD")
                    .arg(&stream_key)
                    .arg("MAXLEN")
                    .arg("~")
                    .arg(1000)
                    .arg("*")
                    .arg(fields)
                    .query_async(&mut *conn)
                    .await?;

                if let Some(config) = &self.config {
                    let _: () = redis::cmd("EXPIRE")
                        .arg(&stream_key)
                        .arg(config.ttl)
                        .query_async(&mut *conn)
                        .await?;
                }
            }
        }
        Ok(())
    }

    pub async fn create_consumer_group(
        &self,
        doc_id: &str,
        group_name: &str,
    ) -> Result<(), anyhow::Error> {
        if let Some(pool) = &self.pool {
            let stream_key = format!("yjs:stream:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
                let result: Result<String, redis::RedisError> = redis::cmd("XGROUP")
                    .arg("CREATE")
                    .arg(&stream_key)
                    .arg(group_name)
                    .arg("0")
                    .arg("MKSTREAM")
                    .query_async(&mut *conn)
                    .await;

                match result {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        if e.to_string().contains("BUSYGROUP") {
                            Ok(())
                        } else {
                            Err(e.into())
                        }
                    }
                }
            } else {
                Err(anyhow::anyhow!("Failed to get Redis connection"))
            }
        } else {
            Err(anyhow::anyhow!("Redis pool is not initialized"))
        }
    }

    pub async fn read_stream_messages(
        &self,
        doc_id: &str,
        group_name: &str,
        consumer_name: &str,
        count: usize,
        block_ms: usize,
    ) -> Result<Vec<(String, Vec<u8>)>, anyhow::Error> {
        if let Some(pool) = &self.pool {
            let stream_key = format!("yjs:stream:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
                let result: RedisStreamResults = redis::cmd("XREADGROUP")
                    .arg("GROUP")
                    .arg(group_name)
                    .arg(consumer_name)
                    .arg("COUNT")
                    .arg(count)
                    .arg("BLOCK")
                    .arg(block_ms)
                    .arg("STREAMS")
                    .arg(&stream_key)
                    .arg(">")
                    .query_async(&mut *conn)
                    .await?;

                let mut updates = Vec::new();
                if !result.is_empty() && !result[0].1.is_empty() {
                    for (msg_id, fields) in &result[0].1 {
                        for (field_name, field_value) in fields {
                            if field_name == "update" {
                                updates.push((msg_id.clone(), field_value.clone()));
                            }
                        }
                    }
                }

                Ok(updates)
            } else {
                Err(anyhow::anyhow!("Failed to get Redis connection"))
            }
        } else {
            Err(anyhow::anyhow!("Redis pool is not initialized"))
        }
    }

    pub async fn ack_message(
        &self,
        doc_id: &str,
        group_name: &str,
        message_id: &str,
    ) -> Result<(), anyhow::Error> {
        if let Some(pool) = &self.pool {
            let stream_key = format!("yjs:stream:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
                let _: () = redis::cmd("XACK")
                    .arg(&stream_key)
                    .arg(group_name)
                    .arg(message_id)
                    .query_async(&mut *conn)
                    .await?;

                Ok(())
            } else {
                Err(anyhow::anyhow!("Failed to get Redis connection"))
            }
        } else {
            Err(anyhow::anyhow!("Redis pool is not initialized"))
        }
    }

    pub async fn read_pending_messages(
        &self,
        doc_id: &str,
        group_name: &str,
        consumer_name: &str,
        count: usize,
    ) -> Result<Vec<(String, Vec<u8>)>, anyhow::Error> {
        if let Some(pool) = &self.pool {
            let stream_key = format!("yjs:stream:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
                let result: RedisStreamResults = redis::cmd("XREADGROUP")
                    .arg("GROUP")
                    .arg(group_name)
                    .arg(consumer_name)
                    .arg("COUNT")
                    .arg(count)
                    .arg("STREAMS")
                    .arg(&stream_key)
                    .arg("0")
                    .query_async(&mut *conn)
                    .await?;

                let mut updates = Vec::new();
                if !result.is_empty() && !result[0].1.is_empty() {
                    for (msg_id, fields) in &result[0].1 {
                        for (field_name, field_value) in fields {
                            if field_name == "update" {
                                updates.push((msg_id.clone(), field_value.clone()));
                            }
                        }
                    }
                }

                Ok(updates)
            } else {
                Err(anyhow::anyhow!("Failed to get Redis connection"))
            }
        } else {
            Err(anyhow::anyhow!("Redis pool is not initialized"))
        }
    }

    pub async fn acquire_lock(
        &self,
        lock_key: &str,
        lock_value: &str,
        ttl_seconds: u64,
    ) -> Result<bool, anyhow::Error> {
        if let Some(pool) = &self.pool {
            if let Ok(mut conn) = pool.get().await {
                let result: Option<String> = redis::cmd("SET")
                    .arg(lock_key)
                    .arg(lock_value)
                    .arg("NX")
                    .arg("EX")
                    .arg(ttl_seconds)
                    .query_async(&mut *conn)
                    .await?;

                return Ok(result.is_some());
            }
        }
        Ok(false)
    }

    pub async fn release_lock(
        &self,
        lock_key: &str,
        lock_value: &str,
    ) -> Result<(), anyhow::Error> {
        if let Some(pool) = &self.pool {
            if let Ok(mut conn) = pool.get().await {
                let script = redis::Script::new(
                    r"
                    if redis.call('get', KEYS[1]) == ARGV[1] then
                        return redis.call('del', KEYS[1])
                    else
                        return 0
                    end
                ",
                );

                let _: () = script
                    .key(lock_key)
                    .arg(lock_value)
                    .invoke_async(&mut *conn)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<(), anyhow::Error> {
        if let Some(pool) = &self.pool {
            if let Ok(mut conn) = pool.get().await {
                let _: () = conn.set(key, value).await?;
            }
        }
        Ok(())
    }

    pub async fn set_with_expiry(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: u64,
    ) -> Result<(), anyhow::Error> {
        if let Some(pool) = &self.pool {
            if let Ok(mut conn) = pool.get().await {
                let _: () = redis::cmd("SET")
                    .arg(key)
                    .arg(value)
                    .arg("EX")
                    .arg(ttl_seconds)
                    .query_async(&mut *conn)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn exists(&self, key: &str) -> Result<bool, anyhow::Error> {
        if let Some(pool) = &self.pool {
            if let Ok(mut conn) = pool.get().await {
                let exists: bool = redis::cmd("EXISTS")
                    .arg(key)
                    .query_async(&mut *conn)
                    .await?;
                return Ok(exists);
            }
        }
        Ok(false)
    }

    pub async fn set_nx(&self, key: &str, value: &str) -> Result<bool, anyhow::Error> {
        if let Some(pool) = &self.pool {
            if let Ok(mut conn) = pool.get().await {
                let result: bool = redis::cmd("SETNX")
                    .arg(key)
                    .arg(value)
                    .query_async(&mut *conn)
                    .await?;
                return Ok(result);
            }
        }
        Ok(false)
    }

    pub async fn set_nx_with_expiry(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: u64,
    ) -> Result<bool, anyhow::Error> {
        if let Some(pool) = &self.pool {
            if let Ok(mut conn) = pool.get().await {
                let result: Option<String> = redis::cmd("SET")
                    .arg(key)
                    .arg(value)
                    .arg("NX")
                    .arg("EX")
                    .arg(ttl_seconds)
                    .query_async(&mut *conn)
                    .await?;

                return Ok(result.is_some());
            }
        }
        Ok(false)
    }

    pub async fn del(&self, key: &str) -> Result<(), anyhow::Error> {
        if let Some(pool) = &self.pool {
            if let Ok(mut conn) = pool.get().await {
                let _: () = redis::cmd("DEL").arg(key).query_async(&mut *conn).await?;
            }
        }
        Ok(())
    }

    pub async fn expire(&self, key: &str, ttl_seconds: u64) -> Result<(), anyhow::Error> {
        if let Some(pool) = &self.pool {
            if let Ok(mut conn) = pool.get().await {
                let _: () = redis::cmd("EXPIRE")
                    .arg(key)
                    .arg(ttl_seconds)
                    .query_async(&mut *conn)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn register_doc_instance(
        &self,
        doc_id: &str,
        instance_id: &str,
        ttl_seconds: u64,
    ) -> Result<bool, anyhow::Error> {
        if let Some(pool) = &self.pool {
            let key = format!("doc:instance:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
                let effective_ttl = if ttl_seconds < 2 { 2 } else { ttl_seconds };
                let result: bool = redis::cmd("SET")
                    .arg(&key)
                    .arg(instance_id)
                    .arg("NX")
                    .arg("EX")
                    .arg(effective_ttl)
                    .query_async(&mut *conn)
                    .await?;

                return Ok(result);
            }
        }
        Ok(false)
    }

    pub async fn get_doc_instance(&self, doc_id: &str) -> Result<Option<String>, anyhow::Error> {
        if let Some(pool) = &self.pool {
            let key = format!("doc:instance:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
                let result: Option<String> = conn.get(&key).await?;
                return Ok(result);
            }
        }
        Ok(None)
    }

    pub async fn refresh_doc_instance(
        &self,
        doc_id: &str,
        instance_id: &str,
        ttl_seconds: u64,
    ) -> Result<(), anyhow::Error> {
        if let Some(pool) = &self.pool {
            let key = format!("doc:instance:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
                let current: Option<String> = conn.get(&key).await?;

                if let Some(current_instance) = current {
                    if current_instance == instance_id {
                        let _: () = redis::cmd("EXPIRE")
                            .arg(&key)
                            .arg(ttl_seconds)
                            .query_async(&mut *conn)
                            .await?;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn release_doc_instance(
        &self,
        doc_id: &str,
        instance_id: &str,
    ) -> Result<(), anyhow::Error> {
        if let Some(pool) = &self.pool {
            let key = format!("doc:instance:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
                let current: Option<String> = conn.get(&key).await?;

                if let Some(current_instance) = current {
                    if current_instance == instance_id {
                        let _: () = redis::cmd("DEL").arg(&key).query_async(&mut *conn).await?;
                    }
                }
            }
        }
        Ok(())
    }
}
