use reearth_flow_geometry::types::{
    coordinate::Coordinate, geometry::Geometry as FlowGeometry, rectangle::Rectangle,
};
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    executor_operation::{ExecutorContext, NodeContext},
    node::DEFAULT_PORT,
};
use reearth_flow_types::{Attribute, AttributeValue, Geometry, GeometryValue};
use serde::{Deserialize, Serialize};

use crate::universal::UniversalProcessor;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ThreeDimentionBoxReplacer {
    min_x: Attribute,
    min_y: Attribute,
    min_z: Attribute,
    max_x: Attribute,
    max_y: Attribute,
    max_z: Attribute,
}

#[typetag::serde(name = "ThreeDimentionBoxReplacer")]
impl UniversalProcessor for ThreeDimentionBoxReplacer {
    fn initialize(&mut self, _ctx: NodeContext) {}
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
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
        let rectangle = Rectangle::new(min, max);
        let geometry = Geometry::with_value(GeometryValue::FlowGeometry(FlowGeometry::Polygon(
            rectangle.to_polygon(),
        )));
        let mut feature = ctx.feature.clone();
        feature.geometry = Some(geometry);
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(
        &self,
        _ctx: NodeContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "ThreeDimentionBoxReplacer"
    }
}

fn parse_f64(value: Option<&AttributeValue>) -> crate::errors::Result<f64> {
    if let Some(AttributeValue::Number(min_x)) = value {
        min_x
            .as_f64()
            .ok_or(crate::errors::ProcessorError::ThreeDimentionBoxReplacer(
                "failed to parse f64".to_string(),
            ))
    } else {
        Err(crate::errors::ProcessorError::ThreeDimentionBoxReplacer(
            "failed to parse f64".to_string(),
        ))
    }
}
