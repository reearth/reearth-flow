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
    ) -> Result<Option<(Vec<u8>, String, Vec<String>)>, FlowProjectRedisDataManagerError> {
        let updates = self.get_flow_updates_from_stream(project_id).await?;
        if updates.is_empty() {
            return Ok(None);
        }

        let doc = Doc::new();
        let mut txn = doc.transact_mut();
        let mut updates_by = Vec::new();
        let mut last_stream_id = String::new();

        for u in updates {
            debug!("Processing update: {:?}", u);
            if let Some(stream_id) = u.stream_id {
                last_stream_id = stream_id;
            }
            if let Some(updated_by) = u.updated_by {
                updates_by.push(updated_by);
            }
            if !u.update.is_empty() {
                txn.apply_update(Update::decode_v2(&u.update)?);
            }
        }
        debug!("Last stream id: {}", last_stream_id);

        Ok(Some((txn.encode_update_v2(), last_stream_id, updates_by)))
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

    pub async fn merge_updates_internal(
        &self,
        project_id: &str,
        state_update: Option<Vec<u8>>,
    ) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError> {
        let doc = Arc::new(Doc::new());
        let merged_stream_update = self.get_merged_update_from_stream(project_id).await?;
        debug!("Merged stream update: {:?}", merged_stream_update);

        let (merged_update, updates_by) = if let Some((merged_update, _, updates_by)) =
            merged_stream_update
        {
            (merged_update, updates_by)
        } else {
            match state_update {
                Some(update) => (update, vec![]),
                None => {
                    return Err(FlowProjectRedisDataManagerError::MissingStateUpdate {
                        key: self.key_manager.state_key(project_id)?,
                        context: format!("No state update available for project: {}", project_id),
                    })
                }
            }
        };

        let doc_clone = Arc::clone(&doc);
        let optimized_merged_state = tokio::task::spawn_blocking(move || {
            let mut txn = doc_clone.transact_mut();
            txn.apply_update(Update::decode_v2(&merged_update)?);
            Ok::<_, FlowProjectRedisDataManagerError>(txn.encode_update_v2())
        })
        .await??;

        Ok((optimized_merged_state, updates_by))
    }
}
