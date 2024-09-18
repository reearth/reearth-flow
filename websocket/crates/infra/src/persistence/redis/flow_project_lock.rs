use std::error::Error;
use std::fmt;

use rslock::{LockError, LockGuard, LockManager};

#[derive(Debug)]
pub struct GlobalLockError(pub LockError);

impl fmt::Display for GlobalLockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Global Lock Error: {:?}", self.0)
    }
}

impl Error for GlobalLockError {}

impl From<LockError> for GlobalLockError {
    fn from(err: LockError) -> Self {
        GlobalLockError(err)
    }
}

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
            .lock(&resource_bytes, duration_ms as usize)
            .await?;
        let guard = LockGuard { lock };
        let result = callback(&guard);
        self.lock_manager.unlock(&guard.lock).await;

        Ok(result)
    }

    pub async fn lock_state<F, T>(
        &self,
        project_id: &str,
        duration_ms: u64,
        callback: F,
    ) -> Result<T, LockError>
    where
        F: FnOnce(&LockGuard) -> T + Send,
        T: Send,
    {
        let lock_key = format!("{}:locks:state", project_id);
        self.with_lock(vec![lock_key], duration_ms, callback).await
    }

    pub async fn lock_updates<F, T>(
        &self,
        project_id: &str,
        duration_ms: u64,
        callback: F,
    ) -> Result<T, LockError>
    where
        F: FnOnce(&LockGuard) -> T + Send,
        T: Send,
    {
        let state_lock_key = format!("{}:locks:state", project_id);
        let updates_lock_key = format!("{}:locks:updates", project_id);
        self.with_lock(
            vec![state_lock_key, updates_lock_key],
            duration_ms,
            callback,
        )
        .await
    }

    pub async fn lock_snapshots<F, T>(
        &self,
        project_id: &str,
        duration_ms: u64,
        callback: F,
    ) -> Result<T, LockError>
    where
        F: FnOnce(&LockGuard) -> T + Send,
        T: Send,
    {
        let state_lock_key = format!("{}:locks:state", project_id);
        let updates_lock_key = format!("{}:locks:updates", project_id);
        let snapshots_lock_key = format!("{}:locks:snapshots", project_id);
        self.with_lock(
            vec![state_lock_key, updates_lock_key, snapshots_lock_key],
            duration_ms,
            callback,
        )
        .await
    }

    pub async fn lock_session<F, T>(
        &self,
        project_id: &str,
        duration_ms: u64,
        callback: F,
    ) -> Result<T, LockError>
    where
        F: FnOnce(&LockGuard) -> T + Send,
        T: Send,
    {
        let state_lock_key = format!("{}:locks:state", project_id);
        let updates_lock_key = format!("{}:locks:updates", project_id);
        let snapshots_lock_key = format!("{}:locks:snapshots", project_id);
        self.with_lock(
            vec![state_lock_key, updates_lock_key, snapshots_lock_key],
            duration_ms,
            callback,
        )
        .await
    }
}
