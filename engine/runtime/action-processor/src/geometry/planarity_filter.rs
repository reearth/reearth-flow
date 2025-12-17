use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_geometry::{
    algorithm::{convex_hull::quick_hull_3d, coords_iter::CoordsIter},
    types::coordinate::Coordinate3D,
    utils::are_points_coplanar,
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{AttributeValue, Expr, Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

pub static PLANARITY_PORT: Lazy<Port> = Lazy::new(|| Port::new("planarity"));
pub static NOT_PLANARITY_PORT: Lazy<Port> = Lazy::new(|| Port::new("notplanarity"));

/// Filter type for planarity check
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum PlanarityFilterType {
    /// Uses covariance matrix eigenvalue analysis (existing algorithm)
    #[default]
    Covariance,
    /// Uses minimum height of the 3D convex hull
    Height,
}

/// # Planarity Filter Parameters
/// Configure how to filter features based on geometry planarity
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlanarityFilterParam {
    /// # Filter Type
    /// The method to use for planarity detection
    #[serde(default)]
    pub filter_type: PlanarityFilterType,
    /// # Threshold
    /// The threshold value for planarity check (as an expression evaluating to f64).
    /// For covariance mode: the maximum allowed eigenvalue (default: 1e-6).
    /// For height mode: the maximum allowed convex hull height.
    pub threshold: Expr,
}

#[derive(Debug, Clone, Default)]
pub struct PlanarityFilterFactory;

impl ProcessorFactory for PlanarityFilterFactory {
    fn name(&self) -> &str {
        "PlanarityFilter"
    }

    fn description(&self) -> &str {
        "Filter Features by Geometry Planarity"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(PlanarityFilterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![PLANARITY_PORT.clone(), NOT_PLANARITY_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: PlanarityFilterParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::PlanarityFilterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::PlanarityFilterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::PlanarityFilterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let threshold_ast = expr_engine
            .compile(params.threshold.as_ref())
            .map_err(|e| {
                GeometryProcessorError::PlanarityFilterFactory(format!(
                    "Failed to compile threshold expression: {e:?}"
                ))
            })?;

        let process = PlanarityFilter {
            global_params: with,
            filter_type: params.filter_type,
            threshold_ast,
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct PlanarityFilter {
    global_params: Option<HashMap<String, serde_json::Value>>,
    filter_type: PlanarityFilterType,
    threshold_ast: rhai::AST,
}

impl Processor for PlanarityFilter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        // Evaluate the threshold expression
        let threshold = self.evaluate_threshold(feature, &ctx)?;

        if geometry.is_empty() {
            send_feature_as_non_planar_surface(feature, &ctx, fw);
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                send_feature_as_non_planar_surface(feature, &ctx, fw);
            }
            GeometryValue::FlowGeometry2D(_) => {
                // 2D geometry is always coplanar (it's 2D)
                fw.send(ctx.new_with_feature_and_port(feature.clone(), PLANARITY_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geometry) => {
                let points: Vec<Coordinate3D<f64>> = geometry.coords_iter().collect();
                let is_planar = match self.filter_type {
                    PlanarityFilterType::Covariance => are_points_coplanar(&points, threshold),
                    PlanarityFilterType::Height => check_planarity_height(points, threshold),
                };

                if let Some(result) = is_planar {
                    let mut feature = feature.clone();
                    let mut insert_number = |key: &str, value: f64| {
                        feature.insert(
                            key.to_string(),
                            AttributeValue::Number(
                                serde_json::Number::from_f64(value)
                                    .unwrap_or_else(|| serde_json::Number::from(0)),
                            ),
                        );
                    };
                    insert_number("surfaceNormalX", result.normal.x());
                    insert_number("surfaceNormalY", result.normal.y());
                    insert_number("surfaceNormalZ", result.normal.z());
                    insert_number("pointOnSurfaceX", result.center.x());
                    insert_number("pointOnSurfaceY", result.center.y());
                    insert_number("pointOnSurfaceZ", result.center.z());
                    fw.send(ctx.new_with_feature_and_port(feature, PLANARITY_PORT.clone()));
                } else {
                    send_feature_as_non_planar_surface(feature, &ctx, fw);
                }
            }
            GeometryValue::CityGmlGeometry(geometry) => {
                // We support only single-polygon geometries for planarity check
                if geometry.gml_geometries.len() != 1 {
                    send_feature_as_non_planar_surface(feature, &ctx, fw);
                    return Ok(());
                };
                let geometry = &geometry.gml_geometries[0];
                if geometry.polygons.len() != 1 {
                    send_feature_as_non_planar_surface(feature, &ctx, fw);
                    return Ok(());
                };
                let polygon = &geometry.polygons[0];
                match self.filter_type {
                    PlanarityFilterType::Covariance => {
                        let points: Vec<Coordinate3D<f64>> = polygon.coords_iter().collect();
                        let is_planar = are_points_coplanar(&points, threshold);
                        let Some(result) = is_planar else {
                            send_feature_as_non_planar_surface(feature, &ctx, fw);
                            return Ok(());
                        };
                        let mut feature = feature.clone();
                        let mut insert_number = |key: &str, value: f64| {
                            feature.insert(
                                key.to_string(),
                                AttributeValue::Number(
                                    serde_json::Number::from_f64(value)
                                        .unwrap_or_else(|| serde_json::Number::from(0)),
                                ),
                            );
                        };
                        insert_number("surfaceNormalX", result.normal.x());
                        insert_number("surfaceNormalY", result.normal.y());
                        insert_number("surfaceNormalZ", result.normal.z());
                        insert_number("pointOnSurfaceX", result.center.x());
                        insert_number("pointOnSurfaceY", result.center.y());
                        insert_number("pointOnSurfaceZ", result.center.z());
                        fw.send(ctx.new_with_feature_and_port(feature, PLANARITY_PORT.clone()))
                    }
                    PlanarityFilterType::Height => {
                        let check_points: Vec<Coordinate3D<f64>> = polygon.coords_iter().collect();
                        let is_planar = check_planarity_height(check_points, threshold);
                        let Some(result) = is_planar else {
                            send_feature_as_non_planar_surface(feature, &ctx, fw);
                            return Ok(());
                        };
                        let mut feature = feature.clone();
                        let mut insert_number = |key: &str, value: f64| {
                            feature.insert(
                                key.to_string(),
                                AttributeValue::Number(
                                    serde_json::Number::from_f64(value)
                                        .unwrap_or_else(|| serde_json::Number::from(0)),
                                ),
                            );
                        };
                        insert_number("surfaceNormalX", result.normal.x());
                        insert_number("surfaceNormalY", result.normal.y());
                        insert_number("surfaceNormalZ", result.normal.z());
                        insert_number("pointOnSurfaceX", result.center.x());
                        insert_number("pointOnSurfaceY", result.center.y());
                        insert_number("pointOnSurfaceZ", result.center.z());
                        fw.send(ctx.new_with_feature_and_port(feature, PLANARITY_PORT.clone()));
                    }
                }
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "PlanarityFilter"
    }
}

impl PlanarityFilter {
    fn evaluate_threshold(
        &self,
        feature: &Feature,
        ctx: &ExecutorContext,
    ) -> Result<f64, BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);
        scope.eval_ast::<f64>(&self.threshold_ast).map_err(|e| {
            GeometryProcessorError::PlanarityFilterFactory(format!(
                "Failed to evaluate threshold expression: {e:?}"
            ))
            .into()
        })
    }
}

/// Check planarity using convex hull minimum height
/// Returns Some(PointsCoplanar) if the minimum height of the convex hull is below threshold
fn check_planarity_height(
    mut points: Vec<Coordinate3D<f64>>,
    threshold: f64,
) -> Option<reearth_flow_geometry::utils::PointsCoplanar> {
    if points.len() < 4 {
        // Less than 4 points: compute plane normal if we have at least 3
        if points.len() < 3 {
            return None;
        }
        // With 3 points, compute the plane they define
        let a = points[0];
        let b = points[1];
        let c = points[2];
        let ab = Coordinate3D::new__(b.x - a.x, b.y - a.y, b.z - a.z);
        let ac = Coordinate3D::new__(c.x - a.x, c.y - a.y, c.z - a.z);
        let normal = ab.cross(&ac);
        let norm = normal.norm();
        if norm < 1e-10 {
            return None; // Collinear points
        }
        let normal_normalized =
            Coordinate3D::new__(normal.x / norm, normal.y / norm, normal.z / norm);
        let center = Coordinate3D::new__(
            (a.x + b.x + c.x) / 3.0,
            (a.y + b.y + c.y) / 3.0,
            (a.z + b.z + c.z) / 3.0,
        );
        return Some(reearth_flow_geometry::utils::PointsCoplanar {
            normal: reearth_flow_geometry::types::point::Point3D::new_(
                normal_normalized.x,
                normal_normalized.y,
                normal_normalized.z,
            ),
            center: reearth_flow_geometry::types::point::Point3D::new_(
                center.x, center.y, center.z,
            ),
        });
    }

    // Translate points to origin to improve numerical stability
    let a = points[0]; // This is safe since we have at least 4 points
    for p in &mut points {
        *p = *p - a;
    }

    // Compute 3D convex hull
    let Some(hull) = quick_hull_3d(&points, threshold * 0.01) else {
        let (triangle, n) = points.windows(3).find_map(|w| {
            let [a, b, c] = [w[0], w[1], w[2]];
            let ab = b - a;
            let ac = c - a;
            let mut n = ab.cross(&ac);
            if n.norm() > threshold * threshold {
                n = n.normalize();
                Some(([a, b, c], n))
            } else {
                None
            }
        })?; // if no such triangle found, then meaningful normal cannot be computed. so return `None`.
        return Some(reearth_flow_geometry::utils::PointsCoplanar {
            normal: reearth_flow_geometry::types::point::Point3D::new_(n.x, n.y, n.z),
            center: reearth_flow_geometry::types::point::Point3D::new_(
                (triangle[0].x + triangle[1].x + triangle[2].x) / 3.0,
                (triangle[0].y + triangle[1].y + triangle[2].y) / 3.0,
                (triangle[0].z + triangle[1].z + triangle[2].z) / 3.0,
            ),
        });
    };
    let vertices = hull.get_vertices();
    let triangles = hull.get_triangles();

    if triangles.is_empty() || vertices.is_empty() {
        // Degenerate case (e.g., all coplanar) - use covariance method as fallback
        return are_points_coplanar(&points, threshold);
    }

    // Compute minimum height of the convex hull
    // The minimum height is the smallest distance from any face to the most distant point
    // perpendicular to that face
    let min_height = compute_convex_hull_min_height(vertices, triangles);

    if min_height <= threshold {
        // Compute the best-fit plane normal using PCA
        are_points_coplanar(&points, f64::MAX)
    } else {
        None
    }
}

/// Compute the minimum height of a convex hull
/// The minimum height is the smallest perpendicular distance between two parallel supporting planes
fn compute_convex_hull_min_height(
    vertices: &[reearth_flow_geometry::types::coordinate::Coordinate<f64, f64>],
    triangles: &[[usize; 3]],
) -> f64 {
    let mut min_height = f64::MAX;

    for tri in triangles {
        let a = vertices[tri[0]];
        let b = vertices[tri[1]];
        let c = vertices[tri[2]];

        // Compute face normal
        let ab = Coordinate3D::new__(b.x - a.x, b.y - a.y, b.z - a.z);
        let ac = Coordinate3D::new__(c.x - a.x, c.y - a.y, c.z - a.z);
        let normal = ab.cross(&ac);
        let norm = normal.norm();

        if norm < 1e-10 {
            continue; // Degenerate triangle
        }

        let unit_normal = Coordinate3D::new__(normal.x / norm, normal.y / norm, normal.z / norm);

        // Find the extent of all vertices along this normal direction
        let mut min_proj = f64::MAX;
        let mut max_proj = f64::MIN;

        for v in vertices {
            let proj = v.x * unit_normal.x + v.y * unit_normal.y + v.z * unit_normal.z;
            min_proj = min_proj.min(proj);
            max_proj = max_proj.max(proj);
        }

        let height = max_proj - min_proj;
        min_height = min_height.min(height);
    }

    min_height
}

fn send_feature_as_non_planar_surface(
    feature: &Feature,
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
) {
    let mut feature = feature.clone();
    feature.insert(
        "issue",
        AttributeValue::String("NonPlanarSurface".to_string()),
    );
    fw.send(ctx.new_with_feature_and_port(feature, NOT_PLANARITY_PORT.clone()));
}

#[cfg(test)]
mod tests {
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Feature;

    use super::*;
    use crate::tests::utils::create_default_execute_context;

    fn create_test_processor(filter_type: PlanarityFilterType, threshold: f64) -> PlanarityFilter {
        // Create a simple AST that returns the threshold value
        let engine = rhai::Engine::new();
        let threshold_ast = engine.compile(format!("{}", threshold)).unwrap();

        PlanarityFilter {
            global_params: None,
            filter_type,
            threshold_ast,
        }
    }

    #[test]
    fn test_process_null_geometry_covariance() {
        let mut processor = create_test_processor(PlanarityFilterType::Covariance, 1e-6);
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        let feature = Feature::default();
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(noop.send_ports.lock().unwrap().len(), 1);
            assert_eq!(
                noop.send_ports.lock().unwrap().first().cloned(),
                Some(NOT_PLANARITY_PORT.clone())
            );
        }
    }

    #[test]
    fn test_process_null_geometry_height() {
        let mut processor = create_test_processor(PlanarityFilterType::Height, 0.001);
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        let feature = Feature::default();
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(noop.send_ports.lock().unwrap().len(), 1);
            assert_eq!(
                noop.send_ports.lock().unwrap().first().cloned(),
                Some(NOT_PLANARITY_PORT.clone())
            );
        }
    }
}
