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
        let (stream_update, stream_updates_by) =
            (self.get_merged_update_from_stream(project_id).await?).unwrap_or_default();

        let redis_update = redis_data.unwrap_or_default();

        let optimized_merged_state = tokio::task::spawn_blocking(
            move || -> Result<Vec<u8>, FlowProjectRedisDataManagerError> {
                let doc = Doc::new();
                let mut txn = doc.transact_mut();

                if !redis_update.is_empty() {
                    txn.apply_update(Update::decode_v2(&redis_update)?)?;
                }

                if !stream_update.is_empty() {
                    txn.apply_update(Update::decode_v2(&stream_update)?)?;
                }

                Ok(txn.encode_update_v2())
            },
        )
        .await??;

        let mut updates_by = stream_updates_by;
        if let Some(new_update_by) = new_update_by {
            updates_by.push(new_update_by);
        }

        debug!("Final merged state: {:?}", optimized_merged_state);
        Ok((optimized_merged_state, updates_by))
    }
}
