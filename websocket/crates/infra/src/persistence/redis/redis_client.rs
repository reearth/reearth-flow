use async_trait::async_trait;
use redis::{aio::MultiplexedConnection, streams::StreamMaxlen, AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RedisClient {
    connection: Arc<Mutex<MultiplexedConnection>>,
    redis_url: String,
}

#[derive(Error, Debug)]
pub enum RedisClientError {
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    #[error("Redis Client Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

#[async_trait]
pub trait RedisClientTrait: Send + Sync {
    fn redis_url(&self) -> &str;
    async fn set<T: Serialize + Send + Sync + 'static>(
        &self,
        key: String,
        value: &T,
    ) -> Result<(), RedisClientError>;
    async fn get<T>(&self, key: &str) -> Result<Option<T>, RedisClientError>
    where
        T: for<'de> Deserialize<'de> + Send + Sync + 'static;
    async fn keys(&self, pattern: String) -> Result<Vec<String>, RedisClientError>;
    async fn xadd(
        &self,
        key: &str,
        id: &str,
        fields: &[(&str, &str)],
    ) -> Result<String, RedisClientError>;
    async fn xread(
        &self,
        key: &str,
        id: &str,
    ) -> Result<Vec<(String, Vec<(String, String)>)>, RedisClientError>;
    async fn xtrim(&self, key: &str, max_len: usize) -> Result<usize, RedisClientError>;
    async fn xdel(&self, key: &str, ids: &[&str]) -> Result<usize, RedisClientError>;
    fn connection(&self) -> Arc<Mutex<MultiplexedConnection>>;
    async fn get_client_count(&self) -> Result<usize, RedisClientError>;
}

#[async_trait]
impl RedisClientTrait for RedisClient {
    fn redis_url(&self) -> &str {
        &self.redis_url
    }

    async fn set<T: Serialize + Send + Sync + 'static>(
        &self,
        key: String,
        value: &T,
    ) -> Result<(), RedisClientError> {
        let mut connection = self.connection.lock().await;
        let _: () = connection.set(key, serde_json::to_string(value)?).await?;
        Ok(())
    }

    async fn get<T>(&self, key: &str) -> Result<Option<T>, RedisClientError>
    where
        T: for<'de> Deserialize<'de> + Send + Sync + 'static,
    {
        let mut connection = self.connection.lock().await;
        let value: Option<String> = connection.get(key).await?;
        match value {
            Some(val) => Ok(Some(serde_json::from_str(&val)?)),
            None => Ok(None),
        }
    }

    async fn keys(&self, pattern: String) -> Result<Vec<String>, RedisClientError> {
        let mut connection = self.connection.lock().await;
        let keys: Vec<String> = connection.keys(pattern).await?;
        Ok(keys)
    }

    async fn xadd(
        &self,
        key: &str,
        id: &str,
        fields: &[(&str, &str)],
    ) -> Result<String, RedisClientError> {
        let mut connection = self.connection.lock().await;
        let id: String = connection.xadd(key, id, fields).await?;
        Ok(id)
    }

    async fn xread(
        &self,
        key: &str,
        id: &str,
    ) -> Result<Vec<(String, Vec<(String, String)>)>, RedisClientError> {
        let mut connection = self.connection.lock().await;
        let result: Vec<(String, Vec<(String, String)>)> = connection.xread(&[key], &[id]).await?;
        Ok(result)
    }

    async fn xtrim(&self, key: &str, max_len: usize) -> Result<usize, RedisClientError> {
        let mut connection = self.connection.lock().await;
        let len: usize = connection.xtrim(key, StreamMaxlen::Equals(max_len)).await?;
        Ok(len)
    }

    async fn xdel(&self, key: &str, ids: &[&str]) -> Result<usize, RedisClientError> {
        let mut connection = self.connection.lock().await;
        let count: usize = connection.xdel(key, ids).await?;
        Ok(count)
    }

    fn connection(&self) -> Arc<Mutex<MultiplexedConnection>> {
        self.connection.clone()
    }

    async fn get_client_count(&self) -> Result<usize, RedisClientError> {
        let mut connection = self.connection.lock().await;

        let client_list: String = redis::cmd("CLIENT")
            .arg("LIST")
            .query_async(&mut *connection)
            .await?;

        let connections: Vec<&str> = client_list.lines().collect();

        Ok(connections.len())
    }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::Mutex;

    #[derive(Clone)]
    pub struct MockRedisClient {
        data: Arc<Mutex<HashMap<String, String>>>,
    }

    impl MockRedisClient {
        pub fn new() -> Self {
            MockRedisClient {
                data: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl RedisClientTrait for MockRedisClient {
        fn redis_url(&self) -> &str {
            "mock://localhost"
        }

        async fn set<T: Serialize + Send + Sync + 'static>(
            &self,
            key: String,
            value: &T,
        ) -> Result<(), RedisClientError> {
            let serialized_value = serde_json::to_string(value)?;
            let mut data = self.data.lock().await;
            data.insert(key, serialized_value);
            Ok(())
        }

        async fn get<T>(&self, key: &str) -> Result<Option<T>, RedisClientError>
        where
            T: for<'de> Deserialize<'de> + Send + Sync + 'static,
        {
            let data = self.data.lock().await;
            if let Some(value) = data.get(key) {
                let deserialized_value: T = serde_json::from_str(value)?;
                Ok(Some(deserialized_value))
            } else {
                Ok(None)
            }
        }

        async fn keys(&self, pattern: String) -> Result<Vec<String>, RedisClientError> {
            let data = self.data.lock().await;
            let keys: Vec<String> = data
                .keys()
                .filter(|k| k.contains(&pattern))
                .cloned()
                .collect();
            Ok(keys)
        }

        async fn xadd(
            &self,
            _key: &str,
            _id: &str,
            _fields: &[(&str, &str)],
        ) -> Result<String, RedisClientError> {
            Ok("mock_id".to_string())
        }

        async fn xread(
            &self,
            _key: &str,
            _id: &str,
        ) -> Result<Vec<(String, Vec<(String, String)>)>, RedisClientError> {
            Ok(vec![(
                "mock_key".to_string(),
                vec![("field".to_string(), "value".to_string())],
            )])
        }

        async fn xtrim(&self, _key: &str, _max_len: usize) -> Result<usize, RedisClientError> {
            Ok(0)
        }

        async fn xdel(&self, _key: &str, _ids: &[&str]) -> Result<usize, RedisClientError> {
            Ok(0)
        }

        fn connection(&self) -> Arc<Mutex<MultiplexedConnection>> {
            unimplemented!()
        }

        async fn get_client_count(&self) -> Result<usize, RedisClientError> {
            Ok(1)
        }
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let client = MockRedisClient::new();

        let result = client.set("test_key".to_string(), &"test_value").await;
        assert!(result.is_ok());

        let get_result: Result<Option<String>, RedisClientError> = client.get("test_key").await;
        assert_eq!(get_result.unwrap(), Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_keys() {
        let client = MockRedisClient::new();

        let _ = client.set("key1".to_string(), &"value1").await;
        let _ = client.set("key2".to_string(), &"value2").await;

        let keys_result = client.keys("key".to_string()).await;
        assert!(keys_result.is_ok());

        let keys = keys_result.unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }

    #[tokio::test]
    async fn test_xadd() {
        let client = MockRedisClient::new();

        let fields = &[("field1", "value1"), ("field2", "value2")];
        let result = client.xadd("test_stream", "*", fields).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "mock_id");
    }

    #[tokio::test]
    async fn test_xread() {
        let client = MockRedisClient::new();

        let result = client.xread("test_stream", "0").await;
        assert!(result.is_ok());

        let entries = result.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "mock_key");
        assert_eq!(entries[0].1[0], ("field".to_string(), "value".to_string()));
    }

    #[tokio::test]
    async fn test_xtrim() {
        let client = MockRedisClient::new();

        let result = client.xtrim("test_stream", 100).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_xdel() {
        let client = MockRedisClient::new();

        let result = client.xdel("test_stream", &["1", "2"]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_get_client_count() {
        let client = MockRedisClient::new();

        let result = client.get_client_count().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
}
