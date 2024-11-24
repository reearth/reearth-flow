use redis::{Client, RedisError};
use thiserror::Error;
use uuid::Uuid;

pub struct LockGuard {
    #[allow(dead_code)]
    redis_client: Client,
    key: String,
    token: String,
}

#[derive(Error, Debug)]
pub enum LockError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),
    #[error("Resource is already locked")]
    AlreadyLocked,
    #[error("Failed to create Redis client: {0}")]
    ClientCreation(String),
}

// Release lock using Lua script to ensure atomic delete with explicit return type
const SCRIPT: &str = r"
    if redis.call('get', KEYS[1]) == ARGV[1] then
        return redis.call('del', KEYS[1])
    else
        return 0
    end";

macro_rules! define_lock_method {
    ($name:ident, $($lock_key:expr),+) => {
        pub async fn $name<F, T>(
            &self,
            project_id: &str,
            duration_ms: u64,
            callback: F,
        ) -> Result<T, LockError>
        where
            F: FnOnce(&LockGuard) -> T + Send,
            T: Send,
        {
            let lock_keys = vec![$(format!("{}:locks:{}", project_id, $lock_key)),+];
            self.with_lock(lock_keys, duration_ms, callback).await
        }
    };
}

#[derive(Clone)]
pub struct FlowProjectLock {
    redis_client: Client,
}

impl FlowProjectLock {
    pub fn new(redis_url: &str) -> Result<Self, LockError> {
        let redis_client =
            Client::open(redis_url).map_err(|e| LockError::ClientCreation(e.to_string()))?;
        Ok(Self { redis_client })
    }

    async fn with_lock<F, T>(
        &self,
        resources: Vec<String>,
        duration_ms: u64,
        callback: F,
    ) -> Result<T, LockError>
    where
        F: FnOnce(&LockGuard) -> T + Send,
        T: Send,
    {
        let key = resources.join(":");
        let token = Uuid::new_v4().to_string();
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;

        // Try to acquire lock using SET NX with expiration
        let acquired: bool = redis::cmd("SET")
            .arg(&key)
            .arg(&token)
            .arg("NX")
            .arg("PX")
            .arg(duration_ms)
            .query_async(&mut conn)
            .await?;

        if !acquired {
            return Err(LockError::AlreadyLocked);
        }

        let guard = LockGuard {
            redis_client: self.redis_client.clone(),
            key,
            token,
        };

        let result = callback(&guard);

        let _: i32 = redis::Script::new(SCRIPT)
            .key(&guard.key)
            .arg(&guard.token)
            .invoke_async(&mut conn)
            .await?;

        Ok(result)
    }

    define_lock_method!(lock_state, "state");
    define_lock_method!(lock_updates, "state", "updates");
    define_lock_method!(lock_snapshots, "state", "updates", "snapshots");
    define_lock_method!(lock_session, "state", "updates", "snapshots");
}
