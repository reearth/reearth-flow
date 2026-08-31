use std::collections::HashMap;

use parking_lot::Mutex;
use petgraph::{graph::NodeIndex, visit::EdgeRef, Direction};
use reearth_flow_state::State;

use crate::{
    executor_operation::ExecutorContext,
    node::{EdgeId, NodeHandle},
};

use super::execution_dag::ExecutionDag;

/// One flush = one storage append (open/write/close), which costs milliseconds
/// on remote-backed filesystems — per-feature appends made large runs I/O-bound
/// (~24ms per feature on Cloud Run). Matches the feature store's granularity.
const FLUSH_LINES: usize = 512;

#[derive(Debug)]
pub struct SourceIntermediateRecorder {
    /// Track incoming edge IDs for source intermediate data
    incoming_edge_ids: Vec<EdgeId>,
    /// Track which upstream nodes are sources
    incoming_is_source: Vec<bool>,
    /// Buffered JSONL lines per edge file, flushed at FLUSH_LINES and on flush().
    buffers: Mutex<HashMap<String, Vec<String>>>,
}

impl SourceIntermediateRecorder {
    pub fn collect(dag: &ExecutionDag, node_index: NodeIndex, node_handles: &[NodeHandle]) -> Self {
        // Collect edge metadata for source intermediate data
        let mut meta_map: HashMap<String, (EdgeId, bool)> = HashMap::new();
        for e in dag.graph().edges_directed(node_index, Direction::Incoming) {
            let src = e.source();
            let w = e.weight();
            let from_handle = &dag.graph()[src].handle;
            let is_source = dag.graph()[src].is_source;
            meta_map.insert(from_handle.id.to_string(), (w.edge_id.clone(), is_source));
        }

        let mut incoming_edge_ids = Vec::new();
        let mut incoming_is_source = Vec::new();
        for nh in node_handles {
            if let Some((edge_id, is_source)) = meta_map.get(&nh.id.to_string()) {
                incoming_edge_ids.push(edge_id.clone());
                incoming_is_source.push(*is_source);
            } else {
                tracing::warn!(
                    "SourceIntermediateRecorder: No edge metadata found for upstream node {}. This may indicate a graph structure issue.",
                    nh.id
                );
                incoming_edge_ids.push(EdgeId::new(uuid::Uuid::new_v4().to_string()));
                incoming_is_source.push(false);
            }
        }

        Self {
            incoming_edge_ids,
            incoming_is_source,
            buffers: Mutex::new(HashMap::new()),
        }
    }

    pub fn record_if_from_source(
        &self,
        feature_state: &State,
        input_index: usize,
        ctx: &ExecutorContext,
        node_name: &str,
        node_id: &str,
    ) {
        let is_source = self
            .incoming_is_source
            .get(input_index)
            .copied()
            .unwrap_or(false);

        if !is_source {
            return;
        }

        let file_id = match self.incoming_edge_ids.get(input_index) {
            Some(edge_id) => edge_id.to_string(),
            None => {
                tracing::warn!(
                    "SourceIntermediateRecorder: incoming_edge_ids is missing index {} for node={}({})",
                    input_index,
                    node_name,
                    node_id,
                );
                return;
            }
        };

        let line = match feature_state.to_jsonl_line(&ctx.feature) {
            Ok(line) => line,
            Err(e) => {
                tracing::warn!(
                    "source-intermediate-serialize failed: node={}({}) edge_id={} feature_id={} err={:?}",
                    node_name,
                    node_id,
                    file_id,
                    ctx.feature.id,
                    e,
                );
                return;
            }
        };

        let full = {
            let mut buffers = self.buffers.lock();
            let buf = buffers.entry(file_id.clone()).or_default();
            buf.push(line);
            if buf.len() >= FLUSH_LINES {
                Some(std::mem::take(buf))
            } else {
                None
            }
        };
        if let Some(lines) = full {
            self.write_lines(feature_state, &file_id, lines, node_name, node_id);
        }
    }

    /// Drains every buffered edge file. Call at node terminate; a node that
    /// dies without terminating loses its tail, the same trade the feature
    /// store makes for batched writes.
    pub fn flush(&self, feature_state: &State, node_name: &str, node_id: &str) {
        let drained: Vec<(String, Vec<String>)> = self.buffers.lock().drain().collect();
        for (file_id, lines) in drained {
            self.write_lines(feature_state, &file_id, lines, node_name, node_id);
        }
    }

    fn write_lines(
        &self,
        feature_state: &State,
        file_id: &str,
        lines: Vec<String>,
        node_name: &str,
        node_id: &str,
    ) {
        if lines.is_empty() {
            return;
        }
        let count = lines.len();
        if let Err(e) = feature_state.append_jsonl_lines_sync(&lines.concat(), file_id) {
            tracing::warn!(
                "source-intermediate-append failed: node={}({}) edge_id={} lines={} err={:?}",
                node_name,
                node_id,
                file_id,
                count,
                e,
            );
        } else {
            tracing::debug!(
                "source-intermediate-append OK: node={}({}) edge_id={} lines={}",
                node_name,
                node_id,
                file_id,
                count,
            );
        }
    }
}
