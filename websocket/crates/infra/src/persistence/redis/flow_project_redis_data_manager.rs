use super::{
    connection::RedisConnection, decode_state_data, default_key_manager::DefaultKeyManager,
    encode_state_data, errors::FlowProjectRedisDataManagerError,
    flow_project_lock::FlowProjectLock, keys::RedisKeyManager, types::FlowEncodedUpdate,
    updates::UpdateManager,
};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use flow_websocket_domain::repository::RedisDataManagerImpl;
use redis::{streams::StreamMaxlen, AsyncCommands};
use std::sync::Arc;
use tracing::debug;
use yrs::{updates::decoder::Decode, Doc, Transact, Update};

#[derive(Clone)]
pub struct FlowProjectRedisDataManager {
    redis_pool: Pool<RedisConnectionManager>,
    global_lock: FlowProjectLock,
    update_manager: Arc<UpdateManager>,
    key_manager: Arc<DefaultKeyManager>,
}

impl FlowProjectRedisDataManager {
    pub async fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
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

    async fn get_state_update_in_redis(
        &self,
        project_id: &str,
        session_id: Option<&str>,
    ) -> Result<Option<Vec<u8>>, FlowProjectRedisDataManagerError> {
        let state_key = self.key_manager.state_key(project_id)?;
        let state_update_string: Option<String> =
            self.get_connection().await?.get(&state_key).await?;

        if let Some(state_update_string) = state_update_string {
            Ok(Some(decode_state_data(state_update_string)?))
        } else {
            Err(FlowProjectRedisDataManagerError::MissingStateUpdate {
                key: state_key,
                context: format!("Project: {}, Session: {:?}", project_id, session_id),
            })
        }
    }

    async fn set_state_data_internal(
        &self,
        project_id: &str,
        encoded_state_update: &str,
        updated_by_json: &str,
    ) -> Result<(), FlowProjectRedisDataManagerError> {
        let mut conn = self.get_connection().await?;
        let _: () = conn
            .set(
                self.key_manager.state_key(project_id)?,
                encoded_state_update,
            )
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
        let encoded_state_update = encode_state_data(state_update)?;
        let updated_by_json = serde_json::to_string(&state_updated_by)?;

        if skip_lock {
            self.set_state_data_internal(project_id, &encoded_state_update, &updated_by_json)
                .await?;
        } else {
            self.global_lock
                .lock_state(project_id, 5000, move |_lock_guard| {
                    Box::pin(async move {
                        self.set_state_data_internal(
                            project_id,
                            &encoded_state_update,
                            &updated_by_json,
                        )
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
        let state_update = self.get_state_update_in_redis(project_id, None).await?;
        let (merged_update, updates_by) = self
            .update_manager
            .merge_updates(project_id, state_update)
            .await?;

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
}

#[async_trait::async_trait]
impl RedisDataManagerImpl for FlowProjectRedisDataManager {
    type Error = FlowProjectRedisDataManagerError;

    async fn get_current_state(
        &self,
        project_id: &str,
        session_id: Option<&str>,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        let state_update = self
            .get_state_update_in_redis(project_id, session_id)
            .await?;
        let merged_update = self.update_manager.get_merged_update(project_id).await?;

        match (state_update, merged_update) {
            (Some(state_update), Some(merged_update)) => {
                let doc = Doc::new();
                let mut txn = doc.transact_mut();
                txn.apply_update(Update::decode_v2(&state_update)?);
                txn.apply_update(Update::decode_v2(&merged_update)?);
                Ok(Some(txn.encode_update_v2()))
            }
            (state_update, _) => Ok(state_update),
        }
    }

    async fn create_session(&self, project_id: &str, session_id: &str) -> Result<(), Self::Error> {
        let _: () = self
            .get_connection()
            .await?
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
        let update_data = FlowEncodedUpdate {
            update: encode_state_data(update_data)?,
            updated_by,
        };

        let value = serde_json::to_string(&update_data)?;
        let fields = &[("value", value.as_str()), ("format", "json")];

        let mut conn = self.get_connection().await?;
        let _: () = conn
            .xadd(self.key_manager.state_updates_key(project_id)?, "*", fields)
            .await?;

        let timestamp = chrono::Utc::now().timestamp().to_string();
        let _: () = conn
            .set(
                self.key_manager.last_updated_at_key(project_id)?,
                &timestamp,
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
