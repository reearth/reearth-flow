use std::sync::Arc;

use flow_websocket_domain::repository::RedisDataManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;
use yrs::{updates::decoder::Decode, Doc, Transact, Update};

use flow_websocket_domain::project::ProjectEditingSession;

use crate::persistence::redis::flow_project_lock::{FlowProjectLock, GlobalLockError};
use crate::persistence::redis::redis_client::RedisClient;

use super::redis_client::RedisClientTrait;

#[derive(Debug, Serialize, Deserialize)]
pub struct FlowUpdate {
    stream_id: Option<String>,
    update: Vec<u8>,
    updated_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlowEncodedUpdate {
    update: String,
    updated_by: Option<String>,
}

pub struct FlowProjectRedisDataManager {
    redis_client: Arc<RedisClient>,
    project_id: String,
    editing_session: Arc<Mutex<ProjectEditingSession>>,
    global_lock: FlowProjectLock,
}

#[derive(Error, Debug)]
pub enum FlowProjectRedisDataManagerError {
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Global lock error: {0}")]
    GlobalLock(#[from] GlobalLockError),
    #[error("Another Editing Session in progress")]
    EditingSessionInProgress,
    #[error("Failed to merge updates")]
    MergeUpdates,
    #[error("Failed to get last update id")]
    LastUpdateId,
    #[error("Missing state update")]
    MissingStateUpdate,
    #[error("Session not set")]
    SessionNotSet,
    #[error(transparent)]
    DecodeUpdate(#[from] yrs::encoding::read::Error),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    RedisClient(#[from] super::redis_client::RedisClientError),
    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl FlowProjectRedisDataManager {
    pub fn new(
        project_id: String,
        editing_session: Arc<Mutex<ProjectEditingSession>>,
        redis_client: Arc<RedisClient>,
    ) -> Self {
        let redis_url = redis_client.redis_url();
        let global_lock = FlowProjectLock::new(redis_url);
        Self {
            redis_client,
            project_id,
            editing_session,
            global_lock,
        }
    }

    fn project_prefix(&self) -> String {
        self.project_id.clone()
    }

    fn active_editing_session_id_key(&self) -> String {
        format!("{}:activeEditingSessionId", self.project_prefix())
    }

    fn session_prefix(&self) -> String {
        let editing_session = self.editing_session.blocking_lock();
        let session_id = editing_session.session_id.as_ref().unwrap();
        format!("{}:{}", self.project_prefix(), session_id)
    }

    fn state_key(&self) -> String {
        format!("{}:state", self.session_prefix())
    }

    fn state_updated_by_key(&self) -> String {
        format!("{}:stateUpdatedBy", self.session_prefix())
    }

    fn state_updates_key(&self) -> String {
        format!("{}:stateUpdates", self.session_prefix())
    }

    fn last_updated_at_key(&self) -> String {
        format!("{}:lastUpdatedAt", self.session_prefix())
    }

    fn encode_state_data(data: Vec<u8>) -> String {
        serde_json::to_string(&data).unwrap()
    }

    fn decode_state_data(data_string: String) -> Vec<u8> {
        serde_json::from_str(&data_string).unwrap()
    }

    async fn get_update_stream_items(
        &self,
    ) -> Result<Vec<(String, Vec<(String, String)>)>, FlowProjectRedisDataManagerError> {
        let stream_data = self
            .redis_client
            .xread(&self.state_updates_key(), "0-0")
            .await?;
        Ok(stream_data)
    }

    async fn get_flow_updates_from_stream(
        &self,
    ) -> Result<Vec<FlowUpdate>, FlowProjectRedisDataManagerError> {
        let stream_items = self.get_update_stream_items().await?;
        let mut updates = Vec::new();
        for stream_item in stream_items {
            let update_id = stream_item.0;
            let value = &stream_item.1[1].1;
            let format = &stream_item.1[3].1;
            let encoded_update: FlowEncodedUpdate = if format == "json" {
                serde_json::from_str(value)?
            } else {
                FlowEncodedUpdate {
                    update: String::new(),
                    updated_by: None,
                }
            };
            let update = FlowUpdate {
                stream_id: Some(update_id),
                update: Self::decode_state_data(encoded_update.update),
                updated_by: encoded_update.updated_by,
            };
            updates.push(update);
        }
        Ok(updates)
    }

    async fn get_merged_update_from_stream(
        &self,
    ) -> Result<Option<(Vec<u8>, String, Vec<String>)>, FlowProjectRedisDataManagerError> {
        let updates = self.get_flow_updates_from_stream().await?;
        if updates.is_empty() {
            return Ok(None);
        }

        // Create an iterator over the cloned updates
        let mut updates_iter = updates.iter().map(|x| x.update.clone());

        // Use the first update as the initial accumulator
        let first_update = updates_iter
            .next()
            .ok_or(FlowProjectRedisDataManagerError::MergeUpdates)?;

        // Fold over the remaining updates
        let merged_update = updates_iter
            .try_fold(first_update, |acc, update| {
                let doc = Doc::new();
                let mut txn = doc.transact_mut();
                txn.apply_update(Update::decode_v2(&acc)?);
                txn.apply_update(Update::decode_v2(&update)?);
                Ok(txn.encode_update_v2())
            })
            .map_err(FlowProjectRedisDataManagerError::DecodeUpdate)?;

        // Collect all updaters into a vector
        let updates_by = updates
            .iter()
            .filter_map(|x| x.updated_by.clone())
            .collect::<Vec<_>>();

        // Get the last update ID, returning an error if it's not available
        let last_update_id = updates
            .last()
            .and_then(|x| x.stream_id.clone())
            .ok_or(FlowProjectRedisDataManagerError::LastUpdateId)?;

        Ok(Some((merged_update, last_update_id, updates_by)))
    }

    async fn get_state_update_in_redis(
        &self,
    ) -> Result<Option<Vec<u8>>, FlowProjectRedisDataManagerError> {
        let state_update_string: Option<String> = self.redis_client.get(&self.state_key()).await?;
        if let Some(state_update_string) = state_update_string {
            Ok(Some(Self::decode_state_data(state_update_string)))
        } else {
            Ok(None)
        }
    }

    pub async fn active_editing_session_id(
        &self,
    ) -> Result<Option<String>, FlowProjectRedisDataManagerError> {
        self.redis_client
            .get(&self.active_editing_session_id_key())
            .await
            .map_err(FlowProjectRedisDataManagerError::from)
    }

    pub async fn get_current_state_updated_by(
        &self,
    ) -> Result<Vec<String>, FlowProjectRedisDataManagerError> {
        let state_updated_by: Option<String> =
            self.redis_client.get(&self.state_updated_by_key()).await?;
        if let Some(state_updated_by) = state_updated_by {
            Ok(serde_json::from_str(&state_updated_by)?)
        } else {
            Ok(vec![])
        }
    }

    pub async fn start_editing_session(
        &self,
        state_update: Vec<u8>,
        state_updated_by: Vec<String>,
        skip_lock: bool,
    ) -> Result<(), FlowProjectRedisDataManagerError> {
        let active_editing_session_id = self.active_editing_session_id().await?;

        let current_session_id = self
            .editing_session
            .lock()
            .await
            .session_id
            .as_ref()
            .ok_or(FlowProjectRedisDataManagerError::SessionNotSet)?
            .clone();

        if let Some(active_editing_session_id) = active_editing_session_id {
            if active_editing_session_id != current_session_id {
                return Err(FlowProjectRedisDataManagerError::EditingSessionInProgress);
            }
        }

        self.set_state_data(state_update, state_updated_by, skip_lock)
            .await?;

        self.redis_client
            .set(&self.active_editing_session_id_key(), &current_session_id)
            .await?;

        Ok(())
    }

    async fn set_state_data_internal(
        &self,
        encoded_state_update: &str,
        updated_by_json: &str,
    ) -> Result<(), FlowProjectRedisDataManagerError> {
        let connection = self.redis_client.connection();
        let mut connection_guard = connection.lock().await;
        let _: () = connection_guard
            .set(self.state_key(), encoded_state_update)
            .await?;
        let _: () = connection_guard
            .set(self.state_updated_by_key(), updated_by_json)
            .await?;
        Ok(())
    }

    async fn set_state_data(
        &self,
        state_update: Vec<u8>,
        state_updated_by: Vec<String>,
        skip_lock: bool,
    ) -> Result<(), FlowProjectRedisDataManagerError> {
        let encoded_state_update = Self::encode_state_data(state_update);
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

    pub async fn last_updated_at(&self) -> Result<i64, FlowProjectRedisDataManagerError> {
        let last_updated_at: Option<String> =
            self.redis_client.get(&self.last_updated_at_key()).await?;
        if let Some(last_updated_at) = last_updated_at {
            Ok(last_updated_at.parse::<i64>()?)
        } else {
            Ok(0)
        }
    }

    async fn execute_merge_updates(
        &self,
    ) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError> {
        let doc = Arc::new(Doc::new());

        let merged_stream_update = self.get_merged_update_from_stream().await?;
        let state_update = self.get_state_update_in_redis().await?;
        let state_updated_by = self.get_current_state_updated_by().await?;

        let (merged_update, last_update_id, updates_by) =
            if let Some((merged_update, last_update_id, updates_by)) = merged_stream_update {
                (Some(merged_update), Some(last_update_id), Some(updates_by))
            } else {
                (None, None, None)
            };

        let merged_update = if let Some(merged_update) = merged_update {
            let doc_clone = Arc::clone(&doc);
            tokio::task::spawn_blocking(move || {
                let mut txn = doc_clone.transact_mut();
                match state_update {
                    Some(ref state_update) => {
                        txn.apply_update(Update::decode_v2(state_update)?);
                    }
                    None => return Err(FlowProjectRedisDataManagerError::MissingStateUpdate),
                }
                txn.apply_update(Update::decode_v2(&merged_update)?);
                Ok::<_, FlowProjectRedisDataManagerError>(txn.encode_update_v2())
            })
            .await??
        } else {
            match state_update {
                Some(state_update) => state_update,
                None => return Err(FlowProjectRedisDataManagerError::MissingStateUpdate),
            }
        };

        let merged_state_updated_by = if let Some(updates_by) = updates_by {
            let mut combined = state_updated_by;
            combined.extend(updates_by);
            combined
        } else {
            state_updated_by
        };

        let doc_clone = Arc::clone(&doc);
        let optimized_merged_state = tokio::task::spawn_blocking(move || {
            let mut txn = doc_clone.transact_mut();
            txn.apply_update(Update::decode_v2(&merged_update)?);
            Ok::<_, FlowProjectRedisDataManagerError>(txn.encode_update_v2())
        })
        .await??;

        self.set_state_data(
            optimized_merged_state.clone(),
            merged_state_updated_by.clone(),
            true,
        )
        .await?;

        if let Some(last_update_id) = last_update_id {
            self.redis_client
                .xtrim(&self.state_updates_key(), 1)
                .await?;
            self.redis_client
                .xdel(&self.state_updates_key(), &[last_update_id.as_str()])
                .await?;
        }

        self.redis_client
            .xtrim(&self.state_updates_key(), 0)
            .await?;

        Ok((optimized_merged_state, merged_state_updated_by))
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

#[async_trait::async_trait]
impl RedisDataManager for FlowProjectRedisDataManager {
    type Error = FlowProjectRedisDataManagerError;

    async fn get_current_state(&self) -> Result<Option<Vec<u8>>, Self::Error> {
        let state_update = self.get_state_update_in_redis().await?;
        if state_update.is_none() {
            return Ok(None);
        }
        let merged_stream_update = self.get_merged_update_from_stream().await?;
        if let Some((merged_update, _, _)) = merged_stream_update {
            let doc = Doc::new();
            let mut txn = doc.transact_mut();
            txn.apply_update(Update::decode_v2(&state_update.unwrap()).unwrap());
            txn.apply_update(Update::decode_v2(&merged_update).unwrap());
            Ok(Some(txn.encode_update_v2()))
        } else {
            Ok(state_update)
        }
    }

    async fn push_update(&self, update: Vec<u8>, updated_by: String) -> Result<(), Self::Error> {
        let update_data: FlowEncodedUpdate = FlowEncodedUpdate {
            update: Self::encode_state_data(update),
            updated_by: Some(updated_by),
        };

        let value = serde_json::to_string(&update_data)?;
        let fields = &[("value", value.as_str()), ("format", "json")];

        let connection = self.redis_client.connection();
        let mut connection_guard = connection.lock().await;

        let _: () = connection_guard
            .xadd(self.state_updates_key(), "*", fields)
            .await?;

        let timestamp = chrono::Utc::now().timestamp().to_string();
        let _: () = connection_guard
            .set(self.last_updated_at_key(), &timestamp)
            .await?;

        Ok(())
    }

    async fn clear_data(&self) -> Result<(), Self::Error> {
        let connection = self.redis_client.connection();
        let mut connection_guard = connection.lock().await;

        let keys_to_delete = vec![
            self.state_key(),
            self.state_updated_by_key(),
            self.state_updates_key(),
            self.last_updated_at_key(),
        ];

        let _: () = connection_guard.del(&keys_to_delete).await?;

        if let Some(active_session_id) = self.active_editing_session_id().await? {
            let editing_session = self.editing_session.lock().await;
            if let Some(current_session_id) = editing_session.session_id.as_ref() {
                if active_session_id == *current_session_id {
                    let _: () = connection_guard
                        .del(&[self.active_editing_session_id_key()])
                        .await?;
                }
            }
        }

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    type MergeUpdatesResult = Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError>;

    #[derive(Clone)]
    pub struct MockFlowProjectRedisDataManager {
        pub current_state: Arc<Mutex<Option<Vec<u8>>>>,
        pub push_update_result: Arc<Mutex<Result<(), FlowProjectRedisDataManagerError>>>,
        pub merge_updates_result: Arc<Mutex<MergeUpdatesResult>>,
    }

    impl MockFlowProjectRedisDataManager {
        pub fn new() -> Self {
            MockFlowProjectRedisDataManager {
                current_state: Arc::new(Mutex::new(Some(vec![1, 2, 3]))),
                push_update_result: Arc::new(Mutex::new(Ok(()))),
                merge_updates_result: Arc::new(Mutex::new(Ok((
                    vec![1, 2, 3],
                    vec!["user1".to_string()],
                )))),
            }
        }

        pub fn set_current_state(&self, state: Option<Vec<u8>>) {
            let mut current_state = self.current_state.lock().unwrap();
            *current_state = state;
        }

        pub fn set_push_update_result(&self, result: Result<(), FlowProjectRedisDataManagerError>) {
            let mut push_update_result = self.push_update_result.lock().unwrap();
            *push_update_result = result;
        }

        pub fn set_merge_updates_result(
            &self,
            result: Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError>,
        ) {
            let mut merge_updates_result = self.merge_updates_result.lock().unwrap();
            *merge_updates_result = result;
        }
    }

    #[async_trait]
    impl RedisDataManager for MockFlowProjectRedisDataManager {
        type Error = FlowProjectRedisDataManagerError;

        async fn get_current_state(&self) -> Result<Option<Vec<u8>>, Self::Error> {
            let state = self.current_state.lock().unwrap();
            Ok(state.clone())
        }

        async fn push_update(
            &self,
            _update: Vec<u8>,
            _updated_by: String,
        ) -> Result<(), Self::Error> {
            let result = self
                .push_update_result
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .to_owned();
            Ok(result)
        }

        async fn clear_data(&self) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn merge_updates(
            &self,
            _skip_lock: bool,
        ) -> Result<(Vec<u8>, Vec<String>), Self::Error> {
            let result = self
                .merge_updates_result
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .to_owned();
            Ok(result)
        }
    }
    #[tokio::test]
    async fn test_get_current_state() {
        let mock_manager = MockFlowProjectRedisDataManager::new();

        mock_manager.set_current_state(Some(vec![1, 2, 3]));

        let result = mock_manager.get_current_state().await;
        assert_eq!(result.unwrap(), Some(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn test_push_update() {
        let mock_manager = MockFlowProjectRedisDataManager::new();

        mock_manager.set_push_update_result(Ok(()));

        let result = mock_manager
            .push_update(vec![1, 2, 3], "user1".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_merge_updates() {
        let mock_manager = MockFlowProjectRedisDataManager::new();

        mock_manager.set_merge_updates_result(Ok((vec![1, 2, 3], vec!["user1".to_string()])));

        let result = mock_manager.merge_updates(false).await;
        assert_eq!(result.unwrap(), (vec![1, 2, 3], vec!["user1".to_string()]));
    }

    #[tokio::test]
    async fn test_get_current_state_empty() {
        let mock_manager = MockFlowProjectRedisDataManager::new();

        mock_manager.set_current_state(None);

        let result = mock_manager.get_current_state().await;
        assert!(result.unwrap().is_none());
    }
}
