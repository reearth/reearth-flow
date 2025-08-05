use std::{collections::HashMap, str::FromStr, sync::Arc};

use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    executor_operation::ExecutorContext, forwarder::ProcessorChannelForwarder, node::DEFAULT_PORT,
};
use reearth_flow_types::Feature;

use super::CompiledCommonReaderParam;

pub(crate) fn read_json(
    ctx: ExecutorContext,
    fw: &ProcessorChannelForwarder,
    global_params: &Option<HashMap<String, serde_json::Value>>,
    params: &CompiledCommonReaderParam,
) -> Result<(), super::errors::FeatureProcessorError> {
    let feature = &ctx.feature;
    let expr_engine = Arc::clone(&ctx.expr_engine);
    let storage_resolver = &ctx.storage_resolver;
    let scope = feature.new_scope(expr_engine.clone(), global_params);
    let json_path = scope
        .eval_ast::<String>(&params.expr)
        .unwrap_or_else(|_| params.original_expr.to_string());
    let input_path = Uri::from_str(json_path.as_str())
        .map_err(|e| super::errors::FeatureProcessorError::FileJsonReader(format!("{e:?}")))?;
    let storage = storage_resolver
        .resolve(&input_path)
        .map_err(|e| super::errors::FeatureProcessorError::FileJsonReader(format!("{e:?}")))?;
    let byte = storage
        .get_sync(input_path.path().as_path())
        .map_err(|e| super::errors::FeatureProcessorError::FileJsonReader(format!("{e:?}")))?;
    let value: serde_json::Value = serde_json::from_str(
        std::str::from_utf8(&byte)
            .map_err(|e| super::errors::FeatureProcessorError::FileJsonReader(format!("{e:?}")))?,
    )
    .map_err(|e| super::errors::FeatureProcessorError::FileJsonReader(format!("{e:?}")))?;
    match value {
        serde_json::Value::Array(arr) => {
            for v in arr {
                let feature = Feature::from(v);
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
        }
        _ => {
            let feature = Feature::from(value);
            fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        }
    }
    Ok(())
}
