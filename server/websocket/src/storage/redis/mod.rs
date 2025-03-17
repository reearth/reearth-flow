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
    pub max_connections: Option<u32>,
    pub min_idle: Option<u32>,
    pub connection_timeout: Option<u64>,
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
            let pool = Self::init_redis_connection(config).await?;
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

    pub async fn init_redis_connection(
        config: &RedisConfig,
    ) -> Result<Arc<RedisPool>, redis::RedisError> {
        let manager = RedisConnectionManager::new(config.url.clone())?;

        let builder = Pool::builder()
            .max_size(config.max_connections.unwrap_or(1024))
            .min_idle(config.min_idle.or(Some(10)))
            .connection_timeout(Duration::from_secs(config.connection_timeout.unwrap_or(5)))
            .idle_timeout(Some(Duration::from_secs(500)))
            .max_lifetime(Some(Duration::from_secs(7200)));

        let pool = builder
            .build(manager)
            .await
            .map_err(|e| redis::RedisError::from(std::io::Error::other(e)))?;

        Ok(Arc::new(pool))
    }

    pub async fn publish_update(&self, doc_id: &str, update: &[u8]) -> Result<(), anyhow::Error> {
        if let Some(pool) = &self.pool {
            let stream_key = format!("yjs:stream:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
                let mut pipe = redis::pipe();

                let fields = &[("update", update)];
                pipe.cmd("XADD")
                    .arg(&stream_key)
                    .arg("MAXLEN")
                    .arg("~")
                    .arg(1000)
                    .arg("NOMKSTREAM")
                    .arg("*")
                    .arg(fields);

                if let Some(config) = &self.config {
                    pipe.cmd("EXPIRE").arg(&stream_key).arg(config.ttl);
                }

                let _: () = pipe.query_async(&mut *conn).await?;
            }
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

        if let Some(pool) = &self.pool {
            let stream_key = format!("yjs:stream:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
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

    pub async fn batch_ack_messages(
        &self,
        doc_id: &str,
        group_name: &str,
        message_ids: &[String],
    ) -> Result<(), anyhow::Error> {
        if message_ids.is_empty() {
            return Ok(());
        }

        if let Some(pool) = &self.pool {
            let stream_key = format!("yjs:stream:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
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
        }
        Err(anyhow::anyhow!("Redis pool is not initialized"))
    }

    pub async fn read_and_ack_messages(
        &self,
        doc_id: &str,
        group_name: &str,
        consumer_name: &str,
        count: usize,
    ) -> Result<Vec<(String, Vec<u8>)>, anyhow::Error> {
        if let Some(pool) = &self.pool {
            let stream_key = format!("yjs:stream:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
                let effective_count = count.max(50);

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
        } else {
            Err(anyhow::anyhow!("Redis pool is not initialized"))
        }
    }

    pub async fn read_batch_messages(
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

    pub async fn read_and_ack_with_lua(
        &self,
        doc_id: &str,
        group_name: &str,
        consumer_name: &str,
        count: usize,
    ) -> Result<Vec<Vec<u8>>, anyhow::Error> {
        if let Some(pool) = &self.pool {
            let stream_key = format!("yjs:stream:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
                let script = redis::Script::new(
                    r#"
                    local stream_key = KEYS[1]
                    local group_name = ARGV[1]
                    local consumer_name = ARGV[2]
                    local count = tonumber(ARGV[3])
                    
                    -- Read messages
                    local result = redis.call('XREADGROUP', 'GROUP', group_name, consumer_name, 'COUNT', count, 'STREAMS', stream_key, '>')
                    
                    -- Check if result is a table and has content
                    if type(result) ~= 'table' then
                        return {}
                    end
                    
                    if #result == 0 then
                        return {}
                    end
                    
                    local messages = result[1][2]
                    local updates = {}
                    local ids_to_ack = {}
                    
                    -- Extract updates and IDs to acknowledge
                    for i, message in ipairs(messages) do
                        local id = message[1]
                        table.insert(ids_to_ack, id)
                        
                        local fields = message[2]
                        for j = 1, #fields, 2 do
                            if fields[j] == "update" then
                                table.insert(updates, fields[j+1])
                            end
                        end
                    end
                    
                    -- Acknowledge messages
                    if #ids_to_ack > 0 then
                        redis.call('XACK', stream_key, group_name, unpack(ids_to_ack))
                    end
                    
                    return updates
                "#,
                );

                let effective_count = count.max(50);
                let updates: Vec<Vec<u8>> = script
                    .key(stream_key)
                    .arg(group_name)
                    .arg(consumer_name)
                    .arg(effective_count)
                    .invoke_async(&mut *conn)
                    .await?;

                Ok(updates)
            } else {
                Err(anyhow::anyhow!("Failed to get Redis connection"))
            }
        } else {
            Err(anyhow::anyhow!("Redis pool is not initialized"))
        }
    }

    pub async fn delete_stream(&self, doc_id: &str) -> Result<(), anyhow::Error> {
        if let Some(pool) = &self.pool {
            let stream_key = format!("yjs:stream:{}", doc_id);
            if let Ok(mut conn) = pool.get().await {
                let _: () = redis::cmd("DEL")
                    .arg(&stream_key)
                    .query_async(&mut *conn)
                    .await?;
            }
        }
        Ok(())
    }
}
