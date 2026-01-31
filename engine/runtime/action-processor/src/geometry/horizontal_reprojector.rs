use std::{cell::RefCell, collections::HashMap, sync::Arc};

use nusamai_projection::crs::EpsgCode;
use proj::Proj;

// Thread-local cache for PROJ transformations.
// Each thread maintains its own cache to ensure thread-safety without requiring
// unsafe Send/Sync implementations on types containing proj::Proj.
thread_local! {
    static PROJ_CACHE: RefCell<HashMap<(String, String), Proj>> = RefCell::new(HashMap::new());
}
use reearth_flow_geometry::types::{
    geometry::{Geometry2D, Geometry3D},
    line::Line,
    line_string::{LineString2D, LineString3D},
    multi_line_string::{MultiLineString2D, MultiLineString3D},
    multi_point::{MultiPoint2D, MultiPoint3D},
    multi_polygon::{MultiPolygon2D, MultiPolygon3D},
    point::{Point2D, Point3D},
    polygon::{Polygon2D, Polygon3D},
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

/// Transform a 2D point using proj
fn transform_point_2d(point: &Point2D<f64>, proj: &proj::Proj) -> Result<Point2D<f64>, BoxedError> {
    let (x, y) = proj.convert((point.x(), point.y()))?;
    Ok(Point2D::from([x, y]))
}

/// Transform a 3D point using proj
/// Note: PROJ transforms the X/Y coordinates, Z is passed through unchanged
fn transform_point_3d(point: &Point3D<f64>, proj: &proj::Proj) -> Result<Point3D<f64>, BoxedError> {
    let (x, y) = proj.convert((point.x(), point.y()))?;
    // Z coordinate is not transformed by horizontal reprojection
    Ok(Point3D::from([x, y, point.z()]))
}

/// Transform a 2D geometry using proj
fn transform_geometry_2d(
    geom: &Geometry2D<f64>,
    proj: &proj::Proj,
) -> Result<Geometry2D<f64>, BoxedError> {
    match geom {
        Geometry2D::Point(p) => Ok(Geometry2D::Point(transform_point_2d(p, proj)?)),
        Geometry2D::Line(line) => {
            let start = transform_point_2d(&line.start_point(), proj)?;
            let end = transform_point_2d(&line.end_point(), proj)?;
            Ok(Geometry2D::Line(Line::new(start.0, end.0)))
        }
        Geometry2D::LineString(ls) => {
            let coords: Result<Vec<_>, BoxedError> = ls
                .coords()
                .map(|c| {
                    let p = Point2D::from([c.x, c.y]);
                    transform_point_2d(&p, proj).map(|tp| tp.0)
                })
                .collect();
            Ok(Geometry2D::LineString(LineString2D::new(coords?)))
        }
        Geometry2D::Polygon(poly) => {
            let exterior_coords: Result<Vec<_>, BoxedError> = poly
                .exterior()
                .coords()
                .map(|c| {
                    let p = Point2D::from([c.x, c.y]);
                    transform_point_2d(&p, proj).map(|tp| tp.0)
                })
                .collect();
            let exterior = LineString2D::new(exterior_coords?);

            let interiors: Result<Vec<_>, BoxedError> = poly
                .interiors()
                .iter()
                .map(|interior| {
                    let coords: Result<Vec<_>, BoxedError> = interior
                        .coords()
                        .map(|c| {
                            let p = Point2D::from([c.x, c.y]);
                            transform_point_2d(&p, proj).map(|tp| tp.0)
                        })
                        .collect();
                    Ok(LineString2D::new(coords?))
                })
                .collect();

            Ok(Geometry2D::Polygon(Polygon2D::new(exterior, interiors?)))
        }
        Geometry2D::MultiPoint(mp) => {
            let points: Result<Vec<_>, BoxedError> =
                mp.iter().map(|p| transform_point_2d(p, proj)).collect();
            Ok(Geometry2D::MultiPoint(MultiPoint2D::new(points?)))
        }
        Geometry2D::MultiLineString(mls) => {
            let line_strings: Result<Vec<_>, BoxedError> = mls
                .iter()
                .map(|ls| {
                    let coords: Result<Vec<_>, BoxedError> = ls
                        .coords()
                        .map(|c| {
                            let p = Point2D::from([c.x, c.y]);
                            transform_point_2d(&p, proj).map(|tp| tp.0)
                        })
                        .collect();
                    Ok(LineString2D::new(coords?))
                })
                .collect();
            Ok(Geometry2D::MultiLineString(MultiLineString2D::new(
                line_strings?,
            )))
        }
        Geometry2D::MultiPolygon(mpoly) => {
            let polygons: Result<Vec<_>, BoxedError> = mpoly
                .iter()
                .map(|poly| {
                    let exterior_coords: Result<Vec<_>, BoxedError> = poly
                        .exterior()
                        .coords()
                        .map(|c| {
                            let p = Point2D::from([c.x, c.y]);
                            transform_point_2d(&p, proj).map(|tp| tp.0)
                        })
                        .collect();
                    let exterior = LineString2D::new(exterior_coords?);

                    let interiors: Result<Vec<_>, BoxedError> = poly
                        .interiors()
                        .iter()
                        .map(|interior| {
                            let coords: Result<Vec<_>, BoxedError> = interior
                                .coords()
                                .map(|c| {
                                    let p = Point2D::from([c.x, c.y]);
                                    transform_point_2d(&p, proj).map(|tp| tp.0)
                                })
                                .collect();
                            Ok(LineString2D::new(coords?))
                        })
                        .collect();

                    Ok(Polygon2D::new(exterior, interiors?))
                })
                .collect();
            Ok(Geometry2D::MultiPolygon(MultiPolygon2D::new(polygons?)))
        }
        Geometry2D::GeometryCollection(_) => {
            Err("GeometryCollection transformation not yet implemented".into())
        }
        _ => Err("Unsupported 2D geometry type for transformation".into()),
    }
}

/// Transform a 3D geometry using proj
fn transform_geometry_3d(
    geom: &Geometry3D<f64>,
    proj: &proj::Proj,
) -> Result<Geometry3D<f64>, BoxedError> {
    match geom {
        Geometry3D::Point(p) => Ok(Geometry3D::Point(transform_point_3d(p, proj)?)),
        Geometry3D::Line(line) => {
            let start = transform_point_3d(&line.start_point(), proj)?;
            let end = transform_point_3d(&line.end_point(), proj)?;
            Ok(Geometry3D::Line(Line::new_(start.0, end.0)))
        }
        Geometry3D::LineString(ls) => {
            let coords: Result<Vec<_>, BoxedError> = ls
                .coords()
                .map(|c| {
                    let p = Point3D::from([c.x, c.y, c.z]);
                    transform_point_3d(&p, proj).map(|tp| tp.0)
                })
                .collect();
            Ok(Geometry3D::LineString(LineString3D::new(coords?)))
        }
        Geometry3D::Polygon(poly) => {
            let exterior_coords: Result<Vec<_>, BoxedError> = poly
                .exterior()
                .coords()
                .map(|c| {
                    let p = Point3D::from([c.x, c.y, c.z]);
                    transform_point_3d(&p, proj).map(|tp| tp.0)
                })
                .collect();
            let exterior = LineString3D::new(exterior_coords?);

            let interiors: Result<Vec<_>, BoxedError> = poly
                .interiors()
                .iter()
                .map(|interior| {
                    let coords: Result<Vec<_>, BoxedError> = interior
                        .coords()
                        .map(|c| {
                            let p = Point3D::from([c.x, c.y, c.z]);
                            transform_point_3d(&p, proj).map(|tp| tp.0)
                        })
                        .collect();
                    Ok(LineString3D::new(coords?))
                })
                .collect();

            Ok(Geometry3D::Polygon(Polygon3D::new(exterior, interiors?)))
        }
        Geometry3D::MultiPoint(mp) => {
            let points: Result<Vec<_>, BoxedError> =
                mp.iter().map(|p| transform_point_3d(p, proj)).collect();
            Ok(Geometry3D::MultiPoint(MultiPoint3D::new(points?)))
        }
        Geometry3D::MultiLineString(mls) => {
            let line_strings: Result<Vec<_>, BoxedError> = mls
                .iter()
                .map(|ls| {
                    let coords: Result<Vec<_>, BoxedError> = ls
                        .coords()
                        .map(|c| {
                            let p = Point3D::from([c.x, c.y, c.z]);
                            transform_point_3d(&p, proj).map(|tp| tp.0)
                        })
                        .collect();
                    Ok(LineString3D::new(coords?))
                })
                .collect();
            Ok(Geometry3D::MultiLineString(MultiLineString3D::new(
                line_strings?,
            )))
        }
        Geometry3D::MultiPolygon(mpoly) => {
            let polygons: Result<Vec<_>, BoxedError> = mpoly
                .iter()
                .map(|poly| {
                    let exterior_coords: Result<Vec<_>, BoxedError> = poly
                        .exterior()
                        .coords()
                        .map(|c| {
                            let p = Point3D::from([c.x, c.y, c.z]);
                            transform_point_3d(&p, proj).map(|tp| tp.0)
                        })
                        .collect();
                    let exterior = LineString3D::new(exterior_coords?);

                    let interiors: Result<Vec<_>, BoxedError> = poly
                        .interiors()
                        .iter()
                        .map(|interior| {
                            let coords: Result<Vec<_>, BoxedError> = interior
                                .coords()
                                .map(|c| {
                                    let p = Point3D::from([c.x, c.y, c.z]);
                                    transform_point_3d(&p, proj).map(|tp| tp.0)
                                })
                                .collect();
                            Ok(LineString3D::new(coords?))
                        })
                        .collect();

                    Ok(Polygon3D::new(exterior, interiors?))
                })
                .collect();
            Ok(Geometry3D::MultiPolygon(MultiPolygon3D::new(polygons?)))
        }
        Geometry3D::GeometryCollection(_) => {
            Err("GeometryCollection transformation not yet implemented".into())
        }
        _ => Err("Unsupported 3D geometry type for transformation".into()),
    }
}

#[derive(Debug, Clone, Default)]
pub struct HorizontalReprojectorFactory;

impl ProcessorFactory for HorizontalReprojectorFactory {
    fn name(&self) -> &str {
        "HorizontalReprojector"
    }

    fn description(&self) -> &str {
        "Reproject Geometry to Different Coordinate System"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(HorizontalReprojectorParam))
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
        let params: HorizontalReprojectorParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::HorizontalReprojectorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::HorizontalReprojectorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::HorizontalReprojectorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);

        // Compile source EPSG expression if provided
        let source_epsg_ast = if let Some(ref source_expr) = params.source_epsg_code {
            Some(expr_engine.compile(source_expr.as_ref()).map_err(|e| {
                GeometryProcessorError::HorizontalReprojectorFactory(format!(
                    "Failed to compile source EPSG expression: {e:?}"
                ))
            })?)
        } else {
            None
        };

        // Compile target EPSG expression
        let target_epsg_ast = expr_engine
            .compile(params.target_epsg_code.as_ref())
            .map_err(|e| {
                GeometryProcessorError::HorizontalReprojectorFactory(format!(
                    "Failed to compile target EPSG expression: {e:?}"
                ))
            })?;

        Ok(Box::new(HorizontalReprojector {
            global_params: with,
            source_epsg_ast,
            target_epsg_ast,
        }))
    }
}

/// # Horizontal Reprojector Parameters
/// Configure the source and target coordinate systems for geometry reprojection
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HorizontalReprojectorParam {
    /// # Source EPSG Code
    /// Source coordinate system EPSG code expression. If not provided, will use the EPSG code from the geometry.
    /// This is optional to maintain backward compatibility but recommended to be explicit.
    /// Can be a constant value (e.g., "4326") or an expression referencing feature attributes.
    #[serde(default)]
    source_epsg_code: Option<Expr>,

    /// # Target EPSG Code
    /// Target coordinate system EPSG code expression for the reprojection.
    /// Can be a constant value (e.g., "4326" for WGS84, "2193" for NZTM2000, "3857" for Web Mercator)
    /// or an expression referencing feature attributes.
    target_epsg_code: Expr,
}

#[derive(Debug, Clone)]
pub struct HorizontalReprojector {
    global_params: Option<HashMap<String, serde_json::Value>>,
    source_epsg_ast: Option<rhai::AST>,
    target_epsg_ast: rhai::AST,
}

/// Helper function to get or create a cached Proj transformation.
/// Uses thread-local storage to ensure thread-safety.
fn get_or_create_proj(from_crs: &str, to_crs: &str) -> Result<(), BoxedError> {
    use std::collections::hash_map::Entry;
    PROJ_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let key = (from_crs.to_string(), to_crs.to_string());
        if let Entry::Vacant(e) = cache.entry(key) {
            let proj = Proj::new_known_crs(from_crs, to_crs, None)?;
            e.insert(proj);
        }
        Ok(())
    })
}

/// Helper function to use a cached Proj transformation.
/// The callback receives a reference to the Proj instance.
fn with_proj<F, R>(from_crs: &str, to_crs: &str, f: F) -> Result<R, BoxedError>
where
    F: FnOnce(&Proj) -> Result<R, BoxedError>,
{
    PROJ_CACHE.with(|cache| {
        let cache = cache.borrow();
        let key = (from_crs.to_string(), to_crs.to_string());
        let proj = cache.get(&key).ok_or_else(|| {
            GeometryProcessorError::HorizontalReprojector(
                "Proj not found in cache - this should not happen".to_string(),
            )
        })?;
        f(proj)
    })
}

impl Processor for HorizontalReprojector {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);

        // Evaluate source EPSG expression if provided
        let source_epsg_from_expr: Option<EpsgCode> = if let Some(ref ast) = self.source_epsg_ast {
            let value: i64 = scope.eval_ast(ast).map_err(|e| {
                GeometryProcessorError::HorizontalReprojector(format!(
                    "Failed to evaluate source EPSG expression: {e}"
                ))
            })?;
            Some(value as EpsgCode)
        } else {
            None
        };

        // Determine source EPSG: from expression, or from geometry
        let source_epsg = source_epsg_from_expr.or(geometry.epsg).ok_or_else(|| {
            GeometryProcessorError::HorizontalReprojector(
                "Source EPSG code not specified and geometry has no EPSG information. \
                Either set sourceEpsgCode parameter or ensure input geometries have EPSG codes."
                    .to_string(),
            )
        })?;

        // Evaluate target EPSG expression
        let target_epsg: i64 = scope.eval_ast(&self.target_epsg_ast).map_err(|e| {
            GeometryProcessorError::HorizontalReprojector(format!(
                "Failed to evaluate target EPSG expression: {e}"
            ))
        })?;
        let target_epsg = target_epsg as EpsgCode;

        // Get or create the projection in thread-local cache
        let from_crs = format!("EPSG:{source_epsg}");
        let to_crs = format!("EPSG:{target_epsg}");
        get_or_create_proj(&from_crs, &to_crs)?;

        match &geometry.value {
            GeometryValue::FlowGeometry2D(geom) => {
                let transformed =
                    with_proj(&from_crs, &to_crs, |proj| transform_geometry_2d(geom, proj))?;
                let mut feature = feature.clone();
                feature.geometry_mut().value = GeometryValue::FlowGeometry2D(transformed);
                feature.geometry_mut().epsg = Some(target_epsg);
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geom) => {
                let transformed =
                    with_proj(&from_crs, &to_crs, |proj| transform_geometry_3d(geom, proj))?;
                let mut feature = feature.clone();
                feature.geometry_mut().value = GeometryValue::FlowGeometry3D(transformed);
                feature.geometry_mut().epsg = Some(target_epsg);
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::CityGmlGeometry(ref geos) => {
                let mut feature = feature.clone();
                let mut transformed_geos = geos.clone();
                with_proj(&from_crs, &to_crs, |proj| {
                    transformed_geos
                        .transform_horizontal(|x, y| {
                            proj.convert((x, y)).map_err(|e| {
                                GeometryProcessorError::HorizontalReprojector(e.to_string())
                            })
                        })
                        .map_err(|e: GeometryProcessorError| -> BoxedError { e.into() })
                })?;
                feature.geometry_mut().value = GeometryValue::CityGmlGeometry(transformed_geos);
                feature.geometry_mut().epsg = Some(target_epsg);
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()))
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "HorizontalReprojector"
    }
}
