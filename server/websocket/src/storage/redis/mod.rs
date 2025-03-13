use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::Duration;

pub mod pubsub;

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

    pub async fn has_pending_updates(&self, doc_id: &str) -> Result<bool, anyhow::Error> {
        if let Some(pool) = &self.pool {
            let redis_key = format!("pending_updates:{}", doc_id);
            let mut retry_count = 0;
            const MAX_RETRIES: usize = 5;

            while retry_count < MAX_RETRIES {
                if let Ok(mut conn) = pool.get().await {
                    match conn.llen::<_, i64>(&redis_key).await {
                        Ok(len) if len > 0 => {
                            return Ok(true);
                        }
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }

                retry_count += 1;
                if retry_count < MAX_RETRIES {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
        Ok(false)
    }

    pub async fn get_pending_updates(&self, doc_id: &str) -> Result<Vec<Vec<u8>>, anyhow::Error> {
        let mut updates = Vec::new();

        if let Some(pool) = &self.pool {
            let redis_key = format!("pending_updates:{}", doc_id);
            let mut retry_count = 0;
            const MAX_RETRIES: usize = 5;

            while retry_count < MAX_RETRIES {
                if let Ok(mut conn) = pool.get().await {
                    if let Ok(result) = conn.lrange::<_, Vec<Vec<u8>>>(&redis_key, 0, -1).await {
                        if !result.is_empty() {
                            updates = result;
                            break;
                        }
                    }
                }

                retry_count += 1;
                if retry_count < MAX_RETRIES {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }

        Ok(updates)
    }

    pub async fn clear_pending_updates(&self, doc_id: &str) -> Result<(), anyhow::Error> {
        if let Some(pool) = &self.pool {
            let redis_key = format!("pending_updates:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
                let _: () = redis::cmd("DEL")
                    .arg(&redis_key)
                    .query_async(&mut *conn)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn add_update(&self, doc_id: &str, update: &[u8]) -> Result<(), anyhow::Error> {
        if let Some(pool) = &self.pool {
            let redis_key = format!("pending_updates:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
                let _: () = conn.rpush(&redis_key, update).await?;

                if let Some(config) = &self.config {
                    let _: () = conn.expire(&redis_key, config.ttl as i64).await?;
                }
            }
        }
        Ok(())
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

    pub async fn add_publish_update(
        &self,
        doc_id: &str,
        update: &[u8],
        ttl_seconds: u64,
    ) -> Result<(), anyhow::Error> {
        if self.pool.is_none() {
            return Ok(());
        }
        let pool = self.pool.as_ref().unwrap();

        let redis_key = format!("pending_updates:{}", doc_id);
        let channel = format!("yjs:updates:{}", doc_id);

        let mut conn = pool.get().await?;

        let mut pipe = redis::pipe();
        pipe.atomic()
            .cmd("LPUSH")
            .arg(&redis_key)
            .arg(update)
            .cmd("EXPIRE")
            .arg(&redis_key)
            .arg(ttl_seconds as i64)
            .cmd("PUBLISH")
            .arg(&channel)
            .arg(update);

        let _: () = pipe.query_async(&mut *conn).await?;
        Ok(())
    }
}
