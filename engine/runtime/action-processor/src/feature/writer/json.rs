use std::sync::Arc;

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{create_batch_feature, Code, CompiledCode, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::FeatureProcessorError;

/// # JsonWriter Parameters
///
/// Configuration for writing features in JSON format with optional custom conversion.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct JsonWriterParam {
    pub(super) converter: Option<Code>,
}

#[derive(Debug, Clone)]
pub(super) struct CompiledJsonWriterParam {
    pub(super) converter: Option<CompiledCode>,
}

pub(super) fn write_json(
    output: &Uri,
    converter: &Option<CompiledCode>,
    storage_resolver: &Arc<StorageResolver>,
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    features: &[Feature],
) -> Result<(), FeatureProcessorError> {
    let json_value: serde_json::Value = if let Some(converter) = converter {
        let synthetic = create_batch_feature(features);
        converter
            .eval(&synthetic, env_vars)
            .map_err(|e| {
                FeatureProcessorError::FeatureWriter(format!("Failed to evaluate converter: {e:?}"))
            })?
            .into()
    } else {
        let attributes = features
            .iter()
            .map(|f| {
                serde_json::Value::Object(
                    f.attributes
                        .iter()
                        .map(|(k, v)| (k.clone().into_inner().to_string(), v.clone().into()))
                        .collect::<serde_json::Map<_, _>>(),
                )
            })
            .collect::<Vec<serde_json::Value>>();
        serde_json::Value::Array(attributes)
    };
    let storage = storage_resolver
        .resolve(output)
        .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))?;
    storage
        .put_sync(output.path().as_path(), Bytes::from(json_value.to_string()))
        .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))?;
    Ok(())
}
