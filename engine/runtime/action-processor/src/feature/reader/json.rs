use std::str::FromStr;

use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    executor_operation::ExecutorContext, forwarder::ProcessorChannelForwarder, node::DEFAULT_PORT,
};
use reearth_flow_types::Feature;

use super::CompiledCommonReaderParam;

pub(crate) fn read_json(
    ctx: ExecutorContext,
    fw: &ProcessorChannelForwarder,
    params: &CompiledCommonReaderParam,
) -> Result<(), super::errors::FeatureProcessorError> {
    let feature = &ctx.feature;
    let storage_resolver = &ctx.storage_resolver;
    let json_path = params
        .dataset
        .eval_string(feature, ctx.env_vars.clone())
        .map_err(|e| super::errors::FeatureProcessorError::FileJsonReader(format!("{e:?}")))?;
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
