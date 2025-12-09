use std::collections::{HashMap, HashSet, VecDeque};

use reearth_flow_common::dir::setup_job_directory;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Workflow;

pub fn copy_upstream_intermediate_data(
    workflow: &Workflow,
    job_id: uuid::Uuid,
    storage_resolver: &StorageResolver,
    previous_job_id: uuid::Uuid,
    start_node_id: uuid::Uuid,
) -> crate::Result<()> {
    tracing::info!(
        "Incremental snapshot: previous_job_id={}, start_node_id={}",
        previous_job_id,
        start_node_id
    );

    let prev_intermediate_data_uri =
        setup_job_directory("engine", "feature-store", previous_job_id)
            .map_err(crate::errors::Error::init)?;
    tracing::info!(
        "Previous intermediate data root = {}",
        prev_intermediate_data_uri.path().display()
    );
    let prev_intermediate_data_state = State::new(&prev_intermediate_data_uri, storage_resolver)
        .map_err(crate::errors::Error::init)?;

    let reuse_intermediate_data_uri =
        setup_job_directory("engine", "previous-feature-store", job_id)
            .map_err(crate::errors::Error::init)?;
    tracing::info!(
        "Reuse intermediate data root = {}",
        reuse_intermediate_data_uri.path().display()
    );
    let reuse_state = State::new(&reuse_intermediate_data_uri, storage_resolver)
        .map_err(crate::errors::Error::init)?;

    let edge_ids = collect_edge_ids_until_node(workflow, start_node_id)?;
    tracing::info!(
        "Incremental snapshot: upstream edge IDs for node {}: {:?}",
        start_node_id,
        edge_ids
    );
    tracing::info!(
        "Incremental snapshot: copying {} edge(s) into previous-feature-store",
        edge_ids.len()
    );

    for edge_id in edge_ids {
        let edge_id_str = edge_id.to_string();
        match reuse_state.copy_jsonl_from_state(&prev_intermediate_data_state, &edge_id_str) {
            Ok(()) => {
                tracing::info!(
                    "Snapshot copied for edge {} into {}",
                    edge_id_str,
                    reuse_intermediate_data_uri.path().display()
                );
            }
            Err(e) => {
                tracing::warn!("Snapshot copy failed for edge {}: {:?}", edge_id_str, e);
            }
        }
    }

    Ok(())
}

pub fn collect_edge_ids_until_node(
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

    let mut nodes_until_start = HashSet::new();
    for id in topo.iter().take(pos + 1) {
        nodes_until_start.insert(*id);
    }

    let mut edge_ids = Vec::new();
    for edge in &graph.edges {
        if nodes_until_start.contains(&edge.to) {
            edge_ids.push(edge.id);
        }
    }

    tracing::info!(
        "Upstream edge IDs for node {}: {:?}",
        start_node_id,
        edge_ids
    );

    Ok(edge_ids)
}
