use std::collections::HashMap;

use nusamai_projection::crs::EpsgCode;
use reearth_flow_geometry::types::{
    geometry::{Geometry2D, Geometry3D},
    point::{Point2D, Point3D},
    line_string::{LineString2D, LineString3D},
    polygon::{Polygon2D, Polygon3D},
    multi_point::{MultiPoint2D, MultiPoint3D},
    multi_line_string::{MultiLineString2D, MultiLineString3D},
    multi_polygon::{MultiPolygon2D, MultiPolygon3D},
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::GeometryValue;
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
fn transform_geometry_2d(geom: &Geometry2D<f64>, proj: &proj::Proj) -> Result<Geometry2D<f64>, BoxedError> {
    match geom {
        Geometry2D::Point(p) => Ok(Geometry2D::Point(transform_point_2d(p, proj)?)),
        Geometry2D::LineString(ls) => {
            let coords: Result<Vec<_>, BoxedError> = ls.coords()
                .map(|c| {
                    let p = Point2D::from([c.x, c.y]);
                    transform_point_2d(&p, proj).map(|tp| tp.0)
                })
                .collect();
            Ok(Geometry2D::LineString(LineString2D::new(coords?)))
        }
        Geometry2D::Polygon(poly) => {
            let exterior_coords: Result<Vec<_>, BoxedError> = poly.exterior().coords()
                .map(|c| {
                    let p = Point2D::from([c.x, c.y]);
                    transform_point_2d(&p, proj).map(|tp| tp.0)
                })
                .collect();
            let exterior = LineString2D::new(exterior_coords?);

            let interiors: Result<Vec<_>, BoxedError> = poly.interiors().iter()
                .map(|interior| {
                    let coords: Result<Vec<_>, BoxedError> = interior.coords()
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
            let points: Result<Vec<_>, BoxedError> = mp.iter()
                .map(|p| transform_point_2d(p, proj))
                .collect();
            Ok(Geometry2D::MultiPoint(MultiPoint2D::new(points?)))
        }
        Geometry2D::MultiLineString(mls) => {
            let line_strings: Result<Vec<_>, BoxedError> = mls.iter()
                .map(|ls| {
                    let coords: Result<Vec<_>, BoxedError> = ls.coords()
                        .map(|c| {
                            let p = Point2D::from([c.x, c.y]);
                            transform_point_2d(&p, proj).map(|tp| tp.0)
                        })
                        .collect();
                    Ok(LineString2D::new(coords?))
                })
                .collect();
            Ok(Geometry2D::MultiLineString(MultiLineString2D::new(line_strings?)))
        }
        Geometry2D::MultiPolygon(mpoly) => {
            let polygons: Result<Vec<_>, BoxedError> = mpoly.iter()
                .map(|poly| {
                    let exterior_coords: Result<Vec<_>, BoxedError> = poly.exterior().coords()
                        .map(|c| {
                            let p = Point2D::from([c.x, c.y]);
                            transform_point_2d(&p, proj).map(|tp| tp.0)
                        })
                        .collect();
                    let exterior = LineString2D::new(exterior_coords?);

                    let interiors: Result<Vec<_>, BoxedError> = poly.interiors().iter()
                        .map(|interior| {
                            let coords: Result<Vec<_>, BoxedError> = interior.coords()
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
        _ => {
            Err("Unsupported 2D geometry type for transformation".into())
        }
    }
}

/// Transform a 3D geometry using proj
fn transform_geometry_3d(geom: &Geometry3D<f64>, proj: &proj::Proj) -> Result<Geometry3D<f64>, BoxedError> {
    match geom {
        Geometry3D::Point(p) => Ok(Geometry3D::Point(transform_point_3d(p, proj)?)),
        Geometry3D::LineString(ls) => {
            let coords: Result<Vec<_>, BoxedError> = ls.coords()
                .map(|c| {
                    let p = Point3D::from([c.x, c.y, c.z]);
                    transform_point_3d(&p, proj).map(|tp| tp.0)
                })
                .collect();
            Ok(Geometry3D::LineString(LineString3D::new(coords?)))
        }
        Geometry3D::Polygon(poly) => {
            let exterior_coords: Result<Vec<_>, BoxedError> = poly.exterior().coords()
                .map(|c| {
                    let p = Point3D::from([c.x, c.y, c.z]);
                    transform_point_3d(&p, proj).map(|tp| tp.0)
                })
                .collect();
            let exterior = LineString3D::new(exterior_coords?);

            let interiors: Result<Vec<_>, BoxedError> = poly.interiors().iter()
                .map(|interior| {
                    let coords: Result<Vec<_>, BoxedError> = interior.coords()
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
            let points: Result<Vec<_>, BoxedError> = mp.iter()
                .map(|p| transform_point_3d(p, proj))
                .collect();
            Ok(Geometry3D::MultiPoint(MultiPoint3D::new(points?)))
        }
        Geometry3D::MultiLineString(mls) => {
            let line_strings: Result<Vec<_>, BoxedError> = mls.iter()
                .map(|ls| {
                    let coords: Result<Vec<_>, BoxedError> = ls.coords()
                        .map(|c| {
                            let p = Point3D::from([c.x, c.y, c.z]);
                            transform_point_3d(&p, proj).map(|tp| tp.0)
                        })
                        .collect();
                    Ok(LineString3D::new(coords?))
                })
                .collect();
            Ok(Geometry3D::MultiLineString(MultiLineString3D::new(line_strings?)))
        }
        Geometry3D::MultiPolygon(mpoly) => {
            let polygons: Result<Vec<_>, BoxedError> = mpoly.iter()
                .map(|poly| {
                    let exterior_coords: Result<Vec<_>, BoxedError> = poly.exterior().coords()
                        .map(|c| {
                            let p = Point3D::from([c.x, c.y, c.z]);
                            transform_point_3d(&p, proj).map(|tp| tp.0)
                        })
                        .collect();
                    let exterior = LineString3D::new(exterior_coords?);

                    let interiors: Result<Vec<_>, BoxedError> = poly.interiors().iter()
                        .map(|interior| {
                            let coords: Result<Vec<_>, BoxedError> = interior.coords()
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
        _ => {
            Err("Unsupported 3D geometry type for transformation".into())
        }
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
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: HorizontalReprojectorParam = if let Some(with) = with {
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

        // Note: We defer projection creation to runtime when we have the actual geometry's EPSG
        // This allows us to handle geometries that have EPSG codes embedded in them
        Ok(Box::new(HorizontalReprojector {
            source_epsg_code: params.source_epsg_code,
            target_epsg_code: params.target_epsg_code,
        }))
    }
}

/// # Horizontal Reprojector Parameters
/// Configure the source and target coordinate systems for geometry reprojection
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HorizontalReprojectorParam {
    /// # Source EPSG Code
    /// Source coordinate system EPSG code. If not provided, will use the EPSG code from the geometry.
    /// This is optional to maintain backward compatibility but recommended to be explicit.
    #[serde(default)]
    source_epsg_code: Option<EpsgCode>,

    /// # Target EPSG Code
    /// Target coordinate system EPSG code for the reprojection.
    /// Supports any valid EPSG code (e.g., 4326 for WGS84, 2193 for NZTM2000, 3857 for Web Mercator).
    #[serde(alias = "epsgCode")]
    target_epsg_code: EpsgCode,
}

#[derive(Debug, Clone)]
pub struct HorizontalReprojector {
    source_epsg_code: Option<EpsgCode>,
    target_epsg_code: EpsgCode,
}

impl Processor for HorizontalReprojector {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        // Determine source EPSG: from parameter or from geometry
        let source_epsg = self.source_epsg_code.or(geometry.epsg).ok_or_else(|| {
            GeometryProcessorError::HorizontalReprojector(
                "Source EPSG code not specified and geometry has no EPSG information. \
                Either set sourceEpsgCode parameter or ensure input geometries have EPSG codes."
                    .to_string(),
            )
        })?;

        // Create projection for this transformation
        let from_crs = format!("EPSG:{}", source_epsg);
        let to_crs = format!("EPSG:{}", self.target_epsg_code);

        let proj_transform = proj::Proj::new_known_crs(&from_crs, &to_crs, None).map_err(
            |e| {
                GeometryProcessorError::HorizontalReprojector(format!(
                    "Failed to create PROJ transformation from {} to {}: {}",
                    from_crs, to_crs, e
                ))
            },
        )?;

        match &geometry.value {
            GeometryValue::FlowGeometry2D(geom) => {
                let mut feature = feature.clone();
                let transformed = transform_geometry_2d(geom, &proj_transform)?;
                feature.geometry.value = GeometryValue::FlowGeometry2D(transformed);
                feature.geometry.epsg = Some(self.target_epsg_code);
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geom) => {
                let mut feature = feature.clone();
                let transformed = transform_geometry_3d(geom, &proj_transform)?;
                feature.geometry.value = GeometryValue::FlowGeometry3D(transformed);
                feature.geometry.epsg = Some(self.target_epsg_code);
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::CityGmlGeometry(_) => {
                return Err(GeometryProcessorError::HorizontalReprojector(
                    "CityGML geometry reprojection with PROJ not yet implemented. Use the legacy Japanese-only reprojector for CityGML.".to_string()
                ).into());
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
