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
use tracing::debug;

use super::errors::GeometryProcessorError;

pub static PLANARITY_PORT: Lazy<Port> = Lazy::new(|| Port::new("planarity"));
pub static NOT_PLANARITY_PORT: Lazy<Port> = Lazy::new(|| Port::new("notplanarity"));

/// Filter type for planarity check
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum PlanarityFilterType {
    /// Uses covariance matrix eigenvalue analysis
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
    /// The threshold value for planarity check.
    /// For covariance mode: the maximum allowed smallest eigenvalue of the covariance matrix.
    /// For height mode: the maximum allowed convex hull minimum height.
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

        let gml_id = feature
            .get("gmlId")
            .or_else(|| feature.get("gml_id"))
            .map(|v| format!("{v:?}"))
            .unwrap_or_else(|| "<none>".to_string());
        let feature_type = feature
            .get("featureType")
            .or_else(|| feature.get("feature_type"))
            .map(|v| format!("{v:?}"))
            .unwrap_or_else(|| "<none>".to_string());
        let lod = feature
            .get("lod")
            .map(|v| format!("{v:?}"))
            .unwrap_or_else(|| "<none>".to_string());

        debug!(
            gml_id = %gml_id,
            feature_type = %feature_type,
            lod = %lod,
            filter_type = ?self.filter_type,
            threshold = threshold,
            "PlanarityFilter: processing feature"
        );

        if geometry.is_empty() {
            debug!(gml_id = %gml_id, "PlanarityFilter: empty geometry → notplanarity");
            send_feature_as_non_planar_surface(feature, &ctx, fw);
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                debug!(gml_id = %gml_id, "PlanarityFilter: None geometry → notplanarity");
                send_feature_as_non_planar_surface(feature, &ctx, fw);
            }
            GeometryValue::FlowGeometry2D(_) => {
                // 2D geometry is always coplanar (it's 2D)
                debug!(gml_id = %gml_id, "PlanarityFilter: 2D geometry → planarity (always coplanar)");
                fw.send(ctx.new_with_feature_and_port(feature.clone(), PLANARITY_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geometry) => {
                let points: Vec<Coordinate3D<f64>> = geometry.coords_iter().collect();
                debug!(
                    gml_id = %gml_id,
                    point_count = points.len(),
                    "PlanarityFilter: FlowGeometry3D"
                );
                let is_planar = match self.filter_type {
                    PlanarityFilterType::Covariance => are_points_coplanar(&points, threshold),
                    PlanarityFilterType::Height => check_planarity_height(points, threshold),
                };

                if let Some(result) = is_planar {
                    debug!(
                        gml_id = %gml_id,
                        normal = ?[result.normal.x(), result.normal.y(), result.normal.z()],
                        center = ?[result.center.x(), result.center.y(), result.center.z()],
                        "PlanarityFilter: FlowGeometry3D → planarity"
                    );
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
                    debug!(gml_id = %gml_id, "PlanarityFilter: FlowGeometry3D → notplanarity");
                    send_feature_as_non_planar_surface(feature, &ctx, fw);
                }
            }
            GeometryValue::CityGmlGeometry(geometry) => {
                let num_gml_geoms = geometry.gml_geometries.len();
                // We support only single-polygon geometries for planarity check
                if num_gml_geoms != 1 {
                    debug!(
                        gml_id = %gml_id,
                        num_gml_geometries = num_gml_geoms,
                        "PlanarityFilter: CityGML geometry has != 1 gml_geometries → notplanarity"
                    );
                    send_feature_as_non_planar_surface(feature, &ctx, fw);
                    return Ok(());
                };
                let geometry = &geometry.gml_geometries[0];
                let num_polygons = geometry.polygons.len();
                if num_polygons != 1 {
                    debug!(
                        gml_id = %gml_id,
                        num_polygons = num_polygons,
                        "PlanarityFilter: CityGML gml_geometry has != 1 polygons → notplanarity"
                    );
                    send_feature_as_non_planar_surface(feature, &ctx, fw);
                    return Ok(());
                };
                let polygon = &geometry.polygons[0];
                let exterior_count = polygon.exterior().len();
                let interior_count = polygon.interiors().len();
                let interior_ring_sizes: Vec<usize> =
                    polygon.interiors().iter().map(|r| r.len()).collect();
                debug!(
                    gml_id = %gml_id,
                    exterior_vertex_count = exterior_count,
                    interior_ring_count = interior_count,
                    interior_ring_sizes = ?interior_ring_sizes,
                    filter_type = ?self.filter_type,
                    threshold = threshold,
                    "PlanarityFilter: CityGML single polygon"
                );
                match self.filter_type {
                    PlanarityFilterType::Covariance => {
                        let points: Vec<Coordinate3D<f64>> = polygon.coords_iter().collect();
                        let is_planar = are_points_coplanar(&points, threshold);
                        let Some(result) = is_planar else {
                            debug!(
                                gml_id = %gml_id,
                                point_count = points.len(),
                                "PlanarityFilter: CityGML Covariance → notplanarity"
                            );
                            send_feature_as_non_planar_surface(feature, &ctx, fw);
                            return Ok(());
                        };
                        debug!(
                            gml_id = %gml_id,
                            normal = ?[result.normal.x(), result.normal.y(), result.normal.z()],
                            center = ?[result.center.x(), result.center.y(), result.center.z()],
                            "PlanarityFilter: CityGML Covariance → planarity"
                        );
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
                        let point_count = check_points.len();
                        let is_planar = check_planarity_height(check_points, threshold);
                        let Some(result) = is_planar else {
                            debug!(
                                gml_id = %gml_id,
                                point_count = point_count,
                                "PlanarityFilter: CityGML Height → notplanarity"
                            );
                            send_feature_as_non_planar_surface(feature, &ctx, fw);
                            return Ok(());
                        };
                        debug!(
                            gml_id = %gml_id,
                            normal = ?[result.normal.x(), result.normal.y(), result.normal.z()],
                            center = ?[result.center.x(), result.center.y(), result.center.z()],
                            "PlanarityFilter: CityGML Height → planarity"
                        );
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

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
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
    // Remove closing point if the polygon ring is closed (last point == first point).
    // This ensures that closed triangles (3 unique vertices + closing = 4 coords)
    // are handled by the triangle shortcut below.
    if points.len() >= 2 {
        let first = points[0];
        let last = *points.last().unwrap();
        let d = first - last;
        if d.x * d.x + d.y * d.y + d.z * d.z < 1e-20 {
            points.pop();
        }
    }

    if points.len() < 4 {
        // Less than 4 points: compute plane normal if we have at least 3
        if points.len() < 3 {
            return None;
        }
        // With 3 points, compute the plane they define
        let a = points[0];
        let b = points[1];
        let c = points[2];
        let ab = b - a;
        let ac = c - a;
        let normal = ab.cross(&ac);
        let norm = normal.norm();
        if norm < 1e-10 {
            return None; // Collinear points
        }
        let normal_normalized = normal / norm;
        let center = (a + b + c) / 3.0;
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

    // Early return optimization: O(n) planarity check
    // Find two base vertices and search for a third that forms a well-defined triangle
    let n = points.len();
    let v0 = points[0]; // At origin after translation
    let v_mid = points[n / 2];

    // We need a non-degenerate triangle to define a reliable normal direction.
    let cross_threshold = threshold * threshold;

    let mut found_normal = None;
    for (i, &v) in points.iter().enumerate() {
        if i == 0 || i == n / 2 {
            continue;
        }
        let edge1 = v_mid - v0;
        let edge2 = v - v0;
        let cross = edge1.cross(&edge2);
        let cross_norm = cross.norm();
        if cross_norm > cross_threshold {
            found_normal = Some(cross / cross_norm);
            break;
        }
    }

    if let Some(unit_normal) = found_normal {
        // Compute projections of all points onto the normal direction
        // Since v0 is at origin, we compute p · unit_normal directly
        let mut min_proj = 0.0_f64;
        let mut max_proj = 0.0_f64;
        for &p in &points {
            let proj = p.dot(&unit_normal);
            min_proj = min_proj.min(proj);
            max_proj = max_proj.max(proj);
        }

        if max_proj - min_proj <= threshold {
            // Planar! Return the coplanar result
            return are_points_coplanar(&points, f64::MAX);
        }
    }

    // Compute 3D convex hull
    // we require quick_hull_3d to compute hull with 1% tolerance of the threshold
    let Some(hull) = quick_hull_3d(&points, threshold * 0.01) else {
        // quick_hull_3d returns None when points are coplanar (can't form a 3D tetrahedron).
        // This means the convex hull minimum height is effectively 0, which is always <= threshold.
        // Use PCA to compute the best-fit plane normal.
        return are_points_coplanar(&points, f64::MAX);
    };
    let vertices = hull.get_vertices();
    let triangles = hull.get_triangles();

    if triangles.is_empty() || vertices.is_empty() {
        // Degenerate case
        return None;
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

/// Compute the minimum height of a convex hull.
///
/// Tests two families of candidate normal directions:
/// 1. **Face normals** — the normal of each hull triangle.
/// 2. **Edge-edge cross products** — for every pair of hull edges, their cross product
///    gives a candidate normal perpendicular to both edges.  For a convex polytope the
///    true minimum width is achieved either along a face normal *or* along a direction
///    that is the cross product of two antipodal edges, so testing all edge pairs yields
///    the correct minimum.
fn compute_convex_hull_min_height(
    vertices: &[reearth_flow_geometry::types::coordinate::Coordinate<f64, f64>],
    triangles: &[[usize; 3]],
) -> f64 {
    // Helper: project all vertices onto `unit_normal` and return the range (max - min).
    let range_along =
        |unit_normal: reearth_flow_geometry::types::coordinate::Coordinate<f64, f64>| -> f64 {
            let mut min_proj = f64::MAX;
            let mut max_proj = f64::NEG_INFINITY;
            for v in vertices {
                let proj = v.dot(&unit_normal);
                min_proj = min_proj.min(proj);
                max_proj = max_proj.max(proj);
            }
            max_proj - min_proj
        };

    let mut min_height = f64::MAX;

    // Collect every edge direction vector from the hull triangles.
    let mut edges = Vec::with_capacity(triangles.len() * 3);
    for tri in triangles {
        for k in 0..3 {
            edges.push(vertices[tri[(k + 1) % 3]] - vertices[tri[k]]);
        }
    }

    // --- Test 1: face normals ---
    for tri in triangles {
        let ab = vertices[tri[1]] - vertices[tri[0]];
        let ac = vertices[tri[2]] - vertices[tri[0]];
        let normal = ab.cross(&ac);
        let norm = normal.norm();
        if norm < 1e-10 {
            continue;
        }
        min_height = min_height.min(range_along(normal / norm));
    }

    // --- Test 2: edge-edge cross products ---
    // For each pair of edges, the cross product is a candidate normal direction.
    for i in 0..edges.len() {
        for j in (i + 1)..edges.len() {
            let normal = edges[i].cross(&edges[j]);
            let norm = normal.norm();
            if norm < 1e-10 {
                continue; // Parallel edges — cross product is degenerate, skip.
            }
            min_height = min_height.min(range_along(normal / norm));
        }
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
    use reearth_flow_types::{feature::Attributes, Feature};

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

        let feature = Feature::new_with_attributes(Attributes::new());
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

        let feature = Feature::new_with_attributes(Attributes::new());
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
