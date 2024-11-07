use super::errors::FlowProjectRedisDataManagerError;
use async_trait::async_trait;

#[async_trait]
pub trait RedisKeyManager: Send + Sync {
    fn project_prefix(&self, project_id: &str) -> String;
    fn active_editing_session_id_key(&self, project_id: &str) -> String;
    fn state_key(&self, project_id: &str) -> Result<String, FlowProjectRedisDataManagerError>;
    fn state_updated_by_key(
        &self,
        project_id: &str,
    ) -> Result<String, FlowProjectRedisDataManagerError>;
    fn state_updates_key(
        &self,
        project_id: &str,
    ) -> Result<String, FlowProjectRedisDataManagerError>;
    fn last_updated_at_key(
        &self,
        project_id: &str,
    ) -> Result<String, FlowProjectRedisDataManagerError>;
}

#[macro_export]
macro_rules! define_key_methods {
    ($($method:ident => $suffix:expr),* $(,)?) => {
        $(
            fn $method(&self, project_id: &str) -> Result<String, $crate::persistence::redis::errors::FlowProjectRedisDataManagerError> {
                Ok(format!("{}:{}", self.project_prefix(project_id), $suffix))
            }
        )*
    };
}
