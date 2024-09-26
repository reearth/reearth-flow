use std::sync::Arc;

use redis::{aio::MultiplexedConnection, streams::StreamMaxlen, AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RedisClient {
    connection: Arc<Mutex<MultiplexedConnection>>,
    redis_url: String,
}

#[derive(Error, Debug)]
pub enum RedisClientError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

impl RedisClient {
    pub async fn new(redis_url: &str) -> Result<Self, RedisClientError> {
        let client = Client::open(redis_url)?;
        let connection = client.get_multiplexed_async_connection().await?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            redis_url: redis_url.to_string(),
        })
    }

    pub fn redis_url(&self) -> &str {
        &self.redis_url
    }

    pub async fn set<T: Serialize>(&self, key: String, value: &T) -> Result<(), RedisClientError> {
        let mut connection = self.connection.lock().await;
        let _: () = connection.set(key, serde_json::to_string(value)?).await?;
        Ok(())
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<Option<T>, RedisClientError> {
        let mut connection = self.connection.lock().await;
        let value: Option<String> = connection.get(key).await?;
        match value {
            Some(val) => Ok(Some(serde_json::from_str(&val)?)),
            None => Ok(None),
        }
    }

    pub async fn keys(&self, pattern: String) -> Result<Vec<String>, RedisClientError> {
        let mut connection = self.connection.lock().await;
        let keys: Vec<String> = connection.keys(pattern).await?;
        Ok(keys)
    }

    pub async fn xadd(
        &self,
        key: &str,
        id: &str,
        fields: &[(&str, &str)],
    ) -> Result<String, RedisClientError> {
        let mut connection = self.connection.lock().await;
        let id: String = connection.xadd(key, id, fields).await?;
        Ok(id)
    }

    pub async fn xread(
        &self,
        key: &str,
        id: &str,
    ) -> Result<Vec<(String, Vec<(String, String)>)>, RedisClientError> {
        let mut connection = self.connection.lock().await;
        let result: Vec<(String, Vec<(String, String)>)> = connection.xread(&[key], &[id]).await?;
        Ok(result)
    }

    pub async fn xtrim(&self, key: &str, max_len: usize) -> Result<usize, RedisClientError> {
        let mut connection = self.connection.lock().await;
        let len: usize = connection.xtrim(key, StreamMaxlen::Equals(max_len)).await?;
        Ok(len)
    }

    pub async fn xdel(&self, key: &str, ids: &[&str]) -> Result<usize, RedisClientError> {
        let mut connection = self.connection.lock().await;
        let count: usize = connection.xdel(key, ids).await?;
        Ok(count)
    }

    pub fn connection(&self) -> Arc<Mutex<MultiplexedConnection>> {
        self.connection.clone()
    }
}
