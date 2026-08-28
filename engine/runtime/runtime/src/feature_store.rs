use std::fmt::Debug;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::{collections::VecDeque, env};

use once_cell::sync::Lazy;
use reearth_flow_state::State;
use reearth_flow_types::Feature;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::node::EdgeId;

static FEATURE_WRITER_DISABLE: Lazy<bool> = Lazy::new(|| {
    env::var("FLOW_RUNTIME_FEATURE_WRITER_DISABLE")
        .ok()
        .map(|s| s.to_lowercase() == "true")
        .unwrap_or(false)
});

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
    async fn write(&mut self, feature: &Feature) -> Result<(), FeatureWriterError>;
    async fn flush(&self) -> Result<(), FeatureWriterError>;
}

/// Creates the writer for one output port.
///
/// `edge_ids` are the ids of the edges leaving this port whose contents must
/// also be readable under `<edge_id>.jsonl`. The port file is written
/// incrementally; the edge files are produced once from it at flush time.
pub fn create_feature_writer(
    port_file_id: EdgeId,
    edge_ids: Vec<EdgeId>,
    state: Arc<State>,
    flush_threshold: usize,
) -> Box<dyn FeatureWriter> {
    Box::new(PrimaryKeyLookupFeatureWriter::new(
        port_file_id,
        edge_ids,
        state,
        flush_threshold,
    ))
}

#[derive(Debug, Clone)]
pub(crate) struct PrimaryKeyLookupFeatureWriter {
    edge_id: EdgeId,
    alias_edge_ids: Vec<EdgeId>,
    state: Arc<State>,
    buffer: Arc<RwLock<VecDeque<String>>>,
    thread_counter: Arc<AtomicU64>,
    flush_threshold: usize,
}

impl PrimaryKeyLookupFeatureWriter {
    pub(crate) fn new(
        edge_id: EdgeId,
        alias_edge_ids: Vec<EdgeId>,
        state: Arc<State>,
        flush_threshold: usize,
    ) -> Self {
        Self {
            edge_id,
            alias_edge_ids,
            state,
            buffer: Arc::new(RwLock::new(VecDeque::new())),
            thread_counter: Arc::new(AtomicU64::new(0)),
            flush_threshold,
        }
    }

    async fn write_inner(&self, item: serde_json::Value) -> Result<(), FeatureWriterError> {
        let item = self
            .state
            .object_to_string(&item)
            .map_err(FeatureWriterError::Serialize)?;
        let mut buffer = self.buffer.write().await;
        buffer.push_back(item);
        if buffer.len() > self.flush_threshold {
            let elements = buffer.drain(..).collect::<Vec<_>>();
            buffer.shrink_to_fit();
            self.state
                .append_strings(&elements, self.edge_id.to_string().as_str())
                .await
                .map_err(|e| FeatureWriterError::Flush(e.to_string()))?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl FeatureWriter for PrimaryKeyLookupFeatureWriter {
    async fn write(&mut self, feature: &Feature) -> Result<(), FeatureWriterError> {
        if *FEATURE_WRITER_DISABLE {
            return Ok(());
        }
        // Serialize directly from reference - no clone needed
        let item = serde_json::to_value(feature).map_err(|e| {
            FeatureWriterError::Serialize(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?;
        self.thread_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let result = self.write_inner(item).await;
        self.thread_counter
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        result
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
        drop(buffer);
        self.publish_edge_files().await
    }
}

impl PrimaryKeyLookupFeatureWriter {
    /// Copies the finished port file to `<edge_id>.jsonl` for every aliased edge.
    /// A port that never received a feature has no file, and then no edge file either.
    async fn publish_edge_files(&self) -> Result<(), FeatureWriterError> {
        if self.alias_edge_ids.is_empty() {
            return Ok(());
        }
        let port_file_id = self.edge_id.to_string();
        if !self
            .state
            .exists_jsonl(&port_file_id)
            .await
            .map_err(|e| FeatureWriterError::Flush(e.to_string()))?
        {
            return Ok(());
        }
        for alias in &self.alias_edge_ids {
            self.state
                .copy_jsonl(&port_file_id, alias.to_string().as_str())
                .await
                .map_err(|e| FeatureWriterError::Flush(e.to_string()))?;
        }
        Ok(())
    }
}
