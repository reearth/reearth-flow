use std::collections::HashMap;

use reearth_flow_common::compress::compress;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct GeometryExtractorFactory;

impl ProcessorFactory for GeometryExtractorFactory {
    fn name(&self) -> &str {
        "Geometry Extractor"
    }

    fn description(&self) -> &str {
        "Serializes a feature's geometry to a compressed representation and stores it in an attribute."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeometryExtractor))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let processor: GeometryExtractor = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::GeometryExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::GeometryExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::GeometryExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(processor))
    }
}

/// # Geometry Extractor Parameters
/// Configure where the serialized geometry is stored.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeometryExtractor {
    /// # Output Attribute
    /// Attribute to store the compressed geometry in. Geometry Replacer reads
    /// the same representation back onto a feature.
    output_attribute: Attribute,
}

impl Processor for GeometryExtractor {
    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        // Nothing to serialize is not a failure to serialize, so the feature
        // passes through untouched rather than being rejected. Geometry Replacer,
        // this action's counterpart, relies on that: it must tolerate a feature
        // arriving with no stored geometry to restore.
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), FEATURES_PORT.clone()));
            return Ok(());
        };
        let mut feature = feature.clone();
        let value = serde_json::to_value(geometry).map_err(|e| {
            GeometryProcessorError::GeometryExtractor(format!("Failed to serialize geometry: {e}"))
        })?;
        let dump = serde_json::to_string(&value)?;
        let dump = compress(&dump)?;
        feature.insert(&self.output_attribute, AttributeValue::String(dump));
        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Geometry Extractor"
    }
}
