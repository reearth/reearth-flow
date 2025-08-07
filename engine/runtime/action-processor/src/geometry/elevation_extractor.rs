use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
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
        "ElevationExtractor"
    }

    fn description(&self) -> &str {
        "Extract Z-Coordinate Elevation to Attribute"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ElevationExtractorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
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
/// Configure where to store the extracted elevation value from geometry coordinates
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ElevationExtractorParam {
    /// # Output Attribute
    /// Name of the attribute where the extracted elevation value will be stored
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

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()))
            }
            GeometryValue::FlowGeometry2D(geometry) => {
                let mut feature = feature.clone();
                feature.attributes.insert(
                    self.output_attribute.clone(),
                    AttributeValue::Number(
                        serde_json::Number::from_f64(geometry.elevation()).ok_or(
                            GeometryProcessorError::ElevationExtractor(
                                "Failed to convert elevation to number".to_string(),
                            ),
                        )?,
                    ),
                );
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geometry) => {
                let mut feature = feature.clone();
                feature.attributes.insert(
                    self.output_attribute.clone(),
                    AttributeValue::Number(
                        serde_json::Number::from_f64(geometry.elevation()).ok_or(
                            GeometryProcessorError::ElevationExtractor(
                                "Failed to convert elevation to number".to_string(),
                            ),
                        )?,
                    ),
                );
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            }
            GeometryValue::CityGmlGeometry(geometry) => {
                let mut feature = feature.clone();
                feature.attributes.insert(
                    self.output_attribute.clone(),
                    AttributeValue::Number(
                        serde_json::Number::from_f64(geometry.elevation()).ok_or(
                            GeometryProcessorError::ElevationExtractor(
                                "Failed to convert elevation to number".to_string(),
                            ),
                        )?,
                    ),
                );
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "ElevationExtractor"
    }
}
