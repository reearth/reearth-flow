use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct ElevationExtractorFactory;

impl ProcessorFactory for ElevationExtractorFactory {
    fn name(&self) -> &str {
        "Elevation Extractor"
    }

    fn description(&self) -> &str {
        "Extracts the elevation of a feature's geometry and stores it in an attribute."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ElevationExtractorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["3d"]
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
        let params: ElevationExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::ElevationExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::ElevationExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::ElevationExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(ElevationExtractor {
            output_attribute: params.output_attribute,
        }))
    }
}

/// # Elevation Extractor Parameters
/// Configure where the extracted elevation is stored.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ElevationExtractorParam {
    /// # Output Attribute
    /// Attribute to store the elevation in.
    output_attribute: Attribute,
}

#[derive(Debug, Clone)]
pub struct ElevationExtractor {
    output_attribute: Attribute,
}

impl Processor for ElevationExtractor {
    fn num_threads(&self) -> usize {
        2
    }

    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        // A geometry carrying no elevation has none to extract. That is nothing
        // to do rather than a failure, so the feature passes through untouched —
        // but without the attribute. Writing zero, as this used to, invents a
        // value indistinguishable from a real sea-level one. The unified geometry
        // model draws the same line: a planar leaf's `elevation()` is `None`.
        let elevation = match &geometry.value {
            GeometryValue::None | GeometryValue::FlowGeometry2D(_) => None,
            GeometryValue::FlowGeometry3D(geometry) => Some(geometry.elevation()),
            GeometryValue::CityGmlGeometry(geometry) => Some(geometry.elevation()),
        };

        let feature = match elevation {
            Some(elevation) => {
                let number = serde_json::Number::from_f64(elevation).ok_or(
                    GeometryProcessorError::ElevationExtractor(
                        "Failed to convert elevation to number".to_string(),
                    ),
                )?;
                let mut feature = feature.clone();
                feature.insert(&self.output_attribute, AttributeValue::Number(number));
                feature
            }
            None => feature.clone(),
        };
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
        "Elevation Extractor"
    }
}
