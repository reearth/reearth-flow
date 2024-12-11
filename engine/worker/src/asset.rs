use std::{path::Path, str::FromStr, sync::Arc};

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;

use crate::types::metadata::Asset;

pub(crate) async fn download_asset(
    storage_resolver: &Arc<StorageResolver>,
    asset: &Asset,
    download_path: &Uri,
) -> crate::errors::Result<()> {
    if asset.is_empty() {
        return Ok(());
    }
    let uris = asset
        .files
        .iter()
        .map(|f| {
            let path = Path::new(&asset.base_url.trim_end_matches("/")).join(f);
            let path =
                path.to_str()
                    .ok_or(crate::errors::Error::failed_to_download_asset_files(
                        "Failed to convert path",
                    ))?;
            let uri = Uri::from_str(path)
                .map_err(crate::errors::Error::failed_to_download_asset_files)?;
            Ok((f.to_string(), uri))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let futures = uris
        .iter()
        .map(|(name, uri)| async move {
            let storage = storage_resolver
                .resolve(uri)
                .map_err(crate::errors::Error::failed_to_download_asset_files)?;
            let bytes = storage
                .get(uri.path().as_path())
                .await
                .map_err(crate::errors::Error::failed_to_download_asset_files)?;
            let bytes = bytes
                .bytes()
                .await
                .map_err(crate::errors::Error::failed_to_download_asset_files)?;
            let location = download_path
                .join(Path::new(name))
                .map_err(crate::errors::Error::failed_to_download_asset_files)?;
            let root_storage = storage_resolver
                .resolve(&location)
                .map_err(crate::errors::Error::failed_to_download_asset_files)?;
            root_storage
                .put(location.path().as_path(), Bytes::from(bytes.to_vec()))
                .await
                .map_err(crate::errors::Error::failed_to_download_asset_files)?;
            Ok(())
        })
        .collect::<Vec<_>>();
    futures::future::try_join_all(futures).await?;
    Ok(())
}
