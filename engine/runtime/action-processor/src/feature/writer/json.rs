use std::sync::Arc;

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Feature;

use super::FeatureProcessorError;

pub(super) fn write_json(
    output: &Uri,
    storage_resolver: &Arc<StorageResolver>,
    features: &[Feature],
) -> Result<(), FeatureProcessorError> {
    let json_value: serde_json::Value = {
        let attributes = features
            .iter()
            .map(|f| {
                serde_json::Value::Object(
                    f.attributes
                        .clone()
                        .into_iter()
                        .map(|(k, v)| (k.into_inner().to_string(), v.into()))
                        .collect::<serde_json::Map<_, _>>(),
                )
            })
            .collect::<Vec<serde_json::Value>>();
        serde_json::Value::Array(attributes)
    };
    let storage = storage_resolver
        .resolve(output)
        .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{:?}", e)))?;
    storage
        .put_sync(output.path().as_path(), Bytes::from(json_value.to_string()))
        .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{:?}", e)))?;
    Ok(())
}
