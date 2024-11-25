use super::{
    errors::FlowProjectRedisDataManagerError,
    keys::RedisKeyManager,
    types::{FlowUpdate, StreamItems},
};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use std::sync::Arc;
use tracing::debug;
use yrs::{updates::decoder::Decode, Doc, ReadTxn, StateVector, Transact, Update};

type RedisStreamResult = Vec<(String, Vec<(String, Vec<(String, Vec<u8>)>)>)>;

pub struct UpdateManager {
    redis_pool: Pool<RedisConnectionManager>,
    key_manager: Arc<dyn RedisKeyManager>,
}

impl UpdateManager {
    pub fn new(
        redis_pool: Pool<RedisConnectionManager>,
        key_manager: Arc<dyn RedisKeyManager>,
    ) -> Self {
        Self {
            redis_pool,
            key_manager,
        }
    }

    pub async fn get_merged_update_from_stream(
        &self,
        project_id: &str,
    ) -> Result<Option<(Vec<Vec<u8>>, Vec<String>)>, FlowProjectRedisDataManagerError> {
        let updates = self.get_flow_updates_from_stream(project_id).await?;
        if updates.is_empty() {
            return Ok(None);
        }

        let mut datas = Vec::new();
        let mut updates_by = Vec::new();

        for u in updates {
            if let Some(updated_by) = u.updated_by {
                updates_by.push(updated_by);
            }
            datas.push(u.update);
        }
        Ok(Some((datas, updates_by)))
    }

    pub async fn get_update_stream_items(
        &self,
        project_id: &str,
    ) -> Result<StreamItems, FlowProjectRedisDataManagerError> {
        let mut conn = self.redis_pool.get().await?;
        let key = self.key_manager.state_updates_key(project_id)?;

        let result: RedisStreamResult = redis::cmd("XREAD")
            .arg("STREAMS")
            .arg(&key)
            .arg("0-0")
            .query_async(&mut *conn)
            .await?;

        let stream_items = result
            .into_iter()
            .flat_map(|(_, entries)| entries)
            .collect();

        Ok(stream_items)
    }

    pub async fn get_flow_updates_from_stream(
        &self,
        project_id: &str,
    ) -> Result<Vec<FlowUpdate>, FlowProjectRedisDataManagerError> {
        let stream_items = self.get_update_stream_items(project_id).await?;
        let mut updates = Vec::new();

        for (update_id, items) in stream_items {
            for (updated_by, update_data) in items {
                updates.push(FlowUpdate {
                    stream_id: Some(update_id.clone()),
                    update: update_data,
                    updated_by: Some(updated_by),
                });
            }
        }

        Ok(updates)
    }

    /// Merge updates from stream and redis
    pub async fn merge_updates_internal(
        &self,
        project_id: &str,
        redis_data: Option<Vec<u8>>,
        new_update_by: Option<String>,
    ) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError> {
        let stream_updates = self.get_merged_update_from_stream(project_id).await?;

        let updates_by = match &stream_updates {
            Some((_, by)) => {
                let mut updates_by = by.clone();
                if let Some(new_update_by) = new_update_by {
                    updates_by.push(new_update_by);
                }
                updates_by
            }
            None => new_update_by.map(|u| vec![u]).unwrap_or_default(),
        };

        let optimized_merged_state = tokio::task::spawn_blocking(
            move || -> Result<Vec<u8>, FlowProjectRedisDataManagerError> {
                let doc = Doc::new();
                let mut txn = doc.transact_mut();

                if let Some(redis_update) = redis_data {
                    txn.apply_update(Update::decode_v2(&redis_update)?)?;
                }

                if let Some((updates, _)) = stream_updates {
                    for update in updates {
                        txn.apply_update(Update::decode_v2(&update)?)?;
                    }
                }

                Ok(txn.encode_update_v2())
            },
        )
        .await??;

        debug!("Final merged state: {:?}", optimized_merged_state);
        Ok((optimized_merged_state, updates_by))
    }

    pub async fn get_current_state(
        &self,
        project_id: &str,
    ) -> Result<Option<Vec<u8>>, FlowProjectRedisDataManagerError> {
        let state_key = self.key_manager.state_key(project_id)?;
        let current_state: Option<Vec<u8>> = self.redis_pool.get().await?.get(state_key).await?;
        Ok(current_state)
    }

    pub async fn handle_state_vector(
        &self,
        project_id: &str,
        state_vector: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, FlowProjectRedisDataManagerError> {
        let current_state = self.get_current_state(project_id).await?;

        if let Some(server_state) = current_state {
            let diff_update = tokio::task::spawn_blocking(move || {
                let doc = Doc::new();
                let mut txn = doc.transact_mut();

                match Update::decode_v2(&server_state) {
                    Ok(update) => {
                        txn.apply_update(update)?;
                    }
                    Err(e) => return Err(FlowProjectRedisDataManagerError::from(e)),
                }

                let client_state_vector = StateVector::decode_v1(&state_vector)?;

                let diff = txn.encode_state_as_update_v2(&client_state_vector);
                Ok(diff)
            })
            .await
            .map_err(FlowProjectRedisDataManagerError::from)??;

            if !diff_update.is_empty() {
                debug!("Generated diff update of size: {}", diff_update.len());
                Ok(Some(diff_update))
            } else {
                debug!("No updates needed for client");
                Ok(None)
            }
        } else {
            debug!("No server state exists yet");
            Ok(None)
        }
    }
}
