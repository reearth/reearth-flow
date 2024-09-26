use rslock::{LockError, LockGuard, LockManager};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GlobalLockError {
    #[error("Global Lock Error: {0:?}")]
    LockError(LockError),
}

impl From<LockError> for GlobalLockError {
    fn from(err: LockError) -> Self {
        GlobalLockError::LockError(err)
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
    ) -> Result<T, GlobalLockError>
    where
        F: FnOnce(&LockGuard) -> T + Send,
        T: Send,
    {
        let resource_bytes: Vec<u8> = resources.join(":").into_bytes();
        let lock = self
            .lock_manager
            .lock(&resource_bytes, duration_ms as usize)
            .await
            .map_err(GlobalLockError::from)?;
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
    ) -> Result<T, GlobalLockError>
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
    ) -> Result<T, GlobalLockError>
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
    ) -> Result<T, GlobalLockError>
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
    ) -> Result<T, GlobalLockError>
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
