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
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Code, CodeType, CompiledCode, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct ThreeDimensionForcerFactory;

impl ProcessorFactory for ThreeDimensionForcerFactory {
    fn name(&self) -> &str {
        "Three Dimension Forcer"
    }

    fn description(&self) -> &str {
        "Adds Z-coordinates to 2D geometries to produce 3D output."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ThreeDimensionForcerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["3d"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: ThreeDimensionForcerParam = if let Some(with) = with {
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

        let elevation = params
            .elevation
            .map(|expr| {
                expr.compile().map_err(|e| {
                    GeometryProcessorError::ThreeDimensionForcerFactory(format!(
                        "Failed to compile elevation expression: {e:?}"
                    ))
                })
            })
            .transpose()?;

        Ok(Box::new(ThreeDimensionForcer {
            elevation,
            preserve_existing_z: params.preserve_existing_z,
        }))
    }
}

/// Geometry that already carries Z is left alone unless the user opts in to
/// replacing it. This matches the standard behaviour of a force-3D operation,
/// where adding a dimension never discards coordinates that are already there.
fn preserve_existing_z_default() -> bool {
    true
}

/// # Three Dimension Forcer Parameters
/// Configure the elevation applied to 2D geometry and how existing Z values are treated.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThreeDimensionForcerParam {
    /// # Elevation
    /// Z-coordinate applied to every point, as a constant or an expression.
    /// Defaults to 0.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation: Option<Code<{ CodeType::FlowExpr as u32 }>>,
    /// # Preserve Existing Z Values
    /// Whether geometry that is already 3D passes through untouched. Defaults to
    /// true, so existing Z is kept. Set it to false to overwrite every Z value
    /// with the elevation.
    #[serde(default = "preserve_existing_z_default")]
    pub preserve_existing_z: bool,
}

impl Default for ThreeDimensionForcerParam {
    fn default() -> Self {
        Self {
            elevation: None,
            preserve_existing_z: preserve_existing_z_default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThreeDimensionForcer {
    elevation: Option<CompiledCode>,
    preserve_existing_z: bool,
}

impl Processor for ThreeDimensionForcer {
    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), FEATURES_PORT.clone()));
            return Ok(());
        }

        let elevation_value = if let Some(ref elevation_ast) = self.elevation {
            elevation_ast
                .eval_float(feature, ctx.variables.clone())
                .map_err(|e| {
                    GeometryProcessorError::ThreeDimensionForcer(format!(
                        "Failed to evaluate elevation expression: {e:?}"
                    ))
                })?
        } else {
            0.0
        };

        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), FEATURES_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geos) => {
                if self.preserve_existing_z {
                    // Pass through unchanged if we're preserving existing Z values
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), FEATURES_PORT.clone()));
                } else {
                    // Convert to 2D then back to 3D with the new elevation
                    let value_2d: Geometry2D = geos.clone().into();
                    let value_3d = convert_2d_to_3d(value_2d, elevation_value);
                    let mut new_geometry = (**geometry).clone();
                    new_geometry.value = GeometryValue::FlowGeometry3D(value_3d);
                    let mut new_feature = feature.clone();
                    new_feature.geometry = Arc::new(new_geometry);
                    fw.send(ctx.new_with_feature_and_port(new_feature, FEATURES_PORT.clone()));
                }
            }
            GeometryValue::FlowGeometry2D(geos) => {
                let value_3d = convert_2d_to_3d(geos.clone(), elevation_value);
                let mut new_geometry = (**geometry).clone();
                new_geometry.value = GeometryValue::FlowGeometry3D(value_3d);
                let mut new_feature = feature.clone();
                new_feature.geometry = Arc::new(new_geometry);
                fw.send(ctx.new_with_feature_and_port(new_feature, FEATURES_PORT.clone()));
            }
            GeometryValue::CityGmlGeometry(gml) => {
                if self.preserve_existing_z {
                    // CityGML is already 3D, pass through unchanged
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), FEATURES_PORT.clone()));
                } else {
                    // Convert to 2D then back to 3D with the new elevation
                    let value_2d: Geometry2D = gml.clone().into();
                    let value_3d = convert_2d_to_3d(value_2d, elevation_value);
                    let mut new_geometry = (**geometry).clone();
                    new_geometry.value = GeometryValue::FlowGeometry3D(value_3d);
                    let mut new_feature = feature.clone();
                    new_feature.geometry = Arc::new(new_geometry);
                    fw.send(ctx.new_with_feature_and_port(new_feature, FEATURES_PORT.clone()));
                }
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    /// Lift 2D geometry into 3D at the elevation expression's value (0.0 when
    /// omitted). A 2D leaf that already lies at an elevation keeps it unless
    /// `preserveExistingZ` is false. 3D geometry passes through untouched unless
    /// `preserveExistingZ` is false, in which case it is flattened and lifted
    /// like 2D geometry. A geometry that cannot be flattened is an error.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let elevation = if let Some(ref elevation_ast) = self.elevation {
            elevation_ast
                .eval_float(feature, ctx.variables.clone())
                .map_err(|e| {
                    GeometryProcessorError::ThreeDimensionForcer(format!(
                        "Failed to evaluate elevation expression: {e:?}"
                    ))
                })?
        } else {
            0.0
        };
        let lifted = force_3d(
            feature.geometry.as_ref().clone(),
            elevation,
            self.preserve_existing_z,
        )
        .map_err(|e| GeometryProcessorError::ThreeDimensionForcer(e.to_string()))?;
        let mut forced = feature.clone();
        forced.set_geometry(lifted);
        fw.send(ctx.new_with_feature_and_port(forced, FEATURES_PORT.clone()));
        Ok(())
    }

    #[cfg(feature = "new-geometry")]
    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Three Dimension Forcer"
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

/// `geometry` in 3D, as [`ThreeDimensionForcer::process`] documents.
#[cfg(feature = "new-geometry")]
fn force_3d(
    geometry: reearth_flow_geometry::Geometry,
    elevation: f64,
    preserve_existing_z: bool,
) -> Result<reearth_flow_geometry::Geometry, reearth_flow_geometry::error::Error> {
    use reearth_flow_geometry::ops::ForceTwoDimension;
    use reearth_flow_geometry::{Geometry, GeometryCollection};

    Ok(match geometry {
        Geometry::None => Geometry::None,
        Geometry::Euclidean2D(g) => {
            Geometry::Euclidean3D(lift_2d(g, elevation, preserve_existing_z)?)
        }
        Geometry::Euclidean3D(g) if preserve_existing_z => Geometry::Euclidean3D(g),
        Geometry::Euclidean3D(mut g) => {
            let flat = g
                .force_2d()
                .map_err(|e| reearth_flow_geometry::error::Error::projection(e.to_string()))?;
            Geometry::Euclidean3D(lift_2d(flat, elevation, false)?)
        }
        Geometry::GeometryCollection(c) => Geometry::GeometryCollection(GeometryCollection::new(
            c.members()
                .iter()
                .cloned()
                .map(|m| force_3d(m, elevation, preserve_existing_z))
                .collect::<Result<Vec<_>, reearth_flow_geometry::error::Error>>()?,
        )),
    })
}

/// Lift a 2D geometry leaf by leaf, each to `elevation` or, when
/// `preserve_existing_z` holds, to the elevation it already lies at.
#[cfg(feature = "new-geometry")]
fn lift_2d(
    geometry: reearth_flow_geometry::Euclidean2DGeometry,
    elevation: f64,
    preserve_existing_z: bool,
) -> Result<reearth_flow_geometry::Euclidean3DGeometry, reearth_flow_geometry::error::Error> {
    use reearth_flow_geometry::collection::Collection3D;
    use reearth_flow_geometry::ops::{Elevation, Translate};
    use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry};

    if let Euclidean2DGeometry::Collection(c) = geometry {
        let members = c
            .members()
            .iter()
            .cloned()
            .map(|m| lift_2d(m, elevation, preserve_existing_z))
            .collect::<Result<Vec<_>, reearth_flow_geometry::error::Error>>()?;
        return Ok(Euclidean3DGeometry::Collection(Collection3D::new(members)));
    }
    let own = geometry.elevation();
    let target = match own {
        Some(z) if preserve_existing_z => z,
        _ => elevation,
    };
    // `into_3d` places the leaf at its own elevation, or at 0.0 without one.
    let mut lifted = geometry.into_3d();
    lifted.translate([0.0, 0.0, target - own.unwrap_or(0.0)])?;
    Ok(lifted)
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::point::{Point2D, Point3D};
    use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

    use super::force_3d;

    fn point_2d(position: [f64; 2]) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            CoordinateFrame::Euclidean,
            position,
        )))
    }

    fn point_3d(position: [f64; 3]) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::Euclidean,
            position,
        )))
    }

    #[test]
    fn a_2d_point_is_placed_at_the_elevation() {
        let forced = force_3d(point_2d([1.0, 2.0]), 5.0, true).unwrap();
        assert_eq!(forced, point_3d([1.0, 2.0, 5.0]));
    }

    #[test]
    fn a_3d_point_keeps_its_z_by_default() {
        let forced = force_3d(point_3d([1.0, 2.0, 3.0]), 5.0, true).unwrap();
        assert_eq!(forced, point_3d([1.0, 2.0, 3.0]));
    }

    #[test]
    fn a_3d_point_takes_the_elevation_when_z_is_not_preserved() {
        let forced = force_3d(point_3d([1.0, 2.0, 3.0]), 5.0, false).unwrap();
        assert_eq!(forced, point_3d([1.0, 2.0, 5.0]));
    }
}
