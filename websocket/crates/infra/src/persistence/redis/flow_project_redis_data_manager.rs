use std::sync::Arc;

use anyhow::{Context, Result};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use yrs::{updates::decoder::Decode, Doc, Transact, Update};

use flow_websocket_domain::project::ProjectEditingSession;

use crate::persistence::redis::flow_project_lock::{FlowProjectLock, GlobalLockError};
use crate::persistence::redis::redis_client::RedisClient;

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

impl FlowProjectRedisDataManager {
    pub fn new(
        project_id: String,
        editing_session: Arc<Mutex<ProjectEditingSession>>,
        redis_client: Arc<RedisClient>,
    ) -> Self {
        let redis_url = redis_client.redis_url();
        let global_lock = FlowProjectLock::new(&redis_url);
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

    async fn get_update_stream_items(&self) -> Result<Vec<(String, Vec<(String, String)>)>> {
        let stream_data = self
            .redis_client
            .xread(&self.state_updates_key(), "0-0")
            .await
            .context("Failed to read update stream")?;
        Ok(stream_data)
    }

    async fn get_flow_updates_from_stream(&self) -> Result<Vec<FlowUpdate>> {
        let stream_items = self.get_update_stream_items().await?;
        let mut updates = Vec::new();
        for stream_item in stream_items {
            let update_id = stream_item.0;
            let value = &stream_item.1[1].1;
            let format = &stream_item.1[3].1;
            let encoded_update: FlowEncodedUpdate = if format == "json" {
                serde_json::from_str(value).context("Failed to deserialize FlowEncodedUpdate")?
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
    ) -> Result<Option<(Vec<u8>, String, Vec<String>)>> {
        let updates = self.get_flow_updates_from_stream().await?;
        if updates.is_empty() {
            return Ok(None);
        }
        let merged_update = updates
            .iter()
            .map(|x| x.update.clone())
            .reduce(|a, b| {
                let doc = Doc::new();
                let mut txn = doc.transact_mut();
                txn.apply_update(Update::decode_v2(&a).unwrap());
                txn.apply_update(Update::decode_v2(&b).unwrap());
                txn.encode_update_v2()
            })
            .unwrap();
        let updates_by = updates
            .iter()
            .filter_map(|x| x.updated_by.clone())
            .collect::<Vec<_>>();
        let last_update_id = updates.last().unwrap().stream_id.clone().unwrap();
        Ok(Some((merged_update, last_update_id, updates_by)))
    }

    async fn get_state_update_in_redis(&self) -> Result<Option<Vec<u8>>> {
        let state_update_string: Option<String> = self.redis_client.get(&self.state_key()).await?;
        if let Some(state_update_string) = state_update_string {
            Ok(Some(Self::decode_state_data(state_update_string)))
        } else {
            Ok(None)
        }
    }

    pub async fn active_editing_session_id(&self) -> Result<Option<String>> {
        self.redis_client
            .get(&self.active_editing_session_id_key())
            .await
            .context("Failed to get active editing session ID")
    }

    pub async fn get_current_state_update(&self) -> Result<Option<Vec<u8>>> {
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

    pub async fn get_current_state_updated_by(&self) -> Result<Vec<String>> {
        let state_updated_by: Option<String> =
            self.redis_client.get(&self.state_updated_by_key()).await?;
        if let Some(state_updated_by) = state_updated_by {
            Ok(serde_json::from_str(&state_updated_by)
                .context("Failed to deserialize state_updated_by")?)
        } else {
            Ok(vec![])
        }
    }

    pub async fn start_editing_session(
        &self,
        state_update: Vec<u8>,
        state_updated_by: Vec<String>,
        skip_lock: bool,
    ) -> Result<()> {
        let active_editing_session_id = self.active_editing_session_id().await?;
        if let Some(active_editing_session_id) = active_editing_session_id {
            if active_editing_session_id
                != self
                    .editing_session
                    .lock()
                    .await
                    .session_id
                    .clone()
                    .unwrap()
            {
                return Err(anyhow::anyhow!("Another Editing Session is in progress"));
            }
        }
        self.set_state_data(state_update, state_updated_by, skip_lock)
            .await?;
        self.redis_client
            .set(
                self.active_editing_session_id_key(),
                &self
                    .editing_session
                    .lock()
                    .await
                    .session_id
                    .clone()
                    .unwrap(),
            )
            .await
            .context("Failed to set active editing session ID")?;
        Ok(())
    }

    async fn set_state_data(
        &self,
        state_update: Vec<u8>,
        state_updated_by: Vec<String>,
        skip_lock: bool,
    ) -> Result<()> {
        let encoded_state_update = Self::encode_state_data(state_update);
        let updated_by_json = serde_json::to_string(&state_updated_by)
            .context("Failed to serialize state_updated_by")?;

        if skip_lock {
            let connection = self.redis_client.connection();
            let mut connection_guard = connection.lock().await;
            connection_guard
                .set(self.state_key(), &encoded_state_update)
                .await
                .context("Failed to set state data")?;
            connection_guard
                .set(self.state_updated_by_key(), &updated_by_json)
                .await
                .context("Failed to set state updated by data")?;
        } else {
            self.global_lock
                .lock_state(&self.project_id, 5000, |_lock_guard| {
                    Box::pin(async {
                        let connection = self.redis_client.connection();
                        let mut connection_guard = connection.lock().await;
                        connection_guard
                            .set(self.state_key(), &encoded_state_update)
                            .await
                            .context("Failed to set state data within lock")?;
                        connection_guard
                            .set(self.state_updated_by_key(), &updated_by_json)
                            .await
                            .context("Failed to set state updated by data within lock")?;
                        Ok::<(), anyhow::Error>(())
                    })
                })
                .await
                .map_err(|e| GlobalLockError(e))?
                .await?;
        }
        Ok(())
    }

    pub async fn push_update(&self, update: Vec<u8>, updated_by: String) -> Result<()> {
        let update_data = FlowEncodedUpdate {
            update: Self::encode_state_data(update),
            updated_by: Some(updated_by),
        };

        let value =
            serde_json::to_string(&update_data).context("Failed to serialize update data")?;
        let fields = &[("value", value.as_str()), ("format", "json")];

        let connection = self.redis_client.connection();
        let mut connection_guard = connection.lock().await;

        connection_guard
            .xadd(&self.state_updates_key(), "*", fields)
            .await
            .context("Failed to add update to stream")?;

        let timestamp = chrono::Utc::now().timestamp().to_string();
        connection_guard
            .set(self.last_updated_at_key(), &timestamp)
            .await
            .context("Failed to set last updated at timestamp")?;

        Ok(())
    }

    pub async fn clear_data(&self) -> Result<()> {
        let connection = self.redis_client.connection();
        let mut connection_guard = connection.lock().await;

        let keys_to_delete = vec![
            self.state_key(),
            self.state_updated_by_key(),
            self.state_updates_key(),
            self.last_updated_at_key(),
        ];

        connection_guard
            .del(&keys_to_delete)
            .await
            .context("Failed to delete keys")?;

        if let Some(active_session_id) = self.active_editing_session_id().await? {
            let editing_session = self.editing_session.lock().await;
            if let Some(current_session_id) = editing_session.session_id.as_ref() {
                if active_session_id == *current_session_id {
                    connection_guard
                        .del(&[self.active_editing_session_id_key()])
                        .await
                        .context("Failed to delete active editing session ID key")?;
                }
            }
        }

        Ok(())
    }

    pub async fn last_updated_at(&self) -> Result<i64> {
        let last_updated_at: Option<String> = self
            .redis_client
            .get(&self.last_updated_at_key())
            .await
            .context("Failed to get last updated at timestamp")?;
        if let Some(last_updated_at) = last_updated_at {
            Ok(last_updated_at
                .parse::<i64>()
                .context("Failed to parse last updated at timestamp")?)
        } else {
            Ok(0)
        }
    }

    async fn execute_merge_updates(&self) -> Result<(Vec<u8>, Vec<String>)> {
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
            let doc = Doc::new();
            let mut txn = doc.transact_mut();
            txn.apply_update(Update::decode_v2(&state_update.unwrap()).unwrap());
            txn.apply_update(Update::decode_v2(&merged_update).unwrap());
            txn.encode_update_v2()
        } else {
            state_update.unwrap()
        };

        let merged_state_updated_by = if let Some(updates_by) = updates_by {
            let mut combined = state_updated_by;
            combined.extend(updates_by);
            combined
        } else {
            state_updated_by
        };

        let doc = Doc::new();
        let mut txn = doc.transact_mut();
        txn.apply_update(Update::decode_v2(&merged_update).unwrap());
        let optimized_merged_state = txn.encode_update_v2();

        self.set_state_data(
            optimized_merged_state.clone(),
            merged_state_updated_by.clone(),
            true,
        )
        .await?;

        if let Some(last_update_id) = last_update_id {
            self.redis_client
                .xtrim(&self.state_updates_key(), 1)
                .await
                .context("Failed to trim stream")?;
            self.redis_client
                .xdel(&self.state_updates_key(), &[last_update_id.as_str()])
                .await
                .context("Failed to delete last update from stream")?;
        }

        self.redis_client
            .xtrim(&self.state_updates_key(), 0)
            .await
            .context("Failed to trim stream to 0")?;

        Ok((optimized_merged_state, merged_state_updated_by))
    }

    pub async fn merge_updates(&self, skip_lock: bool) -> Result<(Vec<u8>, Vec<String>)> {
        if skip_lock {
            self.execute_merge_updates().await
        } else {
            self.lock_and_execute_merge_updates().await
        }
    }

    async fn lock_and_execute_merge_updates(&self) -> Result<(Vec<u8>, Vec<String>)> {
        self.global_lock
            .lock_updates(&self.project_id, 5000, |_| {
                Box::pin(async move { Ok::<(), anyhow::Error>(()) })
            })
            .await
            .map_err(GlobalLockError)?
            .await?;

        self.execute_merge_updates().await
    }
}
