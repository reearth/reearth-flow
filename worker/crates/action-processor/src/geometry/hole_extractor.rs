use std::{collections::HashMap, vec};

use once_cell::sync::Lazy;
use reearth_flow_geometry::types::{
    geometry::{Geometry2D, Geometry3D},
    polygon::{Polygon2D, Polygon3D},
};
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Feature, GeometryValue};
use serde_json::Value;

pub static OUTERSHELL_PORT: Lazy<Port> = Lazy::new(|| Port::new("outershell"));
pub static HOLE_PORT: Lazy<Port> = Lazy::new(|| Port::new("hole"));

#[derive(Debug, Clone, Default)]
pub struct HoleExtractorFactory;

impl ProcessorFactory for HoleExtractorFactory {
    fn name(&self) -> &str {
        "HoleExtractor"
    }

    fn description(&self) -> &str {
        "Extracts holes in a geometry and adds it as an attribute."
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
            OUTERSHELL_PORT.clone(),
            HOLE_PORT.clone(),
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
        Ok(Box::new(HoleExtractor))
    }
}

#[derive(Debug, Clone)]
pub struct HoleExtractor;

impl Processor for HoleExtractor {
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
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()))
            }
            GeometryValue::FlowGeometry2D(geometry) => match geometry {
                Geometry2D::Polygon(polygon) => {
                    handle_polygon2d(polygon, feature, &ctx, fw);
                }
                Geometry2D::MultiPolygon(mpolygon) => {
                    for polygon in mpolygon.iter() {
                        handle_polygon2d(polygon, feature, &ctx, fw);
                    }
                }
                _ => {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                }
            },
            GeometryValue::FlowGeometry3D(geometry) => match geometry {
                Geometry3D::Polygon(polygon) => {
                    handle_polygon3d(polygon, feature, &ctx, fw);
                }
                Geometry3D::MultiPolygon(mpolygon) => {
                    for polygon in mpolygon.iter() {
                        handle_polygon3d(polygon, feature, &ctx, fw);
                    }
                }
                _ => {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                }
            },
            GeometryValue::CityGmlGeometry(geometry) => {
                for geo_feature in geometry.features.iter() {
                    for polygon in &geo_feature.polygons {
                        handle_polygon3d(polygon, feature, &ctx, fw);
                    }
                }
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
        "HoleExtractor"
    }
}

fn handle_polygon2d(
    polygon: &Polygon2D<f64>,
    feature: &Feature,
    ctx: &ExecutorContext,
    fw: &mut dyn ProcessorChannelForwarder,
) {
    let exterior = polygon.exterior();
    let exterior_polygon = Polygon2D::new(exterior.clone(), vec![]);
    if let Some(ref geometry) = &feature.geometry {
        let mut exterior_feature = feature.clone();
        exterior_feature.id = uuid::Uuid::new_v4();
        let mut exterior_geometry = geometry.clone();
        exterior_geometry.value =
            GeometryValue::FlowGeometry2D(Geometry2D::Polygon(exterior_polygon));
        exterior_feature.geometry = Some(exterior_geometry);
        fw.send(ctx.new_with_feature_and_port(exterior_feature, OUTERSHELL_PORT.clone()));
    }
    for interior in polygon.interiors().iter() {
        let interior_polygon = Polygon2D::new(interior.clone(), vec![]);
        if let Some(ref geometry) = &feature.geometry {
            let mut interior_feature = feature.clone();
            interior_feature.id = uuid::Uuid::new_v4();
            let mut interior_geometry = geometry.clone();
            interior_geometry.value =
                GeometryValue::FlowGeometry2D(Geometry2D::Polygon(interior_polygon));
            interior_feature.geometry = Some(interior_geometry);
            fw.send(ctx.new_with_feature_and_port(interior_feature, HOLE_PORT.clone()));
        }
    }
}

fn handle_polygon3d(
    polygon: &Polygon3D<f64>,
    feature: &Feature,
    ctx: &ExecutorContext,
    fw: &mut dyn ProcessorChannelForwarder,
) {
    let exterior = polygon.exterior();
    let exterior_polygon = Polygon3D::new(exterior.clone(), vec![]);
    if let Some(ref geometry) = &feature.geometry {
        let mut exterior_feature = feature.clone();
        exterior_feature.id = uuid::Uuid::new_v4();
        let mut exterior_geometry = geometry.clone();
        exterior_geometry.value =
            GeometryValue::FlowGeometry3D(Geometry3D::Polygon(exterior_polygon));
        exterior_feature.geometry = Some(exterior_geometry);
        fw.send(ctx.new_with_feature_and_port(exterior_feature, OUTERSHELL_PORT.clone()));
    }
    for interior in polygon.interiors().iter() {
        let interior_polygon = Polygon3D::new(interior.clone(), vec![]);
        if let Some(ref geometry) = &feature.geometry {
            let mut interior_feature = feature.clone();
            interior_feature.id = uuid::Uuid::new_v4();
            let mut interior_geometry = geometry.clone();
            interior_geometry.value =
                GeometryValue::FlowGeometry3D(Geometry3D::Polygon(interior_polygon));
            interior_feature.geometry = Some(interior_geometry);
            fw.send(ctx.new_with_feature_and_port(interior_feature, HOLE_PORT.clone()));
        }
    }
}
