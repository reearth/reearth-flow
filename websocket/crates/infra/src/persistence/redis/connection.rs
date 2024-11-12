use bb8::{Pool, PooledConnection};
use bb8_redis::RedisConnectionManager;

#[async_trait::async_trait]
pub trait RedisConnection {
    fn get_pool(&self) -> &Pool<RedisConnectionManager>;

    async fn get_connection(
        &self,
    ) -> Result<PooledConnection<'_, RedisConnectionManager>, bb8::RunError<redis::RedisError>>
    {
        Ok(self.get_pool().get().await?)
    }
}
