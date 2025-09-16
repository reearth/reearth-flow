use bytes::Bytes;
use deadpool_redis::Pool;

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
}

pub type RedisPool = Pool;
