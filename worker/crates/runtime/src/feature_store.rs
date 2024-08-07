use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use reearth_flow_state::State;
use reearth_flow_types::Feature;
use thiserror::Error;

use crate::node::EdgeId;

#[derive(Debug, Error)]
pub enum FeatureWriterError {
    #[error("Feature not found")]
    FeatureNotFound,
    #[error("Flush error: {0}")]
    Flush(String),
}

pub trait FeatureWriterClone {
    fn clone_box(&self) -> Box<dyn FeatureWriter>;
}

impl<T> FeatureWriterClone for T
where
    T: 'static + FeatureWriter + Clone,
{
    fn clone_box(&self) -> Box<dyn FeatureWriter> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn FeatureWriter> {
    fn clone(&self) -> Box<dyn FeatureWriter> {
        self.clone_box()
    }
}

#[async_trait::async_trait]
pub trait FeatureWriter: Send + Sync + Debug + FeatureWriterClone {
    fn write(&mut self, feature: &Feature) -> Result<(), FeatureWriterError>;
    async fn flush(&self) -> Result<(), FeatureWriterError>;
}

pub fn create_feature_writer(edge_id: EdgeId, state: Arc<State>) -> Box<dyn FeatureWriter> {
    Box::new(PrimaryKeyLookupFeatureWriter::new(edge_id, state))
}

#[derive(Debug, Clone)]
pub(crate) struct PrimaryKeyLookupFeatureWriter {
    edge_id: EdgeId,
    index: HashMap<uuid::Uuid, Feature>,
    state: Arc<State>,
}

impl PrimaryKeyLookupFeatureWriter {
    pub(crate) fn new(edge_id: EdgeId, state: Arc<State>) -> Self {
        Self {
            edge_id,
            index: Default::default(),
            state,
        }
    }
}

#[async_trait::async_trait]
impl FeatureWriter for PrimaryKeyLookupFeatureWriter {
    fn write(&mut self, feature: &Feature) -> Result<(), FeatureWriterError> {
        self.index.insert(feature.id, feature.clone());
        Ok(())
    }

    async fn flush(&self) -> Result<(), FeatureWriterError> {
        let item = self.index.values().cloned().collect::<Vec<_>>();
        let item: Vec<serde_json::Value> = item.into_iter().map(|feature| feature.into()).collect();
        self.state
            .save(&item, self.edge_id.to_string().as_str())
            .await
            .map_err(|e| FeatureWriterError::Flush(e.to_string()))
    }
}
