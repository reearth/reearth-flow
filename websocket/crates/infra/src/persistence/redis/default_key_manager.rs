use super::errors::FlowProjectRedisDataManagerError;
use super::keys::RedisKeyManager;
use crate::define_key_methods;

pub struct DefaultKeyManager;

#[async_trait::async_trait]
impl RedisKeyManager for DefaultKeyManager {
    fn project_prefix(&self, project_id: &str) -> String {
        project_id.to_string()
    }

    fn session_prefix(
        &self,
        project_id: &str,
        session_id: Option<&str>,
    ) -> Result<String, FlowProjectRedisDataManagerError> {
        match session_id {
            Some(sid) => Ok(format!("{}:{}", self.project_prefix(project_id), sid)),
            None => Err(FlowProjectRedisDataManagerError::SessionNotSet),
        }
    }

    fn active_editing_session_id_key(&self, project_id: &str) -> String {
        format!("project:{}:active_session", self.project_prefix(project_id))
    }

    define_key_methods! {
        state_key => "state",
        state_updated_by_key => "stateUpdatedBy",
        state_updates_key => "stateUpdates",
        last_updated_at_key => "lastUpdatedAt",
    }
}

pub fn default_key_manager() -> DefaultKeyManager {
    DefaultKeyManager
}
