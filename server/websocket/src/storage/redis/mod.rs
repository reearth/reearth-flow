use anyhow::Result;
use bytes::Bytes;
use deadpool::Runtime;
use deadpool_redis::Config;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};
use uuid;

use crate::{
    RedisConfig, RedisPool, RedisStreamResults, StreamMessages, MESSAGE_TYPE_AWARENESS,
    MESSAGE_TYPE_SYNC, OID_LOCK_KEY,
};

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
        instance_id: &u64,
    ) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let script = redis::Script::new(
            r#"
            local stream_key = KEYS[1]
            local msg_type = ARGV[1]
            local data = ARGV[2]
            local client_id = ARGV[3]
            local timestamp = ARGV[4]
            
            redis.call('XADD', stream_key, '*', 
                'type', msg_type, 
                'data', data, 
                'clientId', client_id, 
                'timestamp', timestamp)
            return 1
            "#,
        );

        let _: () = script
            .key(stream_key)
            .arg(MESSAGE_TYPE_SYNC)
            .arg(update)
            .arg(instance_id)
            .arg(timestamp)
            .invoke_async(&mut *conn)
            .await?;

        Ok(())
    }

    pub async fn publish_update_with_ttl(
        &self,
        stream_key: &str,
        update: &[u8],
        instance_id: &u64,
        ttl: u64,
    ) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let script = redis::Script::new(
            r#"
            local stream_key = KEYS[1]
            local msg_type = ARGV[1]
            local data = ARGV[2]
            local client_id = ARGV[3]
            local timestamp = ARGV[4]
            local ttl = ARGV[5]
            
            redis.call('XADD', stream_key, '*', 
                'type', msg_type, 
                'data', data, 
                'clientId', client_id, 
                'timestamp', timestamp)
            redis.call('EXPIRE', stream_key, ttl)
            return 1
            "#,
        );

        let _: () = script
            .key(stream_key)
            .arg(MESSAGE_TYPE_SYNC)
            .arg(update)
            .arg(instance_id)
            .arg(timestamp)
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
        let key = format!("doc:instance:{doc_id}");
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
        let key = format!("doc:instance:{doc_id}");
        let mut conn = self.pool.get().await?;
        let result: Option<String> = conn.get(&key).await?;
        Ok(result)
    }

    pub async fn read_and_filter(
        &self,
        stream_key: &str,
        count: usize,
        instance_id: &u64,
        last_read_id: &Arc<Mutex<String>>,
    ) -> Result<StreamMessages> {
        let block_ms = 1000;
        let mut conn = self.pool.get().await?;
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
            .query_async(&mut *conn)
            .await?;

        if result.is_empty() || result[0].1.is_empty() {
            return Ok(StreamMessages {
                sync_updates: vec![],
                awareness_updates: vec![],
            });
        }

        let mut sync_updates = Vec::new();
        let mut awareness_updates = Vec::new();
        let mut last_msg_id = String::new();

        for (msg_id, fields) in result[0].1.iter() {
            let mut message_type = String::new();
            let mut client_id = String::new();
            let mut data: Option<Bytes> = None;

            for (field_name, field_value) in fields.iter() {
                match field_name.as_str() {
                    "type" => {
                        if let Ok(type_str) = std::str::from_utf8(field_value) {
                            message_type = type_str.to_string();
                        }
                    }
                    "clientId" => {
                        if let Ok(client_str) = std::str::from_utf8(field_value) {
                            client_id = client_str.to_string();
                        }
                    }
                    "data" => {
                        data = Some(field_value.clone());
                    }
                    _ => {}
                }
            }

            if client_id != instance_id.to_string() {
                if let Some(data) = data {
                    match message_type.as_str() {
                        MESSAGE_TYPE_SYNC => {
                            sync_updates.push(data);
                        }
                        MESSAGE_TYPE_AWARENESS => {
                            awareness_updates.push((client_id, data));
                        }
                        _ => {}
                    }
                }
            }

            last_msg_id = msg_id.clone();
        }

        if !last_msg_id.is_empty() {
            let mut last_id = last_read_id.lock().await;
            *last_id = last_msg_id;
        }

        Ok(StreamMessages {
            sync_updates,
            awareness_updates,
        })
    }

    pub async fn delete_stream(&self, doc_id: &str) -> Result<()> {
        let stream_key = format!("yjs:stream:{doc_id}");
        let mut conn = self.pool.get().await?;
        let _: () = redis::cmd("DEL")
            .arg(&stream_key)
            .query_async(&mut *conn)
            .await?;

        Ok(())
    }

    pub async fn get_stream_last_id(&self, doc_id: &str) -> Result<Option<String>> {
        let stream_key = format!("yjs:stream:{doc_id}");
        let mut conn = self.pool.get().await?;

        // Get the last entry ID from the stream
        type StreamEntry = (String, Vec<(String, Vec<u8>)>);
        let result: Option<Vec<StreamEntry>> = redis::cmd("XREVRANGE")
            .arg(&stream_key)
            .arg("+")
            .arg("-")
            .arg("COUNT")
            .arg(1)
            .query_async(&mut *conn)
            .await?;

        if let Some(entries) = result {
            if !entries.is_empty() {
                return Ok(Some(entries[0].0.clone()));
            }
        }

        Ok(None)
    }

    pub async fn trim_stream_before(&self, doc_id: &str, last_id: &str) -> Result<()> {
        let stream_key = format!("yjs:stream:{doc_id}");
        let mut conn = self.pool.get().await?;

        // Delete all entries before and including the specified ID
        // This keeps only entries after the last saved state
        let script = redis::Script::new(
            r#"
            local stream_key = KEYS[1]
            local last_id = ARGV[1]
            
            -- Get all entries up to and including last_id
            local entries = redis.call('XRANGE', stream_key, '-', last_id)
            
            -- Delete each entry
            for i, entry in ipairs(entries) do
                redis.call('XDEL', stream_key, entry[1])
            end
            
            return #entries
            "#,
        );

        let deleted_count: i32 = script
            .key(&stream_key)
            .arg(last_id)
            .invoke_async(&mut *conn)
            .await?;

        debug!(
            "Trimmed {} entries from stream '{}' after GCS save",
            deleted_count, stream_key
        );
        Ok(())
    }

    pub async fn acquire_doc_lock(&self, doc_id: &str, instance_id: &str) -> Result<bool> {
        let lock_key = format!("lock:doc:{doc_id}");
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
        let lock_key = format!("lock:doc:{doc_id}");

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

    pub async fn update_instance_heartbeat(&self, doc_id: &str, instance_id: &u64) -> Result<()> {
        let key = format!("doc:instances:{doc_id}");
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
        let key = format!("doc:instances:{doc_id}");
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

    pub async fn remove_instance_heartbeat(&self, doc_id: &str, instance_id: &u64) -> Result<bool> {
        let key = format!("doc:instances:{doc_id}");

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
        let stream_key = format!("yjs:stream:{doc_id}");
        let instances_key = format!("doc:instances:{doc_id}");
        let read_lock_key = format!("read:lock:{doc_id}");
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

        let lock_key = format!("lock:doc:{doc_id}");

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
        let stream_key = format!("yjs:stream:{doc_id}");

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

    pub async fn read_all_stream_data(&self, doc_id: &str) -> Result<(Vec<Bytes>, Option<String>)> {
        let stream_key = format!("yjs:stream:{doc_id}");
        let mut conn = self.pool.get().await?;

        let entries: Vec<(String, Vec<(String, bytes::Bytes)>)> = redis::cmd("XRANGE")
            .arg(&stream_key)
            .arg("-")
            .arg("+")
            .query_async(&mut *conn)
            .await?;

        if entries.is_empty() {
            return Ok((Vec::new(), None));
        }

        let mut updates = Vec::new();
        let last_id = entries.last().map(|(id, _)| id.clone());

        for (_entry_id, fields) in entries {
            for (field_name, field_value) in fields {
                if field_name == "data" {
                    updates.push(field_value);
                }
            }
        }

        Ok((updates, last_id))
    }

    pub async fn read_stream_data_in_batches(
        &self,
        doc_id: &str,
        batch_size: usize,
        start_id: &str,
        is_first_batch: bool,
        is_final_batch: bool,
        lock_value: &mut Option<String>,
    ) -> Result<(Vec<Bytes>, String)> {
        let stream_key = format!("yjs:stream:{doc_id}");
        let protection_lock_key = format!("read:lock:{doc_id}");

        if is_first_batch {
            let lock_id = uuid::Uuid::new_v4().to_string();
            let acquired = self
                .acquire_lock(&protection_lock_key, &lock_id, 30)
                .await?;
            if acquired {
                *lock_value = Some(lock_id.clone());
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
            let lock_id = lock_value.clone().unwrap();
            if let Err(e) = self.release_lock(&protection_lock_key, &lock_id).await {
                error!(
                    "Failed to release read lock {} for document '{}': {}",
                    lock_id, doc_id, e
                );
            }
            return Ok((updates, last_id));
        }

        Ok((updates, last_id))
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

    pub async fn publish_awareness(
        &self,
        conn: &mut redis::aio::MultiplexedConnection,
        stream_key: &str,
        awareness_data: &[u8],
        instance_id: &u64,
    ) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let script = redis::Script::new(
            r#"
            local stream_key = KEYS[1]
            local msg_type = ARGV[1]
            local data = ARGV[2]
            local client_id = ARGV[3]
            local timestamp = ARGV[4]
            
            redis.call('XADD', stream_key, '*', 
                'type', msg_type, 
                'data', data, 
                'clientId', client_id, 
                'timestamp', timestamp)
            return 1
            "#,
        );

        let _: () = script
            .key(stream_key)
            .arg(MESSAGE_TYPE_AWARENESS)
            .arg(awareness_data)
            .arg(instance_id)
            .arg(timestamp)
            .invoke_async(&mut *conn)
            .await?;

        Ok(())
    }

    pub async fn trim_stream_by_length(&self, doc_id: &str, max_length: u64) -> Result<u64> {
        let stream_key = format!("yjs:stream:{doc_id}");
        let mut conn = self.pool.get().await?;

        let trimmed_count: u64 = redis::cmd("XTRIM")
            .arg(&stream_key)
            .arg("MAXLEN")
            .arg("~")
            .arg(max_length)
            .query_async(&mut *conn)
            .await?;

        debug!(
            "Trimmed {} entries from stream '{}' by max length {}",
            trimmed_count, stream_key, max_length
        );
        Ok(trimmed_count)
    }

    pub async fn trim_stream_by_min_id(&self, doc_id: &str, min_id: &str) -> Result<u64> {
        let stream_key = format!("yjs:stream:{doc_id}");
        let mut conn = self.pool.get().await?;

        let trimmed_count: u64 = redis::cmd("XTRIM")
            .arg(&stream_key)
            .arg("MINID")
            .arg("~")
            .arg(min_id)
            .query_async(&mut *conn)
            .await?;

        debug!(
            "Trimmed {} entries from stream '{}' by min ID {}",
            trimmed_count, stream_key, min_id
        );
        Ok(trimmed_count)
    }

    pub async fn get_stream_length(&self, doc_id: &str) -> Result<u64> {
        let stream_key = format!("yjs:stream:{doc_id}");
        let mut conn = self.pool.get().await?;

        let length: u64 = redis::cmd("XLEN")
            .arg(&stream_key)
            .query_async(&mut *conn)
            .await?;

        Ok(length)
    }

    pub async fn list_all_streams(&self) -> Result<Vec<String>> {
        let mut conn = self.pool.get().await?;
        let pattern = "yjs:stream:*";

        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut *conn)
            .await?;

        Ok(keys)
    }

    pub async fn trim_streams_comprehensive(
        &self,
        max_message_age_ms: u64,
        max_length: u64,
    ) -> Result<(u64, u64)> {
        let streams = self.list_all_streams().await?;
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let cutoff_time = current_time.saturating_sub(max_message_age_ms);
        let min_id = format!("{}-0", cutoff_time);

        let mut streams_processed = 0u64;
        let mut total_trimmed = 0u64;

        for stream_key in streams {
            if let Some(doc_id) = stream_key.strip_prefix("yjs:stream:") {
                match self.trim_stream_by_min_id(doc_id, &min_id).await {
                    Ok(trimmed) => {
                        total_trimmed += trimmed;
                        if trimmed > 0 {
                            debug!("Trimmed {} old entries from stream '{}'", trimmed, doc_id);
                        }
                    }
                    Err(e) => {
                        error!("Failed to trim stream '{}' by age: {}", doc_id, e);
                        continue;
                    }
                }

                match self.get_stream_length(doc_id).await {
                    Ok(length) if length > max_length => {
                        match self.trim_stream_by_length(doc_id, max_length).await {
                            Ok(trimmed) => {
                                total_trimmed += trimmed;
                                debug!(
                                    "Trimmed {} entries from stream '{}' by length",
                                    trimmed, doc_id
                                );
                            }
                            Err(e) => {
                                error!("Failed to trim stream '{}' by length: {}", doc_id, e);
                                continue;
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        error!("Failed to get length for stream '{}': {}", doc_id, e);
                        continue;
                    }
                }

                streams_processed += 1;
            }
        }

        if streams_processed > 0 {
            info!(
                "Stream trimming completed: processed {} streams, trimmed {} total entries",
                streams_processed, total_trimmed
            );
        }

        Ok((streams_processed, total_trimmed))
    }
}
