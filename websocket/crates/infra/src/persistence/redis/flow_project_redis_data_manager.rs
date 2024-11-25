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

    async fn set_state_with_redis(
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
                self.set_state_with_redis(project_id, state_update, state_updated_by)
                    .await
            })
            .await?
            .await
    }

    async fn get_state_in_redis(
        &self,
        project_id: &str,
    ) -> Result<Option<Vec<u8>>, FlowProjectRedisDataManagerError> {
        let mut conn = self.get_connection().await?;
        debug!("-----------------------------1");
        let state_updates_key = self.key_manager.state_key(project_id)?;
        debug!("-----------------------------2");
        debug!("state_updates_key: {}", state_updates_key);
        let state_updates: Option<Vec<u8>> = conn.get(state_updates_key).await?;
        debug!("-----------------------------3");
        Ok(state_updates)
    }

    async fn execute_merge_updates(
        &self,
        project_id: &str,
        update_data: &Vec<u8>,
        updated_by: Option<String>,
    ) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError> {
        let updated_by = updated_by.unwrap_or_default();
        self.set_update_data(project_id, update_data, Some(updated_by.clone()))
            .await?;
        debug!("Set update data--------------");
        debug!("Update data: {:?}", update_data);

        let data_in_redis = self.get_state_in_redis(project_id).await?;
        debug!("State data in redis--------------");
        debug!("State data in redis: {:?}", data_in_redis);

        let (new_merged_update, new_updates_by) = self
            .update_manager
            .merge_updates_internal(project_id, data_in_redis, Some(updated_by))
            .await?;

        debug!("New merged update: {:?}", new_merged_update);

        let update_by_json = serde_json::to_string(&new_updates_by).unwrap_or_default();

        self.set_state_data(project_id, &new_merged_update, Some(update_by_json))
            .await?;

        self.clear_state_updates_stream(project_id).await?;

        Ok((new_merged_update, new_updates_by))
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
