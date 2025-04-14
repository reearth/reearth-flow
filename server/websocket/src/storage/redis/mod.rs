use anyhow::Result;
use bytes::Bytes;
use deadpool::Runtime;
use deadpool_redis::{Config, Pool};
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};
use uuid;

type RedisField = (String, Bytes);
type RedisFields = Vec<RedisField>;
type RedisStreamMessage = (String, RedisFields);
type RedisStreamMessages = Vec<RedisStreamMessage>;
type RedisStreamResult = (String, RedisStreamMessages);
type RedisStreamResults = Vec<RedisStreamResult>;

const OID_LOCK_KEY: &str = "lock:oid_generation";

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub ttl: u64,
}

pub type RedisPool = Pool;

#[derive(Debug, Clone)]
pub struct RedisStore {
    pool: Arc<RedisPool>,
    config: RedisConfig,
}

impl RedisStore {
    pub async fn new(config: RedisConfig) -> Result<Self> {
        let cfg = Config::from_url(&config.url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        let pool = Arc::new(pool);
        Ok(Self { pool, config })
    }

    pub fn get_pool(&self) -> Arc<RedisPool> {
        self.pool.clone()
    }

    pub fn get_config(&self) -> RedisConfig {
        self.config.clone()
    }

    pub async fn create_dedicated_connection(&self) -> Result<redis::aio::MultiplexedConnection> {
        let client = redis::Client::open(self.config.url.clone())?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(conn)
    }

    pub async fn publish_update(
        &self,
        conn: &mut redis::aio::MultiplexedConnection,
        stream_key: &str,
        update: &[u8],
        instance_id: &str,
    ) -> Result<()> {
        let script = redis::Script::new(
            r#"
            local stream_key = KEYS[1]
            local update = ARGV[1]
            local instance_id = ARGV[2]
            
            redis.call('XADD', stream_key, '*', 'instance_id', update)
            return 1
            "#,
        );

        let _: () = script
            .key(stream_key)
            .arg(update)
            .arg(instance_id)
            .invoke_async(&mut *conn)
            .await?;

        Ok(())
    }

    pub async fn publish_update_with_ttl(
        &self,
        conn: &mut redis::aio::MultiplexedConnection,
        stream_key: &str,
        update: &[u8],
        instance_id: &str,
        ttl: u64,
    ) -> Result<()> {
        let script = redis::Script::new(
            r#"
            local stream_key = KEYS[1]
            local update = ARGV[1]
            local instance_id = ARGV[2]
            local ttl = ARGV[3]
            redis.call('XADD', stream_key, '*', 'instance_id', update)
            redis.call('EXPIRE', stream_key, ttl)
            return 1
            "#,
        );
        let _: () = script
            .key(stream_key)
            .arg(update)
            .arg(instance_id)
            .arg(ttl)
            .invoke_async(&mut *conn)
            .await?;

        Ok(())
    }

    pub async fn acquire_lock(
        &self,
        lock_key: &str,
        lock_value: &str,
        ttl_seconds: u64,
    ) -> Result<bool> {
        let mut conn = self.pool.get().await?;
        let result: Option<String> = redis::cmd("SET")
            .arg(lock_key)
            .arg(lock_value)
            .arg("NX")
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut *conn)
            .await?;

        Ok(result.is_some())
    }

    pub async fn release_lock(&self, lock_key: &str, lock_value: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
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

        Ok(())
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let _: () = conn.set(key, value).await?;

        Ok(())
    }

    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.pool.get().await?;
        let exists: bool = redis::cmd("EXISTS")
            .arg(key)
            .query_async(&mut *conn)
            .await?;
        Ok(exists)
    }

    pub async fn set_nx(&self, key: &str, value: &str) -> Result<bool> {
        let mut conn = self.pool.get().await?;
        let result: bool = redis::cmd("SETNX")
            .arg(key)
            .arg(value)
            .query_async(&mut *conn)
            .await?;
        Ok(result)
    }

    pub async fn del(&self, key: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let _: () = redis::cmd("DEL").arg(key).query_async(&mut *conn).await?;

        Ok(())
    }

    pub async fn expire(&self, key: &str, ttl_seconds: u64) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let _: () = redis::cmd("EXPIRE")
            .arg(key)
            .arg(ttl_seconds)
            .query_async(&mut *conn)
            .await?;

        Ok(())
    }

    pub async fn register_doc_instance(
        &self,
        doc_id: &str,
        instance_id: &str,
        ttl_seconds: u64,
    ) -> Result<bool> {
        let key = format!("doc:instance:{}", doc_id);
        let mut conn = self.pool.get().await?;
        let effective_ttl = if ttl_seconds < 2 { 2 } else { ttl_seconds };
        let result: bool = redis::cmd("SET")
            .arg(&key)
            .arg(instance_id)
            .arg("NX")
            .arg("EX")
            .arg(effective_ttl)
            .query_async(&mut *conn)
            .await?;

        Ok(result)
    }

    pub async fn get_doc_instance(&self, doc_id: &str) -> Result<Option<String>> {
        let key = format!("doc:instance:{}", doc_id);
        let mut conn = self.pool.get().await?;
        let result: Option<String> = conn.get(&key).await?;
        Ok(result)
    }

    pub async fn read_and_filter(
        &self,
        conn: &mut redis::aio::MultiplexedConnection,
        stream_key: &str,
        count: usize,
        instance_id: &str,
        last_read_id: &Arc<Mutex<String>>,
    ) -> Result<Vec<Bytes>> {
        let block_ms = 1600;

        let read_id = {
            let last_id = last_read_id.lock().await;
            last_id.clone()
        };

        let result: RedisStreamResults = redis::cmd("XREAD")
            .arg("COUNT")
            .arg(count)
            .arg("BLOCK")
            .arg(block_ms)
            .arg("STREAMS")
            .arg(stream_key)
            .arg(read_id)
            .query_async(conn)
            .await?;

        if result.is_empty() || result[0].1.is_empty() {
            return Ok(vec![]);
        }

        let mut updates = Vec::with_capacity(result[0].1.len());
        let mut last_msg_id = String::new();

        for (msg_id, fields) in result[0].1.iter() {
            if let Some((_, value)) = fields.iter().find(|(name, _)| name != instance_id) {
                updates.push(value.clone());
            }
            last_msg_id = msg_id.clone();
        }

        if !last_msg_id.is_empty() {
            let mut last_id = last_read_id.lock().await;
            *last_id = last_msg_id;
        }

        Ok(updates)
    }

    pub async fn delete_stream(&self, doc_id: &str) -> Result<()> {
        let stream_key = format!("yjs:stream:{}", doc_id);
        let mut conn = self.pool.get().await?;
        let _: () = redis::cmd("DEL")
            .arg(&stream_key)
            .query_async(&mut *conn)
            .await?;

        Ok(())
    }

    pub async fn acquire_doc_lock(&self, doc_id: &str, instance_id: &str) -> Result<bool> {
        let lock_key = format!("lock:doc:{}", doc_id);
        let ttl = 10;

        let mut conn = self.pool.get().await?;
        let result: Option<String> = redis::cmd("SET")
            .arg(&lock_key)
            .arg(instance_id)
            .arg("NX")
            .arg("EX")
            .arg(ttl)
            .query_async(&mut *conn)
            .await?;

        Ok(result.is_some())
    }

    pub async fn release_doc_lock(&self, doc_id: &str, instance_id: &str) -> Result<bool> {
        let lock_key = format!("lock:doc:{}", doc_id);

        let mut conn = self.pool.get().await?;
        let script = redis::Script::new(
            r#"
            if redis.call('get', KEYS[1]) == ARGV[1] then
                return redis.call('del', KEYS[1])
            else
                return 0
            end
        "#,
        );

        let result: i32 = script
            .key(&lock_key)
            .arg(instance_id)
            .invoke_async(&mut *conn)
            .await?;

        Ok(result == 1)
    }

    pub async fn update_instance_heartbeat(&self, doc_id: &str, instance_id: &str) -> Result<()> {
        let key = format!("doc:instances:{}", doc_id);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut conn = self.pool.get().await?;
        let script = redis::Script::new(
            r#"
            redis.call('HSET', KEYS[1], ARGV[1], ARGV[2])
            return redis.call('EXPIRE', KEYS[1], ARGV[3])
            "#,
        );

        let _: () = script
            .key(&key)
            .arg(instance_id)
            .arg(timestamp)
            .arg(120)
            .invoke_async(&mut *conn)
            .await?;

        Ok(())
    }

    pub async fn get_active_instances(&self, doc_id: &str, timeout_secs: u64) -> Result<i64> {
        let key = format!("doc:instances:{}", doc_id);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut conn = self.pool.get().await?;
        let script = redis::Script::new(
            r#"
            local active_count = 0
            local instances = redis.call('HGETALL', KEYS[1])
            local now = tonumber(ARGV[1])
            local timeout = tonumber(ARGV[2])
            
            for i = 1, #instances, 2 do
                local instance_id = instances[i]
                local last_seen = tonumber(instances[i+1])
                if now - last_seen < timeout then
                    active_count = active_count + 1
                end
            end
            
            return active_count
        "#,
        );

        let count: i64 = script
            .key(&key)
            .arg(now)
            .arg(timeout_secs)
            .invoke_async(&mut *conn)
            .await?;

        Ok(count)
    }

    pub async fn remove_instance_heartbeat(&self, doc_id: &str, instance_id: &str) -> Result<bool> {
        let key = format!("doc:instances:{}", doc_id);

        let mut conn = self.pool.get().await?;

        let script = redis::Script::new(
            r#"
            redis.call('HDEL', KEYS[1], ARGV[1])
            local count = redis.call('HLEN', KEYS[1])
            if count == 0 then
                    redis.call('DEL', KEYS[1])
                    return 1
                else
                    return 0
                end
                "#,
        );

        let is_empty: i32 = script
            .key(&key)
            .arg(instance_id)
            .invoke_async(&mut *conn)
            .await?;

        Ok(is_empty == 1)
    }

    pub async fn safe_delete_stream(&self, doc_id: &str, instance_id: &str) -> Result<()> {
        println!("safe_delete_stream: {:?}", doc_id);
        let stream_key = format!("yjs:stream:{}", doc_id);
        let instances_key = format!("doc:instances:{}", doc_id);
        let read_lock_key = format!("read:lock:{}", doc_id);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut conn = self.pool.get().await?;

        let read_lock_exists: bool = redis::cmd("EXISTS")
            .arg(&read_lock_key)
            .query_async(&mut *conn)
            .await?;

        if read_lock_exists {
            info!(
                "Read operation in progress for '{}', skipping stream deletion",
                doc_id
            );
            return Ok(());
        }

        let script = redis::Script::new(
            r#"
            local lock_key = KEYS[1]
            local instances_key = KEYS[2]
            local stream_key = KEYS[3]
            local read_lock_key = KEYS[4]
            
            local instance_id = ARGV[1]
            local now = tonumber(ARGV[2])
            local timeout = tonumber(ARGV[3])
            
            if redis.call('EXISTS', read_lock_key) == 1 then
                return {acquired=0, deleted=0, reason="read_in_progress"}
            end
            
            if redis.call('GET', lock_key) ~= instance_id then
                if redis.call('SET', lock_key, instance_id, 'NX', 'EX', 10) == false then
                    return {acquired=0, deleted=0, reason="lock_failed"}
                end
            end
            
            local active_count = 0
            local instances = redis.call('HGETALL', instances_key)
            
            for i = 1, #instances, 2 do
                local inst_id = instances[i]
                local last_seen = tonumber(instances[i+1])
                if now - last_seen < timeout then
                    active_count = active_count + 1
                end
            end
            
            if active_count <= 0 then
                local exists = redis.call('EXISTS', stream_key)
                if exists == 1 then
                    redis.call('DEL', stream_key)
                    return {acquired=1, deleted=1, reason="success"}
                else
                    return {acquired=1, deleted=0, reason="stream_not_exists"}
                end
            else
                return {acquired=1, deleted=0, reason="active_instances", count=active_count}
            end
            "#,
        );

        let lock_key = format!("lock:doc:{}", doc_id);

        let _: redis::Value = script
            .key(&lock_key)
            .key(&instances_key)
            .key(&stream_key)
            .key(&read_lock_key)
            .arg(instance_id)
            .arg(now)
            .arg(60)
            .invoke_async(&mut *conn)
            .await?;

        let _ = self.release_doc_lock(doc_id, instance_id).await;

        Ok(())
    }

    pub async fn check_stream_exists(&self, doc_id: &str) -> Result<bool> {
        let stream_key = format!("yjs:stream:{}", doc_id);

        let mut conn = self.pool.get().await?;
        let exists: bool = redis::cmd("EXISTS")
            .arg(&stream_key)
            .query_async(&mut *conn)
            .await?;

        if exists {
            debug!("Redis stream '{}' exists", stream_key);
        } else {
            debug!("Redis stream '{}' does not exist", stream_key);
        }

        Ok(exists)
    }

    pub async fn read_all_stream_data(&self, doc_id: &str) -> Result<Vec<Bytes>> {
        let stream_key = format!("yjs:stream:{}", doc_id);

        let mut conn = self.pool.get().await?;
        let script = redis::Script::new(
            r#"
            if redis.call('EXISTS', KEYS[1]) == 0 then
                return {}
            end
            
            local result = redis.call('XRANGE', KEYS[1], '-', '+')
            local updates = {}
            
            for i, entry in ipairs(result) do
                local fields = entry[2]
                for j = 1, #fields, 2 do
                    table.insert(updates, fields[j+1])
                end
            end
            
            return updates
        "#,
        );

        let updates: Vec<Bytes> = script.key(&stream_key).invoke_async(&mut *conn).await?;

        Ok(updates)
    }

    pub async fn read_stream_data_in_batches(
        &self,
        doc_id: &str,
        batch_size: usize,
        start_id: &str,
        is_first_batch: bool,
        is_final_batch: bool,
    ) -> Result<(Vec<Bytes>, String, Option<String>)> {
        let stream_key = format!("yjs:stream:{}", doc_id);
        let protection_lock_key = format!("read:lock:{}", doc_id);
        let mut lock_value = None;

        if is_first_batch {
            let lock_id = uuid::Uuid::new_v4().to_string();
            let acquired = self
                .acquire_lock(&protection_lock_key, &lock_id, 30)
                .await?;
            if acquired {
                lock_value = Some(lock_id);
                debug!("Acquired read lock for document '{}'", doc_id);
            } else {
                debug!("Failed to acquire read lock for document '{}'", doc_id);
            }
        }

        let mut conn = self.pool.get().await?;

        let script = redis::Script::new(
            r#"
            if redis.call('EXISTS', KEYS[1]) == 0 then
                return {updates={}, last_id=""}
            end
            
            local result = redis.call('XRANGE', KEYS[1], ARGV[1], '+', 'COUNT', ARGV[2])
            local updates = {}
            local last_id = ARGV[1]
            
            if #result > 0 then
                last_id = result[#result][1]
                
                for i, entry in ipairs(result) do
                    local fields = entry[2]
                    for j = 1, #fields, 2 do
                        table.insert(updates, fields[j+1])
                    end
                end
            end
            
            return {updates, last_id}
            "#,
        );

        let result: redis::Value = script
            .key(&stream_key)
            .arg(start_id)
            .arg(batch_size)
            .invoke_async(&mut *conn)
            .await?;

        let mut updates = Vec::new();
        let mut last_id = start_id.to_string();

        if let redis::Value::Array(array) = result {
            if array.len() >= 2 {
                if let redis::Value::Array(entries) = &array[0] {
                    for entry in entries {
                        if let redis::Value::BulkString(bytes) = entry {
                            updates.push(Bytes::from(bytes.clone()));
                        }
                    }
                }

                if let redis::Value::BulkString(id_bytes) = &array[1] {
                    if let Ok(id_str) = std::str::from_utf8(id_bytes) {
                        last_id = id_str.to_string();
                    }
                }
            }
        }

        if is_final_batch && lock_value.is_some() {
            if let Err(e) = self
                .release_lock(&protection_lock_key, &lock_value.unwrap())
                .await
            {
                error!(
                    "Failed to release read lock for document '{}': {}",
                    doc_id, e
                );
            }
            info!("Released read lock for document '{}'", doc_id);
            return Ok((updates, last_id, None));
        }

        Ok((updates, last_id, lock_value))
    }

    pub async fn acquire_oid_lock(&self, ttl_seconds: u64) -> Result<String> {
        let lock_value = uuid::Uuid::new_v4().to_string();
        let mut conn = self.pool.get().await?;

        let script = redis::Script::new(
            r#"
            local result = redis.call('SET', KEYS[1], ARGV[1], 'NX', 'EX', ARGV[2])
            if result then
                return ARGV[1]
            else
                return false
            end
            "#,
        );

        let result: Option<String> = script
            .key(OID_LOCK_KEY)
            .arg(&lock_value)
            .arg(ttl_seconds)
            .invoke_async(&mut *conn)
            .await?;

        if let Some(val) = result {
            Ok(val)
        } else {
            Err(anyhow::anyhow!("Failed to acquire OID generation lock"))
        }
    }

    pub async fn release_oid_lock(&self, lock_value: &str) -> Result<bool> {
        let mut conn = self.pool.get().await?;

        let script = redis::Script::new(
            r#"
            if redis.call('get', KEYS[1]) == ARGV[1] then
                return redis.call('del', KEYS[1])
            else
                return 0
            end
            "#,
        );

        let result: i32 = script
            .key(OID_LOCK_KEY)
            .arg(lock_value)
            .invoke_async(&mut *conn)
            .await?;

        Ok(result == 1)
    }
}
