use std::collections::HashMap;
use std::sync::Arc;

use reearth_flow_common::compress::decode;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Geometry};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct GeometryReplacerFactory;

impl ProcessorFactory for GeometryReplacerFactory {
    fn name(&self) -> &str {
        "Geometry Replacer"
    }

    fn description(&self) -> &str {
        "Replaces a feature's geometry with the compressed geometry data stored in a named attribute."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeometryReplacer))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn tags(&self) -> &[&'static str] {
        &["attribute"]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let processor: GeometryReplacer = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::GeometryReplacerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::GeometryReplacerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::GeometryReplacerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(processor))
    }
}

/// # Geometry Replacer Parameters
/// Configure which attribute holds the geometry that replaces the feature's current geometry.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeometryReplacer {
    /// # Source Attribute
    /// Attribute holding the compressed geometry to apply, as written by Geometry
    /// Extractor. The attribute is removed once its geometry has been applied, and a
    /// feature that does not carry it passes through unchanged.
    source_attribute: Attribute,
}

impl Processor for GeometryReplacer {
    fn num_threads(&self) -> usize {
        2
    }

    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();

        // A feature carrying geometry that cannot be decoded is a failure, and
        // emitting it on `features` would pass it off as transformed. Those go to
        // `rejected`. An absent attribute is not a failure: `Geometry Extractor`
        // deliberately skips features with empty geometry, so its counterpart here
        // has nothing to restore and passes them through untouched.
        let reject = |reason: String| {
            ctx.event_hub.debug_log(
                Some(ctx.error_span()),
                format!("geometry replace rejected: {reason}"),
            );
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
        };

        let Some(source) = feature.attributes.get(&self.source_attribute) else {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), FEATURES_PORT.clone()));
            return Ok(());
        };
        let AttributeValue::String(dump) = source else {
            reject(format!(
                "attribute `{}` is not a string",
                self.source_attribute
            ));
            return Ok(());
        };
        let geometry = match decode(dump).map_err(|e| e.to_string()).and_then(|dump| {
            serde_json::from_str::<Geometry>(&dump).map_err(|e: serde_json::Error| e.to_string())
        }) {
            Ok(geometry) => geometry,
            Err(e) => {
                reject(format!(
                    "attribute `{}` does not decode to geometry: {e}",
                    self.source_attribute
                ));
                return Ok(());
            }
        };

        feature.geometry = Arc::new(geometry);
        feature.remove(&self.source_attribute);
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
        "Geometry Replacer"
    }
}
