use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;
use std::sync::Arc;

use reearth_flow_common::dir::setup_job_directory;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Workflow;

use crate::artifact::artifact_job_subdir_root_uri;
use crate::types::metadata::Metadata;

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

#[allow(clippy::too_many_arguments)]
pub async fn prepare_incremental_feature_store(
    storage_key: &str,
    workflow: &Workflow,
    job_id: uuid::Uuid,
    storage_resolver: &StorageResolver,
    metadata: &Metadata,
    previous_job_id: uuid::Uuid,
    start_node_id: uuid::Uuid,
    feature_state: &State,
) -> crate::errors::Result<Arc<State>> {
    tracing::info!(
        "Incremental run: previous_job_id={}, start_node_id={}",
        previous_job_id,
        start_node_id
    );

    let prev_feature_store_uri =
        artifact_job_subdir_root_uri(metadata, previous_job_id, "feature-store")?;
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

        match feature_state
            .copy_jsonl_from_state_async(&reuse_state, &edge_id_str)
            .await
        {
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

    Ok(Arc::new(reuse_state))
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

/// Copy reusable outputs from the previous job into current job workspace.
/// Then materialize them into <from_subdir> for runtime consumption.
pub async fn prepare_incremental_artifacts(
    storage_key: &str,
    storage_resolver: &StorageResolver,
    metadata: &Metadata,
    previous_job_id: uuid::Uuid,
    job_id: uuid::Uuid,
    specs: &[DirCopySpec],
) -> crate::errors::Result<()> {
    for spec in specs {
        copy_job_subdir_remote_to_local(
            storage_key,
            storage_resolver,
            metadata,
            previous_job_id,
            job_id,
            spec.from_subdir,
            spec.previous_subdir,
        )
        .await?;
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

/// Download a "job subdir tree" from remote previous job into current local previous-subdir.
async fn copy_job_subdir_remote_to_local(
    storage_key: &str,
    storage_resolver: &StorageResolver,
    metadata: &Metadata,
    prev_job_id: uuid::Uuid,
    job_id: uuid::Uuid,
    from_subdir: &str,
    to_subdir: &str,
) -> crate::errors::Result<()> {
    let remote_root = remote_job_subdir_root_uri(metadata, prev_job_id, from_subdir)?;
    let local_prev_root =
        setup_job_directory(storage_key, to_subdir, job_id).map_err(crate::errors::Error::init)?;

    // Ensure local directory exists.
    tokio::fs::create_dir_all(local_prev_root.path())
        .await
        .map_err(crate::errors::Error::init)?;

    download_remote_tree(
        storage_resolver,
        &remote_root,
        local_prev_root.path().as_path(),
        from_subdir,
    )
    .await
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

fn remote_job_subdir_root_uri(
    metadata: &Metadata,
    prev_job_id: uuid::Uuid,
    from_subdir: &str,
) -> crate::errors::Result<reearth_flow_common::uri::Uri> {
    match from_subdir {
        // Remote: <base>/<prev_job_id>/artifacts/
        "artifacts" => artifact_job_subdir_root_uri(metadata, prev_job_id, "artifacts"),
        // Remote: <base>/<prev_job_id>/temp-artifacts/
        "temp-artifacts" => artifact_job_subdir_root_uri(metadata, prev_job_id, "temp-artifacts"),
        _ => Err(crate::errors::Error::init(format!(
            "Unsupported incremental artifact subdir: {from_subdir}"
        ))),
    }
}

/// Download remote subtree rooted at `remote_root` into `local_dst_root`.
async fn download_remote_tree(
    storage_resolver: &StorageResolver,
    remote_root: &reearth_flow_common::uri::Uri,
    local_dst_root: &Path,
    label: &str,
) -> crate::errors::Result<()> {
    tracing::info!(
        "Incremental run: downloading previous {} from {}",
        label,
        remote_root
    );

    let root_storage = storage_resolver
        .resolve(remote_root)
        .map_err(crate::errors::Error::init)?;

    let items = root_storage
        .list_with_result(Some(remote_root.path().as_path()), true)
        .await
        .map_err(|e| {
            crate::errors::Error::init(format!(
                "Incremental run: failed to list previous {label} under {remote_root}: {e}"
            ))
        })?;

    // Filter out directory markers and check emptiness.
    let file_items = items
        .iter()
        .filter(|u| !u.path().to_string_lossy().ends_with('/'))
        .count();
    if file_items == 0 {
        tracing::info!(
            "Incremental run: previous {} is empty under {} (skipping copy).",
            label,
            remote_root
        );
    }

    tokio::fs::create_dir_all(local_dst_root)
        .await
        .map_err(crate::errors::Error::init)?;

    let remote_prefix = remote_root.path().to_string_lossy().to_string();

    for uri in items {
        let p = uri.path();
        let p_str = p.to_string_lossy();

        // Skip directory markers
        if p_str.ends_with('/') {
            continue;
        }

        // Rel path under remote_root
        let rel = match p_str.strip_prefix(remote_prefix.as_str()) {
            Some(s) => s.trim_start_matches('/').to_string(),
            None => {
                tracing::warn!(
                    "Incremental run: skip unexpected {label} path (not under prefix). uri={} prefix={}",
                    uri,
                    remote_prefix
                );
                continue;
            }
        };
        if rel.is_empty() {
            continue;
        }

        let local_path = local_dst_root.join(&rel);
        if let Some(parent) = local_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(crate::errors::Error::init)?;
        }

        let canonical_uri = remote_root.join(&rel).map_err(crate::errors::Error::init)?;

        tracing::info!(
            "Incremental run: downloading previous {label} {} -> {}",
            canonical_uri,
            local_path.display()
        );

        let s = storage_resolver
            .resolve(&canonical_uri)
            .map_err(crate::errors::Error::init)?;
        let res = s
            .get(canonical_uri.path().as_path())
            .await
            .map_err(crate::errors::Error::init)?;
        let bytes = res.bytes().await.map_err(crate::errors::Error::init)?;

        tokio::fs::write(&local_path, bytes)
            .await
            .map_err(crate::errors::Error::init)?;
    }

    Ok(())
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
