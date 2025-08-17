use std::collections::HashMap;

use reearth_flow_geometry::types::{
    coordinate::Coordinate, geometry::Geometry3D as FlowGeometry3D, rect::Rect,
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct ThreeDimensionBoxReplacerFactory;

impl ProcessorFactory for ThreeDimensionBoxReplacerFactory {
    fn name(&self) -> &str {
        "ThreeDimensionBoxReplacer"
    }

    fn description(&self) -> &str {
        "Replace Geometry with 3D Box from Attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ThreeDimensionBoxReplacer))
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
        let processor: ThreeDimensionBoxReplacer = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::ThreeDimensionBoxReplacerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::ThreeDimensionBoxReplacerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::ThreeDimensionBoxReplacerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(processor))
    }
}

/// # 3D Box Replacer Parameters
/// Configure which attributes contain the minimum and maximum coordinates for creating a 3D box
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThreeDimensionBoxReplacer {
    /// # Minimum X Attribute
    /// Name of attribute containing the minimum X coordinate
    min_x: Attribute,
    /// # Minimum Y Attribute
    /// Name of attribute containing the minimum Y coordinate
    min_y: Attribute,
    /// # Minimum Z Attribute
    /// Name of attribute containing the minimum Z coordinate
    min_z: Attribute,
    /// # Maximum X Attribute
    /// Name of attribute containing the maximum X coordinate
    max_x: Attribute,
    /// # Maximum Y Attribute
    /// Name of attribute containing the maximum Y coordinate
    max_y: Attribute,
    /// # Maximum Z Attribute
    /// Name of attribute containing the maximum Z coordinate
    max_z: Attribute,
}

impl Processor for ThreeDimensionBoxReplacer {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let attributes = &ctx.feature.attributes;
        let min_x = parse_f64(attributes.get(&self.min_x))?;
        let min_y = parse_f64(attributes.get(&self.min_y))?;
        let min_z = parse_f64(attributes.get(&self.min_z))?;
        let max_x = parse_f64(attributes.get(&self.max_x))?;
        let max_y = parse_f64(attributes.get(&self.max_y))?;
        let max_z = parse_f64(attributes.get(&self.max_z))?;
        let min = Coordinate::new__(min_x, min_y, min_z);
        let max = Coordinate::new__(max_x, max_y, max_z);
        let rectangle = Rect::new(min, max);
        let geometry = Geometry::with_value(GeometryValue::FlowGeometry3D(
            FlowGeometry3D::Polygon(rectangle.to_polygon()),
        ));
        let mut feature = ctx.feature.clone();
        feature.geometry = geometry;
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "ThreeDimensionBoxReplacer"
    }
}

fn parse_f64(value: Option<&AttributeValue>) -> super::errors::Result<f64> {
    if let Some(AttributeValue::Number(min_x)) = value {
        min_x
            .as_f64()
            .ok_or(GeometryProcessorError::ThreeDimensionBoxReplacer(
                "failed to parse f64".to_string(),
            ))
    } else {
        Err(GeometryProcessorError::ThreeDimensionBoxReplacer(
            "failed to parse f64".to_string(),
        ))
    }
}
