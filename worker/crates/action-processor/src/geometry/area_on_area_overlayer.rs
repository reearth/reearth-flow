use std::collections::HashMap;

use itertools::Itertools;
use once_cell::sync::Lazy;
use reearth_flow_geometry::algorithm::bool_ops::BooleanOps;
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::Polygon2D;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::Attribute;
use reearth_flow_types::AttributeValue;
use reearth_flow_types::{Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use serde_json::Value;

use super::errors::GeometryProcessorError;

pub static AREA_PORT: Lazy<Port> = Lazy::new(|| Port::new("area"));
pub static REMNANTS_PORT: Lazy<Port> = Lazy::new(|| Port::new("remnants"));

#[derive(Debug, Clone, Default)]
pub struct AreaOnAreaOverlayerFactory;

impl ProcessorFactory for AreaOnAreaOverlayerFactory {
    fn name(&self) -> &str {
        "AreaOnAreaOverlayer"
    }

    fn description(&self) -> &str {
        "Overlays an area on another area"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AreaOnAreaOverlayerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            AREA_PORT.clone(),
            REMNANTS_PORT.clone(),
            REJECTED_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: AreaOnAreaOverlayerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::AreaOnAreaOverlayerFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::AreaOnAreaOverlayerFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::AreaOnAreaOverlayerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(AreaOnAreaOverlayer {
            params,
            buffer: HashMap::new(),
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AreaOnAreaOverlayerParam {
    group_by: Option<Vec<Attribute>>,
    output_attribute: Attribute,
}

#[derive(Debug, Clone)]
pub struct AreaOnAreaOverlayer {
    params: AreaOnAreaOverlayerParam,
    buffer: HashMap<String, Vec<Feature>>,
}

impl Processor for AreaOnAreaOverlayer {
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
            GeometryValue::FlowGeometry2D(_) => {
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
        "AreaOnAreaOverlayer"
    }
}

impl AreaOnAreaOverlayer {
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
        let mut target_polygons = others.iter().filter_map(|g| g.as_polygon()).collect_vec();
        target_polygons.extend(
            others
                .iter()
                .filter_map(|g| g.as_multi_polygon().map(|mpoly| mpoly.0))
                .flatten()
                .collect_vec(),
        );

        match geos {
            Geometry2D::Polygon(poly) => {
                let mut feature = feature.clone();
                if target_polygons.is_empty() {
                    feature.attributes.insert(
                        Attribute::new("overlap"),
                        AttributeValue::Number(Number::from(1)),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, AREA_PORT.clone()));
                    return;
                }
                self.handle_2d_polygons(poly, &target_polygons, &feature, ctx, fw)
            }
            Geometry2D::MultiPolygon(mpoly) => {
                let mut feature = feature.clone();
                if target_polygons.is_empty() {
                    feature.attributes.insert(
                        Attribute::new("overlap"),
                        AttributeValue::Number(Number::from(1)),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, AREA_PORT.clone()));
                    return;
                }
                for poly in mpoly.0.iter() {
                    self.handle_2d_polygons(poly, &target_polygons, &feature, ctx, fw)
                }
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
    }

    fn handle_2d_polygons(
        &self,
        target: &Polygon2D<f64>,
        others: &[Polygon2D<f64>],
        feature: &Feature,
        ctx: &ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) {
        let mut overlap = 1;
        let mut remnants = Vec::<MultiPolygon2D<f64>>::new();
        let mut areas = Vec::<Polygon2D<f64>>::new();
        let mut intersections = Vec::<MultiPolygon2D<f64>>::new();
        for other in others.iter() {
            let inter = target.intersection(other);
            if inter.is_empty() {
                continue;
            }
            intersections.push(inter);
            overlap += 1;
            areas.push(other.clone());
            let diff = target.difference(other);
            remnants.push(diff);
        }
        if areas.is_empty() {
            let mut feature = feature.clone();
            feature.attributes.insert(
                Attribute::new("overlap"),
                AttributeValue::Number(Number::from(overlap)),
            );
            fw.send(ctx.new_with_feature_and_port(feature, AREA_PORT.clone()));
            return;
        }
        for intersection in intersections.iter() {
            let Some(geometry) = &feature.geometry else {
                return;
            };
            let mut feature = feature.clone();
            feature.id = uuid::Uuid::new_v4();
            feature.attributes.insert(
                Attribute::new("overlap"),
                AttributeValue::Number(Number::from(overlap)),
            );
            let mut geometry = geometry.clone();
            geometry.value =
                GeometryValue::FlowGeometry2D(Geometry2D::MultiPolygon(intersection.clone()));
            feature.geometry = Some(geometry);
            fw.send(ctx.new_with_feature_and_port(feature, AREA_PORT.clone()));
        }
        for remnant in remnants.iter() {
            let Some(geometry) = &feature.geometry else {
                return;
            };
            let mut feature = feature.clone();
            feature.id = uuid::Uuid::new_v4();
            feature.attributes.insert(
                Attribute::new("overlap"),
                AttributeValue::Number(Number::from(overlap)),
            );
            let mut geometry = geometry.clone();
            geometry.value =
                GeometryValue::FlowGeometry2D(Geometry2D::MultiPolygon(remnant.clone()));
            feature.geometry = Some(geometry);
            fw.send(ctx.new_with_feature_and_port(feature, REMNANTS_PORT.clone()));
        }
    }
}
