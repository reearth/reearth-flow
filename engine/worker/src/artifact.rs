use std::{path::MAIN_SEPARATOR, str::FromStr, sync::Arc};

use bytes::Bytes;
use reearth_flow_common::{dir, uri::Uri};
use reearth_flow_storage::resolve::StorageResolver;
use tokio::sync::Semaphore;
use tracing::info;
use walkdir::WalkDir;

use crate::types::metadata::Metadata;

pub(crate) async fn upload_artifact(
    storage_resolver: &Arc<StorageResolver>,
    metadata: &Metadata,
) -> crate::errors::Result<()> {
    let local_artifact_root_path =
        dir::get_job_root_dir_path("workers", metadata.job_id).map_err(|e| {
            crate::errors::Error::failed_to_upload_artifact(format!(
                "Failed to get job root dir: {e}"
            ))
        })?;
    let remote_artifact_root_path = Uri::from_str(metadata.artifact_base_url.as_str())
        .map_err(crate::errors::Error::failed_to_upload_artifact)?;
    let uris = WalkDir::new(local_artifact_root_path.clone())
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path().is_file()
                && !e
                    .path()
                    .starts_with(local_artifact_root_path.join("assets"))
        })
        .filter_map(|entry| entry.path().as_os_str().to_str().map(String::from))
        .map(|entry| Uri::from_str(entry.as_str()))
        .flat_map(Result::ok)
        .collect::<Vec<_>>();
    let local_artifact_root_path = Uri::from_str(
        local_artifact_root_path
            .to_string_lossy()
            .to_string()
            .as_str(),
    )
    .map_err(crate::errors::Error::failed_to_upload_artifact)?;

    let semaphore = Arc::new(Semaphore::new(5));

    let futures = uris
        .iter()
        .map(|uri| {
            let local_artifact_root_path = local_artifact_root_path.clone();
            let remote_artifact_root_path = remote_artifact_root_path.clone();
            let permit = semaphore.clone().acquire_owned();
            async move {
                let storage = storage_resolver
                    .resolve(uri)
                    .map_err(crate::errors::Error::failed_to_upload_artifact)?;
                let bytes = storage
                    .get(uri.path().as_path())
                    .await
                    .map_err(crate::errors::Error::failed_to_upload_artifact)?;
                let bytes = bytes
                    .bytes()
                    .await
                    .map_err(crate::errors::Error::failed_to_upload_artifact)?;

                let s = uri.to_string();
                let s = s.replace(&local_artifact_root_path.to_string(), "");
                let s = s.trim_start_matches(MAIN_SEPARATOR).to_string();
                let location = remote_artifact_root_path
                    .join(metadata.job_id.to_string())
                    .map_err(crate::errors::Error::failed_to_upload_artifact)?;
                let location = location
                    .join(s)
                    .map_err(crate::errors::Error::failed_to_upload_artifact)?;
                let root_storage = storage_resolver
                    .resolve(&location)
                    .map_err(crate::errors::Error::failed_to_upload_artifact)?;

                let _permit_guard = permit
                    .await
                    .map_err(crate::errors::Error::failed_to_upload_artifact)?;

                info!("Uploading artifact from {:?} to {:?}", uri, location);
                root_storage
                    .put(location.path().as_path(), Bytes::from(bytes.to_vec()))
                    .await
                    .map_err(crate::errors::Error::failed_to_upload_artifact)?;
                Ok(())
            }
        })
        .collect::<Vec<_>>();
    futures::future::try_join_all(futures).await?;
    Ok(())
}
