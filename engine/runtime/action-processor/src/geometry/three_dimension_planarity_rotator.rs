use std::collections::HashMap;

use reearth_flow_geometry::algorithm::centroid::Centroid;
use reearth_flow_geometry::algorithm::normal_3d::compute_normal_3d_from_coords;
use reearth_flow_geometry::algorithm::rotate::rotate_3d::Rotate3D;
use reearth_flow_geometry::algorithm::rotate::rotator_3d::Rotator3D;
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_geometry::types::multi_point::{MultiPoint2D, MultiPoint3D};
use reearth_flow_geometry::types::multi_polygon::{MultiPolygon2D, MultiPolygon3D};
use reearth_flow_geometry::types::no_value::NoValue;
use reearth_flow_geometry::types::point::{Point2D, Point3D};
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Feature, GeometryValue};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct ThreeDimensionPlanarityRotatorFactory;

impl ProcessorFactory for ThreeDimensionPlanarityRotatorFactory {
    fn name(&self) -> &str {
        "ThreeDimensionPlanarityRotator"
    }

    fn description(&self) -> &str {
        "Divides the input geometry into Japanese standard (1km) mesh grid."
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
                if let Some(rotated_feature) = self.rotate_to_horizontal(feature, geometry) {
                    println!("Rotated geometry to horizontal.");
                    fw.send(ctx.new_with_feature_and_port(rotated_feature, DEFAULT_PORT.clone()));
                } else {
                    println!("Failed to rotate geometry to horizontal.");
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
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

impl ThreeDimensionPlanarityRotator {
    fn rotate_to_horizontal(
        &self,
        feature: &Feature,
        geometry: &Geometry3D<f64>,
    ) -> Option<Feature> {
        match geometry {
            Geometry3D::Point(point) => self.process_point(feature, point),
            Geometry3D::MultiPoint(multi_point) => self.process_multi_point(feature, multi_point),
            // Geometry3D::LineString(line_string) => self.process_line_string(feature, line_string),
            // Geometry3D::MultiLineString(multi_line_string) => {
            //     self.process_multi_line_string(feature, multi_line_string)
            // }
            Geometry3D::Polygon(polygon) => self.process_polygon(feature, polygon),
            Geometry3D::MultiPolygon(multi_polygon) => {
                self.process_multi_polygon(feature, multi_polygon)
            }
            _ => None,
        }
    }

    fn process_point(&self, feature: &Feature, point: &Point3D<f64>) -> Option<Feature> {
        let point_2d = Point2D::new(point.x(), point.y(), NoValue);
        let mut new_feature = feature.clone();
        new_feature.geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::Point(point_2d));
        Some(new_feature)
    }

    fn process_multi_point(
        &self,
        feature: &Feature,
        multi_point: &MultiPoint3D<f64>,
    ) -> Option<Feature> {
        let points_2d = multi_point
            .0
            .iter()
            .map(|point| Point2D::new(point.x(), point.y(), NoValue))
            .collect();

        let multi_point_2d = MultiPoint2D::new(points_2d);
        let mut new_feature = feature.clone();
        new_feature.geometry.value =
            GeometryValue::FlowGeometry2D(Geometry2D::MultiPoint(multi_point_2d));
        Some(new_feature)
    }

    fn process_polygon(&self, feature: &Feature, polygon: &Polygon3D<f64>) -> Option<Feature> {
        let polygon_2d = rotate_polygon_to_2d(polygon)?;
        let mut new_feature = feature.clone();
        new_feature.geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon_2d));
        Some(new_feature)
    }

    fn process_multi_polygon(
        &self,
        feature: &Feature,
        multi_polygon: &MultiPolygon3D<f64>,
    ) -> Option<Feature> {
        if multi_polygon.0.is_empty() {
            return None;
        }

        let polygons_2d = multi_polygon
            .0
            .iter()
            .filter_map(rotate_polygon_to_2d)
            .collect::<Vec<_>>();

        if polygons_2d.is_empty() {
            return None;
        }

        let multi_polygon_2d = MultiPolygon2D::new(polygons_2d);
        let mut new_feature = feature.clone();
        new_feature.geometry.value =
            GeometryValue::FlowGeometry2D(Geometry2D::MultiPolygon(multi_polygon_2d));
        Some(new_feature)
    }
}

fn rotate_polygon_to_2d(polygon: &Polygon3D<f64>) -> Option<Polygon2D<f64>> {
    let exterior_coords = polygon.exterior().coords().cloned().collect::<Vec<_>>();
    if exterior_coords.is_empty() {
        return None;
    }
    if exterior_coords.len() < 3 {
        return None;
    }

    let centoroid = polygon.centroid()?;

    let from_vector = compute_normal_3d_from_coords(
        exterior_coords[0],
        exterior_coords[1],
        exterior_coords[2],
        centoroid.into(),
        true,
    )?;
    let to_vector = Point3D::new(0.0, 0.0, 1.0);

    let rotator = Rotator3D::from_vectors_geometry(from_vector, to_vector)?;

    let polygon = polygon.rotate_3d(rotator, Some(centoroid.into()));

    Some(polygon.into())
}
