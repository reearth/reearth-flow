use std::str::FromStr;
use std::sync::Arc;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;

use super::runner::CommonPropertySchema;
use crate::action::ActionValue;

pub(crate) async fn read_text(
    common_props: &CommonPropertySchema,
    storage_resolver: Arc<StorageResolver>,
) -> anyhow::Result<ActionValue> {
    let uri = Uri::from_str(&common_props.dataset)?;
    let storage = storage_resolver.resolve(&uri)?;
    let result = storage.get(uri.path().as_path()).await?;
    let byte = result.bytes().await?;
    let text = String::from_utf8(byte.to_vec())?;
    Ok(ActionValue::String(text))
}
