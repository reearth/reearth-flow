use bytes::Bytes;
use deadpool::managed::{self, Metrics};

pub const OID_LOCK_KEY: &str = "lock:oid_generation";

pub const MESSAGE_TYPE_SYNC: &str = "sync";
pub const MESSAGE_TYPE_AWARENESS: &str = "awareness";
pub type RedisField = (String, Bytes);
pub type RedisFields = Vec<RedisField>;
pub type RedisStreamMessage = (String, RedisFields);
pub type RedisStreamMessages = Vec<RedisStreamMessage>;
pub type RedisStreamResult = (String, RedisStreamMessages);
pub type RedisStreamResults = Vec<RedisStreamResult>;

#[derive(Debug, Clone)]
pub struct StreamMessages {
    pub sync_updates: Vec<Bytes>,
    pub awareness_updates: Vec<(String, Bytes)>,
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub ttl: u64,
    pub stream_trim_interval: u64,
    pub stream_max_message_age: u64,
    pub stream_max_length: u64,
}

pub struct RedisConnectionManager {
    client: redis::Client,
}

impl std::fmt::Debug for RedisConnectionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisConnectionManager").finish()
    }
}

impl RedisConnectionManager {
    pub fn new(url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(url)?;
        Ok(Self { client })
    }
}

impl managed::Manager for RedisConnectionManager {
    type Type = redis::aio::MultiplexedConnection;
    type Error = redis::RedisError;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        self.client.get_multiplexed_async_connection().await
    }

    async fn recycle(
        &self,
        conn: &mut Self::Type,
        _metrics: &Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        // Check if the connection is still alive by sending a PING
        redis::cmd("PING")
            .query_async::<String>(conn)
            .await
            .map_err(managed::RecycleError::Backend)?;
        Ok(())
    }
}

pub type RedisPool = managed::Pool<RedisConnectionManager>;
