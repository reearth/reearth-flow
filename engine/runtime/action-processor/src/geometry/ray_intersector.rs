use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use once_cell::sync::Lazy;
use rayon::prelude::*;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_geometry::algorithm::bvh_acceleration::AcceleratedGeometrySet;
use reearth_flow_geometry::algorithm::ray_intersection::{IncludeOrigin, Ray3D, RayHit};
use reearth_flow_geometry::types::coordinate::Coordinate3D;
use reearth_flow_geometry::types::geometry::Geometry3D;
use reearth_flow_geometry::types::line::Line;
use reearth_flow_geometry::types::point::Point3D;
use reearth_flow_geometry::types::triangular_mesh::TriangularMesh;
use reearth_flow_runtime::cache::executor_cache_subdir;
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
use crate::ACCUMULATOR_BUFFER_BYTE_THRESHOLD;

static RAY_PORT: Lazy<Port> = Lazy::new(|| Port::new("ray"));
static GEOM_PORT: Lazy<Port> = Lazy::new(|| Port::new("geom"));
static INTERSECTION_PORT: Lazy<Port> = Lazy::new(|| Port::new("intersection"));
static NO_INTERSECTION_PORT: Lazy<Port> = Lazy::new(|| Port::new("no_intersection"));

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
        vec![
            INTERSECTION_PORT.clone(),
            NO_INTERSECTION_PORT.clone(),
            REJECTED_PORT.clone(),
        ]
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
            output_geometry_type: params.output_geometry_type,
            pair_ids: Vec::new(),
            pair_id_set: HashSet::new(),
            ray_buffer: HashMap::new(),
            geom_buffer: HashMap::new(),
            buffer_bytes: 0,
            temp_dir: None,
            executor_id: None,
        }))
    }
}

/// Output geometry type for ray intersection results
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum OutputGeometryType {
    /// Output a point at the intersection location (default behavior)
    #[default]
    PointOfIntersection,
    /// Output a line segment from ray origin to intersection point
    LineSegmentToIntersection,
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

    /// Type of geometry to output for intersection results.
    /// - "pointOfIntersection" (default): Output a point at the intersection location
    /// - "lineSegmentToIntersection": Output a line segment from ray origin to intersection point
    #[serde(default)]
    pub output_geometry_type: OutputGeometryType,
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

/// Disk record for rays — stores pre-extracted ray values to avoid re-evaluating
/// attribute lookups in finish().
#[derive(Serialize, Deserialize)]
struct DiskRayRecord {
    feature: Feature,
    origin: [f64; 3],
    direction: [f64; 3],
}

pub struct RayIntersector {
    // Immutable config
    global_params: Option<HashMap<String, Value>>,
    ray_definition: RayDefinition,
    pair_id_ast: rhai::AST,
    closest_only_ast: Option<rhai::AST>,
    tolerance_ast: Option<rhai::AST>,
    include_ray_origin_ast: Option<rhai::AST>,
    output_geometry_type: OutputGeometryType,

    // Disk-backed state
    pair_ids: Vec<String>,
    pair_id_set: HashSet<String>,
    ray_buffer: HashMap<String, Vec<String>>,
    geom_buffer: HashMap<String, Vec<String>>,
    buffer_bytes: usize,
    temp_dir: Option<PathBuf>,
    executor_id: Option<uuid::Uuid>,
}

impl fmt::Debug for RayIntersector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RayIntersector")
            .field("pair_ids", &self.pair_ids.len())
            .field("buffer_bytes", &self.buffer_bytes)
            .field("temp_dir", &self.temp_dir)
            .finish()
    }
}

impl Clone for RayIntersector {
    fn clone(&self) -> Self {
        Self {
            global_params: self.global_params.clone(),
            ray_definition: self.ray_definition.clone(),
            pair_id_ast: self.pair_id_ast.clone(),
            closest_only_ast: self.closest_only_ast.clone(),
            tolerance_ast: self.tolerance_ast.clone(),
            include_ray_origin_ast: self.include_ray_origin_ast.clone(),
            output_geometry_type: self.output_geometry_type.clone(),
            pair_ids: Vec::new(),
            pair_id_set: HashSet::new(),
            ray_buffer: HashMap::new(),
            geom_buffer: HashMap::new(),
            buffer_bytes: 0,
            temp_dir: None,
            executor_id: None,
        }
    }
}

impl Drop for RayIntersector {
    fn drop(&mut self) {
        if let Some(ref dir) = self.temp_dir {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

/// Executor-specific engine cache folder for accumulating processors
fn engine_cache_dir(executor_id: uuid::Uuid) -> PathBuf {
    executor_cache_subdir(executor_id, "processors")
}

impl RayIntersector {
    fn ensure_temp_dir(&mut self) -> Result<&PathBuf, BoxedError> {
        if self.temp_dir.is_none() {
            let executor_id = self.executor_id.unwrap_or_else(uuid::Uuid::nil);
            let dir = engine_cache_dir(executor_id)
                .join(format!("ray-intersector-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(dir.join("rays"))?;
            std::fs::create_dir_all(dir.join("geoms"))?;
            self.temp_dir = Some(dir);
        }
        Ok(self.temp_dir.as_ref().unwrap())
    }

    fn flush_buffer(&mut self) -> Result<(), BoxedError> {
        if self.ray_buffer.is_empty() && self.geom_buffer.is_empty() {
            return Ok(());
        }

        let dir = self.ensure_temp_dir()?.clone();

        // Flush ray buffer
        for (pair_id, lines) in self.ray_buffer.drain() {
            let path = dir.join("rays").join(format!("{pair_id}.jsonl"));
            let file = File::options().create(true).append(true).open(&path)?;
            let mut writer = BufWriter::new(file);
            for line in &lines {
                writer.write_all(line.as_bytes())?;
                writer.write_all(b"\n")?;
            }
            writer.flush()?;
        }

        // Flush geom buffer
        for (pair_id, lines) in self.geom_buffer.drain() {
            let path = dir.join("geoms").join(format!("{pair_id}.jsonl"));
            let file = File::options().create(true).append(true).open(&path)?;
            let mut writer = BufWriter::new(file);
            for line in &lines {
                writer.write_all(line.as_bytes())?;
                writer.write_all(b"\n")?;
            }
            writer.flush()?;
        }

        self.buffer_bytes = 0;
        Ok(())
    }

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
            None => Ok(true),
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

    /// Creates an intersection feature (pure function — no fw.send()).
    fn create_intersection_feature(
        &self,
        ray_feature: &Feature,
        ray: &Ray3D,
        hit: RayHit,
    ) -> Feature {
        let mut output_feature = ray_feature.clone();

        let geometry_value = match self.output_geometry_type {
            OutputGeometryType::PointOfIntersection => GeometryValue::FlowGeometry3D(
                Geometry3D::Point(Point3D::new(hit.point.x, hit.point.y, hit.point.z)),
            ),
            OutputGeometryType::LineSegmentToIntersection => {
                let (origin, _) = ray.origin_and_direction();
                GeometryValue::FlowGeometry3D(Geometry3D::Line(Line::new_(origin, hit.point)))
            }
        };

        let geometry = Geometry {
            value: geometry_value,
            ..Default::default()
        };
        output_feature.geometry = Arc::new(geometry);

        output_feature.attributes_mut().insert(
            Attribute::new("ray_intersection_t"),
            AttributeValue::Number(
                serde_json::Number::from_f64(hit.t).unwrap_or_else(|| serde_json::Number::from(0)),
            ),
        );

        output_feature
    }

    /// Creates an output feature with ray geometry when lineSegmentToIntersection is set,
    /// or returns the original feature otherwise.
    fn create_ray_output_feature(&self, ray_feature: &Feature, ray: &Ray3D) -> Feature {
        match self.output_geometry_type {
            OutputGeometryType::PointOfIntersection => ray_feature.clone(),
            OutputGeometryType::LineSegmentToIntersection => {
                let mut output_feature = ray_feature.clone();
                let (origin, direction) = ray.origin_and_direction();
                let end = Coordinate3D::new__(
                    origin.x + direction.x,
                    origin.y + direction.y,
                    origin.z + direction.z,
                );
                let geometry = Geometry {
                    value: GeometryValue::FlowGeometry3D(Geometry3D::Line(Line::new_(origin, end))),
                    ..Default::default()
                };
                let geometry = Arc::new(geometry);
                output_feature.geometry = geometry;
                output_feature
            }
        }
    }
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
        if self.executor_id.is_none() {
            self.executor_id = Some(fw.executor_id());
        }

        let feature = &ctx.feature;
        let expr_engine = Arc::clone(&ctx.expr_engine);

        let pair_id = self.evaluate_pair_id(&expr_engine, feature)?;

        // Register pair_id
        if self.pair_id_set.insert(pair_id.clone()) {
            self.pair_ids.push(pair_id.clone());
        }

        match &ctx.port {
            port if port == &*RAY_PORT => match self.extract_ray(feature) {
                Ok(ray) => {
                    let (origin, direction) = ray.origin_and_direction();
                    let record = DiskRayRecord {
                        feature: feature.clone(),
                        origin: [origin.x, origin.y, origin.z],
                        direction: [direction.x, direction.y, direction.z],
                    };
                    let json = serde_json::to_string(&record).map_err(|e| {
                        GeometryProcessorError::RayIntersector(format!(
                            "Failed to serialize ray record: {e}"
                        ))
                    })?;
                    self.buffer_bytes += json.len();
                    self.ray_buffer.entry(pair_id).or_default().push(json);
                }
                Err(_) => {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                }
            },
            port if port == &*GEOM_PORT => {
                if let GeometryValue::FlowGeometry3D(geo) = &feature.geometry.value {
                    if let Some(mesh) = to_triangle_mesh(geo.clone(), DEFAULT_TOLERANCE) {
                        let json = serde_json::to_string(&mesh).map_err(|e| {
                            GeometryProcessorError::RayIntersector(format!(
                                "Failed to serialize mesh: {e}"
                            ))
                        })?;
                        self.buffer_bytes += json.len();
                        self.geom_buffer.entry(pair_id).or_default().push(json);
                    } else {
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()),
                        );
                    }
                } else if let GeometryValue::CityGmlGeometry(geo) = &feature.geometry.value {
                    if geo.gml_geometries.len() != 1 {
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()),
                        );
                        return Ok(());
                    }

                    let polygons = &geo.gml_geometries[0].polygons;
                    let geom =
                        match polygons.len() {
                            0 => {
                                fw.send(ctx.new_with_feature_and_port(
                                    feature.clone(),
                                    REJECTED_PORT.clone(),
                                ));
                                return Ok(());
                            }
                            1 => Geometry3D::Polygon(polygons[0].clone()),
                            _ => Geometry3D::MultiPolygon(
                                reearth_flow_geometry::types::multi_polygon::MultiPolygon3D::new(
                                    polygons.to_vec(),
                                ),
                            ),
                        };
                    if let Some(mesh) = to_triangle_mesh(geom, DEFAULT_TOLERANCE) {
                        let json = serde_json::to_string(&mesh).map_err(|e| {
                            GeometryProcessorError::RayIntersector(format!(
                                "Failed to serialize mesh: {e}"
                            ))
                        })?;
                        self.buffer_bytes += json.len();
                        self.geom_buffer.entry(pair_id).or_default().push(json);
                    } else {
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()),
                        );
                    }
                } else {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                }
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }

        if self.buffer_bytes >= ACCUMULATOR_BUFFER_BYTE_THRESHOLD {
            self.flush_buffer()?;
        }

        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let finish_start = Instant::now();

        // Flush remaining in-memory buffer
        self.flush_buffer()?;
        // Reclaim buffer memory
        self.ray_buffer = HashMap::new();
        self.geom_buffer = HashMap::new();

        let dir = match &self.temp_dir {
            Some(d) => d.clone(),
            None => {
                tracing::info!("RayIntersector finish: no data received");
                return Ok(());
            }
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let pair_ids = std::mem::take(&mut self.pair_ids);

        tracing::info!("RayIntersector finish: {} pair groups", pair_ids.len(),);

        let intersection_path = dir.join("intersection.jsonl");
        let no_intersection_path = dir.join("no_intersection.jsonl");

        let mut intersection_writer = BufWriter::new(File::create(&intersection_path)?);
        let mut no_intersection_writer = BufWriter::new(File::create(&no_intersection_path)?);

        let mut total_intersections = 0usize;
        let mut total_no_intersections = 0usize;

        for pair_id in &pair_ids {
            let group_start = Instant::now();

            let ray_path = dir.join("rays").join(format!("{pair_id}.jsonl"));
            let geom_path = dir.join("geoms").join(format!("{pair_id}.jsonl"));

            // Load all geometry meshes for this pair
            let geoms: Vec<TriangularMesh<f64, f64>> = if geom_path.exists() {
                let file = File::open(&geom_path)?;
                let reader = BufReader::new(file);
                let mut meshes = Vec::new();
                for line in reader.lines() {
                    let line = line?;
                    if line.is_empty() {
                        continue;
                    }
                    let mesh: TriangularMesh<f64, f64> = serde_json::from_str(&line)?;
                    meshes.push(mesh);
                }
                meshes
            } else {
                Vec::new()
            };

            if !ray_path.exists() {
                continue;
            }

            // If no geometries, emit all rays to no_intersection
            if geoms.is_empty() {
                let file = File::open(&ray_path)?;
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    let line = line?;
                    if line.is_empty() {
                        continue;
                    }
                    let record: DiskRayRecord = serde_json::from_str(&line)?;
                    let ray = Ray3D::new(
                        Coordinate3D::new__(record.origin[0], record.origin[1], record.origin[2]),
                        Coordinate3D::new__(
                            record.direction[0],
                            record.direction[1],
                            record.direction[2],
                        ),
                    );
                    let output_feature = self.create_ray_output_feature(&record.feature, &ray);
                    let json = serde_json::to_string(&output_feature)?;
                    no_intersection_writer.write_all(json.as_bytes())?;
                    no_intersection_writer.write_all(b"\n")?;
                    total_no_intersections += 1;
                }
                tracing::info!(
                    "RayIntersector pair_id={}: no geometries — skipped in {:.3}ms",
                    pair_id,
                    group_start.elapsed().as_secs_f64() * 1000.0,
                );
                continue;
            }

            // Build BVH
            let bvh_start = Instant::now();
            let accel_set = AcceleratedGeometrySet::build(&geoms);
            let bvh_elapsed = bvh_start.elapsed();

            // Stream rays in chunks
            let intersect_start = Instant::now();
            let file = File::open(&ray_path)?;
            let reader = BufReader::new(file);
            let mut chunk: Vec<DiskRayRecord> = Vec::new();
            let mut chunk_bytes: usize = 0;
            let mut group_intersections = 0usize;
            let mut group_no_intersections = 0usize;

            let mut lines_iter = reader.lines();
            loop {
                // Read lines until chunk is big enough or EOF
                let mut eof = false;
                while chunk_bytes < ACCUMULATOR_BUFFER_BYTE_THRESHOLD {
                    match lines_iter.next() {
                        Some(Ok(line)) => {
                            if line.is_empty() {
                                continue;
                            }
                            chunk_bytes += line.len();
                            let record: DiskRayRecord = serde_json::from_str(&line)?;
                            chunk.push(record);
                        }
                        Some(Err(e)) => return Err(e.into()),
                        None => {
                            eof = true;
                            break;
                        }
                    }
                }

                if chunk.is_empty() {
                    break;
                }

                // Process chunk in parallel
                let results: Vec<(Feature, Ray3D, Vec<RayHit>)> = chunk
                    .par_iter()
                    .map(|record| {
                        let ray = Ray3D::new(
                            Coordinate3D::new__(
                                record.origin[0],
                                record.origin[1],
                                record.origin[2],
                            ),
                            Coordinate3D::new__(
                                record.direction[0],
                                record.direction[1],
                                record.direction[2],
                            ),
                        );

                        let closest_only = self
                            .evaluate_closest_only(&expr_engine, &record.feature)
                            .unwrap_or(true);
                        let tolerance = self
                            .evaluate_tolerance(&expr_engine, &record.feature)
                            .unwrap_or(DEFAULT_TOLERANCE);
                        let include_ray_origin = self
                            .evaluate_include_ray_origin(&expr_engine, &record.feature)
                            .unwrap_or(true);

                        let include_origin = if include_ray_origin {
                            IncludeOrigin::Yes
                        } else {
                            IncludeOrigin::No { tolerance }
                        };

                        let hits = if closest_only {
                            accel_set
                                .closest_ray_intersection(&ray, tolerance, include_origin)
                                .map(|(_, hit)| vec![hit])
                                .unwrap_or_default()
                        } else {
                            accel_set
                                .ray_intersections(&ray, tolerance, include_origin)
                                .into_iter()
                                .map(|(_, hit)| hit)
                                .collect()
                        };

                        (record.feature.clone(), ray, hits)
                    })
                    .collect();

                // Write results to disk
                for (ray_feature, ray, hits) in results {
                    if hits.is_empty() {
                        group_no_intersections += 1;
                        let output_feature = self.create_ray_output_feature(&ray_feature, &ray);
                        let json = serde_json::to_string(&output_feature)?;
                        no_intersection_writer.write_all(json.as_bytes())?;
                        no_intersection_writer.write_all(b"\n")?;
                    } else {
                        group_intersections += hits.len();
                        for hit in hits {
                            let output_feature =
                                self.create_intersection_feature(&ray_feature, &ray, hit);
                            let json = serde_json::to_string(&output_feature)?;
                            intersection_writer.write_all(json.as_bytes())?;
                            intersection_writer.write_all(b"\n")?;
                        }
                    }
                }

                chunk.clear();
                chunk_bytes = 0;

                if eof {
                    break;
                }
            }

            total_intersections += group_intersections;
            total_no_intersections += group_no_intersections;

            // BVH and geoms drop here at end of scope
            tracing::info!(
                "RayIntersector pair_id={}: {} geoms — \
                 BVH build: {:.3}ms, ray intersect: {:.3}ms, \
                 hits: {}, misses: {}, total: {:.3}ms",
                pair_id,
                geoms.len(),
                bvh_elapsed.as_secs_f64() * 1000.0,
                intersect_start.elapsed().as_secs_f64() * 1000.0,
                group_intersections,
                group_no_intersections,
                group_start.elapsed().as_secs_f64() * 1000.0,
            );
        }

        intersection_writer.flush()?;
        no_intersection_writer.flush()?;

        // Send output files
        if total_intersections > 0 {
            fw.send_file(
                intersection_path,
                INTERSECTION_PORT.clone(),
                ctx.as_context(),
            );
        }
        if total_no_intersections > 0 {
            fw.send_file(
                no_intersection_path,
                NO_INTERSECTION_PORT.clone(),
                ctx.as_context(),
            );
        }

        tracing::info!(
            "RayIntersector finish complete: {} intersections, {} no-intersections, total {:.3}ms",
            total_intersections,
            total_no_intersections,
            finish_start.elapsed().as_secs_f64() * 1000.0,
        );
        Ok(())
    }

    fn name(&self) -> &str {
        "RayIntersector"
    }
}

fn to_triangle_mesh(geom: Geometry3D<f64>, tolerance: f64) -> Option<TriangularMesh<f64, f64>> {
    match geom {
        Geometry3D::Polygon(p) => TriangularMesh::try_from_polygons(vec![p], Some(tolerance)).ok(),
        Geometry3D::MultiPolygon(mp) => {
            TriangularMesh::try_from_polygons(mp.0, Some(tolerance)).ok()
        }
        Geometry3D::Solid(s) => s.as_triangle_mesh(Some(tolerance)).ok(),
        Geometry3D::Triangle(t) => Some(TriangularMesh::from_single_triangle(t)),
        Geometry3D::TriangularMesh(mesh) => Some(mesh),
        _ => None,
    }
}
