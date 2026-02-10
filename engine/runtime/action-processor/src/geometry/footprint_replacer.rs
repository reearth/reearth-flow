use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::{
    algorithm::{area2d::Area2D, bool_ops::BooleanOps},
    types::{
        coordinate::Coordinate2D,
        geometry::{Geometry2D, Geometry3D},
        line_string::LineString2D,
        multi_polygon::MultiPolygon2D,
        polygon::{Polygon2D, Polygon3D},
    },
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{CityGmlGeometry, Feature, GeometryValue};
use serde_json::Value;

pub static FOOTPRINT_PORT: Lazy<Port> = Lazy::new(|| Port::new("footprint"));

#[derive(Debug, Clone, Default)]
pub struct FootprintReplacerFactory;

impl ProcessorFactory for FootprintReplacerFactory {
    fn name(&self) -> &str {
        "FootprintReplacer"
    }

    fn description(&self) -> &str {
        "Projects 3D geometry to XY plane and computes the union footprint (supports solids, surfaces, and CityGML)"
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
        vec![FOOTPRINT_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(FootprintReplacer))
    }
}

#[derive(Debug, Clone)]
pub struct FootprintReplacer;

impl Processor for FootprintReplacer {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geom) => {
                if let Some(footprint) = create_footprint_from_geometry3d(feature, geom) {
                    fw.send(ctx.new_with_feature_and_port(footprint, FOOTPRINT_PORT.clone()));
                } else {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                }
            }
            GeometryValue::CityGmlGeometry(citygml) => {
                if let Some(footprint) = create_footprint_from_citygml(feature, citygml) {
                    fw.send(ctx.new_with_feature_and_port(footprint, FOOTPRINT_PORT.clone()));
                } else {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                }
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(&mut self, _: NodeContext, _: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "FootprintReplacer"
    }
}

/// Extract 3D polygons from a Geometry3D, handling solids, surfaces, and other geometry types.
fn extract_polygons_from_geometry3d(geom: &Geometry3D<f64>) -> Vec<Polygon3D<f64>> {
    match geom {
        Geometry3D::Polygon(poly) => vec![poly.clone()],
        Geometry3D::MultiPolygon(mpoly) => mpoly.0.clone(),
        Geometry3D::Solid(solid) => {
            // Extract all faces from the solid and convert to polygons
            solid
                .all_faces()
                .into_iter()
                .map(|face| {
                    let coords = face.0;
                    Polygon3D::new(
                        reearth_flow_geometry::types::line_string::LineString3D::new(coords),
                        vec![],
                    )
                })
                .collect()
        }
        Geometry3D::Triangle(triangle) => {
            // Convert triangle to polygon
            let arr = triangle.to_array();
            let coords = vec![arr[0], arr[1], arr[2], arr[0]];
            vec![Polygon3D::new(
                reearth_flow_geometry::types::line_string::LineString3D::new(coords),
                vec![],
            )]
        }
        Geometry3D::GeometryCollection(gc) => {
            // Recursively extract polygons from geometry collection
            gc.iter()
                .flat_map(extract_polygons_from_geometry3d)
                .collect()
        }
        _ => vec![],
    }
}

/// Create footprint from FlowGeometry3D
fn create_footprint_from_geometry3d(feature: &Feature, geom: &Geometry3D<f64>) -> Option<Feature> {
    let polygons = extract_polygons_from_geometry3d(geom);

    if polygons.is_empty() {
        return None;
    }

    create_footprint_from_polygons(feature, &polygons)
}

/// Create footprint from CityGML geometry
fn create_footprint_from_citygml(feature: &Feature, citygml: &CityGmlGeometry) -> Option<Feature> {
    // Collect all polygons from all GML geometries
    let polygons: Vec<Polygon3D<f64>> = citygml
        .gml_geometries
        .iter()
        .flat_map(|gml_geom| gml_geom.polygons.clone())
        .collect();

    if polygons.is_empty() {
        return None;
    }

    create_footprint_from_polygons(feature, &polygons)
}

/// Project a 3D polygon to the XY plane (drop Z coordinate)
fn project_polygon_to_2d(polygon: &Polygon3D<f64>) -> Polygon2D<f64> {
    let exterior: Vec<Coordinate2D<f64>> = polygon
        .exterior()
        .coords()
        .map(|c| Coordinate2D::new_(c.x, c.y))
        .collect();

    let interiors: Vec<LineString2D<f64>> = polygon
        .interiors()
        .iter()
        .map(|interior| {
            let coords: Vec<Coordinate2D<f64>> = interior
                .coords()
                .map(|c| Coordinate2D::new_(c.x, c.y))
                .collect();
            LineString2D::new(coords)
        })
        .collect();

    Polygon2D::new(LineString2D::new(exterior), interiors)
}

/// Minimum area threshold for projected polygons.  Polygons whose XY
/// projection has an area smaller than this are considered degenerate
/// (e.g. vertical wall faces) and are excluded from the footprint.
const MIN_PROJECTED_AREA: f64 = 1e-6;

/// Create footprint from a collection of 3D polygons
fn create_footprint_from_polygons(
    feature: &Feature,
    polygons: &[Polygon3D<f64>],
) -> Option<Feature> {
    let mut projected_polygons = Vec::new();

    // Project each polygon to the XY plane
    for polygon in polygons {
        let projected = project_polygon_to_2d(polygon);

        // Skip degenerate polygons (less than 3 points in exterior)
        if projected.exterior().0.len() < 3 {
            continue;
        }

        // Skip near-zero-area projections (e.g. vertical wall faces)
        if projected.unsigned_area2d() < MIN_PROJECTED_AREA {
            continue;
        }

        projected_polygons.push(projected);
    }

    if projected_polygons.is_empty() {
        return None;
    }

    // Union all projected polygons in a single pass.
    // Splitting into two MultiPolygons and calling union() once lets i_overlay's
    // sweep-line dissolve all overlaps in one pass with the NonZero fill rule,
    // instead of doing N-1 sequential union operations that accumulate vertex noise.
    let combined_polygons = if projected_polygons.len() == 1 {
        MultiPolygon2D::new(projected_polygons)
    } else {
        let mid = projected_polygons.len() / 2;
        let group_b = MultiPolygon2D::new(projected_polygons.split_off(mid));
        let group_a = MultiPolygon2D::new(projected_polygons);
        group_a.union(&group_b)
    };

    let mut result_feature = feature.clone();
    result_feature.geometry_mut().value =
        GeometryValue::FlowGeometry2D(Geometry2D::MultiPolygon(combined_polygons));

    Some(result_feature)
}
