use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use reearth_flow_common::runtime_config::FEATURE_WRITER_DISABLE;
use reearth_flow_state::State;
use reearth_flow_types::Feature;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::node::{EdgeId, Port};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FeatureWriterKey(pub(crate) Port, pub(crate) Port);

#[derive(Debug, Error)]
pub enum FeatureWriterError {
    #[error("Feature not found")]
    FeatureNotFound,
    #[error(transparent)]
    Serialize(#[from] std::io::Error),
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
    fn edge_id(&self) -> EdgeId;
    async fn write(&mut self, feature: &Feature) -> Result<(), FeatureWriterError>;
    async fn flush(&self) -> Result<(), FeatureWriterError>;
}

pub fn create_feature_writer(
    edge_id: EdgeId,
    state: Arc<State>,
    flush_threshold: usize,
) -> Box<dyn FeatureWriter> {
    Box::new(PrimaryKeyLookupFeatureWriter::new(
        edge_id,
        state,
        flush_threshold,
    ))
}

#[derive(Debug, Clone)]
pub(crate) struct PrimaryKeyLookupFeatureWriter {
    edge_id: EdgeId,
    state: Arc<State>,
    buffer: Arc<RwLock<VecDeque<String>>>,
    thread_counter: Arc<AtomicU64>,
    flush_threshold: usize,
}

impl PrimaryKeyLookupFeatureWriter {
    pub(crate) fn new(edge_id: EdgeId, state: Arc<State>, flush_threshold: usize) -> Self {
        Self {
            edge_id,
            state,
            buffer: Arc::new(RwLock::new(VecDeque::new())),
            thread_counter: Arc::new(AtomicU64::new(0)),
            flush_threshold,
        }
    }
}

#[async_trait::async_trait]
impl FeatureWriter for PrimaryKeyLookupFeatureWriter {
    fn edge_id(&self) -> EdgeId {
        self.edge_id.clone()
    }

    async fn write(&mut self, feature: &Feature) -> Result<(), FeatureWriterError> {
        if *FEATURE_WRITER_DISABLE {
            return Ok(());
        }
        let item: serde_json::Value = feature.clone().into();
        self.thread_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let item = self
            .state
            .object_to_string(&item)
            .map_err(FeatureWriterError::Serialize)?;
        let mut buffer = self.buffer.write().await;
        buffer.push_back(item);
        if buffer.len() > self.flush_threshold {
            let elements = buffer.drain(..).collect::<Vec<_>>();
            self.state
                .append_strings(&elements, self.edge_id.to_string().as_str())
                .await
                .map_err(|e| FeatureWriterError::Flush(e.to_string()))?;
        }
        self.thread_counter
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    async fn flush(&self) -> Result<(), FeatureWriterError> {
        if *FEATURE_WRITER_DISABLE {
            return Ok(());
        }
        while self
            .thread_counter
            .load(std::sync::atomic::Ordering::Relaxed)
            > 0
        {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        let buffer = self.buffer.read().await;
        let items = buffer.iter().cloned().collect::<Vec<_>>();
        self.state
            .append_strings(&items, self.edge_id.to_string().as_str())
            .await
            .map_err(|e| FeatureWriterError::Flush(e.to_string()))?;
        Ok(())
    }
}
