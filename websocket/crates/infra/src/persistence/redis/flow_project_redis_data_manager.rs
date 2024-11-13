use crate::persistence::repository::RedisDataManagerImpl;

use super::{
    connection::RedisConnection, default_key_manager::DefaultKeyManager,
    errors::FlowProjectRedisDataManagerError, flow_project_lock::FlowProjectLock,
    keys::RedisKeyManager, updates::UpdateManager,
};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::{streams::StreamMaxlen, AsyncCommands};
use std::sync::Arc;
use tracing::debug;
use yrs::{updates::decoder::Decode, Doc, Transact, Update};

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
        let global_lock = FlowProjectLock::new(redis_url);
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
            redis::Value::Array(outer) => {
                let mut mapped_results = Vec::new();
                for stream in outer {
                    if let redis::Value::Array(stream_data) = stream {
                        if stream_data.len() >= 2 {
                            if let redis::Value::Array(entries) = &stream_data[1] {
                                mapped_results.extend(entries.iter().filter_map(|entry| {
                                    if let redis::Value::Array(entry_fields) = entry {
                                        Self::parse_stream_entry(entry_fields)
                                    } else {
                                        None
                                    }
                                }));
                            }
                        }
                    }
                }
                Ok(mapped_results)
            }
            _ => Ok(vec![]),
        }
    }

    async fn get_state_update_in_redis(
        &self,
        project_id: &str,
    ) -> Result<Option<Vec<Vec<u8>>>, FlowProjectRedisDataManagerError> {
        let state_update_key = self.key_manager.state_updates_key(project_id)?;
        debug!(
            "Getting state update from Redis stream: {}",
            state_update_key
        );

        let entries = self.xread_map(&state_update_key, "0").await?;
        debug!("State update entries: {:?}", entries);

        if entries.is_empty() {
            return Ok(None);
        }

        let mut updates = Vec::new();
        for (_, fields) in entries {
            if let Some((_, update_data)) = fields.first() {
                if !update_data.is_empty() {
                    updates.push(update_data.clone());
                }
            }
        }

        if updates.is_empty() {
            Ok(None)
        } else {
            Ok(Some(updates))
        }
    }

    async fn set_state_data_internal(
        &self,
        project_id: &str,
        state_update: &Vec<u8>,
        updated_by_json: &str,
    ) -> Result<(), FlowProjectRedisDataManagerError> {
        let mut conn = self.get_connection().await?;
        let _: () = conn
            .set(self.key_manager.state_key(project_id)?, state_update)
            .await?;
        let _: () = conn
            .set(
                self.key_manager.state_updated_by_key(project_id)?,
                updated_by_json,
            )
            .await?;
        Ok(())
    }

    async fn set_state_data(
        &self,
        project_id: &str,
        state_update: Vec<u8>,
        state_updated_by: Vec<String>,
        skip_lock: bool,
    ) -> Result<(), FlowProjectRedisDataManagerError> {
        let updated_by_json = serde_json::to_string(&state_updated_by)?;

        if skip_lock {
            self.set_state_data_internal(project_id, &state_update, &updated_by_json)
                .await?;
        } else {
            self.global_lock
                .lock_state(project_id, 5000, move |_lock_guard| {
                    Box::pin(async move {
                        self.set_state_data_internal(project_id, &state_update, &updated_by_json)
                            .await?;
                        Ok::<(), FlowProjectRedisDataManagerError>(())
                    })
                })
                .await
                .map_err(FlowProjectRedisDataManagerError::from)?
                .await?;
        }
        Ok(())
    }

    async fn execute_merge_updates(
        &self,
        project_id: &str,
    ) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError> {
        let state_updates = self.get_state_update_in_redis(project_id).await?;
        let mut merged_update: Vec<u8> = Vec::new();
        let mut updates_by: Vec<String> = Vec::new();
        if let Some(state_updates) = state_updates {
            for state_update in state_updates {
                let (new_merged_update, new_updates_by) = self
                    .update_manager
                    .merge_updates(project_id, Some(state_update))
                    .await?;
                merged_update = new_merged_update;
                updates_by = new_updates_by;
            }
        }

        self.set_state_data(project_id, merged_update.clone(), updates_by.clone(), true)
            .await?;

        // Clear the update stream
        let mut conn = self.get_connection().await?;
        let _: () = conn
            .xtrim(
                &self.key_manager.state_updates_key(project_id)?,
                StreamMaxlen::Equals(0),
            )
            .await?;

        Ok((merged_update, updates_by))
    }

    /// Merge updates by user id
    async fn execute_merge_updates_by_user_id(
        &self,
        project_id: &str,
        user_id: &str,
    ) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError> {
        let mut merged_update: Vec<u8> = Vec::new();
        let mut updates_by: Vec<String> = Vec::new();

        // Fetch all updates from Redis
        let entries = self
            .xread_map(&self.key_manager.state_updates_key(project_id)?, "0")
            .await?;

        // Filter entries by user_id and collect their updates
        for (_, fields) in entries {
            if let Some((updated_by, update_data)) = fields.first() {
                if updated_by == user_id {
                    let (new_merged_update, new_updates_by) = self
                        .update_manager
                        .merge_updates(project_id, Some(update_data.clone()))
                        .await?;
                    merged_update = new_merged_update;
                    updates_by = new_updates_by;
                }
            }
        }

        Ok((merged_update, updates_by))
    }

    async fn lock_and_execute_merge_updates(
        &self,
        project_id: &str,
    ) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError> {
        self.global_lock
            .lock_updates(project_id, 5000, |_| async {
                Ok::<(), FlowProjectRedisDataManagerError>(())
            })
            .await?
            .await?;

        self.execute_merge_updates(project_id).await
    }

    async fn lock_and_execute_merge_updates_by_user_id(
        &self,
        project_id: &str,
        user_id: &str,
    ) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError> {
        self.global_lock
            .lock_updates(project_id, 5000, |_| async {
                Ok::<(), FlowProjectRedisDataManagerError>(())
            })
            .await?
            .await?;

        self.execute_merge_updates_by_user_id(project_id, user_id)
            .await
    }
}

#[async_trait::async_trait]
impl RedisDataManagerImpl for FlowProjectRedisDataManager {
    type Error = FlowProjectRedisDataManagerError;

    async fn get_current_state(
        &self,
        project_id: &str,
        _session_id: Option<&str>,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        let state_updates = self.get_state_update_in_redis(project_id).await?;
        let merged_update = self.update_manager.get_merged_update(project_id).await?;
        match (state_updates, merged_update) {
            (Some(state_updates), Some(merged_update)) => {
                let doc = Doc::new();
                let mut txn = doc.transact_mut();

                for state_update in state_updates {
                    txn.apply_update(Update::decode_v2(&state_update)?);
                }
                if !merged_update.is_empty() {
                    txn.apply_update(Update::decode_v2(&merged_update)?);
                }

                Ok(Some(txn.encode_update_v2()))
            }
            (state_updates, _) => Ok(state_updates.and_then(|updates| updates.into_iter().next())),
        }
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

    async fn push_update(
        &self,
        project_id: &str,
        update_data: Vec<u8>,
        updated_by: Option<String>,
    ) -> Result<(), Self::Error> {
        let fields = &[(updated_by.as_deref().unwrap_or(""), &update_data)];
        debug!("Pushing update to Redis stream: {:?}", fields);

        let mut conn = self.get_connection().await?;
        let _: () = conn
            .xadd(self.key_manager.state_updates_key(project_id)?, "*", fields)
            .await?;

        debug!("Update pushed to Redis stream");

        let timestamp = chrono::Utc::now().timestamp().to_string();
        let _: () = conn
            .set(
                self.key_manager.last_updated_at_key(project_id)?,
                &timestamp,
            )
            .await?;

        let _ = self.execute_merge_updates(project_id).await?;
        debug!("Updates merged after push");

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
        skip_lock: bool,
    ) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError> {
        if skip_lock {
            self.execute_merge_updates(project_id).await
        } else {
            self.lock_and_execute_merge_updates(project_id).await
        }
    }

    async fn merge_updates_by_user_id(
        &self,
        project_id: &str,
        user_id: &str,
        skip_lock: bool,
    ) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError> {
        if skip_lock {
            self.execute_merge_updates_by_user_id(project_id, user_id)
                .await
        } else {
            self.lock_and_execute_merge_updates_by_user_id(project_id, user_id)
                .await
        }
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
}
