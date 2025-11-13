use std::collections::HashMap;

use reearth_flow_geometry::algorithm::centroid::Centroid;
use reearth_flow_geometry::algorithm::normal_3d::compute_normal_3d_from_coords;
use reearth_flow_geometry::algorithm::rotate::query::RotateQuery3D;
use reearth_flow_geometry::algorithm::rotate::rotate_3d::Rotate3D;
use reearth_flow_geometry::types::csg::CSGChild;
use reearth_flow_geometry::types::geometry::Geometry3D;
use reearth_flow_geometry::types::line_string::LineString3D;
use reearth_flow_geometry::types::point::Point3D;
use reearth_flow_geometry::types::polygon::Polygon3D;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::GeometryValue;
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct ThreeDimensionPlanarityRotatorFactory;

impl ProcessorFactory for ThreeDimensionPlanarityRotatorFactory {
    fn name(&self) -> &str {
        "ThreeDimensionPlanarityRotator"
    }

    fn description(&self) -> &str {
        "Rotates a single or a set of 2D geometries in 3D space to align them horizontally."
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
        vec![DEFAULT_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(ThreeDimensionPlanarityRotator))
    }
}

#[derive(Debug, Clone)]
pub struct ThreeDimensionPlanarityRotator;

impl Processor for ThreeDimensionPlanarityRotator {
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
            GeometryValue::FlowGeometry3D(geometry) => {
                for rotated_geometry in rotate_geometry(geometry) {
                    match rotated_geometry {
                        RotatedGeometry::Success(rotated_geometry) => {
                            let mut feature = feature.clone();
                            feature.geometry.value =
                                GeometryValue::FlowGeometry3D(rotated_geometry);
                            fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                        }
                        RotatedGeometry::Failure(geometry) => {
                            let mut feature = feature.clone();
                            feature.geometry.value = GeometryValue::FlowGeometry3D(geometry);
                            fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
                        }
                    }
                }
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "ThreeDimensionPlanarityRotator"
    }
}

enum RotatedGeometry {
    Success(Geometry3D<f64>),
    Failure(Geometry3D<f64>),
}

impl RotatedGeometry {
    fn from_original(geometry: Geometry3D<f64>) -> Self {
        match rotate_single_geometry(&geometry) {
            Some(rotated_geometry) => RotatedGeometry::Success(rotated_geometry),
            None => RotatedGeometry::Failure(geometry),
        }
    }
}

fn rotate_geometry(geometry: &Geometry3D<f64>) -> Vec<RotatedGeometry> {
    match geometry {
        Geometry3D::Point(_) => vec![RotatedGeometry::from_original(geometry.clone())],
        Geometry3D::Line(_) => vec![RotatedGeometry::from_original(geometry.clone())],
        Geometry3D::LineString(_) => vec![RotatedGeometry::from_original(geometry.clone())],
        Geometry3D::Polygon(_) => vec![RotatedGeometry::from_original(geometry.clone())],
        Geometry3D::Rect(_) => vec![RotatedGeometry::from_original(geometry.clone())],
        Geometry3D::Triangle(_) => vec![RotatedGeometry::from_original(geometry.clone())],
        Geometry3D::TriangularMesh(_) => vec![RotatedGeometry::from_original(geometry.clone())],
        Geometry3D::Solid(solid) => solid
            .all_faces()
            .iter()
            .map(|f| {
                let coords = LineString3D::from(f.0.clone());
                let geometry = Geometry3D::Polygon(Polygon3D::new(coords, vec![]));
                RotatedGeometry::from_original(geometry)
            })
            .collect(),
        Geometry3D::MultiPoint(multi_point) => multi_point
            .iter()
            .map(|point| RotatedGeometry::from_original(Geometry3D::Point(*point)))
            .collect(),
        Geometry3D::MultiPolygon(multi_polygon) => multi_polygon
            .iter()
            .map(|polygon| RotatedGeometry::from_original(Geometry3D::Polygon(polygon.clone())))
            .collect(),
        Geometry3D::MultiLineString(multi_line_string) => multi_line_string
            .iter()
            .map(|line_string| {
                RotatedGeometry::from_original(Geometry3D::LineString(line_string.clone()))
            })
            .collect(),
        Geometry3D::GeometryCollection(geometry_collection) => geometry_collection
            .iter()
            .flat_map(rotate_geometry)
            .collect(),
        Geometry3D::CSG(csg) => {
            let mut left = match csg.left() {
                CSGChild::CSG(csg) => rotate_geometry(&Geometry3D::CSG(csg.clone())),
                CSGChild::Solid(solid) => rotate_geometry(&Geometry3D::Solid(solid.clone())),
            };
            let mut right = match csg.right() {
                CSGChild::CSG(csg) => rotate_geometry(&Geometry3D::CSG(csg.clone())),
                CSGChild::Solid(solid) => rotate_geometry(&Geometry3D::Solid(solid.clone())),
            };
            left.append(&mut right);
            left
        }
    }
}

fn rotate_single_geometry(geometry: &Geometry3D<f64>) -> Option<Geometry3D<f64>> {
    let centoroid = geometry.centroid()?;
    let surface_coords = match geometry {
        Geometry3D::Point(point) => vec![point.0],
        Geometry3D::Line(line) => vec![line.start, line.end],
        Geometry3D::LineString(line_string) => line_string.coords().cloned().collect(),
        Geometry3D::Polygon(polygon) => polygon.exterior().coords().cloned().collect(),
        Geometry3D::Rect(rect) => rect.to_polygon().exterior().coords().cloned().collect(),
        Geometry3D::Triangle(triangle) => triangle
            .to_polygon()
            .exterior()
            .into_iter()
            .cloned()
            .collect(),
        // other geometries has multiple surfaces
        Geometry3D::TriangularMesh(_) => return None,
        Geometry3D::Solid(_) => return None,
        Geometry3D::MultiPoint(_) => return None,
        Geometry3D::MultiPolygon(_) => return None,
        Geometry3D::MultiLineString(_) => return None,
        Geometry3D::GeometryCollection(_) => return None,
        Geometry3D::CSG(_) => return None,
    };

    let surface_points = surface_coords
        .into_iter()
        .map(Into::into)
        .collect::<Vec<Point3D<f64>>>();

    if surface_points.len() < 2 {
        return None;
    }

    let from_vector = if surface_points.len() == 2 {
        let vector = surface_points[0] - surface_points[1];
        Point3D::new(vector.y(), -vector.x(), vector.z())
    } else {
        compute_normal_3d_from_coords(surface_points, centoroid, true, 1e-2)?
    };

    let to_vector = Point3D::new(0.0, 0.0, 1.0);

    let rotate_query = RotateQuery3D::from_vectors_geometry(from_vector, to_vector)?;

    Some(geometry.rotate_3d(rotate_query, Some(centoroid)))
}
