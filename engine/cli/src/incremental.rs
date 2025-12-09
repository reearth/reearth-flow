use std::collections::{HashMap, HashSet, VecDeque};

use reearth_flow_common::dir::setup_job_directory;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Workflow;

pub fn prepare_incremental_feature_store(
    workflow: &Workflow,
    job_id: uuid::Uuid,
    storage_resolver: &StorageResolver,
    previous_job_id: uuid::Uuid,
    start_node_id: uuid::Uuid,
) -> crate::Result<()> {
    tracing::info!(
        "Incremental run: previous_job_id={}, start_node_id={}",
        previous_job_id,
        start_node_id
    );

    let prev_feature_store_uri = setup_job_directory("engine", "feature-store", previous_job_id)
        .map_err(crate::errors::Error::init)?;
    tracing::info!(
        "Incremental run: previous feature-store root = {}",
        prev_feature_store_uri.path().display()
    );
    let prev_feature_store_state = State::new(&prev_feature_store_uri, storage_resolver)
        .map_err(crate::errors::Error::init)?;

    let reuse_feature_store_uri = setup_job_directory("engine", "previous-feature-store", job_id)
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
        match reuse_state.copy_jsonl_from_state(&prev_feature_store_state, &edge_id_str) {
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
) -> crate::Result<Vec<uuid::Uuid>> {
    let graph = workflow
        .graphs
        .iter()
        .find(|g| g.id == workflow.entry_graph_id)
        .ok_or_else(|| crate::errors::Error::init("Entry graph not found"))?;

    let mut in_degree: HashMap<uuid::Uuid, usize> = HashMap::new();
    let mut adj: HashMap<uuid::Uuid, Vec<uuid::Uuid>> = HashMap::new();

    for node in &graph.nodes {
        in_degree.entry(node.id()).or_insert(0);
        adj.entry(node.id()).or_default();
    }

    for edge in &graph.edges {
        adj.entry(edge.from).or_default().push(edge.to);
        *in_degree.entry(edge.to).or_insert(0) += 1;
        in_degree.entry(edge.from).or_insert(0);
    }

    let mut queue = VecDeque::new();
    for (node_id, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(*node_id);
        }
    }

    let mut topo: Vec<uuid::Uuid> = Vec::new();
    let mut in_degree_mut = in_degree.clone();

    while let Some(node) = queue.pop_front() {
        topo.push(node);
        if let Some(neighbors) = adj.get(&node) {
            for &next in neighbors {
                if let Some(d) = in_degree_mut.get_mut(&next) {
                    *d -= 1;
                    if *d == 0 {
                        queue.push_back(next);
                    }
                }
            }
        }
    }

    let pos = topo
        .iter()
        .position(|id| *id == start_node_id)
        .ok_or_else(|| crate::errors::Error::init("start_node_id not found in workflow graph"))?;

    let mut nodes_before_start = HashSet::new();
    for id in topo.iter().take(pos) {
        nodes_before_start.insert(*id);
    }

    let mut edge_ids = Vec::new();
    for edge in &graph.edges {
        if nodes_before_start.contains(&edge.from) {
            edge_ids.push(edge.id);
        }
    }

    Ok(edge_ids)
}
