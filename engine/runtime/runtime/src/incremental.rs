use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use reearth_flow_state::State;

use crate::node::{FEATURE_FILTER_ACTION, OUTPUT_ROUTING_ACTION, ROUTING_PARAM_KEY};

#[derive(Clone, Debug)]
pub struct IncrementalRunConfig {
    pub start_node_id: uuid::Uuid,
    pub previous_feature_state: Arc<State>,
    pub available_edge_ids: HashSet<uuid::Uuid>,
}

#[derive(Debug, Clone)]
pub struct ReusableIds {
    pub edge_ids: Vec<uuid::Uuid>,
    pub port_file_ids: Vec<String>,
}

pub fn collect_reusable_ids(
    workflow: &reearth_flow_types::Workflow,
    start_node_id: uuid::Uuid,
) -> Result<ReusableIds, String> {
    let graphs: HashMap<uuid::Uuid, &reearth_flow_types::Graph> =
        workflow.graphs.iter().map(|g| (g.id, g)).collect();

    let mut node_to_graph: HashMap<uuid::Uuid, uuid::Uuid> = HashMap::new();
    for g in &workflow.graphs {
        for n in &g.nodes {
            node_to_graph.insert(n.id(), g.id);
        }
    }

    let start_graph_id = node_to_graph.get(&start_node_id).copied().ok_or_else(|| {
        format!("start_node_id {} not found in any graph", start_node_id)
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

    let prefix_chains = build_all_prefix_chains(workflow, &graphs);

    let mut edge_ids = HashSet::<uuid::Uuid>::new();
    let mut port_file_ids = HashSet::<String>::new();

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
            &mut port_file_ids,
            &prefix_chains,
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
    let mut port_vec: Vec<_> = port_file_ids.into_iter().collect();
    port_vec.sort();
    Ok(ReusableIds {
        edge_ids: v,
        port_file_ids: port_vec,
    })
}

/// Collects reusable edges and port file IDs in a graph, treating nodes upstream of
/// `start_node_id` as reusable. Also recursively processes upstream subworkflow nodes.
fn collect_reusable_in_graph_and_upstream_subworkflows(
    graphs: &HashMap<uuid::Uuid, &reearth_flow_types::Graph>,
    graph_id: uuid::Uuid,
    start_node_id: uuid::Uuid,
    edge_ids: &mut HashSet<uuid::Uuid>,
    port_file_ids: &mut HashSet<String>,
    prefix_chains: &HashMap<uuid::Uuid, Vec<Vec<uuid::Uuid>>>,
) -> Result<(), String> {
    let graph = graphs
        .get(&graph_id)
        .ok_or_else(|| format!("graph {} not found", graph_id))?;

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

    // For upstream nodes, compute their port file IDs and recurse into subworkflows
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

        if let Some(chains) = prefix_chains.get(&graph_id) {
            for chain in chains {
                port_file_ids.extend(compute_port_file_ids(graph, node, chain));
            }
        }

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
                port_file_ids,
                prefix_chains,
                &mut visited_subgraphs,
            )?;
        }
    }

    Ok(())
}

/// Recursively collects all edges and port file IDs in a graph and its nested subgraphs.
fn collect_all_in_graph_recursive(
    graphs: &HashMap<uuid::Uuid, &reearth_flow_types::Graph>,
    graph_id: uuid::Uuid,
    edge_ids: &mut HashSet<uuid::Uuid>,
    port_file_ids: &mut HashSet<String>,
    prefix_chains: &HashMap<uuid::Uuid, Vec<Vec<uuid::Uuid>>>,
    visited: &mut HashSet<uuid::Uuid>,
) -> Result<(), String> {
    if !visited.insert(graph_id) {
        tracing::info!(
            "Skipping already-visited subgraph {} (cycle detected)",
            graph_id
        );
        return Ok(());
    }

    let graph = graphs
        .get(&graph_id)
        .ok_or_else(|| format!("graph {} not found", graph_id))?;

    for edge in &graph.edges {
        edge_ids.insert(edge.id);
    }

    for node in &graph.nodes {
        if let Some(chains) = prefix_chains.get(&graph_id) {
            for chain in chains {
                port_file_ids.extend(compute_port_file_ids(graph, node, chain));
            }
        }
        if let Some(sub_graph_id) = extract_subgraph_id_if_subworkflow_node(node) {
            collect_all_in_graph_recursive(
                graphs,
                sub_graph_id,
                edge_ids,
                port_file_ids,
                prefix_chains,
                visited,
            )?;
        }
    }

    Ok(())
}

fn extract_subgraph_id_if_subworkflow_node(node: &reearth_flow_types::Node) -> Option<uuid::Uuid> {
    match node {
        reearth_flow_types::Node::SubGraph { sub_graph_id, .. } => Some(*sub_graph_id),
        _ => None,
    }
}

/// Builds a mapping from graph_id to all prefix chains reaching that graph.
/// Each prefix chain is the sequence of SubGraph entity IDs from the entry
/// graph down to the target graph. The entry graph itself has an empty chain.
fn build_all_prefix_chains(
    workflow: &reearth_flow_types::Workflow,
    graphs: &HashMap<uuid::Uuid, &reearth_flow_types::Graph>,
) -> HashMap<uuid::Uuid, Vec<Vec<uuid::Uuid>>> {
    let mut result: HashMap<uuid::Uuid, Vec<Vec<uuid::Uuid>>> = HashMap::new();
    result
        .entry(workflow.entry_graph_id)
        .or_default()
        .push(vec![]);

    // Track (graph_id, caller_entity_id) to avoid infinite loops on cyclic subgraph references.
    let mut visited: HashSet<(uuid::Uuid, Option<uuid::Uuid>)> = HashSet::new();
    visited.insert((workflow.entry_graph_id, None));

    let mut queue: VecDeque<(uuid::Uuid, Vec<uuid::Uuid>)> = VecDeque::new();
    queue.push_back((workflow.entry_graph_id, vec![]));

    while let Some((graph_id, current_chain)) = queue.pop_front() {
        if let Some(graph) = graphs.get(&graph_id) {
            for node in &graph.nodes {
                if let reearth_flow_types::Node::SubGraph {
                    entity,
                    sub_graph_id,
                } = node
                {
                    if !visited.insert((*sub_graph_id, Some(entity.id))) {
                        continue;
                    }
                    let mut child_chain = current_chain.clone();
                    child_chain.push(entity.id);
                    result
                        .entry(*sub_graph_id)
                        .or_default()
                        .push(child_chain.clone());
                    queue.push_back((*sub_graph_id, child_chain));
                }
            }
        }
    }

    result
}

/// Computes port-based file ID strings for a node, matching the naming convention
/// in execution_dag.rs:
///   - (Some(prefix), is_subgraph_output=true)  => "{prefix}.{port}"
///   - (Some(prefix), is_subgraph_output=false)  => "{prefix}.{node_id}.{port}"
///   - (None, _)                                  => "{node_id}.{port}"
fn compute_port_file_ids(
    graph: &reearth_flow_types::Graph,
    node: &reearth_flow_types::Node,
    prefix_chain: &[uuid::Uuid],
) -> Vec<String> {
    if matches!(node, reearth_flow_types::Node::SubGraph { .. }) {
        return vec![];
    }

    let node_id = node.id();
    let ports = collect_output_ports(graph, node);
    if ports.is_empty() {
        return vec![];
    }

    let prefix_str = if prefix_chain.is_empty() {
        None
    } else {
        Some(
            prefix_chain
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join("."),
        )
    };

    let is_subgraph_output = prefix_str.is_some() && is_output_router(node);

    ports
        .into_iter()
        .map(|port| match (&prefix_str, is_subgraph_output) {
            (Some(pfx), true) => format!("{}.{}", pfx, port),
            (Some(pfx), false) => format!("{}.{}.{}", pfx, node_id, port),
            (None, _) => format!("{}.{}", node_id, port),
        })
        .collect()
}

/// Determines the output port names for a node by inspecting outgoing edges.
/// For OutputRouter and FeatureFilter nodes, also includes ports derived from
/// their configuration parameters (routingPort / conditions[].outputPort).
fn collect_output_ports(
    graph: &reearth_flow_types::Graph,
    node: &reearth_flow_types::Node,
) -> Vec<String> {
    let node_id = node.id();
    let mut ports: HashSet<String> = graph
        .edges
        .iter()
        .filter(|e| e.from == node_id)
        .map(|e| e.from_port.clone())
        .collect();

    if let reearth_flow_types::Node::Action { entity, action } = node {
        if let Some(with) = &entity.with {
            if action == OUTPUT_ROUTING_ACTION {
                if let Some(serde_json::Value::String(rp)) = with.get(ROUTING_PARAM_KEY) {
                    ports.insert(rp.clone());
                }
            } else if action == FEATURE_FILTER_ACTION {
                if let Some(serde_json::Value::Array(conditions)) = with.get("conditions") {
                    for condition in conditions {
                        if let Some(serde_json::Value::String(port)) = condition.get("outputPort") {
                            ports.insert(port.clone());
                        }
                    }
                }
            }
        }
    }

    let mut v: Vec<_> = ports.into_iter().collect();
    v.sort();
    v
}

fn is_output_router(node: &reearth_flow_types::Node) -> bool {
    matches!(node, reearth_flow_types::Node::Action { action, .. } if action == OUTPUT_ROUTING_ACTION)
}
