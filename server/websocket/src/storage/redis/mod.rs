use bytes::Bytes;
use deadpool::Runtime;
use deadpool_redis::{Config, Connection, Pool};
use redis::AsyncCommands;
use std::sync::Arc;
use tracing::{debug, warn};

type RedisField = (String, Bytes);
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

pub type RedisPool = Pool;

#[derive(Debug, Clone)]
pub struct RedisStore {
    pool: Arc<RedisPool>,
    config: RedisConfig,
}

impl RedisStore {
    pub async fn new(config: RedisConfig) -> Result<Self, anyhow::Error> {
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

    pub async fn publish_update(
        &self,
        stream_key: &str,
        update: &[u8],
        conn: &mut Connection,
    ) -> Result<(), anyhow::Error> {
        let script = redis::Script::new(
            r#"
            local stream_key = KEYS[1]
            local update = ARGV[1]
            
            redis.call('XADD', stream_key, '*', 'update', update)
            return 1
            "#,
        );

        let _: () = script
            .key(stream_key)
            .arg(update)
            .invoke_async(&mut *conn)
            .await?;

        Ok(())
    }

    pub async fn create_consumer_group(
        &self,
        doc_id: &str,
        group_name: &str,
    ) -> Result<(), anyhow::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);
        if let Ok(mut conn) = self.pool.get().await {
            let script = redis::Script::new(
                r#"
                local stream_key = KEYS[1]
                local group_name = ARGV[1]
                local ttl = ARGV[2]
                
                local ok, err = pcall(function()
                    redis.call('XGROUP', 'CREATE', stream_key, group_name, '0', 'MKSTREAM')
                end)
                
                if ok or (not ok and string.find(err, "BUSYGROUP")) then
                    redis.call('EXPIRE', stream_key, ttl)
                    return "OK"
                else
                    return {err=err}
                end
                "#,
            );

            let _: () = script
                .key(&stream_key)
                .arg(group_name)
                .arg(self.config.ttl)
                .invoke_async(&mut *conn)
                .await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to get Redis connection"))
        }
    }

    pub async fn read_stream_messages(
        &self,
        doc_id: &str,
        group_name: &str,
        consumer_name: &str,
        count: usize,
        block_ms: usize,
    ) -> Result<Vec<(String, Bytes)>, anyhow::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);
        if let Ok(mut conn) = self.pool.get().await {
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
    }

    pub async fn batch_ack_messages(
        &self,
        doc_id: &str,
        group_name: &str,
        message_ids: &[String],
    ) -> Result<(), anyhow::Error> {
        if message_ids.is_empty() {
            return Ok(());
        }

        let stream_key = format!("yjs:stream:{}", doc_id);
        if let Ok(mut conn) = self.pool.get().await {
            let mut pipe = redis::pipe();

            let mut cmd = redis::cmd("XACK");
            cmd.arg(&stream_key).arg(group_name);

            for id in message_ids {
                cmd.arg(id);
            }

            pipe.add_command(cmd);

            let _: () = pipe.query_async(&mut *conn).await?;
            return Ok(());
        }
        Err(anyhow::anyhow!("Failed to get Redis connection"))
    }

    pub async fn read_batch_messages(
        &self,
        doc_id: &str,
        group_name: &str,
        consumer_name: &str,
        count: usize,
        block_ms: usize,
    ) -> Result<Vec<(String, Bytes)>, anyhow::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);
        if let Ok(mut conn) = self.pool.get().await {
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
            let mut message_ids = Vec::new();

            if !result.is_empty() && !result[0].1.is_empty() {
                for (msg_id, fields) in &result[0].1 {
                    message_ids.push(msg_id.clone());

                    for (field_name, field_value) in fields {
                        if field_name == "update" {
                            updates.push((msg_id.clone(), field_value.clone()));
                        }
                    }
                }

                if !message_ids.is_empty() {
                    let mut pipe = redis::pipe();
                    let mut cmd = redis::cmd("XACK");
                    cmd.arg(&stream_key).arg(group_name);

                    for id in &message_ids {
                        cmd.arg(id);
                    }

                    pipe.add_command(cmd);
                    let _: () = pipe.query_async(&mut *conn).await?;
                }
            }

            Ok(updates)
        } else {
            Err(anyhow::anyhow!("Failed to get Redis connection"))
        }
    }

    pub async fn ack_message(
        &self,
        doc_id: &str,
        group_name: &str,
        message_id: &str,
    ) -> Result<(), anyhow::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);
        if let Ok(mut conn) = self.pool.get().await {
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
    }

    pub async fn read_pending_messages(
        &self,
        doc_id: &str,
        group_name: &str,
        consumer_name: &str,
        count: usize,
    ) -> Result<Vec<(String, Bytes)>, anyhow::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);
        if let Ok(mut conn) = self.pool.get().await {
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
    }

    pub async fn acquire_lock(
        &self,
        lock_key: &str,
        lock_value: &str,
        ttl_seconds: u64,
    ) -> Result<bool, anyhow::Error> {
        if let Ok(mut conn) = self.pool.get().await {
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
        Ok(false)
    }

    pub async fn release_lock(
        &self,
        lock_key: &str,
        lock_value: &str,
    ) -> Result<(), anyhow::Error> {
        if let Ok(mut conn) = self.pool.get().await {
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
        Ok(())
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<(), anyhow::Error> {
        if let Ok(mut conn) = self.pool.get().await {
            let _: () = conn.set(key, value).await?;
        }
        Ok(())
    }

    pub async fn exists(&self, key: &str) -> Result<bool, anyhow::Error> {
        if let Ok(mut conn) = self.pool.get().await {
            let exists: bool = redis::cmd("EXISTS")
                .arg(key)
                .query_async(&mut *conn)
                .await?;
            return Ok(exists);
        }
        Ok(false)
    }

    pub async fn set_nx(&self, key: &str, value: &str) -> Result<bool, anyhow::Error> {
        if let Ok(mut conn) = self.pool.get().await {
            let result: bool = redis::cmd("SETNX")
                .arg(key)
                .arg(value)
                .query_async(&mut *conn)
                .await?;
            return Ok(result);
        }
        Ok(false)
    }

    pub async fn set_nx_with_expiry(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: u64,
    ) -> Result<bool, anyhow::Error> {
        if let Ok(mut conn) = self.pool.get().await {
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
        Ok(false)
    }

    pub async fn del(&self, key: &str) -> Result<(), anyhow::Error> {
        if let Ok(mut conn) = self.pool.get().await {
            let _: () = redis::cmd("DEL").arg(key).query_async(&mut *conn).await?;
        }
        Ok(())
    }

    pub async fn expire(&self, key: &str, ttl_seconds: u64) -> Result<(), anyhow::Error> {
        if let Ok(mut conn) = self.pool.get().await {
            let _: () = redis::cmd("EXPIRE")
                .arg(key)
                .arg(ttl_seconds)
                .query_async(&mut *conn)
                .await?;
        }
        Ok(())
    }

    pub async fn register_doc_instance(
        &self,
        doc_id: &str,
        instance_id: &str,
        ttl_seconds: u64,
    ) -> Result<bool, anyhow::Error> {
        let key = format!("doc:instance:{}", doc_id);
        if let Ok(mut conn) = self.pool.get().await {
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
        Ok(false)
    }

    pub async fn get_doc_instance(&self, doc_id: &str) -> Result<Option<String>, anyhow::Error> {
        let key = format!("doc:instance:{}", doc_id);
        if let Ok(mut conn) = self.pool.get().await {
            let result: Option<String> = conn.get(&key).await?;
            return Ok(result);
        }
        Ok(None)
    }

    pub async fn read_and_ack(
        &self,
        conn: &mut Connection,
        stream_key: &str,
        group_name: &str,
        consumer_name: &str,
        count: usize,
    ) -> Result<Vec<Bytes>, anyhow::Error> {
        let block_ms = 1600;

        let result: RedisStreamResults = redis::cmd("XREADGROUP")
            .arg("GROUP")
            .arg(group_name)
            .arg(consumer_name)
            .arg("COUNT")
            .arg(count)
            .arg("BLOCK")
            .arg(block_ms)
            .arg("STREAMS")
            .arg(stream_key)
            .arg(">")
            .query_async(&mut *conn)
            .await?;

        if result.is_empty() || result[0].1.is_empty() {
            return Ok(vec![]);
        }

        let mut updates = Vec::with_capacity(result[0].1.len());
        let mut message_ids = Vec::with_capacity(result[0].1.len());

        for (msg_id, fields) in result[0].1.iter() {
            message_ids.push(msg_id.clone());
            if let Some((_, update)) = fields.iter().find(|(name, _)| name == "update") {
                updates.push(update.clone());
            }
        }

        if !message_ids.is_empty() {
            let lua_script = r#"
            local key = KEYS[1]
            local group = ARGV[1]
            local result = 0
            for i=2, #ARGV do
                result = result + redis.call('XACK', key, group, ARGV[i])
            end
            return result
            "#;

            let script = redis::Script::new(lua_script);

            let mut cmd = script.prepare_invoke();
            cmd.key(stream_key);
            cmd.arg(group_name);

            for msg_id in &message_ids {
                cmd.arg(msg_id);
            }

            let _: i64 = cmd.invoke_async(&mut *conn).await?;
        }
        Ok(updates)
    }

    pub async fn delete_stream(&self, doc_id: &str) -> Result<(), anyhow::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);
        if let Ok(mut conn) = self.pool.get().await {
            let _: () = redis::cmd("DEL")
                .arg(&stream_key)
                .query_async(&mut *conn)
                .await?;
        }
        Ok(())
    }

    pub async fn delete_consumer(
        &self,
        doc_id: &str,
        group_name: &str,
        consumer_name: &str,
    ) -> Result<i64, anyhow::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);
        if let Ok(mut conn) = self.pool.get().await {
            let exists: bool = redis::cmd("EXISTS")
                .arg(&stream_key)
                .query_async(&mut *conn)
                .await?;

            if !exists {
                return Ok(0);
            }

            let result: i64 = redis::cmd("XGROUP")
                .arg("DELCONSUMER")
                .arg(&stream_key)
                .arg(group_name)
                .arg(consumer_name)
                .query_async(&mut *conn)
                .await?;

            return Ok(result);
        }

        Err(anyhow::anyhow!("Failed to get Redis connection"))
    }

    pub async fn acquire_doc_lock(
        &self,
        doc_id: &str,
        instance_id: &str,
    ) -> Result<bool, anyhow::Error> {
        let lock_key = format!("lock:doc:{}", doc_id);
        let ttl = 10;

        if let Ok(mut conn) = self.pool.get().await {
            let result: Option<String> = redis::cmd("SET")
                .arg(&lock_key)
                .arg(instance_id)
                .arg("NX")
                .arg("EX")
                .arg(ttl)
                .query_async(&mut *conn)
                .await?;

            return Ok(result.is_some());
        }

        Ok(false)
    }

    pub async fn release_doc_lock(
        &self,
        doc_id: &str,
        instance_id: &str,
    ) -> Result<bool, anyhow::Error> {
        let lock_key = format!("lock:doc:{}", doc_id);

        if let Ok(mut conn) = self.pool.get().await {
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

            return Ok(result == 1);
        }

        Ok(false)
    }

    pub async fn update_instance_heartbeat(
        &self,
        doc_id: &str,
        instance_id: &str,
    ) -> Result<(), anyhow::Error> {
        let key = format!("doc:instances:{}", doc_id);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let Ok(mut conn) = self.pool.get().await {
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

            return Ok(());
        }

        Err(anyhow::anyhow!("Failed to get Redis connection"))
    }

    pub async fn get_active_instances(
        &self,
        doc_id: &str,
        timeout_secs: u64,
    ) -> Result<i64, anyhow::Error> {
        let key = format!("doc:instances:{}", doc_id);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let Ok(mut conn) = self.pool.get().await {
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

            return Ok(count);
        }

        Err(anyhow::anyhow!("Failed to get Redis connection"))
    }

    pub async fn remove_instance_heartbeat(
        &self,
        doc_id: &str,
        instance_id: &str,
    ) -> Result<bool, anyhow::Error> {
        let key = format!("doc:instances:{}", doc_id);

        if let Ok(mut conn) = self.pool.get().await {
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

            return Ok(is_empty == 1);
        }

        Err(anyhow::anyhow!("Failed to get Redis connection"))
    }

    pub async fn safe_delete_stream(
        &self,
        doc_id: &str,
        instance_id: &str,
    ) -> Result<(), anyhow::Error> {
        if self.acquire_doc_lock(doc_id, instance_id).await? {
            let connections = self.get_active_instances(doc_id, 60).await?;

            if connections <= 0 {
                let stream_key = format!("yjs:stream:{}", doc_id);
                if let Ok(mut conn) = self.pool.get().await {
                    let exists: bool = redis::cmd("EXISTS")
                        .arg(&stream_key)
                        .query_async(&mut *conn)
                        .await?;

                    if exists {
                        let _: () = redis::cmd("DEL")
                            .arg(&stream_key)
                            .query_async(&mut *conn)
                            .await?;
                    }
                }

                let _ = self.release_doc_lock(doc_id, instance_id).await;
                return Ok(());
            }

            let _ = self.release_doc_lock(doc_id, instance_id).await;
        }

        warn!("still using stream");

        Ok(())
    }

    pub async fn check_stream_exists(&self, doc_id: &str) -> Result<bool, anyhow::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);

        if let Ok(mut conn) = self.pool.get().await {
            let exists: bool = redis::cmd("EXISTS")
                .arg(&stream_key)
                .query_async(&mut *conn)
                .await?;

            if exists {
                debug!("Redis stream '{}' exists", stream_key);
            } else {
                debug!("Redis stream '{}' does not exist", stream_key);
            }

            return Ok(exists);
        }

        Err(anyhow::anyhow!("Failed to get Redis connection"))
    }

    pub async fn read_all_stream_data(&self, doc_id: &str) -> Result<Vec<Bytes>, anyhow::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);

        if let Ok(mut conn) = self.pool.get().await {
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
                        if fields[j] == "update" then
                            table.insert(updates, fields[j+1])
                        end
                    end
                end
                
                return updates
            "#,
            );

            let updates: Vec<Bytes> = script.key(&stream_key).invoke_async(&mut *conn).await?;

            return Ok(updates);
        }

        Err(anyhow::anyhow!("Failed to get Redis connection"))
    }
}
