use super::errors::FlowProjectRedisDataManagerError;
use async_trait::async_trait;

#[async_trait]
pub trait RedisKeyManager: Send + Sync {
    fn project_prefix(&self) -> String;
    fn session_prefix(&self) -> Result<String, FlowProjectRedisDataManagerError>;
    fn active_editing_session_id_key(&self) -> String;
    fn state_key(&self) -> Result<String, FlowProjectRedisDataManagerError>;
    fn state_updated_by_key(&self) -> Result<String, FlowProjectRedisDataManagerError>;
    fn state_updates_key(&self) -> Result<String, FlowProjectRedisDataManagerError>;
    fn last_updated_at_key(&self) -> Result<String, FlowProjectRedisDataManagerError>;
}

#[macro_export]
macro_rules! define_key_methods {
    ($($method:ident => $suffix:expr),* $(,)?) => {
        $(
            fn $method(&self) -> Result<String, $crate::persistence::redis::errors::FlowProjectRedisDataManagerError> {
                Ok(format!("{}:{}", self.session_prefix()?, $suffix))
            }
        )*
    };
}
