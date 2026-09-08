use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use reearth_flow_common::dir::{get_job_root_dir_path, setup_job_directory};
use reearth_flow_runtime::incremental::collect_reusable_ids;
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

/// Copies the reusable port files of `previous_job_id` into `feature_state` for an
/// incremental run starting at `start_node_id`. Returns the previous run's feature
/// store and the ids of the port files that were copied.
pub fn prepare_incremental_feature_store(
    storage_key: &str,
    workflow: &Workflow,
    job_id: uuid::Uuid,
    storage_resolver: &StorageResolver,
    previous_job_id: uuid::Uuid,
    start_node_id: uuid::Uuid,
    feature_state: &State,
) -> crate::Result<(Arc<State>, HashSet<String>)> {
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

    let reusable_ids =
        collect_reusable_ids(workflow, start_node_id).map_err(crate::errors::Error::init)?;
    // --- Port-based file copying ---
    let port_file_ids = &reusable_ids.port_file_ids;
    let mut copied_port_file_ids: HashSet<String> = HashSet::new();

    tracing::info!(
        "Incremental run: {} port-based file IDs to copy",
        port_file_ids.len()
    );

    for file_id in port_file_ids {
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
                copied_port_file_ids.insert(file_id.clone());
            }
            Err(e) => {
                return Err(crate::errors::Error::init(format!(
                    "Failed to copy port file {} into feature-store: {:?}",
                    file_id, e
                )));
            }
        }
    }

    tracing::info!(
        "Incremental run: {} of {} reusable port files copied",
        copied_port_file_ids.len(),
        port_file_ids.len()
    );
    Ok((Arc::new(reuse_state), copied_port_file_ids))
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
