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
        key: &str,
        value: &T,
    ) -> Result<(), RedisClientError>;
    async fn get<T: for<'de> Deserialize<'de> + Send + Sync + 'static>(
        &self,
        key: &str,
    ) -> Result<Option<T>, RedisClientError>;
    async fn keys(&self, pattern: &str) -> Result<Vec<String>, RedisClientError>;
    async fn xadd(
        &self,
        key: &str,
        id: &str,
        fields: &[(String, String)],
    ) -> Result<String, RedisClientError>;
    async fn xread(
        &self,
        key: &str,
        id: &str,
    ) -> Result<Vec<(String, Vec<(String, String)>)>, RedisClientError>;
    async fn xtrim(&self, key: &str, max_len: usize) -> Result<usize, RedisClientError>;
    async fn xdel(&self, key: &str, ids: &[String]) -> Result<usize, RedisClientError>;
    fn connection(&self) -> &Arc<Mutex<MultiplexedConnection>>;
    async fn get_client_count(&self) -> Result<usize, RedisClientError>;
}

#[async_trait]
impl RedisClientTrait for RedisClient {
    fn redis_url(&self) -> &str {
        &self.redis_url
    }

    async fn set<T: Serialize + Send + Sync + 'static>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), RedisClientError> {
        let serialized = serde_json::to_string(value)?;
        let mut connection = self.connection.lock().await;
        connection.set::<_, _, ()>(key, serialized).await?;
        Ok(())
    }

    async fn get<T: for<'de> Deserialize<'de> + Send + Sync + 'static>(
        &self,
        key: &str,
    ) -> Result<Option<T>, RedisClientError> {
        let mut connection = self.connection.lock().await;
        let value: Option<String> = connection.get(key).await?;
        value
            .map(|val| serde_json::from_str(&val))
            .transpose()
            .map_err(Into::into)
    }

    async fn keys(&self, pattern: &str) -> Result<Vec<String>, RedisClientError> {
        let mut connection = self.connection.lock().await;
        connection.keys(pattern).await.map_err(Into::into)
    }

    async fn xadd(
        &self,
        key: &str,
        id: &str,
        fields: &[(String, String)],
    ) -> Result<String, RedisClientError> {
        let mut connection = self.connection.lock().await;
        let fields: Vec<(&str, &str)> = fields
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        connection.xadd(key, id, &fields).await.map_err(Into::into)
    }

    async fn xread(
        &self,
        key: &str,
        id: &str,
    ) -> Result<Vec<(String, Vec<(String, String)>)>, RedisClientError> {
        let mut connection = self.connection.lock().await;
        connection.xread(&[key], &[id]).await.map_err(Into::into)
    }

    async fn xtrim(&self, key: &str, max_len: usize) -> Result<usize, RedisClientError> {
        let mut connection = self.connection.lock().await;
        connection
            .xtrim(key, StreamMaxlen::Equals(max_len))
            .await
            .map_err(Into::into)
    }

    async fn xdel(&self, key: &str, ids: &[String]) -> Result<usize, RedisClientError> {
        let mut connection = self.connection.lock().await;
        connection.xdel(key, ids).await.map_err(Into::into)
    }

    fn connection(&self) -> &Arc<Mutex<MultiplexedConnection>> {
        &self.connection
    }

    async fn get_client_count(&self) -> Result<usize, RedisClientError> {
        let mut connection = self.connection.lock().await;
        let client_list: String = redis::cmd("CLIENT")
            .arg("LIST")
            .arg("TYPE")
            .arg("normal")
            .query_async(&mut *connection)
            .await?;
        Ok(client_list.lines().count())
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
            Self {
                data: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl RedisClientTrait for MockRedisClient {
        fn redis_url(&self) -> &str {
            "mock://localhost"
        }

        async fn set<T: Serialize + Send + Sync>(
            &self,
            key: &str,
            value: &T,
        ) -> Result<(), RedisClientError> {
            let serialized_value = serde_json::to_string(value)?;
            self.data
                .lock()
                .await
                .insert(key.to_string(), serialized_value);
            Ok(())
        }

        async fn get<T: for<'de> Deserialize<'de> + Send + Sync>(
            &self,
            key: &str,
        ) -> Result<Option<T>, RedisClientError> {
            self.data
                .lock()
                .await
                .get(key)
                .map(|value| serde_json::from_str(value))
                .transpose()
                .map_err(Into::into)
        }

        async fn keys(&self, pattern: &str) -> Result<Vec<String>, RedisClientError> {
            Ok(self
                .data
                .lock()
                .await
                .keys()
                .filter(|k| k.contains(pattern))
                .cloned()
                .collect())
        }

        async fn xadd(
            &self,
            _key: &str,
            _id: &str,
            _fields: &[(String, String)],
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

        async fn xdel(&self, _key: &str, _ids: &[String]) -> Result<usize, RedisClientError> {
            Ok(0)
        }

        fn connection(&self) -> &Arc<Mutex<MultiplexedConnection>> {
            unimplemented!()
        }

        async fn get_client_count(&self) -> Result<usize, RedisClientError> {
            Ok(1)
        }
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let client = MockRedisClient::new();
        client.set("test_key", &"test_value").await.unwrap();
        let result: Option<String> = client.get("test_key").await.unwrap();
        assert_eq!(result, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_keys() {
        let client = MockRedisClient::new();
        client.set("key1", &"value1").await.unwrap();
        client.set("key2", &"value2").await.unwrap();
        let keys = client.keys("key").await.unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }

    #[tokio::test]
    async fn test_xadd() {
        let client = MockRedisClient::new();
        let result = client
            .xadd(
                "test_stream",
                "*",
                &[
                    ("field1".to_string(), "value1".to_string()),
                    ("field2".to_string(), "value2".to_string()),
                ],
            )
            .await;
        assert_eq!(result.unwrap(), "mock_id");
    }

    #[tokio::test]
    async fn test_xread() {
        let client = MockRedisClient::new();
        let entries = client.xread("test_stream", "0").await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "mock_key");
        assert_eq!(entries[0].1[0], ("field".to_string(), "value".to_string()));
    }

    #[tokio::test]
    async fn test_xtrim() {
        let client = MockRedisClient::new();
        assert_eq!(client.xtrim("test_stream", 100).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_xdel() {
        let client = MockRedisClient::new();
        assert_eq!(
            client
                .xdel("test_stream", &["1".to_string(), "2".to_string()])
                .await
                .unwrap(),
            0
        );
    }

    #[tokio::test]
    async fn test_get_client_count() {
        let client = MockRedisClient::new();
        assert_eq!(client.get_client_count().await.unwrap(), 1);
    }
}
