use super::{
    errors::FlowProjectRedisDataManagerError,
    keys::RedisKeyManager,
    types::{FlowUpdate, StreamItems},
};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use std::sync::Arc;
use tracing::debug;
use yrs::{updates::decoder::Decode, Doc, Transact, Update};

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
    ) -> Result<Option<(Vec<u8>, Vec<String>)>, FlowProjectRedisDataManagerError> {
        let updates = self.get_flow_updates_from_stream(project_id).await?;
        if updates.is_empty() {
            return Ok(None);
        }

        let doc = Doc::new();
        let mut txn = doc.transact_mut();
        let mut updates_by = Vec::new();

        for u in updates {
            debug!("Processing update: {:?}", u);
            if let Some(updated_by) = u.updated_by {
                updates_by.push(updated_by);
            }
            if !u.update.is_empty() {
                let _ = txn.apply_update(Update::decode_v2(&u.update)?);
            }
        }
        Ok(Some((txn.encode_update_v2(), updates_by)))
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
        debug!(
            "Starting merge_updates_internal for project_id: {}, has_redis_data: {}, new_update_by: {:?}",
            project_id,
            redis_data.is_some(),
            new_update_by
        );
    
        let (stream_update, stream_updates_by) = match self.get_merged_update_from_stream(project_id).await {
            Ok(Some((update, updates_by))) => {
                debug!(
                    "Retrieved stream update for project {}: size={}, updates_by={:?}",
                    project_id,
                    update.len(),
                    updates_by
                );
                (update, updates_by)
            }
            Ok(None) => {
                debug!("No existing stream updates found for project {}", project_id);
                Default::default()
            }
            Err(e) => {
                debug!(
                    "Error getting merged update from stream for project {}: {:?}",
                    project_id, e
                );
                return Err(e);
            }
        };
    
        let redis_update = redis_data.unwrap_or_default();
        debug!(
            "Redis update size for project {}: {}",
            project_id,
            redis_update.len()
        );
    
        debug!(
            "Spawning blocking task to merge updates for project {}",
            project_id
        );
        let optimized_merged_state = tokio::task::spawn_blocking(move || {
            debug!("Starting merge operation in blocking task");
            let doc = Doc::new();
            let mut txn = doc.transact_mut();
    
            if !redis_update.is_empty() {
                debug!("Applying redis update of size {}", redis_update.len());
                match Update::decode_v2(&redis_update) {
                    Ok(update) => {
                        if let Err(e) = txn.apply_update(update) {
                            debug!("Error applying redis update: {:?}", e);
                            return Err(FlowProjectRedisDataManagerError::from(e));
                        }
                    }
                    Err(e) => {
                        debug!("Error decoding redis update: {:?}", e);
                        return Err(FlowProjectRedisDataManagerError::from(e));
                    }
                }
            }
    
            if !stream_update.is_empty() {
                debug!("Applying stream update of size {}", stream_update.len());
                match Update::decode_v2(&stream_update) {
                    Ok(update) => {
                        if let Err(e) = txn.apply_update(update) {
                            debug!("Error applying stream update: {:?}", e);
                            return Err(FlowProjectRedisDataManagerError::from(e));
                        }
                    }
                    Err(e) => {
                        debug!("Error decoding stream update: {:?}", e);
                        return Err(FlowProjectRedisDataManagerError::from(e));
                    }
                }
            }
    
            let result = txn.encode_update_v2();
            debug!("Successfully encoded merged update of size {}", result.len());
            Ok(result)
        })
        .await
        .map_err(|e| {
            debug!("Join error from blocking task: {:?}", e);
            FlowProjectRedisDataManagerError::from(e)
        })??;
    
        let mut updates_by = stream_updates_by;
        if let Some(new_update_by) = new_update_by {
            debug!(
                "Adding new update attribution for project {}: {}",
                project_id, new_update_by
            );
            updates_by.push(new_update_by);
        }
    
        debug!(
            "Completed merge_updates_internal for project {}: final_size={}, updates_by={:?}",
            project_id,
            optimized_merged_state.len(),
            updates_by
        );
    
        Ok((optimized_merged_state, updates_by))
    }
}
