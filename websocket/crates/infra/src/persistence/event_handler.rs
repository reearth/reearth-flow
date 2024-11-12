use async_trait::async_trait;
use std::error::Error;
use std::fmt::Debug;

#[async_trait]
pub trait EventHandler: Debug {
    type Error: Error + Send + Sync + 'static;

    async fn record_snapshot_created(
        &self,
        project_id: &str,
        user_id: &str,
        version: u64,
        snapshot_name: Option<&str>,
    ) -> Result<(), Self::Error>;
}
