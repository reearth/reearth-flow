use std::collections::HashMap;
use std::fmt::Debug;

use reearth_flow_types::Feature;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FeatureWriterError {
    #[error("Feature not found")]
    FeatureNotFound,
}

pub trait FeatureWriter: Send + Sync + Debug {
    fn write(&mut self, feature: &Feature) -> Result<(), FeatureWriterError>;
}

pub fn create_feature_writer() -> Box<dyn FeatureWriter> {
    Box::new(PrimaryKeyLookupFeatureWriter::new())
}

#[derive(Debug)]
pub(crate) struct PrimaryKeyLookupFeatureWriter {
    index: HashMap<uuid::Uuid, Feature>,
}

impl PrimaryKeyLookupFeatureWriter {
    pub(crate) fn new() -> Self {
        Self {
            index: Default::default(),
        }
    }
}

impl FeatureWriter for PrimaryKeyLookupFeatureWriter {
    fn write(&mut self, feature: &Feature) -> Result<(), FeatureWriterError> {
        self.index.insert(feature.id, feature.clone());
        Ok(())
    }
}
