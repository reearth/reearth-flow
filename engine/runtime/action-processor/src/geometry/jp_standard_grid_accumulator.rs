use std::collections::HashMap;

use reearth_flow_geometry::algorithm::bool_ops::BooleanOps;
use reearth_flow_geometry::algorithm::bounding_rect::BoundingRect;
use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::geometry::{Geometry, Geometry2D};
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::Polygon2D;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::jpmesh::{JPMeshCode, JPMeshType};
use reearth_flow_types::{Attribute, AttributeValue, GeometryValue};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct JPStandardGridAccumulatorFactory;

impl ProcessorFactory for JPStandardGridAccumulatorFactory {
    fn name(&self) -> &str {
        "JPStandardGridAccumulator"
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
        Ok(Box::new(JPStandardGridAccumulator))
    }
}

#[derive(Debug, Clone)]
pub struct JPStandardGridAccumulator;

impl Processor for JPStandardGridAccumulator {
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
            GeometryValue::FlowGeometry2D(geometry) => {
                let bounds = if let Some(bounds) = geometry.bounding_rect() {
                    bounds
                } else {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                };

                let meshes_1km = JPMeshCode::from_inside_bounds(bounds, JPMeshType::Mesh1km);

                for meshcode in meshes_1km {
                    let binded_geometry = if let Some(binded_geometry) =
                        self.bind_geometry_into_mesh_2d(geometry, meshcode.clone())
                    {
                        binded_geometry
                    } else {
                        continue;
                    };

                    let mut attributes = feature.attributes.clone();
                    attributes.insert(
                        Attribute::new("meshcode"),
                        AttributeValue::String(meshcode.to_number().to_string()),
                    );

                    let mut new_feature = feature.clone();
                    new_feature.geometry.value = GeometryValue::FlowGeometry2D(binded_geometry);
                    new_feature.attributes = attributes;

                    fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
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
        "JPStandardGridAccumulator"
    }
}

impl JPStandardGridAccumulator {
    fn bind_geometry_into_mesh_2d(
        &self,
        geometry: &Geometry2D,
        mesh: JPMeshCode,
    ) -> Option<Geometry2D> {
        let bounds = mesh.bounds();

        let bounds_polygon = Polygon2D::new(
            LineString2D::new(vec![
                Coordinate2D::new_(bounds.min().x, bounds.min().y),
                Coordinate2D::new_(bounds.max().x, bounds.min().y),
                Coordinate2D::new_(bounds.max().x, bounds.max().y),
                Coordinate2D::new_(bounds.min().x, bounds.max().y),
                Coordinate2D::new_(bounds.min().x, bounds.min().y),
            ]),
            vec![],
        );

        let bind_geometry = match geometry {
            Geometry::Point(_) => geometry.clone(),
            Geometry::LineString(line_string) => {
                let multi_line_string =
                    reearth_flow_geometry::types::multi_line_string::MultiLineString2D::new(vec![
                        line_string.clone(),
                    ]);

                let clipped = bounds_polygon.clip(&multi_line_string, false);

                if clipped.0.is_empty() {
                    return None;
                } else if clipped.0.len() == 1 {
                    Geometry::LineString(clipped.0[0].clone())
                } else {
                    Geometry::MultiLineString(clipped)
                }
            }

            Geometry::MultiLineString(multi_line_string) => {
                let clipped = bounds_polygon.clip(multi_line_string, false);

                if clipped.0.is_empty() {
                    return None;
                } else if clipped.0.len() == 1 {
                    Geometry::LineString(clipped.0[0].clone())
                } else {
                    Geometry::MultiLineString(clipped)
                }
            }

            Geometry::Polygon(polygon) => {
                let intersection = polygon.intersection(&bounds_polygon);

                if intersection.0.is_empty() {
                    return None;
                } else if intersection.0.len() == 1 {
                    Geometry::Polygon(intersection.0[0].clone())
                } else {
                    Geometry::MultiPolygon(intersection)
                }
            }

            Geometry::MultiPolygon(multi_polygon) => {
                let intersection =
                    multi_polygon.intersection(&MultiPolygon2D::new(vec![bounds_polygon]));
                if intersection.0.is_empty() {
                    return None;
                } else if intersection.0.len() == 1 {
                    Geometry::Polygon(intersection.0[0].clone())
                } else {
                    Geometry::MultiPolygon(intersection)
                }
            }

            _ => {
                return None;
            }
        };

        Some(bind_geometry)
    }
}
