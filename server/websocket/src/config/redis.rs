//! Redis configuration module.

/// Redis-related configuration.
#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// Redis connection URL
    pub url: String,
    /// TTL for Redis keys in seconds
    pub ttl: u64,
    /// Interval for trimming Redis streams in seconds
    pub stream_trim_interval: u64,
    /// Maximum age of stream messages in milliseconds
    pub stream_max_message_age: u64,
    /// Maximum number of messages in stream
    pub stream_max_length: u64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            ttl: 43200,                      // 12 hours
            stream_trim_interval: 60,        // 60 seconds
            stream_max_message_age: 3600000, // 1 hour in milliseconds
            stream_max_length: 100,
        }
    }
}

/// Convert to domain RedisConfig for use with RedisStore
impl From<RedisConfig> for crate::domain::value_objects::redis::RedisConfig {
    fn from(config: RedisConfig) -> Self {
        Self {
            url: config.url,
            ttl: config.ttl,
            stream_trim_interval: config.stream_trim_interval,
            stream_max_message_age: config.stream_max_message_age,
            stream_max_length: config.stream_max_length,
        }
    }
}

impl From<&RedisConfig> for crate::domain::value_objects::redis::RedisConfig {
    fn from(config: &RedisConfig) -> Self {
        Self {
            url: config.url.clone(),
            ttl: config.ttl,
            stream_trim_interval: config.stream_trim_interval,
            stream_max_message_age: config.stream_max_message_age,
            stream_max_length: config.stream_max_length,
        }
    }
}
