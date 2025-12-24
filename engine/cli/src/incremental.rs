use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;

use reearth_flow_common::dir::{get_job_root_dir_path, setup_job_directory};
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Workflow;

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
) -> crate::Result<()> {
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
