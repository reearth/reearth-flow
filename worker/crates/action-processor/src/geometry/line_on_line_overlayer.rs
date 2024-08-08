use std::collections::HashMap;

use itertools::Itertools;
use once_cell::sync::Lazy;
use reearth_flow_geometry::algorithm::line_intersection::{
    self, line_intersection, line_intersection3d,
};
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::geometry::Geometry3D;
use reearth_flow_geometry::types::multi_line_string::{MultiLineString2D, MultiLineString3D};
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
use reearth_flow_types::{Attribute, AttributeValue, Feature, GeometryValue};
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
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::LineOnLineOverlayerFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
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
            params,
            buffer: HashMap::new(),
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LineOnLineOverlayerParam {
    group_by: Option<Vec<Attribute>>,
    output_attribute: Attribute,
}

#[derive(Debug, Clone)]
pub struct LineOnLineOverlayer {
    params: LineOnLineOverlayerParam,
    buffer: HashMap<String, Vec<Feature>>,
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
            GeometryValue::FlowGeometry2D(_) | GeometryValue::FlowGeometry3D(_) => {
                let key = if let Some(group_by) = &self.params.group_by {
                    group_by
                        .iter()
                        .map(|k| feature.get(&k).map(|v| v.to_string()).unwrap_or_default())
                        .collect::<Vec<_>>()
                        .join(",")
                } else {
                    "_all".to_string()
                };
                if let Some(values) = self.buffer.get(&key) {
                    self.handle_geometry(feature, values, &ctx, fw);
                    {
                        if let Some(buffer) = self.buffer.get_mut(&key) {
                            buffer.push(feature.clone());
                        }
                    }
                } else {
                    self.buffer.insert(key, vec![feature.clone()]);
                    self.handle_geometry(feature, &[], &ctx, fw);
                }
            }
            _ => fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone())),
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
    fn handle_geometry(
        &self,
        feature: &Feature,
        others: &[Feature],
        ctx: &ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) {
        let Some(geometry) = feature.geometry.as_ref() else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return;
        };
        match &geometry.value {
            GeometryValue::FlowGeometry2D(geos) => {
                let others = others
                    .iter()
                    .filter_map(|f| {
                        f.geometry
                            .as_ref()
                            .and_then(|g| g.value.as_flow_geometry_2d().cloned())
                    })
                    .collect::<Vec<_>>();
                self.handle_2d_geometry(geos, &others, feature, ctx, fw);
            }
            GeometryValue::FlowGeometry3D(geos) => {
                let others = others
                    .iter()
                    .filter_map(|f| {
                        f.geometry
                            .as_ref()
                            .and_then(|g| g.value.as_flow_geometry_3d().cloned())
                    })
                    .collect::<Vec<_>>();
                self.handle_3d_geometry(geos, &others, feature, ctx, fw);
            }
            _ => fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone())),
        }
    }

    fn handle_2d_geometry(
        &self,
        geos: &Geometry2D,
        others: &[Geometry2D],
        feature: &Feature,
        ctx: &ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) {
        let mut target_line_strings = others
            .iter()
            .filter_map(|g| g.as_multi_line_string())
            .collect_vec();
        target_line_strings.extend(
            others
                .iter()
                .filter_map(|g| {
                    g.as_line_string()
                        .map(|line| MultiLineString2D::new(vec![line]))
                })
                .collect_vec(),
        );
        match geos {
            Geometry2D::MultiLineString(line_strings) => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), LINE_PORT.clone()));
                if target_line_strings.is_empty() {
                    return;
                }
                self.handle_2d_line_strings(line_strings, target_line_strings, feature, ctx, fw);
            }
            Geometry2D::LineString(line_string) => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), LINE_PORT.clone()));
                if target_line_strings.is_empty() {
                    return;
                }
                self.handle_2d_line_strings(
                    &MultiLineString2D::new(vec![line_string.clone()]),
                    target_line_strings,
                    feature,
                    ctx,
                    fw,
                );
            }
            _ => fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone())),
        }
    }

    fn handle_2d_line_strings(
        &self,
        target: &MultiLineString2D<f64>,
        others: Vec<MultiLineString2D<f64>>,
        feature: &Feature,
        ctx: &ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) {
        let mut overlap = 1;
        let mut points = Vec::<Point2D<f64>>::new();
        for other in others.iter() {
            for line_target in target.lines() {
                for line_other in other.lines() {
                    match line_intersection(line_target, line_other) {
                        Some(line_intersection::LineIntersection::SinglePoint {
                            intersection,
                            ..
                        }) => {
                            overlap += 1;
                            points.push(intersection.into());
                        }
                        Some(line_intersection::LineIntersection::Collinear { .. }) => {
                            fw.send(ctx.new_with_feature_and_port(
                                feature.clone(),
                                COLLINEAR_PORT.clone(),
                            ));
                            return;
                        }
                        None => {}
                    }
                }
            }
        }
        if !points.is_empty() {
            let Some(geometry) = &feature.geometry else {
                return;
            };
            let mut geometry = geometry.clone();
            let mut feature = feature.clone();
            feature.id = uuid::Uuid::new_v4();
            geometry.value =
                GeometryValue::FlowGeometry2D(Geometry2D::MultiPoint(MultiPoint2D::new(points)));
            feature.geometry = Some(geometry);
            feature.attributes.insert(
                self.params.output_attribute.clone(),
                AttributeValue::Number(Number::from(overlap)),
            );
            fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
        }
    }

    fn handle_3d_geometry(
        &self,
        geos: &Geometry3D,
        others: &[Geometry3D],
        feature: &Feature,
        ctx: &ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) {
        let mut target_line_strings = others
            .iter()
            .filter_map(|g| g.as_multi_line_string())
            .collect_vec();
        target_line_strings.extend(
            others
                .iter()
                .filter_map(|g| {
                    g.as_line_string()
                        .map(|line| MultiLineString3D::new(vec![line]))
                })
                .collect_vec(),
        );
        match geos {
            Geometry3D::MultiLineString(line_strings) => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), LINE_PORT.clone()));
                if target_line_strings.is_empty() {
                    return;
                }
                self.handle_3d_line_strings(line_strings, target_line_strings, feature, ctx, fw);
            }
            Geometry3D::LineString(line_string) => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), LINE_PORT.clone()));
                if target_line_strings.is_empty() {
                    return;
                }
                self.handle_3d_line_strings(
                    &MultiLineString3D::new(vec![line_string.clone()]),
                    target_line_strings,
                    feature,
                    ctx,
                    fw,
                );
            }
            _ => fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone())),
        }
    }

    fn handle_3d_line_strings(
        &self,
        target: &MultiLineString3D<f64>,
        others: Vec<MultiLineString3D<f64>>,
        feature: &Feature,
        ctx: &ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) {
        let mut overlap = 1;
        let mut points = Vec::<Point3D<f64>>::new();
        for other in others.iter() {
            for line_target in target.lines() {
                for line_other in other.lines() {
                    if let Some(point) = line_intersection3d(line_target, line_other) {
                        overlap += 1;
                        points.push(point.into());
                    }
                }
            }
        }
        if !points.is_empty() {
            let Some(geometry) = &feature.geometry else {
                return;
            };
            let mut geometry = geometry.clone();
            let mut feature = feature.clone();
            feature.id = uuid::Uuid::new_v4();
            geometry.value =
                GeometryValue::FlowGeometry3D(Geometry3D::MultiPoint(MultiPoint3D::new(points)));
            feature.geometry = Some(geometry);
            feature.attributes.insert(
                self.params.output_attribute.clone(),
                AttributeValue::Number(Number::from(overlap)),
            );
            fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
        }
    }
}
