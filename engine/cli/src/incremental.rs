use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;
use std::sync::Arc;

use reearth_flow_common::dir::{get_job_root_dir_path, setup_job_directory};
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Workflow;

#[derive(Debug, Clone)]
pub struct ReusableIds {
    pub edge_ids: Vec<uuid::Uuid>,
    pub node_ids: HashSet<uuid::Uuid>,
}

#[derive(Debug, Clone)]
pub struct DirCopySpec {
    pub from_subdir: &'static str,
    pub previous_subdir: &'static str,
}

impl DirCopySpec {
    pub const fn new(from_subdir: &'static str, previous_subdir: &'static str) -> Self {
        Self {
            from_subdir,
            previous_subdir,
        }
    }

    pub const fn materialize_target(&self) -> &'static str {
        self.from_subdir
    }
}

pub fn prepare_incremental_feature_store(
    storage_key: &str,
    workflow: &Workflow,
    job_id: uuid::Uuid,
    storage_resolver: &StorageResolver,
    previous_job_id: uuid::Uuid,
    start_node_id: uuid::Uuid,
    feature_state: &State,
) -> crate::Result<(Arc<State>, HashSet<uuid::Uuid>)> {
    tracing::info!(
        "Incremental run: previous_job_id={}, start_node_id={}",
        previous_job_id,
        start_node_id
    );

    let prev_feature_store_uri = setup_job_directory(storage_key, "feature-store", previous_job_id)
        .map_err(crate::errors::Error::init)?;
    tracing::info!(
        "Incremental run: previous feature-store root = {}",
        prev_feature_store_uri.path().display()
    );
    let prev_feature_store_state = State::new(&prev_feature_store_uri, storage_resolver)
        .map_err(crate::errors::Error::init)?;

    let reuse_feature_store_uri =
        setup_job_directory(storage_key, "previous-feature-store", job_id)
            .map_err(crate::errors::Error::init)?;
    tracing::info!(
        "Incremental run: reuse feature-store root = {}",
        reuse_feature_store_uri.path().display()
    );
    let reuse_state = State::new(&reuse_feature_store_uri, storage_resolver)
        .map_err(crate::errors::Error::init)?;

    let reusable_ids = collect_reusable_ids(workflow, start_node_id)?;
    let candidate_edge_ids = &reusable_ids.edge_ids;
    tracing::info!(
        "Incremental run: candidate reusable edge IDs for node {}: {:?}",
        start_node_id,
        candidate_edge_ids
    );

    // Filter candidate edges by checking which ones actually exist in the previous feature store
    let mut actually_copied_edge_ids = Vec::new();

    for edge_id in candidate_edge_ids {
        let edge_id_str = edge_id.to_string();
        match reuse_state.copy_jsonl_from_state(&prev_feature_store_state, &edge_id_str) {
            Ok(()) => {
                tracing::info!(
                    "Incremental run: copied edge {} into {}",
                    edge_id_str,
                    reuse_feature_store_uri.path().display()
                );
                actually_copied_edge_ids.push(*edge_id);
            }
            Err(e) => {
                tracing::info!(
                    "Incremental run: edge {} does not exist in previous feature-store, skipping: {:?}",
                    edge_id_str,
                    e
                );
                continue;
            }
        }

        match feature_state.copy_jsonl_from_state(&reuse_state, &edge_id_str) {
            Ok(()) => {
                tracing::info!("Copied edge {} into feature-store", edge_id_str);
            }
            Err(e) => {
                return Err(crate::errors::Error::init(format!(
                    "Failed to copy edge {} into feature-store: {:?}",
                    edge_id_str, e
                )));
            }
        }
    }

    let actually_copied_edges: HashSet<uuid::Uuid> = actually_copied_edge_ids.into_iter().collect();

    tracing::info!(
        "Incremental run: successfully copied {} out of {} candidate edges",
        actually_copied_edges.len(),
        candidate_edge_ids.len()
    );
    tracing::info!(
        "Incremental run: actually copied edge IDs: {:?}",
        actually_copied_edges
    );

    // --- Port-based file copying ---
    let all_prev_ids = prev_feature_store_state.list_jsonl_ids_sync();
    let port_file_ids: Vec<&str> = all_prev_ids
        .iter()
        .filter(|stem| is_port_file_for_reusable_node(stem, &reusable_ids.node_ids))
        .map(|s| s.as_str())
        .collect();

    tracing::info!(
        "Incremental run: found {} port-based files to copy",
        port_file_ids.len()
    );

    for file_id in &port_file_ids {
        match reuse_state.copy_jsonl_from_state(&prev_feature_store_state, file_id) {
            Ok(()) => {
                tracing::info!(
                    "Incremental run: copied port file {} into {}",
                    file_id,
                    reuse_feature_store_uri.path().display()
                );
            }
            Err(e) => {
                tracing::info!(
                    "Incremental run: port file {} not found in previous feature-store, skipping: {:?}",
                    file_id,
                    e
                );
                continue;
            }
        }

        match feature_state.copy_jsonl_from_state(&reuse_state, file_id) {
            Ok(()) => {
                tracing::info!("Copied port file {} into feature-store", file_id);
            }
            Err(e) => {
                return Err(crate::errors::Error::init(format!(
                    "Failed to copy port file {} into feature-store: {:?}",
                    file_id, e
                )));
            }
        }
    }

    Ok((Arc::new(reuse_state), actually_copied_edges))
}

pub fn collect_reusable_ids(
    workflow: &Workflow,
    start_node_id: uuid::Uuid,
) -> crate::Result<ReusableIds> {
    let graphs: HashMap<uuid::Uuid, &reearth_flow_types::Graph> =
        workflow.graphs.iter().map(|g| (g.id, g)).collect();

    let mut node_to_graph: HashMap<uuid::Uuid, uuid::Uuid> = HashMap::new();
    for g in &workflow.graphs {
        for n in &g.nodes {
            node_to_graph.insert(n.id(), g.id);
        }
    }

    let start_graph_id = node_to_graph.get(&start_node_id).copied().ok_or_else(|| {
        crate::errors::Error::init(format!(
            "start_node_id {} not found in any graph",
            start_node_id
        ))
    })?;

    // Build subgraph callsite map: sub_graph_id -> [(parent_graph_id, caller_node_id)]
    let mut callsites: HashMap<uuid::Uuid, Vec<(uuid::Uuid, uuid::Uuid)>> = HashMap::new();
    for g in &workflow.graphs {
        for n in &g.nodes {
            if let reearth_flow_types::Node::SubGraph {
                entity,
                sub_graph_id,
                ..
            } = n
            {
                callsites
                    .entry(*sub_graph_id)
                    .or_default()
                    .push((g.id, entity.id));
            }
        }
    }

    let mut edge_ids = HashSet::<uuid::Uuid>::new();
    let mut node_ids = HashSet::<uuid::Uuid>::new();

    // BFS traversal from start node up to parent graphs
    let mut q: VecDeque<(uuid::Uuid, uuid::Uuid)> = VecDeque::new();
    let mut visited: HashSet<(uuid::Uuid, uuid::Uuid)> = HashSet::new();

    q.push_back((start_graph_id, start_node_id));
    visited.insert((start_graph_id, start_node_id));

    while let Some((gid, sid)) = q.pop_front() {
        collect_reusable_in_graph_and_upstream_subworkflows(
            &graphs,
            gid,
            sid,
            &mut edge_ids,
            &mut node_ids,
        )?;

        // If current graph is a subworkflow, traverse up to parent graphs
        if let Some(parents) = callsites.get(&gid) {
            for &(pgid, caller_node_id) in parents {
                if visited.insert((pgid, caller_node_id)) {
                    q.push_back((pgid, caller_node_id));
                }
            }
        }
    }

    let mut v: Vec<_> = edge_ids.into_iter().collect();
    v.sort();
    Ok(ReusableIds {
        edge_ids: v,
        node_ids,
    })
}

/// Collects reusable edges and node IDs in a graph, treating nodes upstream of
/// `start_node_id` as reusable. Also recursively processes upstream subworkflow nodes.
fn collect_reusable_in_graph_and_upstream_subworkflows(
    graphs: &HashMap<uuid::Uuid, &reearth_flow_types::Graph>,
    graph_id: uuid::Uuid,
    start_node_id: uuid::Uuid,
    edge_ids: &mut HashSet<uuid::Uuid>,
    node_ids: &mut HashSet<uuid::Uuid>,
) -> crate::Result<()> {
    let graph = graphs
        .get(&graph_id)
        .ok_or_else(|| crate::errors::Error::init(format!("graph {} not found", graph_id)))?;

    // Build adjacency list for BFS
    let mut adj: HashMap<uuid::Uuid, Vec<uuid::Uuid>> = HashMap::new();
    for node in &graph.nodes {
        adj.entry(node.id()).or_default();
    }
    for edge in &graph.edges {
        adj.entry(edge.from).or_default().push(edge.to);
    }

    // Find all downstream nodes from start_node via BFS
    let mut downstream = HashSet::new();
    let mut q = VecDeque::new();
    downstream.insert(start_node_id);
    q.push_back(start_node_id);

    while let Some(n) = q.pop_front() {
        if let Some(nexts) = adj.get(&n) {
            for &nx in nexts {
                if downstream.insert(nx) {
                    q.push_back(nx);
                }
            }
        }
    }

    // Collect edges whose source is NOT downstream (i.e., upstream edges)
    for edge in &graph.edges {
        if !downstream.contains(&edge.from) {
            edge_ids.insert(edge.id);
        }
    }

    // Track visited subgraphs to prevent infinite recursion in case of cycles
    let mut visited_subgraphs = HashSet::new();

    // For upstream nodes, collect their IDs and recurse into subworkflows
    for node in &graph.nodes {
        let nid = node.id();
        if downstream.contains(&nid) {
            tracing::info!(
                "Skipping node {} in graph {} as it is downstream of start node {}",
                nid,
                graph_id,
                start_node_id
            );
            continue;
        }

        node_ids.insert(nid);

        tracing::info!(
            "Processing upstream node {} in graph {} for reusable data",
            nid,
            graph_id
        );

        if let Some(sub_graph_id) = extract_subgraph_id_if_subworkflow_node(node) {
            tracing::info!(
                "Node {} in graph {} is a subworkflow node calling subgraph {}",
                nid,
                graph_id,
                sub_graph_id
            );
            collect_all_in_graph_recursive(
                graphs,
                sub_graph_id,
                edge_ids,
                node_ids,
                &mut visited_subgraphs,
            )?;
        }
    }

    Ok(())
}

/// Recursively collects all edges and node IDs in a graph and its nested subgraphs.
/// Uses cycle detection to prevent infinite recursion if subgraphs form circular references.
fn collect_all_in_graph_recursive(
    graphs: &HashMap<uuid::Uuid, &reearth_flow_types::Graph>,
    graph_id: uuid::Uuid,
    edge_ids: &mut HashSet<uuid::Uuid>,
    node_ids: &mut HashSet<uuid::Uuid>,
    visited: &mut HashSet<uuid::Uuid>,
) -> crate::Result<()> {
    if !visited.insert(graph_id) {
        tracing::info!(
            "Skipping already-visited subgraph {} (cycle detected)",
            graph_id
        );
        return Ok(());
    }

    let graph = graphs
        .get(&graph_id)
        .ok_or_else(|| crate::errors::Error::init(format!("graph {} not found", graph_id)))?;

    for edge in &graph.edges {
        edge_ids.insert(edge.id);
    }

    for node in &graph.nodes {
        node_ids.insert(node.id());
        if let Some(sub_graph_id) = extract_subgraph_id_if_subworkflow_node(node) {
            collect_all_in_graph_recursive(graphs, sub_graph_id, edge_ids, node_ids, visited)?;
        }
    }

    Ok(())
}

/// Extracts the subgraph ID from a node if it's a SubGraph node type.
fn extract_subgraph_id_if_subworkflow_node(node: &reearth_flow_types::Node) -> Option<uuid::Uuid> {
    match node {
        reearth_flow_types::Node::SubGraph { sub_graph_id, .. } => Some(*sub_graph_id),
        _ => None,
    }
}

/// Determines if a JSONL file stem represents a port-based file belonging
/// to one of the given reusable node IDs.
fn is_port_file_for_reusable_node(stem: &str, reusable_node_ids: &HashSet<uuid::Uuid>) -> bool {
    let segments: Vec<&str> = stem.split('.').collect();
    if segments.len() < 2 {
        return false;
    }

    // If the last segment parses as a UUID, this is an edge-based file
    let last = segments.last().unwrap();
    if uuid::Uuid::parse_str(last).is_ok() {
        return false;
    }

    // Last segment is the port name. Check if any preceding UUID segment
    // is a reusable node ID.
    for seg in &segments[..segments.len() - 1] {
        if let Ok(id) = uuid::Uuid::parse_str(seg) {
            if reusable_node_ids.contains(&id) {
                return true;
            }
        }
    }

    false
}

/// Copy reusable outputs from the previous job into current job workspace.
/// Then materialize them into <from_subdir> for runtime consumption.
pub fn prepare_incremental_artifacts(
    storage_key: &str,
    previous_job_id: uuid::Uuid,
    job_id: uuid::Uuid,
    specs: &[DirCopySpec],
) -> crate::Result<()> {
    for spec in specs {
        copy_job_subdir(
            storage_key,
            previous_job_id,
            job_id,
            spec.from_subdir,
            spec.previous_subdir,
        )
        .map_err(crate::errors::Error::init)?;
        materialize_job_subdir(
            storage_key,
            job_id,
            spec.previous_subdir,
            spec.materialize_target(),
        )
        .map_err(crate::errors::Error::init)?;
    }
    Ok(())
}

fn copy_job_subdir(
    storage_key: &str,
    prev_job_id: uuid::Uuid,
    job_id: uuid::Uuid,
    from_subdir: &str,
    to_subdir: &str,
) -> std::io::Result<()> {
    let prev = get_job_root_dir_path(storage_key, prev_job_id)
        .map_err(|e| io_err(format!("get_job_root_dir_path prev: {e}")))?
        .join(from_subdir);

    let cur_prev = setup_job_directory(storage_key, to_subdir, job_id)
        .map_err(|e| io_err(format!("setup_job_directory cur {to_subdir}: {e}")))?;

    copy_dir_all_overwrite(prev.as_path(), cur_prev.path().as_path())
}

/// Materialize local previous-subdir into the current runtime subdir.
fn materialize_job_subdir(
    storage_key: &str,
    job_id: uuid::Uuid,
    from_subdir: &str,
    to_subdir: &str,
) -> std::io::Result<()> {
    let cur_prev = setup_job_directory(storage_key, from_subdir, job_id)
        .map_err(|e| io_err(format!("setup_job_directory from {from_subdir}: {e}")))?;
    let cur = setup_job_directory(storage_key, to_subdir, job_id)
        .map_err(|e| io_err(format!("setup_job_directory to {to_subdir}: {e}")))?;

    copy_dir_all_overwrite(cur_prev.path().as_path(), cur.path().as_path())
}

fn copy_dir_all_overwrite(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.exists() {
        return Ok(());
    }
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all_overwrite(&from, &to)?;
        } else if ty.is_file() {
            fs::copy(&from, &to)?;
        } else if ty.is_symlink() {
            tracing::warn!("Skipping symlink during copy: {}", from.display());
        } else {
            tracing::warn!("Skipping non-file entry during copy: {}", from.display());
        }
    }
    Ok(())
}

fn io_err(msg: String) -> std::io::Error {
    std::io::Error::other(msg)
}
