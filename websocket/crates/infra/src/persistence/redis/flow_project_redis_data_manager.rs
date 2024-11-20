use crate::persistence::repository::RedisDataManagerImpl;
use redis::AsyncCommands;

use super::{
    connection::RedisConnection, default_key_manager::DefaultKeyManager,
    errors::FlowProjectRedisDataManagerError, flow_project_lock::FlowProjectLock,
    keys::RedisKeyManager, updates::UpdateManager,
};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::streams::StreamMaxlen;
use std::sync::Arc;
use tracing::debug;

type StreamEntry = (String, Vec<(String, Vec<u8>)>);

#[derive(Clone)]
pub struct FlowProjectRedisDataManager {
    redis_pool: Pool<RedisConnectionManager>,
    global_lock: FlowProjectLock,
    update_manager: Arc<UpdateManager>,
    key_manager: Arc<DefaultKeyManager>,
}

impl FlowProjectRedisDataManager {
    pub async fn new(redis_url: &str) -> Result<Self, FlowProjectRedisDataManagerError> {
        let manager = RedisConnectionManager::new(redis_url)?;
        let redis_pool = Pool::builder().build(manager).await?;
        let global_lock = FlowProjectLock::new(redis_url)?;
        let key_manager = Arc::new(DefaultKeyManager);

        let instance = Self {
            redis_pool: redis_pool.clone(),
            global_lock,
            update_manager: Arc::new(UpdateManager::new(redis_pool.clone(), key_manager.clone())),
            key_manager,
        };

        Ok(instance)
    }

    fn parse_entry_fields(fields: &[redis::Value]) -> Option<Vec<(String, Vec<u8>)>> {
        fields
            .chunks(2)
            .filter(|chunk| chunk.len() == 2)
            .filter_map(|chunk| match (&chunk[0], &chunk[1]) {
                (redis::Value::BulkString(key), redis::Value::BulkString(val)) => {
                    Some((String::from_utf8_lossy(key).into_owned(), val.clone()))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
            .into()
    }

    fn parse_stream_entry(entry_fields: &[redis::Value]) -> Option<StreamEntry> {
        if entry_fields.len() < 2 {
            return None;
        }

        let entry_id = match &entry_fields[0] {
            redis::Value::BulkString(bytes) => String::from_utf8_lossy(bytes).into_owned(),
            _ => return None,
        };

        match &entry_fields[1] {
            redis::Value::Array(fields) => Some((entry_id, Self::parse_entry_fields(fields)?)),
            _ => None,
        }
    }

    async fn xread_map(
        &self,
        key: &str,
        id: &str,
    ) -> Result<Vec<(String, Vec<(String, Vec<u8>)>)>, FlowProjectRedisDataManagerError> {
        let result: redis::Value = {
            let mut connection = self.get_connection().await?;
            connection.xread(&[key], &[id]).await?
        };

        match result {
            redis::Value::Array(outer) => Ok(outer
                .into_iter()
                .filter_map(|stream| match stream {
                    redis::Value::Array(mut stream_data) if stream_data.len() >= 2 => {
                        match stream_data.remove(1) {
                            redis::Value::Array(entries) => Some(entries),
                            _ => None,
                        }
                    }
                    _ => None,
                })
                .flat_map(|entries| {
                    entries.into_iter().filter_map(|entry| match entry {
                        redis::Value::Array(entry_fields) => {
                            Self::parse_stream_entry(&entry_fields)
                        }
                        _ => None,
                    })
                })
                .collect()),
            _ => Ok(vec![]),
        }
    }

    async fn get_state_update_in_redis(
        &self,
        project_id: &str,
    ) -> Result<Option<Vec<Vec<u8>>>, FlowProjectRedisDataManagerError> {
        let state_update_key = self.key_manager.state_updates_key(project_id)?;

        let entries = self.xread_map(&state_update_key, "0").await?;

        debug!("State update entries: {:?}", entries);

        let updates: Vec<Vec<u8>> = entries
            .into_iter()
            .filter_map(|(_, fields)| {
                fields.first().and_then(|(_, update_data)| {
                    if !update_data.is_empty() {
                        Some(update_data.clone())
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok((!updates.is_empty()).then_some(updates))
    }

    async fn set_state_with_stream(
        &self,
        project_id: &str,
        state_data: &Vec<u8>,
        updated_by: Option<String>,
    ) -> Result<(), FlowProjectRedisDataManagerError> {
        let mut conn = self.get_connection().await?;
        let state_key = self.key_manager.state_key(project_id)?;
        let stream_update_key_by = self.key_manager.state_updated_by_key(project_id)?;

        let _: () = conn.set(&state_key, state_data).await?;
        let _: () = conn.set(&stream_update_key_by, updated_by).await?;

        Ok(())
    }
    async fn set_state_data(
        &self,
        project_id: &str,
        state_update: &Vec<u8>,
        state_updated_by: Option<String>,
    ) -> Result<(), FlowProjectRedisDataManagerError> {
        self.global_lock
            .lock_state(project_id, 5000, |_| async {
                self.set_state_with_stream(project_id, state_update, state_updated_by)
                    .await
            })
            .await?
            .await
    }

    async fn execute_merge_updates(
        &self,
        project_id: &str,
        update_data: &Vec<u8>,
        updated_by: Option<String>,
    ) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError> {
        let updated_by = updated_by.unwrap_or_default();
        self.set_update_data(project_id, update_data, Some(updated_by))
            .await?;
        let state_updates = self.get_state_update_in_redis(project_id).await?;
        debug!("State updates in redis: {:?}", state_updates);
        let mut merged_update: Vec<u8> = Vec::new();
        let mut updates_by: Vec<String> = Vec::new();
        if let Some(state_updates) = state_updates {
            for state_update in state_updates {
                let (new_merged_update, new_updates_by) = self
                    .update_manager
                    .merge_updates_internal(project_id, Some(state_update))
                    .await?;
                merged_update = new_merged_update;
                updates_by = new_updates_by;
            }
        }

        let update_by_json = serde_json::to_string(&updates_by).unwrap_or_default();

        self.set_state_data(project_id, &merged_update, Some(update_by_json))
            .await?;

        self.clear_state_updates_stream(project_id).await?;

        Ok((merged_update, updates_by))
    }

    async fn set_update_data(
        &self,
        project_id: &str,
        update_data: &Vec<u8>,
        updated_by: Option<String>,
    ) -> Result<(), FlowProjectRedisDataManagerError> {
        let mut conn = self.get_connection().await?;
        let stream_key = self.key_manager.state_updates_key(project_id)?;
        let updated_by = updated_by.unwrap_or_default();
        let fields = &[(updated_by, update_data)];
        let _: () = conn.xadd(&stream_key, "*", fields).await?;

        Ok(())
    }

    async fn clear_state_updates_stream(
        &self,
        project_id: &str,
    ) -> Result<(), FlowProjectRedisDataManagerError> {
        let mut conn = self.get_connection().await?;
        let _: () = conn
            .xtrim(
                &self.key_manager.state_updates_key(project_id)?,
                StreamMaxlen::Equals(0),
            )
            .await?;
        Ok(())
    }

    async fn lock_and_execute_merge_updates(
        &self,
        project_id: &str,
        update_data: &Vec<u8>,
        updated_by: Option<String>,
    ) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError> {
        self.global_lock
            .lock_updates(project_id, 5000, |_| async {
                self.execute_merge_updates(project_id, update_data, updated_by)
                    .await
            })
            .await?
            .await
    }
}

#[async_trait::async_trait]
impl RedisDataManagerImpl for FlowProjectRedisDataManager {
    type Error = FlowProjectRedisDataManagerError;

    async fn get_current_state(&self, project_id: &str) -> Result<Option<Vec<u8>>, Self::Error> {
        let state_key = self.key_manager.state_key(project_id)?;
        let current_state: Option<Vec<u8>> = self.redis_pool.get().await?.get(state_key).await?;
        Ok(current_state)
    }

    async fn get_state_updates_by(&self, project_id: &str) -> Result<Option<String>, Self::Error> {
        let state_updates_key = self.key_manager.state_updated_by_key(project_id)?;
        let state_updates = self.redis_pool.get().await?.get(state_updates_key).await?;
        Ok(state_updates)
    }

    async fn create_session(&self, project_id: &str, session_id: &str) -> Result<(), Self::Error> {
        let mut conn = self.get_connection().await?;
        let _: () = conn
            .set(
                self.key_manager.active_editing_session_id_key(project_id),
                session_id,
            )
            .await?;
        Ok(())
    }

    async fn clear_data(
        &self,
        project_id: &str,
        session_id: Option<&str>,
    ) -> Result<(), Self::Error> {
        let mut connection = self.get_connection().await?;

        debug!("Starting to clear Redis data for project: {}", project_id);

        let keys_to_delete = vec![
            self.key_manager.state_key(project_id)?,
            self.key_manager.state_updated_by_key(project_id)?,
            self.key_manager.state_updates_key(project_id)?,
            self.key_manager.last_updated_at_key(project_id)?,
        ];

        debug!("Deleting keys: {:?}", keys_to_delete);

        let _: () = connection.del(&keys_to_delete).await?;

        if let Some(active_session_id) = self.get_active_session_id(project_id).await? {
            if let Some(current_session_id) = session_id {
                if active_session_id == current_session_id {
                    debug!("Clearing active session ID: {}", active_session_id);
                    let _: () = connection
                        .del(&[self.key_manager.active_editing_session_id_key(project_id)])
                        .await?;
                }
            }
        }

        debug!(
            "Successfully cleared all Redis data for project: {}",
            project_id
        );
        Ok(())
    }

    /// Merge all updates in the stream
    async fn merge_updates(
        &self,
        project_id: &str,
        update_data: Vec<u8>,
        updated_by: Option<String>,
    ) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError> {
        self.lock_and_execute_merge_updates(project_id, &update_data, updated_by)
            .await
    }

    async fn get_active_session_id(
        &self,
        project_id: &str,
    ) -> Result<Option<String>, FlowProjectRedisDataManagerError> {
        let mut conn = self.get_connection().await?;
        let result: Option<String> = conn
            .get(self.key_manager.active_editing_session_id_key(project_id))
            .await?;
        Ok(result)
    }
}

#[async_trait::async_trait]
impl RedisConnection for FlowProjectRedisDataManager {
    fn get_pool(&self) -> &Pool<RedisConnectionManager> {
        &self.redis_pool
    }

    async fn get_connection(
        &self,
    ) -> Result<bb8::PooledConnection<'_, RedisConnectionManager>, bb8::RunError<redis::RedisError>>
    {
        Ok(self.get_pool().get().await?)
    }
}
