use anyhow::{Context, Result};
use redis::{aio::MultiplexedConnection, streams::StreamMaxlen, AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RedisClient {
    connection: Arc<Mutex<MultiplexedConnection>>,
    redis_url: String,
}

impl RedisClient {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url).context("Failed to open Redis client")?;
        let connection = client
            .get_multiplexed_async_connection()
            .await
            .context("Failed to get Redis multiplexed async connection")?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            redis_url: redis_url.to_string(),
        })
    }

    pub fn redis_url(&self) -> &str {
        &self.redis_url
    }

    pub async fn set<T: Serialize>(&self, key: String, value: &T) -> Result<()> {
        let mut connection = self.connection.lock().await;
        connection
            .set(
                &key,
                serde_json::to_string(value).context("Failed to serialize value")?,
            )
            .await
            .context(format!("Failed to set key: {}", key))?;
        Ok(())
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        let mut connection = self.connection.lock().await;
        let value: Option<String> = connection
            .get(key)
            .await
            .context(format!("Failed to get key: {}", key))?;
        match value {
            Some(val) => Ok(Some(
                serde_json::from_str(&val).context("Failed to deserialize value")?,
            )),
            None => Ok(None),
        }
    }

    pub async fn keys(&self, pattern: String) -> Result<Vec<String>> {
        let mut connection = self.connection.lock().await;
        let keys: Vec<String> = connection
            .keys(&pattern)
            .await
            .context(format!("Failed to get keys with pattern: {}", pattern))?;
        Ok(keys)
    }

    pub async fn xadd(&self, key: &str, id: &str, fields: &[(&str, &str)]) -> Result<String> {
        let mut connection = self.connection.lock().await;
        let id: String = connection
            .xadd(key, id, fields)
            .await
            .context(format!("Failed to xadd fields with key: {}", key))?;
        Ok(id)
    }

    pub async fn xread(&self, key: &str, id: &str) -> Result<Vec<(String, Vec<(String, String)>)>> {
        let mut connection = self.connection.lock().await;
        let result: Vec<(String, Vec<(String, String)>)> = connection
            .xread(&[key], &[id])
            .await
            .context(format!("Failed to xread from key: {}, id: {}", key, id))?;
        Ok(result)
    }

    pub async fn xtrim(&self, key: &str, max_len: usize) -> Result<usize> {
        let mut connection = self.connection.lock().await;
        let len: usize = connection
            .xtrim(key, StreamMaxlen::Equals(max_len))
            .await
            .context(format!(
                "Failed to xtrim key: {} to max_len: {}",
                key, max_len
            ))?;
        Ok(len)
    }

    pub async fn xdel(&self, key: &str, ids: &[&str]) -> Result<usize> {
        let mut connection = self.connection.lock().await;
        let count: usize = connection
            .xdel(key, ids)
            .await
            .context(format!("Failed to xdel ids from key: {}", key))?;
        Ok(count)
    }

    pub fn connection(&self) -> Arc<Mutex<MultiplexedConnection>> {
        self.connection.clone()
    }
}
