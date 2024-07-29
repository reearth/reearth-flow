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
use reearth_flow_types::{Feature, Geometry, GeometryValue};
use serde::{Deserialize, Serialize};
use serde_json::Number;
use serde_json::Value;

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
        None
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
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(AreaOnAreaOverlayer))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AreaOnAreaOverlayer;

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
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geos) => {
                self.handle_2d_geometry(geos, feature, geometry, &ctx, fw);
            }
            GeometryValue::FlowGeometry3D(_) => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::CityGmlGeometry(_) => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()))
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
        "AreaOnAreaOverlayer"
    }
}

impl AreaOnAreaOverlayer {
    fn handle_2d_geometry(
        &self,
        geos: &Geometry2D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) {
        match geos {
            Geometry2D::Polygon(_) => {
                let mut feature = feature.clone();
                feature.attributes.insert(
                    Attribute::new("overlap"),
                    AttributeValue::Number(Number::from(1)),
                );
                fw.send(ctx.new_with_feature_and_port(feature, AREA_PORT.clone()));
            }
            Geometry2D::MultiPolygon(mpoly) => {
                let mut overlap = 1;
                let mut intersections = Vec::<MultiPolygon2D<f64>>::new();
                let mut remnants = Vec::<MultiPolygon2D<f64>>::new();
                let mut areas = Vec::<Polygon2D<f64>>::new();
                let polygons: Vec<Polygon2D<f64>> = mpoly.iter().cloned().collect();
                for combo in polygons.iter().combinations(2) {
                    let inter = combo[0].intersection(combo[1]);
                    if inter.is_empty() {
                        continue;
                    }
                    overlap += 1;
                    areas.extend(
                        combo
                            .iter()
                            .map(|&p| p.clone())
                            .collect::<Vec<Polygon2D<f64>>>(),
                    );
                    intersections.push(inter);
                    let diff1 = combo[0].difference(combo[1]);
                    let diff2 = combo[1].difference(combo[0]);
                    remnants.push(diff1);
                    remnants.push(diff2);
                }
                for intersection in intersections.iter() {
                    let mut feature = feature.clone();
                    feature.attributes.insert(
                        Attribute::new("overlap"),
                        AttributeValue::Number(Number::from(overlap)),
                    );
                    let mut geometry = geometry.clone();
                    geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::MultiPolygon(
                        intersection.clone(),
                    ));
                    feature.geometry = Some(geometry);
                    fw.send(ctx.new_with_feature_and_port(feature, AREA_PORT.clone()));
                }
                for remnant in remnants.iter() {
                    let mut feature = feature.clone();
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
                for polygon in polygons.iter() {
                    if areas.contains(polygon) {
                        continue;
                    }
                    let mut feature = feature.clone();
                    feature.attributes.insert(
                        Attribute::new("overlap"),
                        AttributeValue::Number(Number::from(overlap)),
                    );
                    let mut geometry = geometry.clone();
                    geometry.value =
                        GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon.clone()));
                    feature.geometry = Some(geometry);
                    fw.send(ctx.new_with_feature_and_port(feature, AREA_PORT.clone()));
                }
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
    }
}
