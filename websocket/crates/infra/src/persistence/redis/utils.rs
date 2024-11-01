use crate::{define_key_methods, persistence::redis::errors::FlowProjectRedisDataManagerError};

use super::keys::RedisKeyManager;

pub fn default_key_manager() -> impl RedisKeyManager {
    struct DefaultKeyManager;
    impl RedisKeyManager for DefaultKeyManager {
        fn project_prefix(&self) -> String {
            String::new()
        }
        fn session_prefix(&self) -> Result<String, FlowProjectRedisDataManagerError> {
            Ok(String::new())
        }
        fn active_editing_session_id_key(&self) -> String {
            String::new()
        }
        define_key_methods! {
            state_key => "state",
            state_updated_by_key => "stateUpdatedBy",
            state_updates_key => "stateUpdates",
            last_updated_at_key => "lastUpdatedAt",
        }
    }
    DefaultKeyManager
}

pub fn encode_state_data(data: Vec<u8>) -> Result<String, FlowProjectRedisDataManagerError> {
    Ok(serde_json::to_string(&data)?)
}

pub fn decode_state_data(data_string: String) -> Result<Vec<u8>, FlowProjectRedisDataManagerError> {
    Ok(serde_json::from_str(&data_string)?)
}
