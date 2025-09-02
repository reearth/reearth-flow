use crate::domain::entity::awareness::AwarenessServer as aw;

use crate::domain::repository::{AwarenessRepository, RedisRepository};
use crate::domain::services::kv::DocOps;
use crate::domain::value_objects::document_name::DocumentName;
use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::RwLock;
use yrs::sync::Awareness;
use yrs::{Doc, ReadTxn, StateVector, Transact};

#[async_trait::async_trait]
impl<D: for<'a> DocOps<'a>> AwarenessRepository for aw<D> {
    async fn load_awareness(&self, document_name: &DocumentName) -> Result<Arc<RwLock<Awareness>>> {
        let doc = Doc::new();
        let storage: &Arc<D> = self.storage();
        let mut txn = doc.transact_mut();
        storage.load_doc(document_name.as_str(), &mut txn).await?;
        Ok(Arc::new(RwLock::new(Awareness::new(doc.clone()))))
    }

    async fn save_awareness_state<T: RedisRepository>(
        &self,
        document_name: &DocumentName,
        awareness: &Awareness,
        redis: &T,
    ) -> Result<()> {
        let doc = awareness.doc();
        let state = {
            let txn = doc.transact();
            txn.encode_state_as_update_v1(&yrs::StateVector::default())
        };

        let storage: &Arc<D> = self.storage();
        storage
            .push_update(document_name.as_str(), &state, redis)
            .await?;
        Ok(())
    }

    async fn get_awareness_update(&self, document_name: &DocumentName) -> Result<Option<Bytes>> {
        let storage: &Arc<D> = self.storage();
        let update = storage
            .get_diff(document_name.as_str(), &StateVector::default())
            .await?;
        Ok(update.map(Bytes::from))
    }
}
