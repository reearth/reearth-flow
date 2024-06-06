use crate::persistence::redis::redis_client::RedisClient;
use crate::persistence::redis::flow_project_lock::FlowProjectLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use yrs::{Doc, Transact, Update, updates::decoder::Decode};
use std::error::Error;

use flow_websocket_domain::project::ProjectEditingSession;


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
    pub fn new(project_id: String, editing_session: Arc<Mutex<ProjectEditingSession>>, redis_client: Arc<RedisClient>) -> Self {
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

    async fn get_update_stream_items(&self) -> Result<Vec<(String, Vec<(String, String)>)>, Box<dyn Error>> {
        let stream_data = self.redis_client.xread(&self.state_updates_key(), "0-0").await?;
        Ok(stream_data)
    }

    async fn get_flow_updates_from_stream(&self) -> Result<Vec<FlowUpdate>, Box<dyn Error>> {
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

    async fn get_merged_update_from_stream(&self) -> Result<Option<(Vec<u8>, String, Vec<String>)>, Box<dyn Error>> {
        let updates = self.get_flow_updates_from_stream().await?;
        if updates.is_empty() {
            return Ok(None);
        }
        let merged_update = updates.iter().map(|x| x.update.clone()).reduce(|a, b| {
            let doc = Doc::new();
            let mut txn = doc.transact_mut();
            txn.apply_update(Update::decode_v2(&a).unwrap());
            txn.apply_update(Update::decode_v2(&b).unwrap());
            txn.encode_update_v2()
        }).unwrap();
        let updates_by = updates.iter().filter_map(|x| x.updated_by.clone()).collect::<Vec<_>>();
        let last_update_id = updates.last().unwrap().stream_id.clone().unwrap();
        Ok(Some((merged_update, last_update_id, updates_by)))
    }

    async fn get_state_update_in_redis(&self) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        let state_update_string: Option<String> = self.redis_client.get(&self.state_key()).await?;
        if let Some(state_update_string) = state_update_string {
            Ok(Some(Self::decode_state_data(state_update_string)))
        } else {
            Ok(None)
        }
    }

    pub async fn active_editing_session_id(&self) -> Result<Option<String>, Box<dyn Error>> {
        self.redis_client.get(&self.active_editing_session_id_key()).await
    }

    pub async fn get_current_state_update(&self) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
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

    pub async fn get_current_state_updated_by(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let state_updated_by: Option<String> = self.redis_client.get(&self.state_updated_by_key()).await?;
        if let Some(state_updated_by) = state_updated_by {
            Ok(serde_json::from_str(&state_updated_by)?)
        } else {
            Ok(vec![])
        }
    }

    pub async fn start_editing_session(&self, state_update: Vec<u8>, state_updated_by: Vec<String>, skip_lock: bool) -> Result<(), Box<dyn Error>> {
        let active_editing_session_id = self.active_editing_session_id().await?;
        if let Some(active_editing_session_id) = active_editing_session_id {
            if active_editing_session_id != self.editing_session.lock().await.session_id.clone().unwrap() {
                return Err("Another Editing Session in progress".into());
            }
        }
        self.set_state_data(state_update, state_updated_by, skip_lock).await?;
        self.redis_client.set(self.active_editing_session_id_key(), &self.editing_session.lock().await.session_id.clone().unwrap()).await?;
        Ok(())
    }

    async fn set_state_data(&self, state_update: Vec<u8>, state_updated_by: Vec<String>, skip_lock: bool) -> Result<(), Box<dyn Error>> {
        let encoded_state_update = Self::encode_state_data(state_update);
        if skip_lock {
            self.redis_client.set(self.state_key(), &encoded_state_update).await?;
            self.redis_client.set(self.state_updated_by_key(), &serde_json::to_string(&state_updated_by)?).await?;
        } else {
            self.global_lock.lock_state(&self.project_id, 5000, |lock| {
                Box::pin(async {
                    self.redis_client.set(self.state_key(), &encoded_state_update).await?;
                    self.redis_client.set(self.state_updated_by_key(), &serde_json::to_string(&state_updated_by)?).await?;
                    Ok(())
                })
            }).await?;
        }
        Ok(())
    }

    pub async fn push_update(&self, update: Vec<u8>, updated_by: String) -> Result<(), Box<dyn Error>> {
        let update_data = FlowEncodedUpdate {
            update: Self::encode_state_data(update),
            updated_by: Some(updated_by),
        };
        let fields = &[("value", &serde_json::to_string(&update_data)?), ("format", "json")];
        self.redis_client.xadd(&self.state_updates_key(), "*", fields).await?;
        self.redis_client.set(self.last_updated_at_key(), &format!("{}", chrono::Utc::now().timestamp())).await?;
        Ok(())
    }

    pub async fn clear_data(&self) -> Result<(), Box<dyn Error>> {
        let mut cmds = self.redis_client.connection().lock();
        cmds.del(&self.state_key());
        cmds.del(&self.state_updated_by_key());
        cmds.del(&self.state_updates_key());
        cmds.del(&self.last_updated_at_key());

        if self.active_editing_session_id().await? == Some(self.editing_session.lock().await.session_id.clone().unwrap()) {
            cmds.del(&self.active_editing_session_id_key());
        }

        cmds.exec().await?;
        Ok(())
    }

    pub async fn last_updated_at(&self) -> Result<i64, Box<dyn Error>> {
        let last_updated_at: Option<String> = self.redis_client.get(&self.last_updated_at_key()).await?;
        if let Some(last_updated_at) = last_updated_at {
            Ok(last_updated_at.parse::<i64>()?)
        } else {
            Ok(0)
        }
    }

    pub async fn merge_updates(&self, skip_lock: bool) -> Result<(Vec<u8>, Vec<String>), Box<dyn Error>> {
        let execute_merge_updates = || {
            Box::pin(async {
                let merged_stream_update = self.get_merged_update_from_stream().await?;
                let state_update = self.get_state_update_in_redis().await?;
                let state_updated_by = self.get_current_state_updated_by().await?;
                let merged_update = if let Some((merged_update, _, _)) = merged_stream_update {
                    let mut doc = Doc::new();
                    let mut txn = doc.transact_mut();
                    txn.apply_update(Update::decode_v2(&state_update.unwrap()).unwrap());
                    txn.apply_update(Update::decode_v2(&merged_update).unwrap());
                    txn.encode_update_v2()
                } else {
                    state_update.unwrap()
                };
                let merged_state_updated_by = if let Some((_, _, updates_by)) = merged_stream_update {
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
                self.set_state_data(optimized_merged_state.clone(), merged_state_updated_by.clone(), true).await?;
                if let Some((_, last_update_id, _)) = merged_stream_update {
                    self.redis_client.xtrim(&self.state_updates_key(), 1).await?;
                    self.redis_client.xdel(&self.state_updates_key(), &last_update_id).await?;
                }
                self.redis_client.xtrim(&self.state_updates_key(), 0).await?;
                Ok((optimized_merged_state, merged_state_updated_by))
            })
        };
        if skip_lock {
            execute_merge_updates().await
        } else {
            self.global_lock.lock_updates(&self.project_id, 5000, |_| execute_merge_updates()).await
        }
    }
}
