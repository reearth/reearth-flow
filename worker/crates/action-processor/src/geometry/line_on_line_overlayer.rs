use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::algorithm::line_intersection::{self, line_intersection};
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::geometry::Geometry3D;
use reearth_flow_geometry::types::line::{Line2D, Line3D};
use reearth_flow_geometry::types::multi_point::{MultiPoint2D, MultiPoint3D};
use reearth_flow_geometry::types::point::{Point2D, Point3D};
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

use super::errors::GeometryProcessorError;

pub static POINT_PORT: Lazy<Port> = Lazy::new(|| Port::new("point"));
pub static LINE_PORT: Lazy<Port> = Lazy::new(|| Port::new("line"));
pub static COLLINEAR_PORT: Lazy<Port> = Lazy::new(|| Port::new("collinear"));

#[derive(Debug, Clone, Default)]
pub struct LineOnLineOverlayerFactory;

impl ProcessorFactory for LineOnLineOverlayerFactory {
    fn name(&self) -> &str {
        "LineOnLineOverlayer"
    }

    fn description(&self) -> &str {
        "Intersection points are turned into point features that can contain the merged list of attributes of the original intersected lines."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(LineOnLineOverlayerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![POINT_PORT.clone(), LINE_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: LineOnLineOverlayerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::LineOnLineOverlayerFactory(format!(
                    "Failed to serialize with: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::LineOnLineOverlayerFactory(format!(
                    "Failed to deserialize with: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::LineOnLineOverlayerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(LineOnLineOverlayer {
            output_attribute: params.output_attribute,
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LineOnLineOverlayerParam {
    output_attribute: Attribute,
}

#[derive(Debug, Clone)]
pub struct LineOnLineOverlayer {
    output_attribute: Attribute,
}

impl Processor for LineOnLineOverlayer {
    fn initialize(&mut self, _ctx: NodeContext) {}

    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let Some(geometry) = &feature.geometry else {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::Null => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geos) => {
                self.handle_2d_geometry(geos, feature, geometry, &ctx, fw);
            }
            GeometryValue::FlowGeometry3D(geos) => {
                self.handle_3d_geometry(geos, feature, geometry, &ctx, fw);
            }
            GeometryValue::CityGmlGeometry(_) => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
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
        "LineOnLineOverlayer"
    }
}

impl LineOnLineOverlayer {
    fn handle_2d_geometry(
        &self,
        geos: &Geometry2D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) {
        match geos {
            Geometry2D::MultiLineString(line_strings) => {
                let mut lines = Vec::new();
                for line_string in line_strings.iter() {
                    lines.extend(line_string.lines());
                }
                self.handle_2d_lines(feature, geometry, lines, ctx, fw);
            }
            Geometry2D::LineString(line_string) => {
                let lines = line_string.lines();
                self.handle_2d_lines(feature, geometry, lines.into_iter().collect(), ctx, fw);
            }
            _ => fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone())),
        }
        fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
    }

    fn handle_2d_lines(
        &self,
        feature: &Feature,
        geometry: &Geometry,
        lines: Vec<Line2D<f64>>,
        ctx: &ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) {
        let mut overlap = 0;
        let mut points = Vec::<Point2D<f64>>::new();
        for (i, current) in lines.iter().enumerate() {
            for next in &lines[i + 1..] {
                match line_intersection(*current, *next) {
                    Some(line_intersection::LineIntersection::SinglePoint {
                        intersection, ..
                    }) => {
                        overlap += 1;
                        points.push(intersection.into());
                    }
                    Some(line_intersection::LineIntersection::Collinear { .. }) => {
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), COLLINEAR_PORT.clone()),
                        );
                        return;
                    }
                    None => {}
                }
            }
        }
        if points.is_empty() {
            let mut feature = feature.clone();
            feature.attributes.insert(
                self.output_attribute.clone(),
                AttributeValue::Number(Number::from(overlap)),
            );
            fw.send(ctx.new_with_feature_and_port(feature, LINE_PORT.clone()));
            return;
        }
        let mut geometry = geometry.clone();
        let mut feature = feature.clone();
        geometry.value =
            GeometryValue::FlowGeometry2D(Geometry2D::MultiPoint(MultiPoint2D::new(points)));
        feature.geometry = Some(geometry);
        feature.attributes.insert(
            self.output_attribute.clone(),
            AttributeValue::Number(Number::from(overlap)),
        );
        fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
    }

    fn handle_3d_geometry(
        &self,
        geos: &Geometry3D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) {
        match geos {
            Geometry3D::MultiLineString(line_strings) => {
                let mut lines = Vec::new();
                for line_string in line_strings.iter() {
                    lines.extend(line_string.lines());
                }
                self.handle_3d_lines(feature, geometry, lines, ctx, fw);
            }
            Geometry3D::LineString(line_string) => {
                let lines = line_string.lines();
                self.handle_3d_lines(feature, geometry, lines.into_iter().collect(), ctx, fw);
            }
            _ => fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone())),
        }
        fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
    }

    fn handle_3d_lines(
        &self,
        feature: &Feature,
        geometry: &Geometry,
        lines: Vec<Line3D<f64>>,
        ctx: &ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) {
        let mut overlap = 0;
        let mut points = Vec::<Point3D<f64>>::new();
        for (i, current) in lines.iter().enumerate() {
            for next in &lines[i + 1..] {
                match line_intersection(*current, *next) {
                    Some(line_intersection::LineIntersection::SinglePoint {
                        intersection, ..
                    }) => {
                        overlap += 1;
                        points.push(intersection.into());
                    }
                    Some(line_intersection::LineIntersection::Collinear { .. }) => {
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), COLLINEAR_PORT.clone()),
                        );
                        return;
                    }
                    None => {}
                }
            }
        }
        if points.is_empty() {
            let mut feature = feature.clone();
            feature.attributes.insert(
                self.output_attribute.clone(),
                AttributeValue::Number(Number::from(overlap)),
            );
            fw.send(ctx.new_with_feature_and_port(feature, LINE_PORT.clone()));
            return;
        }
        let mut geometry = geometry.clone();
        let mut feature = feature.clone();
        geometry.value =
            GeometryValue::FlowGeometry3D(Geometry3D::MultiPoint(MultiPoint3D::new(points)));
        feature.geometry = Some(geometry);
        feature.attributes.insert(
            self.output_attribute.clone(),
            AttributeValue::Number(Number::from(overlap)),
        );
        fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
    }
}
