use std::collections::HashMap;
use std::sync::Arc;

use once_cell::sync::Lazy;
use rayon::prelude::*;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_geometry::algorithm::bvh_acceleration::AcceleratedGeometrySet;
use reearth_flow_geometry::algorithm::ray_intersection::{IncludeOrigin, Ray3D, RayHit};
use reearth_flow_geometry::types::coordinate::Coordinate3D;
use reearth_flow_geometry::types::geometry::Geometry3D;
use reearth_flow_geometry::types::point::Point3D;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

static RAY_PORT: Lazy<Port> = Lazy::new(|| Port::new("ray"));
static GEOM_PORT: Lazy<Port> = Lazy::new(|| Port::new("geom"));
static INTERSECTION_PORT: Lazy<Port> = Lazy::new(|| Port::new("intersection"));

const DEFAULT_TOLERANCE: f64 = 1e-10;

#[derive(Debug, Clone, Default)]
pub(super) struct RayIntersectorFactory;

impl ProcessorFactory for RayIntersectorFactory {
    fn name(&self) -> &str {
        "RayIntersector"
    }

    fn description(&self) -> &str {
        "Computes intersection points between rays and geometries"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(RayIntersectorParams))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![RAY_PORT.clone(), GEOM_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![INTERSECTION_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: RayIntersectorParams = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::RayIntersectorFactory(format!(
                    "Failed to serialize parameters: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::RayIntersectorFactory(format!(
                    "Failed to deserialize parameters: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::RayIntersectorFactory(
                "Missing required parameters".to_string(),
            )
            .into());
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);

        let pair_id_ast = expr_engine.compile(params.pair_id.as_ref()).map_err(|e| {
            GeometryProcessorError::RayIntersectorFactory(format!(
                "Failed to compile pairId expression: {e}"
            ))
        })?;

        let closest_only_ast = params
            .closest_intersection_only
            .map(|expr| {
                expr_engine.compile(expr.as_ref()).map_err(|e| {
                    GeometryProcessorError::RayIntersectorFactory(format!(
                        "Failed to compile closestIntersectionOnly expression: {e}"
                    ))
                })
            })
            .transpose()?;

        let tolerance_ast = params
            .tolerance
            .map(|expr| {
                expr_engine.compile(expr.as_ref()).map_err(|e| {
                    GeometryProcessorError::RayIntersectorFactory(format!(
                        "Failed to compile tolerance expression: {e}"
                    ))
                })
            })
            .transpose()?;

        let include_ray_origin_ast = params
            .include_ray_origin
            .map(|expr| {
                expr_engine.compile(expr.as_ref()).map_err(|e| {
                    GeometryProcessorError::RayIntersectorFactory(format!(
                        "Failed to compile includeRayOrigin expression: {e}"
                    ))
                })
            })
            .transpose()?;

        Ok(Box::new(RayIntersector {
            global_params: with,
            ray_definition: params.ray,
            pair_id_ast,
            closest_only_ast,
            tolerance_ast,
            include_ray_origin_ast,
            ray_buffer: HashMap::new(),
            geom_buffer: HashMap::new(),
        }))
    }
}

/// RayIntersector Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RayIntersectorParams {
    /// Defines how to extract ray data from feature attributes
    pub ray: RayDefinition,

    /// Expression that evaluates to a pair ID (int or string) for grouping rays with geometries.
    /// Only rays and geometries with matching pairId values are tested against each other.
    pub pair_id: Expr,

    /// When true (default), return only the closest intersection point per ray-geometry pair.
    /// When false, return all intersection points.
    #[serde(default)]
    pub closest_intersection_only: Option<Expr>,

    /// Tolerance for intersection calculations (evaluates to f64).
    /// If not specified, a default tolerance is used.
    #[serde(default)]
    pub tolerance: Option<Expr>,

    /// When true (default), include intersections at the ray origin.
    /// When false, exclude intersections where t < tolerance.
    #[serde(default)]
    pub include_ray_origin: Option<Expr>,
}

/// Defines how ray data is extracted from feature attributes.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RayDefinition {
    /// Attribute containing ray origin X coordinate
    pub pos_x: Attribute,
    /// Attribute containing ray origin Y coordinate
    pub pos_y: Attribute,
    /// Attribute containing ray origin Z coordinate
    pub pos_z: Attribute,
    /// Attribute containing ray direction X component
    pub dir_x: Attribute,
    /// Attribute containing ray direction Y component
    pub dir_y: Attribute,
    /// Attribute containing ray direction Z component
    pub dir_z: Attribute,
}

/// Internal data structure for buffered rays
#[derive(Debug, Clone)]
struct RayData {
    feature: Feature,
    ray: Ray3D,
}

#[derive(Clone, Debug)]
pub struct RayIntersector {
    global_params: Option<HashMap<String, Value>>,
    ray_definition: RayDefinition,
    pair_id_ast: rhai::AST,
    closest_only_ast: Option<rhai::AST>,
    tolerance_ast: Option<rhai::AST>,
    include_ray_origin_ast: Option<rhai::AST>,
    ray_buffer: HashMap<String, Vec<RayData>>,
    geom_buffer: HashMap<String, Vec<Geometry3D<f64>>>,
}

impl Processor for RayIntersector {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let expr_engine = Arc::clone(&ctx.expr_engine);

        let pair_id = self.evaluate_pair_id(&expr_engine, feature)?;

        match &ctx.port {
            port if port == &*RAY_PORT => match self.extract_ray(feature) {
                Ok(ray) => {
                    self.ray_buffer.entry(pair_id).or_default().push(RayData {
                        feature: feature.clone(),
                        ray,
                    });
                }
                Err(_) => {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                }
            },
            port if port == &*GEOM_PORT => {
                if let GeometryValue::FlowGeometry3D(geo) = &feature.geometry.value {
                    self.geom_buffer
                        .entry(pair_id)
                        .or_default()
                        .push(geo.clone());
                } else {
                    // TODO (After geometry type refactor):
                    // Currently only FlowGeometry3D is supported.
                    // This is because the current geometry type makes it difficult to integrate
                    // the ray intersection logic for 2D and 3D geometries.
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                }
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);

        // Process each pair group
        for (pair_id, rays) in &self.ray_buffer {
            let Some(geoms) = self.geom_buffer.get(pair_id) else {
                continue; // No matching geometries for this pair
            };

            if geoms.is_empty() || rays.is_empty() {
                continue;
            }

            let accel_set = AcceleratedGeometrySet::build(geoms);

            let results: Vec<(Feature, Vec<RayHit>)> = rays
                .par_iter()
                .filter_map(|ray_data| {
                    let closest_only = self
                        .evaluate_closest_only(&expr_engine, &ray_data.feature)
                        .unwrap_or(true);
                    let tolerance = self
                        .evaluate_tolerance(&expr_engine, &ray_data.feature)
                        .unwrap_or(DEFAULT_TOLERANCE);
                    let include_ray_origin = self
                        .evaluate_include_ray_origin(&expr_engine, &ray_data.feature)
                        .unwrap_or(true);

                    let include_origin = if include_ray_origin {
                        IncludeOrigin::Yes
                    } else {
                        IncludeOrigin::No { tolerance }
                    };

                    let hits = if closest_only {
                        accel_set
                            .closest_ray_intersection(&ray_data.ray, tolerance, include_origin)
                            .map(|(_, hit)| vec![hit])
                            .unwrap_or_default()
                    } else {
                        accel_set
                            .ray_intersections(&ray_data.ray, tolerance, include_origin)
                            .into_iter()
                            .map(|(_, hit)| hit)
                            .collect()
                    };

                    if hits.is_empty() {
                        None
                    } else {
                        Some((ray_data.feature.clone(), hits))
                    }
                })
                .collect();

            for (ray_feature, hits) in results {
                for hit in hits {
                    self.emit_intersection(&ray_feature, hit, &ctx, fw);
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "RayIntersector"
    }
}

impl RayIntersector {
    fn evaluate_pair_id(
        &self,
        expr_engine: &Arc<Engine>,
        feature: &Feature,
    ) -> Result<String, BoxedError> {
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);
        let result: rhai::Dynamic = scope.eval_ast(&self.pair_id_ast).map_err(|e| {
            GeometryProcessorError::RayIntersector(format!("Failed to evaluate pairId: {e}"))
        })?;
        Ok(result.to_string())
    }

    fn evaluate_closest_only(
        &self,
        expr_engine: &Arc<Engine>,
        feature: &Feature,
    ) -> Result<bool, BoxedError> {
        match &self.closest_only_ast {
            Some(ast) => {
                let scope = feature.new_scope(expr_engine.clone(), &self.global_params);
                let result: rhai::Dynamic = scope.eval_ast(ast).map_err(|e| {
                    GeometryProcessorError::RayIntersector(format!(
                        "Failed to evaluate closestIntersectionOnly: {e}"
                    ))
                })?;
                Ok(result.as_bool().unwrap_or(true))
            }
            None => Ok(true),
        }
    }

    fn evaluate_tolerance(
        &self,
        expr_engine: &Arc<Engine>,
        feature: &Feature,
    ) -> Result<f64, BoxedError> {
        match &self.tolerance_ast {
            Some(ast) => {
                let scope = feature.new_scope(expr_engine.clone(), &self.global_params);
                let result: rhai::Dynamic = scope.eval_ast(ast).map_err(|e| {
                    GeometryProcessorError::RayIntersector(format!(
                        "Failed to evaluate tolerance: {e}"
                    ))
                })?;
                result.as_float().map_err(|_| {
                    GeometryProcessorError::RayIntersector(
                        "tolerance must evaluate to a number".to_string(),
                    )
                    .into()
                })
            }
            None => Ok(DEFAULT_TOLERANCE),
        }
    }

    fn evaluate_include_ray_origin(
        &self,
        expr_engine: &Arc<Engine>,
        feature: &Feature,
    ) -> Result<bool, BoxedError> {
        match &self.include_ray_origin_ast {
            Some(ast) => {
                let scope = feature.new_scope(expr_engine.clone(), &self.global_params);
                let result: rhai::Dynamic = scope.eval_ast(ast).map_err(|e| {
                    GeometryProcessorError::RayIntersector(format!(
                        "Failed to evaluate includeRayOrigin: {e}"
                    ))
                })?;
                Ok(result.as_bool().unwrap_or(true))
            }
            None => Ok(true), // Default: include ray origin
        }
    }

    fn extract_ray(&self, feature: &Feature) -> Result<Ray3D, BoxedError> {
        let px = self.get_f64_attribute(feature, &self.ray_definition.pos_x)?;
        let py = self.get_f64_attribute(feature, &self.ray_definition.pos_y)?;
        let pz = self.get_f64_attribute(feature, &self.ray_definition.pos_z)?;
        let dx = self.get_f64_attribute(feature, &self.ray_definition.dir_x)?;
        let dy = self.get_f64_attribute(feature, &self.ray_definition.dir_y)?;
        let dz = self.get_f64_attribute(feature, &self.ray_definition.dir_z)?;

        Ok(Ray3D::new(
            Coordinate3D::new__(px, py, pz),
            Coordinate3D::new__(dx, dy, dz),
        ))
    }

    fn get_f64_attribute(&self, feature: &Feature, attr: &Attribute) -> Result<f64, BoxedError> {
        let value = feature.attributes.get(attr).ok_or_else(|| {
            GeometryProcessorError::RayIntersector(format!("Missing attribute: {attr}"))
        })?;

        match value {
            AttributeValue::Number(n) => n.as_f64().ok_or_else(|| {
                GeometryProcessorError::RayIntersector(format!(
                    "Attribute {attr} is not a valid number"
                ))
                .into()
            }),
            _ => Err(GeometryProcessorError::RayIntersector(format!(
                "Attribute {attr} is not a number"
            ))
            .into()),
        }
    }

    fn emit_intersection(
        &self,
        ray_feature: &Feature,
        hit: RayHit,
        ctx: &NodeContext,
        fw: &ProcessorChannelForwarder,
    ) {
        let mut output_feature = ray_feature.clone();
        output_feature.geometry = Arc::new(Geometry {
            value: GeometryValue::FlowGeometry3D(Geometry3D::Point(Point3D::new(
                hit.point.x,
                hit.point.y,
                hit.point.z,
            ))),
            ..Default::default()
        });

        output_feature.attributes_mut().insert(
            Attribute::new("ray_intersection_t"),
            AttributeValue::Number(
                serde_json::Number::from_f64(hit.t).unwrap_or_else(|| serde_json::Number::from(0)),
            ),
        );

        fw.send(ExecutorContext::new_with_node_context_feature_and_port(
            ctx,
            output_feature,
            INTERSECTION_PORT.clone(),
        ));
    }
}
