use std::sync::Arc;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;

use crate::action::ActionValue;

pub(crate) async fn read_text(
    input_path: Uri,
    storage_resolver: Arc<StorageResolver>,
) -> anyhow::Result<ActionValue> {
    let storage = storage_resolver.resolve(&input_path)?;
    let result = storage.get(input_path.path().as_path()).await?;
    let byte = result.bytes().await?;
    let text = String::from_utf8(byte.to_vec())?;
    Ok(ActionValue::String(text))
}
