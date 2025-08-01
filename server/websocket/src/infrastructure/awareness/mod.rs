use crate::domain::entity::awareness::AwarenessServer as aw;
use crate::domain::repository::AwarenessRepository;
use crate::domain::services::kv::DocOps;
use crate::domain::value_objects::document_name::DocumentName;
use anyhow::Result;
use std::sync::Arc;
use yrs::sync::Awareness;
use yrs::{Doc, ReadTxn, Transact};

#[async_trait::async_trait]
impl<D: for<'a> DocOps<'a>> AwarenessRepository for aw<D> {
    async fn load_awareness(&self, document_name: &DocumentName, doc: &Doc) -> Result<()> {
        let storage: &Arc<D> = self.storage();
        let mut txn = doc.transact_mut();
        storage.load_doc(document_name.as_str(), &mut txn).await?;
        Ok(())
    }

    async fn save_awareness_state(
        &self,
        document_name: &DocumentName,
        awareness: &Awareness,
    ) -> Result<()> {
        let doc = awareness.doc();
        let state = {
            let txn = doc.transact();
            txn.encode_state_as_update_v1(&yrs::StateVector::default())
        };

        let storage: &Arc<D> = self.storage();
        storage
            .save_awareness_state(document_name.as_str(), &state)
            .await?;
        Ok(())
    }

    async fn get_awareness_update(&self, document_name: &DocumentName) -> Result<Option<Bytes>> {
        // This would typically get the latest awareness changes
        // For now, return None as this requires more complex awareness state tracking
        Ok(None)
    }
}
