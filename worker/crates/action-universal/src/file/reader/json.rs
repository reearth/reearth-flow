use std::sync::Arc;

use reearth_flow_action::{error::Error, ActionValue, Result};
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;

pub(crate) async fn read_json(
    input_path: Uri,
    storage_resolver: Arc<StorageResolver>,
) -> Result<ActionValue> {
    let storage = storage_resolver
        .resolve(&input_path)
        .map_err(Error::input)?;
    let result = storage
        .get(input_path.path().as_path())
        .await
        .map_err(Error::internal_runtime)?;
    let byte = result.bytes().await.map_err(Error::internal_runtime)?;
    let text = String::from_utf8(byte.to_vec()).map_err(Error::internal_runtime)?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(Error::internal_runtime)?;
    Ok(value.into())
}
