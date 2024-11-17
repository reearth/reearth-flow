use rslock::{LockError, LockGuard, LockManager};
use std::time::Duration;

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
    lock_manager: LockManager,
}

impl FlowProjectLock {
    pub fn new(redis_url: &str) -> Self {
        let lock_manager = LockManager::new(vec![redis_url]);
        Self { lock_manager }
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
        let resource_bytes: Vec<u8> = resources.join(":").into_bytes();
        let lock = self
            .lock_manager
            .lock(&resource_bytes, Duration::from_millis(duration_ms))
            .await?;
        let guard = LockGuard { lock };
        let result = callback(&guard);
        self.lock_manager.unlock(&guard.lock).await;

        Ok(result)
    }

    define_lock_method!(lock_state, "state");
    define_lock_method!(lock_updates, "state", "updates");
    define_lock_method!(lock_snapshots, "state", "updates", "snapshots");
    define_lock_method!(lock_session, "state", "updates", "snapshots");
}
