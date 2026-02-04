use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use indexmap::IndexMap;
use reearth_flow_geometry::algorithm::bool_ops::BooleanOps;
use reearth_flow_geometry::algorithm::centroid::Centroid;
use reearth_flow_geometry::types::coordinate::{Coordinate2D, Coordinate3D};
use reearth_flow_geometry::types::geometry::Geometry3D;
use reearth_flow_geometry::types::line_string::{LineString2D, LineString3D};
use reearth_flow_geometry::types::multi_polygon::{MultiPolygon2D, MultiPolygon3D};
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use spade::{DelaunayTriangulation, Point2, Triangulation};

use super::errors::PlateauProcessorError;

/// Check if a point is inside a polygon using ray casting algorithm
fn point_in_polygon(x: f64, y: f64, polygon: &Polygon3D<f64>) -> bool {
    let exterior = polygon.exterior();
    let mut inside = ray_cast_test(x, y, &exterior.0);

    // Check holes - if inside a hole, the point is outside the polygon
    for interior in polygon.interiors() {
        if ray_cast_test(x, y, &interior.0) {
            inside = !inside;
        }
    }

    inside
}

fn ray_cast_test(x: f64, y: f64, ring: &[Coordinate3D<f64>]) -> bool {
    let mut inside = false;
    let n = ring.len();
    if n < 3 {
        return false;
    }

    let mut j = n - 1;
    for i in 0..n {
        let xi = ring[i].x;
        let yi = ring[i].y;
        let xj = ring[j].x;
        let yj = ring[j].y;

        if ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

#[derive(Debug, Clone, Default)]
pub struct FloodingAreaSurfaceGeneratorFactory;

impl ProcessorFactory for FloodingAreaSurfaceGeneratorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.FloodingAreaSurfaceGenerator"
    }

    fn description(&self) -> &str {
        "Generates TIN-based surfaces from flood area polygons for efficient 3D tile generation. \
         Optionally groups features by attribute before combining and triangulating."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FloodingAreaSurfaceGeneratorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
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
        let params: FloodingAreaSurfaceGeneratorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::FloodingAreaSurfaceGeneratorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::FloodingAreaSurfaceGeneratorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            FloodingAreaSurfaceGeneratorParam::default()
        };

        let process = FloodingAreaSurfaceGenerator {
            point_spacing: params.point_spacing.unwrap_or(50.0),
            sample_interior: params.sample_interior.unwrap_or(false),
            group_by: params.group_by,
            buffer: Arc::new(Mutex::new(HashMap::new())),
            epsg: Arc::new(Mutex::new(None)),
        };
        Ok(Box::new(process))
    }
}

/// # FloodingAreaSurfaceGenerator Parameters
///
/// Configuration for generating TIN surfaces from flood area polygons.
/// This processor converts polygons to triangulated surfaces by:
/// 1. Optionally grouping features by an attribute (e.g., udxDirs)
/// 2. Combining all polygons in each group
/// 3. Sampling points along polygon boundaries at regular intervals
/// 4. Optionally adding interior grid points within polygons
/// 5. Performing Delaunay triangulation to create a TIN surface
/// 6. Filtering triangles to keep only those inside the original polygons
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct FloodingAreaSurfaceGeneratorParam {
    /// Spacing between sampled points in meters (default: 50.0).
    /// Points are sampled along polygon boundaries and optionally on an interior grid
    /// at this spacing interval.
    pub point_spacing: Option<f64>,
    /// Enable interior grid sampling (default: false).
    /// When enabled, points are added on a regular grid inside the polygon
    /// to create a more uniform triangulation. This can be slow for large polygons.
    #[serde(default)]
    pub sample_interior: Option<bool>,
    /// Attribute name to group features by before combining and triangulating.
    /// Features with the same value for this attribute will be processed together.
    /// If not specified, each feature is processed individually.
    pub group_by: Option<Attribute>,
}

/// Data stored for each group
#[derive(Debug, Clone)]
struct GroupData {
    polygons: Vec<Polygon3D<f64>>,
    attributes: IndexMap<Attribute, AttributeValue>,
}

#[derive(Debug)]
pub struct FloodingAreaSurfaceGenerator {
    point_spacing: f64,
    sample_interior: bool,
    group_by: Option<Attribute>,
    buffer: Arc<Mutex<HashMap<AttributeValue, GroupData>>>,
    epsg: Arc<Mutex<Option<u16>>>,
}

impl Clone for FloodingAreaSurfaceGenerator {
    fn clone(&self) -> Self {
        Self {
            point_spacing: self.point_spacing,
            sample_interior: self.sample_interior,
            group_by: self.group_by.clone(),
            buffer: Arc::clone(&self.buffer),
            epsg: Arc::clone(&self.epsg),
        }
    }
}

impl Processor for FloodingAreaSurfaceGenerator {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        // Store EPSG if not set
        if let Ok(mut epsg_guard) = self.epsg.lock() {
            if epsg_guard.is_none() {
                *epsg_guard = geometry.epsg;
            }
        }

        // Extract polygons from geometry
        let polygons: Vec<Polygon3D<f64>> = match &geometry.value {
            GeometryValue::FlowGeometry3D(geom3d) => match geom3d {
                Geometry3D::Polygon(polygon) => vec![polygon.clone()],
                Geometry3D::MultiPolygon(multi_polygon) => multi_polygon.0.clone(),
                _ => {
                    // Non-polygon geometry, pass through
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
                    return Ok(());
                }
            },
            _ => {
                // Non-3D geometry, pass through
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
                return Ok(());
            }
        };

        // If grouping is enabled, buffer the feature
        if let Some(group_attr) = &self.group_by {
            let key = feature
                .attributes
                .get(group_attr)
                .cloned()
                .unwrap_or(AttributeValue::Null);

            if let Ok(mut buffer) = self.buffer.lock() {
                let entry = buffer.entry(key).or_insert_with(|| GroupData {
                    polygons: Vec::new(),
                    attributes: (*feature.attributes).clone(),
                });
                entry.polygons.extend(polygons);
            }
            return Ok(());
        }

        // No grouping - process immediately
        let epsg = geometry.epsg;
        let mut out_feature = feature.clone();

        let mut all_triangles = Vec::new();
        for polygon in &polygons {
            let triangles = self.triangulate_polygon(polygon)?;
            all_triangles.extend(triangles);
        }

        if !all_triangles.is_empty() {
            out_feature.geometry = Arc::new(Geometry {
                epsg,
                value: GeometryValue::FlowGeometry3D(Geometry3D::MultiPolygon(
                    MultiPolygon3D::new(all_triangles),
                )),
            });
        }

        fw.send(ctx.new_with_feature_and_port(out_feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        // If no grouping, nothing to do
        if self.group_by.is_none() {
            return Ok(());
        }

        let buffer = if let Ok(buffer) = self.buffer.lock() {
            buffer.clone()
        } else {
            return Ok(());
        };

        let epsg = self.epsg.lock().ok().and_then(|g| *g);

        // Process each group
        for (_key, group_data) in buffer {
            if group_data.polygons.is_empty() {
                continue;
            }

            // Step 1: Dissolve/merge polygons using 2D boolean union
            let dissolved_polygons = self.dissolve_polygons(&group_data.polygons);

            let mut all_triangles = Vec::new();

            // Step 2: Triangulate each dissolved polygon
            for polygon in &dissolved_polygons {
                if let Ok(triangles) = self.triangulate_polygon(polygon) {
                    all_triangles.extend(triangles);
                }
            }

            if all_triangles.is_empty() {
                continue;
            }

            // Create output feature
            let mut feature = Feature::new_with_attributes(group_data.attributes);
            feature.geometry = Arc::new(Geometry {
                epsg,
                value: GeometryValue::FlowGeometry3D(Geometry3D::MultiPolygon(
                    MultiPolygon3D::new(all_triangles),
                )),
            });

            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                DEFAULT_PORT.clone(),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "FloodingAreaSurfaceGenerator"
    }
}

impl FloodingAreaSurfaceGenerator {
    fn triangulate_polygon(
        &self,
        polygon: &Polygon3D<f64>,
    ) -> Result<Vec<Polygon3D<f64>>, BoxedError> {
        // Sample points from the polygon boundary and interior
        let points = self.sample_points_from_polygon(polygon);

        if points.len() < 3 {
            // Not enough points for triangulation, return original polygon
            return Ok(vec![polygon.clone()]);
        }

        // Perform Delaunay triangulation in 2D (x, y), preserving z values
        // Then filter triangles to keep only those inside the original polygon
        let triangles = self.delaunay_triangulate(&points, polygon)?;

        Ok(triangles)
    }

    fn sample_points_from_polygon(&self, polygon: &Polygon3D<f64>) -> Vec<Coordinate3D<f64>> {
        let mut points = Vec::new();
        let exterior = polygon.exterior();

        // Sample points along the exterior ring
        self.sample_ring(&exterior.0, &mut points);

        // Sample interior holes
        for interior in polygon.interiors() {
            self.sample_ring(&interior.0, &mut points);
        }

        // Add interior grid points if enabled
        if self.sample_interior {
            self.add_interior_grid_points(polygon, &mut points);
        }

        points
    }

    fn sample_ring(&self, ring: &[Coordinate3D<f64>], points: &mut Vec<Coordinate3D<f64>>) {
        for window in ring.windows(2) {
            let p1 = window[0];
            let p2 = window[1];

            let dx = p2.x - p1.x;
            let dy = p2.y - p1.y;
            let dz = p2.z - p1.z;
            let length = (dx * dx + dy * dy).sqrt();

            if length < self.point_spacing {
                points.push(p1);
            } else {
                let num_segments = (length / self.point_spacing).ceil() as usize;
                for i in 0..num_segments {
                    let t = i as f64 / num_segments as f64;
                    let x = p1.x + t * dx;
                    let y = p1.y + t * dy;
                    let z = p1.z + t * dz;
                    points.push(Coordinate3D::new__(x, y, z));
                }
            }
        }
    }

    fn add_interior_grid_points(
        &self,
        polygon: &Polygon3D<f64>,
        points: &mut Vec<Coordinate3D<f64>>,
    ) {
        // Maximum interior grid points to avoid performance issues with large polygons
        const MAX_INTERIOR_GRID_POINTS: usize = 1000;

        let exterior = polygon.exterior();
        if exterior.0.is_empty() {
            return;
        }

        // Calculate bounding box
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for coord in &exterior.0 {
            min_x = min_x.min(coord.x);
            min_y = min_y.min(coord.y);
            max_x = max_x.max(coord.x);
            max_y = max_y.max(coord.y);
        }

        let width = max_x - min_x;
        let height = max_y - min_y;

        // Adjust spacing for large polygons to avoid too many points
        let estimated_grid_points = (width / self.point_spacing) * (height / self.point_spacing);
        let effective_spacing = if estimated_grid_points > MAX_INTERIOR_GRID_POINTS as f64 {
            // Increase spacing to limit points
            let scale = (estimated_grid_points / MAX_INTERIOR_GRID_POINTS as f64).sqrt();
            self.point_spacing * scale
        } else {
            self.point_spacing
        };

        // Calculate average z value for interior points
        let avg_z: f64 = exterior.0.iter().map(|p| p.z).sum::<f64>() / exterior.0.len() as f64;

        // Create grid points within the bounding box
        let mut interior_count = 0;
        let mut y = min_y + effective_spacing;
        while y < max_y && interior_count < MAX_INTERIOR_GRID_POINTS {
            let mut x = min_x + effective_spacing;
            while x < max_x && interior_count < MAX_INTERIOR_GRID_POINTS {
                // Check if point is inside the polygon
                if point_in_polygon(x, y, polygon) {
                    // Interpolate z value from nearby boundary points
                    let z = self.interpolate_z(x, y, &exterior.0, avg_z);
                    points.push(Coordinate3D::new__(x, y, z));
                    interior_count += 1;
                }
                x += effective_spacing;
            }
            y += effective_spacing;
        }

        // Also add centroid if it's inside the polygon
        if let Some(centroid) = polygon.centroid() {
            let cx = centroid.x();
            let cy = centroid.y();
            if point_in_polygon(cx, cy, polygon) {
                let z = self.interpolate_z(cx, cy, &exterior.0, avg_z);
                points.push(Coordinate3D::new__(cx, cy, z));
            }
        }
    }

    fn interpolate_z(&self, x: f64, y: f64, ring: &[Coordinate3D<f64>], default_z: f64) -> f64 {
        // Simple inverse distance weighting interpolation
        let mut sum_weight = 0.0;
        let mut sum_z = 0.0;

        for coord in ring {
            let dx = coord.x - x;
            let dy = coord.y - y;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq < 1e-10 {
                // Very close to a boundary point, use its z value
                return coord.z;
            }

            let weight = 1.0 / dist_sq;
            sum_weight += weight;
            sum_z += weight * coord.z;
        }

        if sum_weight > 0.0 {
            sum_z / sum_weight
        } else {
            default_z
        }
    }

    fn delaunay_triangulate(
        &self,
        points: &[Coordinate3D<f64>],
        polygon: &Polygon3D<f64>,
    ) -> Result<Vec<Polygon3D<f64>>, BoxedError> {
        if points.len() < 3 {
            return Ok(Vec::new());
        }

        // Create a mapping from 2D points to 3D coordinates with z values
        let mut point_map: HashMap<(i64, i64), f64> = HashMap::new();
        let scale = 1_000_000.0; // Scale for hash key precision

        for p in points {
            let key = ((p.x * scale) as i64, (p.y * scale) as i64);
            point_map.insert(key, p.z);
        }

        // Create 2D points for Delaunay triangulation
        let points_2d: Vec<Point2<f64>> = points.iter().map(|p| Point2::new(p.x, p.y)).collect();

        // Build Delaunay triangulation
        let triangulation: DelaunayTriangulation<Point2<f64>> =
            DelaunayTriangulation::bulk_load(points_2d).map_err(|e| {
                PlateauProcessorError::FloodingAreaSurfaceGenerator(format!(
                    "Delaunay triangulation failed: {:?}",
                    e
                ))
            })?;

        // Extract triangles, filtering to keep only those inside the original polygon
        let mut triangles = Vec::new();
        for face in triangulation.inner_faces() {
            let vertices = face.vertices();
            let v0 = vertices[0].position();
            let v1 = vertices[1].position();
            let v2 = vertices[2].position();

            // Calculate centroid of the triangle
            let cx = (v0.x + v1.x + v2.x) / 3.0;
            let cy = (v0.y + v1.y + v2.y) / 3.0;

            // Only keep triangles whose centroid is inside the original polygon
            if !point_in_polygon(cx, cy, polygon) {
                continue;
            }

            // Look up z values
            let z0 = self.get_z_value(&point_map, v0.x, v0.y, scale);
            let z1 = self.get_z_value(&point_map, v1.x, v1.y, scale);
            let z2 = self.get_z_value(&point_map, v2.x, v2.y, scale);

            let coords = vec![
                Coordinate3D::new__(v0.x, v0.y, z0),
                Coordinate3D::new__(v1.x, v1.y, z1),
                Coordinate3D::new__(v2.x, v2.y, z2),
                Coordinate3D::new__(v0.x, v0.y, z0), // Close the ring
            ];

            let triangle = Polygon3D::new(LineString3D::new(coords), Vec::new());
            triangles.push(triangle);
        }

        Ok(triangles)
    }

    fn get_z_value(&self, point_map: &HashMap<(i64, i64), f64>, x: f64, y: f64, scale: f64) -> f64 {
        let key = ((x * scale) as i64, (y * scale) as i64);
        if let Some(&z) = point_map.get(&key) {
            return z;
        }

        // Try nearby keys for floating point precision issues
        for dx in -1..=1 {
            for dy in -1..=1 {
                let key = ((x * scale) as i64 + dx, (y * scale) as i64 + dy);
                if let Some(&z) = point_map.get(&key) {
                    return z;
                }
            }
        }

        0.0 // Default z value
    }

    /// Dissolves/merges a collection of 3D polygons by performing 2D union operations.
    /// Z values are preserved by storing them in a HashMap keyed by (x, y) coordinates.
    fn dissolve_polygons(&self, polygons: &[Polygon3D<f64>]) -> Vec<Polygon3D<f64>> {
        if polygons.is_empty() {
            return Vec::new();
        }

        if polygons.len() == 1 {
            return polygons.to_vec();
        }

        // Build a Z-value map from all input polygons for later reconstruction
        let mut z_map: HashMap<(i64, i64), f64> = HashMap::new();
        let scale = 1_000_000.0; // For coordinate precision

        // Convert 3D polygons to 2D and store Z values
        let mut polygons_2d: Vec<Polygon2D<f64>> = Vec::with_capacity(polygons.len());
        for poly3d in polygons {
            // Store Z values for all exterior coordinates
            for coord in &poly3d.exterior().0 {
                let key = ((coord.x * scale) as i64, (coord.y * scale) as i64);
                z_map.entry(key).or_insert(coord.z);
            }
            // Store Z values for all interior (hole) coordinates
            for interior in poly3d.interiors() {
                for coord in &interior.0 {
                    let key = ((coord.x * scale) as i64, (coord.y * scale) as i64);
                    z_map.entry(key).or_insert(coord.z);
                }
            }

            // Convert to 2D polygon
            let poly2d = self.polygon_3d_to_2d(poly3d);
            polygons_2d.push(poly2d);
        }

        // Perform incremental union of all polygons
        let mut result = MultiPolygon2D::new(vec![polygons_2d[0].clone()]);
        for poly in polygons_2d.iter().skip(1) {
            let other = MultiPolygon2D::new(vec![poly.clone()]);
            result = result.union(&other);
        }

        // Convert dissolved 2D polygons back to 3D
        let mut result_3d: Vec<Polygon3D<f64>> = Vec::with_capacity(result.0.len());
        for poly2d in &result.0 {
            let poly3d = self.polygon_2d_to_3d(poly2d, &z_map, scale);
            result_3d.push(poly3d);
        }

        result_3d
    }

    /// Converts a 3D polygon to a 2D polygon by dropping the Z coordinate.
    fn polygon_3d_to_2d(&self, poly3d: &Polygon3D<f64>) -> Polygon2D<f64> {
        let exterior_2d: Vec<Coordinate2D<f64>> = poly3d
            .exterior()
            .0
            .iter()
            .map(|c| Coordinate2D::new_(c.x, c.y))
            .collect();

        let interiors_2d: Vec<LineString2D<f64>> = poly3d
            .interiors()
            .iter()
            .map(|ring| {
                let coords: Vec<Coordinate2D<f64>> = ring
                    .0
                    .iter()
                    .map(|c| Coordinate2D::new_(c.x, c.y))
                    .collect();
                LineString2D::new(coords)
            })
            .collect();

        Polygon2D::new(LineString2D::new(exterior_2d), interiors_2d)
    }

    /// Converts a 2D polygon back to 3D by looking up Z values from the Z map.
    fn polygon_2d_to_3d(
        &self,
        poly2d: &Polygon2D<f64>,
        z_map: &HashMap<(i64, i64), f64>,
        scale: f64,
    ) -> Polygon3D<f64> {
        let exterior_3d: Vec<Coordinate3D<f64>> = poly2d
            .exterior()
            .0
            .iter()
            .map(|c| {
                let z = self.lookup_z(c.x, c.y, z_map, scale);
                Coordinate3D::new__(c.x, c.y, z)
            })
            .collect();

        let interiors_3d: Vec<LineString3D<f64>> = poly2d
            .interiors()
            .iter()
            .map(|ring| {
                let coords: Vec<Coordinate3D<f64>> = ring
                    .0
                    .iter()
                    .map(|c| {
                        let z = self.lookup_z(c.x, c.y, z_map, scale);
                        Coordinate3D::new__(c.x, c.y, z)
                    })
                    .collect();
                LineString3D::new(coords)
            })
            .collect();

        Polygon3D::new(LineString3D::new(exterior_3d), interiors_3d)
    }

    /// Looks up the Z value for a 2D coordinate from the Z map.
    /// Tries exact match first, then nearby coordinates.
    fn lookup_z(&self, x: f64, y: f64, z_map: &HashMap<(i64, i64), f64>, scale: f64) -> f64 {
        let key = ((x * scale) as i64, (y * scale) as i64);

        // Try exact match
        if let Some(&z) = z_map.get(&key) {
            return z;
        }

        // Try nearby coordinates (for floating point precision issues)
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nearby_key = (key.0 + dx, key.1 + dy);
                if let Some(&z) = z_map.get(&nearby_key) {
                    return z;
                }
            }
        }

        // Interpolate from nearby points if no exact match found
        // Find the closest point with a Z value
        let mut closest_dist = f64::MAX;
        let mut closest_z = 0.0;
        for (&(kx, ky), &z) in z_map.iter() {
            let px = kx as f64 / scale;
            let py = ky as f64 / scale;
            let dist = (x - px).powi(2) + (y - py).powi(2);
            if dist < closest_dist {
                closest_dist = dist;
                closest_z = z;
            }
        }

        closest_z
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangulate_simple_polygon() {
        let generator = FloodingAreaSurfaceGenerator {
            point_spacing: 10.0,
            sample_interior: true,
            group_by: None,
            buffer: Arc::new(Mutex::new(HashMap::new())),
            epsg: Arc::new(Mutex::new(None)),
        };

        let coords = vec![
            Coordinate3D::new__(0.0, 0.0, 1.0),
            Coordinate3D::new__(100.0, 0.0, 2.0),
            Coordinate3D::new__(100.0, 100.0, 3.0),
            Coordinate3D::new__(0.0, 100.0, 4.0),
            Coordinate3D::new__(0.0, 0.0, 1.0),
        ];
        let polygon = Polygon3D::new(LineString3D::new(coords), Vec::new());

        let triangles = generator.triangulate_polygon(&polygon).unwrap();
        assert!(!triangles.is_empty());
        // With interior grid sampling at 10m spacing for a 100x100 polygon,
        // we should have significantly more triangles than just boundary-based
        assert!(triangles.len() > 10);
    }

    #[test]
    fn test_point_in_polygon() {
        let coords = vec![
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(10.0, 0.0, 0.0),
            Coordinate3D::new__(10.0, 10.0, 0.0),
            Coordinate3D::new__(0.0, 10.0, 0.0),
            Coordinate3D::new__(0.0, 0.0, 0.0),
        ];
        let polygon = Polygon3D::new(LineString3D::new(coords), Vec::new());

        // Inside
        assert!(point_in_polygon(5.0, 5.0, &polygon));
        // Outside
        assert!(!point_in_polygon(15.0, 5.0, &polygon));
        assert!(!point_in_polygon(-1.0, 5.0, &polygon));
    }
}
