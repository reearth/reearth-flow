use std::{collections::HashMap, sync::Arc};

use reearth_flow_geometry::types::{
    coordinate::Coordinate3D,
    geometry::{Geometry2D, Geometry3D},
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Expr, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct ThreeDimensionForcerFactory;

impl ProcessorFactory for ThreeDimensionForcerFactory {
    fn name(&self) -> &str {
        "ThreeDimensionForcer"
    }

    fn description(&self) -> &str {
        "Convert 2D Geometry to 3D by Adding Z-Coordinates"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ThreeDimensionForcerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: ThreeDimensionForcerParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::ThreeDimensionForcerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::ThreeDimensionForcerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            ThreeDimensionForcerParam::default()
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let elevation_ast = if let Some(ref elevation_expr) = params.elevation {
            Some(expr_engine.compile(elevation_expr.as_ref()).map_err(|e| {
                GeometryProcessorError::ThreeDimensionForcerFactory(format!(
                    "Failed to compile elevation expression '{}': {:?}",
                    elevation_expr.as_ref(),
                    e
                ))
            })?)
        } else {
            None
        };

        Ok(Box::new(ThreeDimensionForcer {
            global_params: with,
            elevation: elevation_ast,
            preserve_existing_z: params.preserve_existing_z,
        }))
    }
}

/// # ThreeDimensionForcer Parameters
/// Configure how to convert 2D geometries to 3D by adding Z-coordinates
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct ThreeDimensionForcerParam {
    /// # Elevation
    /// The Z-coordinate (elevation) value to add to all points. Can be a constant value or an expression. Defaults to 0.0 if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation: Option<Expr>,
    /// # Preserve Existing Z Values
    /// If true, geometries that are already 3D will pass through unchanged. If false, existing Z values will be replaced with the specified elevation. Defaults to false.
    #[serde(default)]
    pub preserve_existing_z: bool,
}

#[derive(Debug, Clone)]
pub struct ThreeDimensionForcer {
    global_params: Option<HashMap<String, serde_json::Value>>,
    elevation: Option<rhai::AST>,
    preserve_existing_z: bool,
}

impl Processor for ThreeDimensionForcer {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            return Ok(());
        }

        // Calculate the elevation value to use
        let elevation_value = if let Some(ref elevation_ast) = self.elevation {
            let scope = feature.new_scope(expr_engine.clone(), &self.global_params);
            // Try to evaluate as f64 first, if that fails try i64 and convert
            match scope.eval_ast::<f64>(elevation_ast) {
                Ok(val) => val,
                Err(_) => {
                    // If f64 evaluation fails, try i64 and convert to f64
                    scope
                        .eval_ast::<i64>(elevation_ast)
                        .map(|i| i as f64)
                        .map_err(|e| {
                            GeometryProcessorError::ThreeDimensionForcer(format!(
                                "Failed to evaluate elevation expression: {e:?}"
                            ))
                        })?
                }
            }
        } else {
            0.0
        };

        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geos) => {
                if self.preserve_existing_z {
                    // Pass through unchanged if we're preserving existing Z values
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
                } else {
                    // Convert to 2D then back to 3D with the new elevation
                    let value_2d: Geometry2D = geos.clone().into();
                    let value_3d = convert_2d_to_3d(value_2d, elevation_value);
                    let mut new_geometry = geometry.clone();
                    new_geometry.value = GeometryValue::FlowGeometry3D(value_3d);
                    let mut new_feature = feature.clone();
                    new_feature.geometry = new_geometry;
                    fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
                }
            }
            GeometryValue::FlowGeometry2D(geos) => {
                let value_3d = convert_2d_to_3d(geos.clone(), elevation_value);
                let mut new_geometry = geometry.clone();
                new_geometry.value = GeometryValue::FlowGeometry3D(value_3d);
                let mut new_feature = feature.clone();
                new_feature.geometry = new_geometry;
                fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::CityGmlGeometry(gml) => {
                if self.preserve_existing_z {
                    // CityGML is already 3D, pass through unchanged
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
                } else {
                    // Convert to 2D then back to 3D with the new elevation
                    let value_2d: Geometry2D = gml.clone().into();
                    let value_3d = convert_2d_to_3d(value_2d, elevation_value);
                    let mut new_geometry = geometry.clone();
                    new_geometry.value = GeometryValue::FlowGeometry3D(value_3d);
                    let mut new_feature = feature.clone();
                    new_feature.geometry = new_geometry;
                    fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
                }
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "ThreeDimensionForcer"
    }
}

/// Convert a 2D geometry to 3D by adding the specified Z coordinate to all points
fn convert_2d_to_3d(geom: Geometry2D, z: f64) -> Geometry3D {
    use reearth_flow_geometry::types::{
        geometry::Geometry3D, line::Line, line_string::LineString,
        multi_line_string::MultiLineString, multi_point::MultiPoint, multi_polygon::MultiPolygon,
        point::Point, polygon::Polygon, rect::Rect, triangle::Triangle,
    };

    match geom {
        Geometry2D::Point(p) => Geometry3D::Point(Point(Coordinate3D::new__(p.0.x, p.0.y, z))),
        Geometry2D::Line(l) => Geometry3D::Line(Line {
            start: Coordinate3D::new__(l.start.x, l.start.y, z),
            end: Coordinate3D::new__(l.end.x, l.end.y, z),
        }),
        Geometry2D::LineString(ls) => {
            let coords: Vec<Coordinate3D<f64>> =
                ls.0.into_iter()
                    .map(|c| Coordinate3D::new__(c.x, c.y, z))
                    .collect();
            Geometry3D::LineString(LineString(coords))
        }
        Geometry2D::Polygon(poly) => {
            let (exterior, interiors) = poly.into_inner();
            let exterior_coords: Vec<Coordinate3D<f64>> = exterior
                .0
                .into_iter()
                .map(|c| Coordinate3D::new__(c.x, c.y, z))
                .collect();
            let interior_coords: Vec<LineString<f64, f64>> = interiors
                .into_iter()
                .map(|interior| {
                    let coords: Vec<Coordinate3D<f64>> = interior
                        .0
                        .into_iter()
                        .map(|c| Coordinate3D::new__(c.x, c.y, z))
                        .collect();
                    LineString(coords)
                })
                .collect();
            Geometry3D::Polygon(Polygon::new(LineString(exterior_coords), interior_coords))
        }
        Geometry2D::MultiPoint(mp) => {
            let points: Vec<Point<f64, f64>> =
                mp.0.into_iter()
                    .map(|p| Point(Coordinate3D::new__(p.0.x, p.0.y, z)))
                    .collect();
            Geometry3D::MultiPoint(MultiPoint(points))
        }
        Geometry2D::MultiLineString(mls) => {
            let line_strings: Vec<LineString<f64, f64>> = mls
                .0
                .into_iter()
                .map(|ls| {
                    let coords: Vec<Coordinate3D<f64>> =
                        ls.0.into_iter()
                            .map(|c| Coordinate3D::new__(c.x, c.y, z))
                            .collect();
                    LineString(coords)
                })
                .collect();
            Geometry3D::MultiLineString(MultiLineString(line_strings))
        }
        Geometry2D::MultiPolygon(mp) => {
            let polygons: Vec<Polygon<f64, f64>> =
                mp.0.into_iter()
                    .map(|poly| {
                        let (exterior, interiors) = poly.into_inner();
                        let exterior_coords: Vec<Coordinate3D<f64>> = exterior
                            .0
                            .into_iter()
                            .map(|c| Coordinate3D::new__(c.x, c.y, z))
                            .collect();
                        let interior_coords: Vec<LineString<f64, f64>> = interiors
                            .into_iter()
                            .map(|interior| {
                                let coords: Vec<Coordinate3D<f64>> = interior
                                    .0
                                    .into_iter()
                                    .map(|c| Coordinate3D::new__(c.x, c.y, z))
                                    .collect();
                                LineString(coords)
                            })
                            .collect();
                        Polygon::new(LineString(exterior_coords), interior_coords)
                    })
                    .collect();
            Geometry3D::MultiPolygon(MultiPolygon(polygons))
        }
        Geometry2D::Rect(rect) => {
            let min = rect.min();
            let max = rect.max();
            Geometry3D::Rect(Rect::new(
                Coordinate3D::new__(min.x, min.y, z),
                Coordinate3D::new__(max.x, max.y, z),
            ))
        }
        Geometry2D::Triangle(tri) => {
            let [c1, c2, c3] = tri.to_array();
            Geometry3D::Triangle(Triangle::new(
                Coordinate3D::new__(c1.x, c1.y, z),
                Coordinate3D::new__(c2.x, c2.y, z),
                Coordinate3D::new__(c3.x, c3.y, z),
            ))
        }
        Geometry2D::Solid(_solid) => {
            // Solids in 2D don't really make sense, return empty collection
            Geometry3D::GeometryCollection(vec![])
        }
        Geometry2D::GeometryCollection(gc) => {
            let geometries: Vec<Geometry3D> =
                gc.into_iter().map(|g| convert_2d_to_3d(g, z)).collect();
            Geometry3D::GeometryCollection(geometries)
        }
        Geometry2D::CSG(_) => {
            // CSG in 2D doesn't exist, unreachable
            Geometry3D::GeometryCollection(vec![])
        }
        Geometry2D::TriangularMesh(_) => {
            // TriangularMesh in 2D doesn't exist, unreachable
            Geometry3D::GeometryCollection(vec![])
        }
    }
}
