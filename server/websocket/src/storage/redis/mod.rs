use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::Duration;

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
            let channel = format!("yjs:updates:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
                let _: () = redis::cmd("PUBLISH")
                    .arg(&channel)
                    .arg(update)
                    .query_async(&mut *conn)
                    .await?;
            }
        }
        Ok(())
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
