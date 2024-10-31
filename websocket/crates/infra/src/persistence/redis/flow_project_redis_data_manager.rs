use super::{
    connection::RedisConnection, errors::FlowProjectRedisDataManagerError,
    flow_project_lock::FlowProjectLock, keys::RedisKeyManager, types::FlowEncodedUpdate,
    updates::UpdateManager,
};
use crate::define_key_methods;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use flow_websocket_domain::repository::RedisDataManager;
use redis::{streams::StreamMaxlen, AsyncCommands};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;
use yrs::{updates::decoder::Decode, Doc, Transact, Update};

pub struct FlowProjectRedisDataManager {
    redis_pool: Pool<RedisConnectionManager>,
    project_id: String,
    session_id: Arc<Mutex<Option<String>>>,
    global_lock: FlowProjectLock,
    update_manager: Arc<UpdateManager>,
}

impl FlowProjectRedisDataManager {
    pub async fn new(
        project_id: String,
        session_id: Option<String>,
        redis_url: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let manager = RedisConnectionManager::new(redis_url)?;
        let redis_pool = Pool::builder().build(manager).await?;
        let global_lock = FlowProjectLock::new(redis_url);

        let mut instance = Self {
            redis_pool: redis_pool.clone(),
            project_id,
            session_id: Arc::new(Mutex::new(session_id)),
            global_lock,
            update_manager: Arc::new(UpdateManager::new(
                redis_pool.clone(),
                Arc::new(Self::default_key_manager()),
            )),
        };

        instance.update_manager =
            Arc::new(UpdateManager::new(redis_pool, Arc::new(instance.clone())));

        Ok(instance)
    }

    fn default_key_manager() -> impl RedisKeyManager {
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

    async fn get_state_update_in_redis(
        &self,
    ) -> Result<Option<Vec<u8>>, FlowProjectRedisDataManagerError> {
        let state_key = self.state_key()?;
        let state_update_string: Option<String> =
            self.get_connection().await?.get(&state_key).await?;

        if let Some(state_update_string) = state_update_string {
            Ok(Some(Self::decode_state_data(state_update_string)?))
        } else {
            let session_id = self.session_id.lock().await;
            Err(FlowProjectRedisDataManagerError::MissingStateUpdate {
                key: state_key,
                context: format!(
                    "Project: {}, Session: {:?}",
                    self.project_id,
                    session_id.as_ref()
                ),
            })
        }
    }

    fn encode_state_data(data: Vec<u8>) -> Result<String, FlowProjectRedisDataManagerError> {
        Ok(serde_json::to_string(&data)?)
    }

    fn decode_state_data(data_string: String) -> Result<Vec<u8>, FlowProjectRedisDataManagerError> {
        Ok(serde_json::from_str(&data_string)?)
    }

    async fn set_state_data_internal(
        &self,
        encoded_state_update: &str,
        updated_by_json: &str,
    ) -> Result<(), FlowProjectRedisDataManagerError> {
        let mut conn = self.get_connection().await?;
        let _: () = conn.set(self.state_key()?, encoded_state_update).await?;
        let _: () = conn
            .set(self.state_updated_by_key()?, updated_by_json)
            .await?;
        Ok(())
    }

    async fn set_state_data(
        &self,
        state_update: Vec<u8>,
        state_updated_by: Vec<String>,
        skip_lock: bool,
    ) -> Result<(), FlowProjectRedisDataManagerError> {
        let encoded_state_update = Self::encode_state_data(state_update)?;
        let updated_by_json = serde_json::to_string(&state_updated_by)?;

        if skip_lock {
            self.set_state_data_internal(&encoded_state_update, &updated_by_json)
                .await?;
        } else {
            let project_id = self.project_id.clone();
            self.global_lock
                .lock_state(&project_id, 5000, move |_lock_guard| {
                    Box::pin(async move {
                        self.set_state_data_internal(&encoded_state_update, &updated_by_json)
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
    ) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError> {
        let state_update = self.get_state_update_in_redis().await?;
        let (merged_update, updates_by) = self.update_manager.merge_updates(state_update).await?;

        self.set_state_data(merged_update.clone(), updates_by.clone(), true)
            .await?;

        // Clear the update stream
        let mut conn = self.get_connection().await?;
        let _: () = conn
            .xtrim(&self.state_updates_key()?, StreamMaxlen::Equals(0))
            .await?;

        Ok((merged_update, updates_by))
    }

    async fn lock_and_execute_merge_updates(
        &self,
    ) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError> {
        self.global_lock
            .lock_updates(&self.project_id, 5000, |_| async {
                Ok::<(), FlowProjectRedisDataManagerError>(())
            })
            .await?
            .await?;

        self.execute_merge_updates().await
    }
}

impl Clone for FlowProjectRedisDataManager {
    fn clone(&self) -> Self {
        Self {
            redis_pool: self.redis_pool.clone(),
            project_id: self.project_id.clone(),
            session_id: self.session_id.clone(),
            global_lock: self.global_lock.clone(),
            update_manager: self.update_manager.clone(),
        }
    }
}

#[async_trait::async_trait]
impl RedisKeyManager for FlowProjectRedisDataManager {
    fn project_prefix(&self) -> String {
        self.project_id.clone()
    }

    fn session_prefix(&self) -> Result<String, FlowProjectRedisDataManagerError> {
        self.session_id
            .try_lock()
            .map_err(|_| FlowProjectRedisDataManagerError::LockError)?
            .as_ref()
            .ok_or(FlowProjectRedisDataManagerError::SessionNotSet)
            .map(|session_id| format!("{}:{}", self.project_prefix(), session_id))
    }

    fn active_editing_session_id_key(&self) -> String {
        format!("{}:activeEditingSessionId", self.project_prefix())
    }

    define_key_methods! {
        state_key => "state",
        state_updated_by_key => "stateUpdatedBy",
        state_updates_key => "stateUpdates",
        last_updated_at_key => "lastUpdatedAt",
    }
}

#[async_trait::async_trait]
impl RedisDataManager for FlowProjectRedisDataManager {
    type Error = FlowProjectRedisDataManagerError;

    async fn get_current_state(&self) -> Result<Option<Vec<u8>>, Self::Error> {
        let state_update = self.get_state_update_in_redis().await?;
        let merged_update = self.update_manager.get_merged_update().await?;

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

    async fn create_session(&self, session_id: &str) -> Result<(), Self::Error> {
        let _: () = self
            .get_connection()
            .await?
            .set(self.active_editing_session_id_key(), session_id)
            .await?;
        Ok(())
    }

    async fn push_update(
        &self,
        update_data: Vec<u8>,
        updated_by: Option<String>,
    ) -> Result<(), Self::Error> {
        let update_data = FlowEncodedUpdate {
            update: Self::encode_state_data(update_data)?,
            updated_by,
        };

        let value = serde_json::to_string(&update_data)?;
        let fields = &[("value", value.as_str()), ("format", "json")];

        let mut conn = self.get_connection().await?;
        let _: () = conn.xadd(self.state_updates_key()?, "*", fields).await?;

        let timestamp = chrono::Utc::now().timestamp().to_string();
        let _: () = conn.set(self.last_updated_at_key()?, &timestamp).await?;

        Ok(())
    }

    async fn clear_data(&self) -> Result<(), Self::Error> {
        let mut connection = self.get_connection().await?;

        debug!(
            "Starting to clear Redis data for project: {}",
            self.project_id
        );

        let keys_to_delete = vec![
            self.state_key()?,
            self.state_updated_by_key()?,
            self.state_updates_key()?,
            self.last_updated_at_key()?,
        ];

        debug!("Deleting keys: {:?}", keys_to_delete);

        let _: () = connection.del(&keys_to_delete).await?;

        if let Some(active_session_id) = self.get_active_session_id().await? {
            let editing_session = self.session_id.lock().await;
            if let Some(current_session_id) = editing_session.as_ref() {
                if active_session_id == *current_session_id {
                    debug!("Clearing active session ID: {}", active_session_id);
                    let _: () = connection
                        .del(&[self.active_editing_session_id_key()])
                        .await?;
                }
            }
        }

        debug!(
            "Successfully cleared all Redis data for project: {}",
            self.project_id
        );
        Ok(())
    }

    async fn merge_updates(
        &self,
        skip_lock: bool,
    ) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError> {
        if skip_lock {
            self.execute_merge_updates().await
        } else {
            self.lock_and_execute_merge_updates().await
        }
    }

    async fn get_active_session_id(
        &self,
    ) -> Result<Option<String>, FlowProjectRedisDataManagerError> {
        let mut conn = self.get_connection().await?;
        let result: Option<String> = conn.get(self.active_editing_session_id_key()).await?;
        Ok(result)
    }

    async fn set_active_session_id(
        &self,
        session_id: String,
    ) -> Result<(), FlowProjectRedisDataManagerError> {
        let _: () = self
            .get_connection()
            .await?
            .set(self.active_editing_session_id_key(), &session_id)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl RedisConnection for FlowProjectRedisDataManager {
    fn get_pool(&self) -> &Pool<RedisConnectionManager> {
        &self.redis_pool
    }
}
