use bytes::Bytes;
use deadpool::managed::Pool;
use deadpool::Runtime;
use deadpool_redis::{Connection, Manager};
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::Duration;

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
    pub max_connections: Option<u32>,
    pub min_idle: Option<u32>,
    pub connection_timeout: Option<u64>,
}

pub type RedisPool = Pool<Manager, Connection>;

#[derive(Debug, Clone)]
pub struct RedisStore {
    pool: Arc<RedisPool>,
    config: Option<RedisConfig>,
}

impl RedisStore {
    pub async fn new(config: Option<RedisConfig>) -> Result<Self, anyhow::Error> {
        let pool = if let Some(config) = &config {
            Self::init_redis_connection(config).await?
        } else {
            return Err(anyhow::anyhow!("Redis configuration is missing"));
        };
        Ok(Self { pool, config })
    }

    pub fn get_pool(&self) -> Arc<RedisPool> {
        self.pool.clone()
    }

    pub fn get_config(&self) -> Option<RedisConfig> {
        self.config.clone()
    }

    pub async fn init_redis_connection(
        config: &RedisConfig,
    ) -> Result<Arc<RedisPool>, anyhow::Error> {
        let manager = Manager::new(config.url.clone())?;

        let pool = Pool::builder(manager)
            .max_size(config.max_connections.unwrap_or(2048) as usize)
            .wait_timeout(Some(Duration::from_secs(
                config.connection_timeout.unwrap_or(5),
            )))
            .create_timeout(Some(Duration::from_secs(
                config.connection_timeout.unwrap_or(5),
            )))
            .recycle_timeout(Some(Duration::from_secs(500)))
            .runtime(Runtime::Tokio1)
            .build()?;

        Ok(Arc::new(pool))
    }

    pub async fn publish_update(&self, doc_id: &str, update: &[u8]) -> Result<(), anyhow::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);
        if let Ok(mut conn) = self.pool.get().await {
            let mut pipe = redis::pipe();

            let fields = &[("update", update)];
            pipe.cmd("XADD")
                .arg(&stream_key)
                .arg("NOMKSTREAM")
                .arg("*")
                .arg(fields);

            if let Some(config) = &self.config {
                pipe.cmd("EXPIRE").arg(&stream_key).arg(config.ttl);
            }

            let _: () = pipe.query_async(&mut *conn).await?;
        }
        Ok(())
    }

    pub async fn publish_batch_updates(
        &self,
        doc_id: &str,
        updates: &[&[u8]],
    ) -> Result<(), anyhow::Error> {
        if updates.is_empty() {
            return Ok(());
        }

        let stream_key = format!("yjs:stream:{}", doc_id);
        if let Ok(mut conn) = self.pool.get().await {
            let ttl = self.config.as_ref().map(|c| c.ttl).unwrap_or(3600);

            let mut pipe = redis::pipe();

            pipe.atomic();

            for update in updates {
                let fields = &[("update", *update)];
                pipe.cmd("XADD")
                    .arg(&stream_key)
                    .arg("MAXLEN")
                    .arg("~")
                    .arg(1000)
                    .arg("NOMKSTREAM")
                    .arg("*")
                    .arg(fields);
            }

            pipe.cmd("EXPIRE").arg(&stream_key).arg(ttl);

            let _: () = pipe.query_async(&mut *conn).await?;
        }
        Ok(())
    }

    pub async fn create_consumer_group(
        &self,
        doc_id: &str,
        group_name: &str,
    ) -> Result<(), anyhow::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);
        if let Ok(mut conn) = self.pool.get().await {
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

    pub async fn read_and_ack_messages(
        &self,
        doc_id: &str,
        group_name: &str,
        consumer_name: &str,
        count: usize,
    ) -> Result<Vec<(String, Bytes)>, anyhow::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);
        if let Ok(mut conn) = self.pool.get().await {
            let effective_count = count.max(30);

            let result: RedisStreamResults = redis::cmd("XREADGROUP")
                .arg("GROUP")
                .arg(group_name)
                .arg(consumer_name)
                .arg("COUNT")
                .arg(effective_count)
                .arg("STREAMS")
                .arg(&stream_key)
                .arg(">")
                .query_async(&mut *conn)
                .await?;

            let mut updates = Vec::new();
            let mut message_ids = Vec::new();

            if !result.is_empty() && !result[0].1.is_empty() {
                message_ids.reserve(result[0].1.len());
                updates.reserve(result[0].1.len());

                for (msg_id, fields) in &result[0].1 {
                    message_ids.push(msg_id.clone());

                    for (field_name, field_value) in fields {
                        if field_name == "update" {
                            updates.push((msg_id.clone(), field_value.clone()));
                        }
                    }
                }

                if !message_ids.is_empty() {
                    let ack_result = self
                        .batch_ack_messages(doc_id, group_name, &message_ids)
                        .await;
                    if let Err(e) = ack_result {
                        tracing::warn!("Failed to acknowledge messages: {}", e);
                    }
                }
            }

            Ok(updates)
        } else {
            Err(anyhow::anyhow!("Failed to get Redis connection"))
        }
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

    pub async fn set_with_expiry(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: u64,
    ) -> Result<(), anyhow::Error> {
        if let Ok(mut conn) = self.pool.get().await {
            let _: () = redis::cmd("SET")
                .arg(key)
                .arg(value)
                .arg("EX")
                .arg(ttl_seconds)
                .query_async(&mut *conn)
                .await?;
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

    pub async fn read_and_ack_with_lua(
        &self,
        doc_id: &str,
        group_name: &str,
        consumer_name: &str,
        count: usize,
    ) -> Result<Vec<Bytes>, anyhow::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);

        let mut conn = self.pool.get().await?;

        let script = redis::Script::new(
            r#"
            local stream_key = KEYS[1]
            local group_name = ARGV[1]
            local consumer_name = ARGV[2]
            local count = tonumber(ARGV[3])
                
                local result = redis.call('XREADGROUP', 'GROUP', group_name, consumer_name, 'COUNT', count, 'STREAMS', stream_key, '>')
                if not result or #result == 0 then return {} end
                
                local messages = result[1][2]
                if not messages or #messages == 0 then return {} end
                
                local updates = {}
                local ids_to_ack = {}
                local n_messages = #messages
                
                for i = 1, n_messages do
                    local message = messages[i]
                    ids_to_ack[i] = message[1]
                    
                    local fields = message[2]
                    local n_fields = #fields
                    for j = 1, n_fields, 2 do
                        if fields[j] == "update" then
                            updates[#updates + 1] = fields[j+1]
                            break
                        end
                    end
                end
                
                if n_messages > 0 then
                    redis.call('XACK', stream_key, group_name, unpack(ids_to_ack))
                end
                
                return updates
                "#,
        );

        let updates = script
            .key(stream_key)
            .arg(group_name)
            .arg(consumer_name)
            .arg(count)
            .invoke_async(&mut *conn)
            .await?;

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

    pub async fn increment_doc_connections(&self, doc_id: &str) -> Result<i64, anyhow::Error> {
        let key = format!("connections:doc:{}", doc_id);

        if let Ok(mut conn) = self.pool.get().await {
            let script = redis::Script::new(
                r#"
                local count = redis.call('INCR', KEYS[1])
                redis.call('EXPIRE', KEYS[1], ARGV[1])
                return count
                "#,
            );

            let count: i64 = script.key(&key).arg(60).invoke_async(&mut *conn).await?;

            tracing::debug!(
                "Redis: Incremented connections for doc '{}' to {}",
                doc_id,
                count
            );
            return Ok(count);
        }

        Err(anyhow::anyhow!("Failed to get Redis connection"))
    }

    pub async fn refresh_doc_connections(&self, doc_id: &str) -> Result<(), anyhow::Error> {
        let key = format!("connections:doc:{}", doc_id);

        if let Ok(mut conn) = self.pool.get().await {
            let script = redis::Script::new(
                r#"
                if redis.call('EXISTS', KEYS[1]) == 1 then
                    return redis.call('EXPIRE', KEYS[1], ARGV[1])
                end
                return 0
                "#,
            );

            let _: i64 = script.key(&key).arg(60).invoke_async(&mut *conn).await?;

            tracing::debug!("Redis: Refreshed TTL for doc connections '{}'", doc_id);
            return Ok(());
        }

        Err(anyhow::anyhow!("Failed to get Redis connection"))
    }

    pub async fn decrement_doc_connections(&self, doc_id: &str) -> Result<i64, anyhow::Error> {
        let key = format!("connections:doc:{}", doc_id);

        if let Ok(mut conn) = self.pool.get().await {
            let script = redis::Script::new(
                r#"
                local current = redis.call('get', KEYS[1])
                if current and tonumber(current) > 0 then
                    return redis.call('decr', KEYS[1])
                elseif current then
                    return 0
                else
                    return 0
                end
            "#,
            );

            let count: i64 = script.key(&key).invoke_async(&mut *conn).await?;
            tracing::info!(
                "Redis: Decremented connections for doc '{}' to {}",
                doc_id,
                count
            );

            return Ok(count);
        }

        Err(anyhow::anyhow!("Failed to get Redis connection"))
    }

    pub async fn get_doc_connections(&self, doc_id: &str) -> Result<i64, anyhow::Error> {
        let key = format!("connections:doc:{}", doc_id);

        if let Ok(mut conn) = self.pool.get().await {
            let count: Option<i64> = redis::cmd("GET").arg(&key).query_async(&mut *conn).await?;
            let count_value = count.unwrap_or(0);
            tracing::debug!(
                "Redis: Current connections for doc '{}': {}",
                doc_id,
                count_value
            );

            return Ok(count_value);
        }

        Err(anyhow::anyhow!("Failed to get Redis connection"))
    }

    pub async fn safe_delete_stream(
        &self,
        doc_id: &str,
        instance_id: &str,
    ) -> Result<bool, anyhow::Error> {
        if self.acquire_doc_lock(doc_id, instance_id).await? {
            let connections = self.get_doc_connections(doc_id).await?;

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

                        tracing::info!("Safely deleted Redis stream for '{}'", doc_id);
                    }
                }

                let _ = self.release_doc_lock(doc_id, instance_id).await;
                return Ok(true);
            }

            tracing::info!(
                "Not deleting Redis stream for '{}' as there are still {} connections",
                doc_id,
                connections
            );

            let _ = self.release_doc_lock(doc_id, instance_id).await;
        } else {
            tracing::debug!(
                "Could not acquire lock for doc '{}', skipping deletion attempt",
                doc_id
            );
        }

        Ok(false)
    }

    pub async fn check_stream_exists(&self, doc_id: &str) -> Result<bool, anyhow::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);

        if let Ok(mut conn) = self.pool.get().await {
            let exists: bool = redis::cmd("EXISTS")
                .arg(&stream_key)
                .query_async(&mut *conn)
                .await?;

            if exists {
                tracing::debug!("Redis stream '{}' exists", stream_key);
            } else {
                tracing::debug!("Redis stream '{}' does not exist", stream_key);
            }

            return Ok(exists);
        }

        Err(anyhow::anyhow!("Failed to get Redis connection"))
    }

    pub async fn read_all_stream_data(&self, doc_id: &str) -> Result<Vec<Bytes>, anyhow::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);

        if let Ok(mut conn) = self.pool.get().await {
            let exists: bool = redis::cmd("EXISTS")
                .arg(&stream_key)
                .query_async(&mut *conn)
                .await?;

            if !exists {
                return Ok(Vec::new());
            }

            type RawStreamEntry = (String, Vec<(String, Bytes)>);
            let result: Vec<RawStreamEntry> = redis::cmd("XRANGE")
                .arg(&stream_key)
                .arg("-")
                .arg("+")
                .query_async(&mut *conn)
                .await?;

            let mut updates = Vec::new();

            for (_, fields) in result {
                for (field_name, field_value) in fields {
                    if field_name == "update" {
                        updates.push(field_value);
                    }
                }
            }

            return Ok(updates);
        }

        Err(anyhow::anyhow!("Failed to get Redis connection"))
    }
}
