use std::collections::HashMap;

use petgraph::{graph::NodeIndex, visit::EdgeRef, Direction};
use reearth_flow_state::State;

use crate::{
    executor_operation::ExecutorContext,
    node::{EdgeId, NodeHandle},
};

use super::execution_dag::ExecutionDag;

#[derive(Debug)]
pub struct SourceIntermediateRecorder {
    /// Track incoming edge IDs for source intermediate data
    incoming_edge_ids: Vec<EdgeId>,
    /// Track which upstream nodes are sources
    incoming_is_source: Vec<bool>,
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

        if let Err(e) = feature_state.append_sync(&ctx.feature, &file_id) {
            tracing::warn!(
                "source-intermediate-append failed: node={}({}) edge_id={} err={:?}",
                node_name,
                node_id,
                file_id,
                e,
            );
        } else {
            tracing::debug!(
                "source-intermediate-append OK: node={}({}) edge_id={} feature_id={}",
                node_name,
                node_id,
                file_id,
                ctx.feature.id,
            );
        }
    }
}
