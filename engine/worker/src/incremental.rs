use std::collections::{HashMap, HashSet, VecDeque};

use reearth_flow_common::dir::setup_job_directory;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Workflow;

use crate::artifact::artifact_feature_store_root_uri;
use crate::types::metadata::Metadata;

pub async fn prepare_incremental_feature_store(
    workflow: &Workflow,
    job_id: uuid::Uuid,
    storage_resolver: &StorageResolver,
    metadata: &Metadata,
    previous_job_id: uuid::Uuid,
    start_node_id: uuid::Uuid,
) -> crate::errors::Result<()> {
    tracing::info!(
        "Incremental run: previous_job_id={}, start_node_id={}",
        previous_job_id,
        start_node_id
    );

    let prev_feature_store_uri = artifact_feature_store_root_uri(metadata, previous_job_id)?;
    tracing::info!(
        "Incremental run: previous feature-store root = {}",
        prev_feature_store_uri.path().display()
    );
    let prev_feature_store_state = State::new(&prev_feature_store_uri, storage_resolver)
        .map_err(crate::errors::Error::init)?;

    let reuse_feature_store_uri = setup_job_directory("workers", "previous-feature-store", job_id)
        .map_err(crate::errors::Error::init)?;
    tracing::info!(
        "Incremental run: reuse feature-store root = {}",
        reuse_feature_store_uri.path().display()
    );
    let reuse_state = State::new(&reuse_feature_store_uri, storage_resolver)
        .map_err(crate::errors::Error::init)?;

    let edge_ids = collect_reusable_edge_ids(workflow, start_node_id)?;
    tracing::info!(
        "Incremental run: reusable edge IDs for node {}: {:?}",
        start_node_id,
        edge_ids
    );
    tracing::info!(
        "Incremental run: copying {} edge(s) into previous-feature-store",
        edge_ids.len()
    );

    for edge_id in edge_ids {
        let edge_id_str = edge_id.to_string();

        match reuse_state
            .copy_jsonl_from_state_async(&prev_feature_store_state, &edge_id_str)
            .await
        {
            Ok(()) => {
                tracing::info!(
                    "Incremental run: copied edge {} into {}",
                    edge_id_str,
                    reuse_feature_store_uri.path().display()
                );
            }
            Err(e) => {
                tracing::warn!(
                    "Incremental run: failed to copy edge {} from previous feature-store: {:?}",
                    edge_id_str,
                    e
                );
            }
        }
    }

    Ok(())
}

pub fn collect_reusable_edge_ids(
    workflow: &Workflow,
    start_node_id: uuid::Uuid,
) -> crate::errors::Result<Vec<uuid::Uuid>> {
    let graph = workflow
        .graphs
        .iter()
        .find(|g| g.id == workflow.entry_graph_id)
        .ok_or_else(|| crate::errors::Error::init("Entry graph not found"))?;

    let mut adj: HashMap<uuid::Uuid, Vec<uuid::Uuid>> = HashMap::new();
    for node in &graph.nodes {
        adj.entry(node.id()).or_default();
    }
    for edge in &graph.edges {
        adj.entry(edge.from).or_default().push(edge.to);
    }

    let mut downstream_nodes = HashSet::new();
    let mut queue = VecDeque::new();

    downstream_nodes.insert(start_node_id);
    queue.push_back(start_node_id);

    while let Some(node) = queue.pop_front() {
        if let Some(neighbors) = adj.get(&node) {
            for &next in neighbors {
                if downstream_nodes.insert(next) {
                    queue.push_back(next);
                }
            }
        }
    }

    let mut reusable_edges = Vec::new();
    for edge in &graph.edges {
        if !downstream_nodes.contains(&edge.from) {
            reusable_edges.push(edge.id);
        }
    }

    Ok(reusable_edges)
}
