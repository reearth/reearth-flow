use std::sync::Arc;

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_eval_expr::{engine::Engine, utils::dynamic_to_value};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{Expr, Feature};
use rhai::Dynamic;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::FeatureProcessorError;

/// # JsonWriter Parameters
///
/// Configuration for writing features in JSON format with optional custom conversion.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct JsonWriterParam {
    pub(super) converter: Option<Expr>,
}

#[derive(Debug, Clone)]
pub(super) struct CompiledJsonWriterParam {
    pub(super) converter: Option<rhai::AST>,
}

pub(super) fn write_json(
    output: &Uri,
    converter: &Option<rhai::AST>,
    storage_resolver: &Arc<StorageResolver>,
    expr_engine: &Arc<Engine>,
    features: &[Feature],
) -> Result<(), FeatureProcessorError> {
    let json_value: serde_json::Value = if let Some(converter) = converter {
        let scope = expr_engine.new_scope();
        let value: serde_json::Value = serde_json::Value::Array(
            features
                .iter()
                .map(|feature| {
                    serde_json::Value::Object(
                        feature
                            .attributes
                            .clone()
                            .into_iter()
                            .map(|(k, v)| (k.into_inner().to_string(), v.into()))
                            .collect::<serde_json::Map<_, _>>(),
                    )
                })
                .collect::<Vec<_>>(),
        );
        scope.set("__features", value);
        let convert = scope.eval_ast::<Dynamic>(converter).map_err(|e| {
            FeatureProcessorError::FeatureWriter(format!("Failed to evaluate converter: {e:?}"))
        })?;
        dynamic_to_value(&convert)
    } else {
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
        .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))?;
    storage
        .put_sync(output.path().as_path(), Bytes::from(json_value.to_string()))
        .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))?;
    Ok(())
}
