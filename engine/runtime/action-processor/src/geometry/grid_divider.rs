use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use rayon::prelude::*;
use reearth_flow_geometry::algorithm::bounding_rect::BoundingRect;
use reearth_flow_geometry::types::coordinate::{Coordinate2D, Coordinate3D};
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_geometry::types::line_string::{LineString2D, LineString3D};
use reearth_flow_geometry::types::multi_polygon::{MultiPolygon2D, MultiPolygon3D};
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_geometry::types::rect::Rect2D;
use reearth_flow_runtime::cache::executor_cache_subdir;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{
    Attribute, AttributeValue, CityGmlGeometry, Feature, Geometry, GeometryValue, GmlGeometry,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;
use crate::ACCUMULATOR_BUFFER_BYTE_THRESHOLD;

#[derive(Debug, Clone, Default)]
pub struct GridDividerFactory;

impl ProcessorFactory for GridDividerFactory {
    fn name(&self) -> &str {
        "GridDivider"
    }

    fn description(&self) -> &str {
        "Divide Polygons into Regular Grid Cells"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GridDividerParam))
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
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: GridDividerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::GridDividerFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::GridDividerFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::GridDividerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let unit_square_size = param.unit_square_size;

        if unit_square_size <= 0.0 {
            return Err(GeometryProcessorError::GridDividerFactory(format!(
                "unit_square_size must be positive, got: {}",
                unit_square_size
            ))
            .into());
        }

        let processor = GridDivider {
            unit_square_size,
            keep_square_only: param.keep_square_only.unwrap_or(false),
            group_by: param.group_by,
            bounds_per_group: HashMap::new(),
            group_map: HashMap::new(),
            group_keys: Vec::new(),
            group_count: 0,
            buffer: HashMap::new(),
            buffer_bytes: 0,
            temp_dir: None,
            executor_id: None,
        };

        Ok(Box::new(processor))
    }
}

/// # GridDivider Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GridDividerParam {
    /// # Unit Square Size
    /// Side length of each grid cell (in the same units as the geometry coordinates)
    pub unit_square_size: f64,
    /// # Keep Square Only
    /// If true, only output complete grid squares (discard edge pieces). Default: false
    pub keep_square_only: Option<bool>,
    /// # Group By Attributes
    /// Attributes used to group features - each group gets its own grid origin
    pub group_by: Option<Vec<Attribute>>,
}

pub struct GridDivider {
    unit_square_size: f64,
    keep_square_only: bool,
    group_by: Option<Vec<Attribute>>,

    // Disk-backed state
    bounds_per_group: HashMap<AttributeValue, Rect2D<f64>>,
    group_map: HashMap<AttributeValue, usize>,
    group_keys: Vec<AttributeValue>,
    group_count: usize,
    buffer: HashMap<usize, Vec<String>>,
    buffer_bytes: usize,
    temp_dir: Option<PathBuf>,
    executor_id: Option<uuid::Uuid>,
}

impl fmt::Debug for GridDivider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GridDivider")
            .field("unit_square_size", &self.unit_square_size)
            .field("group_count", &self.group_count)
            .field("buffer_bytes", &self.buffer_bytes)
            .field("temp_dir", &self.temp_dir)
            .finish()
    }
}

impl Clone for GridDivider {
    fn clone(&self) -> Self {
        Self {
            unit_square_size: self.unit_square_size,
            keep_square_only: self.keep_square_only,
            group_by: self.group_by.clone(),
            bounds_per_group: HashMap::new(),
            group_map: HashMap::new(),
            group_keys: Vec::new(),
            group_count: 0,
            buffer: HashMap::new(),
            buffer_bytes: 0,
            temp_dir: None,
            executor_id: None,
        }
    }
}

impl Drop for GridDivider {
    fn drop(&mut self) {
        if let Some(ref dir) = self.temp_dir {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

/// Represents a single grid cell
#[derive(Debug, Clone)]
struct GridCell {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
    row: usize,
    col: usize,
}

/// Result of clipping a geometry against a grid cell
struct ClipResult {
    geometry: GeometryValue,
    is_complete_square: bool,
}

/// Executor-specific engine cache folder for accumulating processors
fn engine_cache_dir(executor_id: uuid::Uuid) -> PathBuf {
    executor_cache_subdir(executor_id, "processors")
}

impl GridDivider {
    fn ensure_temp_dir(&mut self) -> Result<&PathBuf, BoxedError> {
        if self.temp_dir.is_none() {
            let executor_id = self.executor_id.unwrap_or_else(uuid::Uuid::nil);
            let dir = engine_cache_dir(executor_id)
                .join(format!("grid-divider-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&dir)?;
            self.temp_dir = Some(dir);
        }
        Ok(self.temp_dir.as_ref().unwrap())
    }

    fn flush_buffer(&mut self) -> Result<(), BoxedError> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let dir = self.ensure_temp_dir()?.clone();

        for (group_idx, lines) in self.buffer.drain() {
            let path = dir.join(format!("group_{group_idx:06}.jsonl"));
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
}

impl Processor for GridDivider {
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
        let geometry = &feature.geometry;

        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        let bounds_opt = get_geometry_bounds_2d(&geometry.value);

        match bounds_opt {
            Some(bounds) => {
                let key = if let Some(group_by) = &self.group_by {
                    AttributeValue::Array(
                        group_by
                            .iter()
                            .filter_map(|attr| feature.attributes.get(attr).cloned())
                            .collect(),
                    )
                } else {
                    AttributeValue::Null
                };

                // Update group bounds
                self.bounds_per_group
                    .entry(key.clone())
                    .and_modify(|existing| *existing = existing.merge(bounds))
                    .or_insert(bounds);

                // Get or assign group index
                let group_idx = if let Some(&idx) = self.group_map.get(&key) {
                    idx
                } else {
                    let idx = self.group_count;
                    self.group_map.insert(key.clone(), idx);
                    self.group_keys.push(key);
                    self.group_count += 1;
                    idx
                };

                // Serialize feature to buffer
                let json = serde_json::to_string(&feature).map_err(|e| {
                    GeometryProcessorError::GridDivider(format!("Failed to serialize feature: {e}"))
                })?;
                self.buffer_bytes += json.len();
                self.buffer.entry(group_idx).or_default().push(json);

                if self.buffer_bytes >= ACCUMULATOR_BUFFER_BYTE_THRESHOLD {
                    self.flush_buffer()?;
                }
            }
            None => {
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
        // Flush remaining buffer
        self.flush_buffer()?;
        self.buffer = HashMap::new();

        let dir = match &self.temp_dir {
            Some(d) => d.clone(),
            None => {
                // No data was received
                return Ok(());
            }
        };

        let group_keys = std::mem::take(&mut self.group_keys);
        let bounds_per_group = std::mem::take(&mut self.bounds_per_group);
        let group_map = std::mem::take(&mut self.group_map);

        let output_path = dir.join("output.jsonl");
        let mut output_writer = BufWriter::new(File::create(&output_path)?);
        let mut total_output = 0usize;

        for key in &group_keys {
            let group_idx = match group_map.get(key) {
                Some(&idx) => idx,
                None => continue,
            };
            let bounds = match bounds_per_group.get(key) {
                Some(b) => b,
                None => continue,
            };

            let group_path = dir.join(format!("group_{group_idx:06}.jsonl"));
            if !group_path.exists() {
                continue;
            }

            // Compute grid parameters from group bounds
            let grid_origin_x = bounds.min().x;
            let grid_origin_y = bounds.min().y;

            let file = File::open(&group_path)?;
            let reader = BufReader::new(file);

            // Process features in parallel chunks
            let mut chunk: Vec<Feature> = Vec::new();
            let mut chunk_bytes: usize = 0;
            let mut lines_iter = reader.lines();

            loop {
                let mut eof = false;
                while chunk_bytes < ACCUMULATOR_BUFFER_BYTE_THRESHOLD {
                    match lines_iter.next() {
                        Some(Ok(line)) => {
                            if line.is_empty() {
                                continue;
                            }
                            chunk_bytes += line.len();
                            let feature: Feature = serde_json::from_str(&line)?;
                            chunk.push(feature);
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

                let unit_size = self.unit_square_size;
                let keep_square_only = self.keep_square_only;
                let group_by = &self.group_by;

                // Process chunk in parallel
                let results: Vec<Feature> = chunk
                    .par_iter()
                    .flat_map(|feature| {
                        let feature_bounds = match get_geometry_bounds_2d(&feature.geometry.value) {
                            Some(b) => b,
                            None => return vec![],
                        };

                        // Compute overlapping cell range
                        let min_col =
                            ((feature_bounds.min().x - grid_origin_x) / unit_size).floor() as isize;
                        let max_col =
                            ((feature_bounds.max().x - grid_origin_x) / unit_size).ceil() as isize;
                        let min_row =
                            ((feature_bounds.min().y - grid_origin_y) / unit_size).floor() as isize;
                        let max_row =
                            ((feature_bounds.max().y - grid_origin_y) / unit_size).ceil() as isize;

                        let min_col = min_col.max(0) as usize;
                        let min_row = min_row.max(0) as usize;
                        let max_col = max_col.max(0) as usize;
                        let max_row = max_row.max(0) as usize;

                        let mut results = Vec::new();
                        for row in min_row..max_row {
                            for col in min_col..max_col {
                                let cell = GridCell {
                                    min_x: grid_origin_x + (col as f64) * unit_size,
                                    min_y: grid_origin_y + (row as f64) * unit_size,
                                    max_x: grid_origin_x + (col as f64 + 1.0) * unit_size,
                                    max_y: grid_origin_y + (row as f64 + 1.0) * unit_size,
                                    row,
                                    col,
                                };

                                for clip_result in
                                    clip_geometry_by_cell(&feature.geometry.value, &cell)
                                {
                                    if keep_square_only && !clip_result.is_complete_square {
                                        continue;
                                    }
                                    results.push(create_output_feature(
                                        feature,
                                        clip_result.geometry,
                                        &cell,
                                        group_by,
                                    ));
                                }
                            }
                        }
                        results
                    })
                    .collect();

                // Write results to output file
                for feature in &results {
                    let json = serde_json::to_string(feature)?;
                    output_writer.write_all(json.as_bytes())?;
                    output_writer.write_all(b"\n")?;
                }
                total_output += results.len();

                chunk.clear();
                chunk_bytes = 0;

                if eof {
                    break;
                }
            }
        }

        output_writer.flush()?;

        if total_output > 0 {
            fw.send_file(output_path, DEFAULT_PORT.clone(), ctx.as_context());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "GridDivider"
    }
}

/// Extract 2D bounding box from geometry value
fn get_geometry_bounds_2d(geometry: &GeometryValue) -> Option<Rect2D<f64>> {
    match geometry {
        GeometryValue::FlowGeometry2D(geo) => match geo {
            Geometry2D::Polygon(poly) => poly.bounding_rect(),
            Geometry2D::MultiPolygon(mpoly) => mpoly.bounding_rect(),
            _ => None,
        },
        GeometryValue::FlowGeometry3D(geo) => match geo {
            Geometry3D::Polygon(poly) => {
                // Convert 3D bounding rect to 2D
                poly.bounding_rect().map(Rect2D::from)
            }
            Geometry3D::MultiPolygon(mpoly) => mpoly.bounding_rect().map(Rect2D::from),
            _ => None,
        },
        GeometryValue::CityGmlGeometry(citygml) => {
            // Compute bounds from all polygons in CityGML
            let mut combined_bounds: Option<Rect2D<f64>> = None;
            for gml in &citygml.gml_geometries {
                for poly in &gml.polygons {
                    if let Some(rect) = poly.bounding_rect() {
                        let rect_2d = Rect2D::from(rect);
                        combined_bounds = Some(match combined_bounds {
                            Some(existing) => existing.merge(rect_2d),
                            None => rect_2d,
                        });
                    }
                }
            }
            combined_bounds
        }
        GeometryValue::None => None,
    }
}

/// Clip geometry by AABB cell
/// Returns a Vec because CityGmlGeometry may produce multiple output geometries (one per polygon)
fn clip_geometry_by_cell(geometry: &GeometryValue, cell: &GridCell) -> Vec<ClipResult> {
    match geometry {
        GeometryValue::FlowGeometry2D(geo) => clip_geometry_2d(geo, cell)
            .map(|clipped| {
                let is_complete = is_complete_square_2d(&clipped, cell);
                ClipResult {
                    geometry: GeometryValue::FlowGeometry2D(clipped),
                    is_complete_square: is_complete,
                }
            })
            .into_iter()
            .collect(),
        GeometryValue::FlowGeometry3D(geo) => clip_geometry_3d(geo, cell)
            .map(|clipped| {
                let is_complete = is_complete_square_3d(&clipped, cell);
                ClipResult {
                    geometry: GeometryValue::FlowGeometry3D(clipped),
                    is_complete_square: is_complete,
                }
            })
            .into_iter()
            .collect(),
        GeometryValue::CityGmlGeometry(citygml) => clip_citygml_geometry_per_polygon(citygml, cell),
        GeometryValue::None => vec![],
    }
}

/// Clip 2D geometry
fn clip_geometry_2d(geo: &Geometry2D<f64>, cell: &GridCell) -> Option<Geometry2D<f64>> {
    match geo {
        Geometry2D::Polygon(poly) => clip_polygon_2d(poly, cell).map(Geometry2D::Polygon),
        Geometry2D::MultiPolygon(mpoly) => {
            let clipped: Vec<Polygon2D<f64>> = mpoly
                .iter()
                .filter_map(|poly| clip_polygon_2d(poly, cell))
                .collect();
            if clipped.is_empty() {
                None
            } else {
                Some(Geometry2D::MultiPolygon(MultiPolygon2D::new(clipped)))
            }
        }
        _ => None,
    }
}

/// Clip 3D geometry
fn clip_geometry_3d(geo: &Geometry3D<f64>, cell: &GridCell) -> Option<Geometry3D<f64>> {
    match geo {
        Geometry3D::Polygon(poly) => clip_polygon_3d(poly, cell).map(Geometry3D::Polygon),
        Geometry3D::MultiPolygon(mpoly) => {
            let clipped: Vec<Polygon3D<f64>> = mpoly
                .iter()
                .filter_map(|poly| clip_polygon_3d(poly, cell))
                .collect();
            if clipped.is_empty() {
                None
            } else {
                Some(Geometry3D::MultiPolygon(MultiPolygon3D::new(clipped)))
            }
        }
        _ => None,
    }
}

/// Clip CityGML geometry and return one ClipResult per polygon
/// This ensures each output feature has exactly one polygon for PolygonNormalExtractor
fn clip_citygml_geometry_per_polygon(
    citygml: &CityGmlGeometry,
    cell: &GridCell,
) -> Vec<ClipResult> {
    let mut results = Vec::new();

    for gml in &citygml.gml_geometries {
        // Clip each polygon individually and create a separate CityGmlGeometry for each
        for poly in &gml.polygons {
            if let Some(clipped_poly) = clip_polygon_3d(poly, cell) {
                // Check if this clipped polygon is a complete square
                let is_complete = is_complete_square_3d_polygon(&clipped_poly, cell);

                let single_gml = GmlGeometry {
                    id: gml.id.clone(),
                    ty: gml.ty,
                    gml_trait: gml.gml_trait.clone(),
                    lod: gml.lod,
                    pos: 0,
                    len: 1,
                    points: gml.points.clone(),
                    polygons: vec![clipped_poly.clone()],
                    line_strings: vec![],
                    feature_id: gml.feature_id.clone(),
                    feature_type: gml.feature_type.clone(),
                    composite_surfaces: vec![],
                    polygon_ring_ids: vec![],
                };

                // Create placeholder UV polygon matching the structure of clipped_poly
                let uv_polygon = create_placeholder_uv_polygon(&clipped_poly);

                let single_citygml = CityGmlGeometry {
                    gml_geometries: vec![single_gml],
                    materials: citygml.materials.clone(),
                    textures: citygml.textures.clone(),
                    polygon_materials: vec![None],
                    polygon_textures: vec![None],
                    polygon_uvs: MultiPolygon2D::new(vec![uv_polygon]),
                };

                results.push(ClipResult {
                    geometry: GeometryValue::CityGmlGeometry(single_citygml),
                    is_complete_square: is_complete,
                });
            }
        }
    }

    results
}

/// Create a placeholder UV polygon that matches the ring structure of a 3D polygon.
fn create_placeholder_uv_polygon(poly3d: &Polygon3D<f64>) -> Polygon2D<f64> {
    let exterior_uv_coords: Vec<Coordinate2D<f64>> = poly3d
        .exterior()
        .0
        .iter()
        .map(|_| Coordinate2D::new_(0.0, 0.0))
        .collect();
    let exterior_uv = LineString2D::new(exterior_uv_coords);

    let interior_uvs: Vec<LineString2D<f64>> = poly3d
        .interiors()
        .iter()
        .map(|interior| {
            let coords: Vec<Coordinate2D<f64>> = interior
                .0
                .iter()
                .map(|_| Coordinate2D::new_(0.0, 0.0))
                .collect();
            LineString2D::new(coords)
        })
        .collect();

    Polygon2D::new(exterior_uv, interior_uvs)
}

/// Check if a 2D geometry is a complete square matching the grid cell
fn is_complete_square_2d(geo: &Geometry2D<f64>, cell: &GridCell) -> bool {
    match geo {
        Geometry2D::Polygon(poly) => is_complete_square_2d_polygon(poly, cell),
        Geometry2D::MultiPolygon(mpoly) => mpoly
            .iter()
            .all(|poly| is_complete_square_2d_polygon(poly, cell)),
        _ => false,
    }
}

/// Check if a 3D geometry is a complete square matching the grid cell (in XY)
fn is_complete_square_3d(geo: &Geometry3D<f64>, cell: &GridCell) -> bool {
    match geo {
        Geometry3D::Polygon(poly) => is_complete_square_3d_polygon(poly, cell),
        Geometry3D::MultiPolygon(mpoly) => mpoly
            .iter()
            .all(|poly| is_complete_square_3d_polygon(poly, cell)),
        _ => false,
    }
}

/// Check if a 2D polygon is a complete square matching the grid cell
fn is_complete_square_2d_polygon(poly: &Polygon2D<f64>, cell: &GridCell) -> bool {
    let exterior = &poly.exterior().0;
    if exterior.len() != 5 {
        return false;
    }

    if !poly.interiors().is_empty() {
        return false;
    }

    let cell_corners = [
        (cell.min_x, cell.min_y),
        (cell.max_x, cell.min_y),
        (cell.max_x, cell.max_y),
        (cell.min_x, cell.max_y),
    ];

    let tolerance = 1e-9;
    for (cx, cy) in &cell_corners {
        let found = exterior
            .iter()
            .take(4)
            .any(|coord| (coord.x - cx).abs() < tolerance && (coord.y - cy).abs() < tolerance);
        if !found {
            return false;
        }
    }

    true
}

/// Check if a 3D polygon is a complete square matching the grid cell (in XY)
fn is_complete_square_3d_polygon(poly: &Polygon3D<f64>, cell: &GridCell) -> bool {
    let exterior = &poly.exterior().0;
    if exterior.len() != 5 {
        return false;
    }

    if !poly.interiors().is_empty() {
        return false;
    }

    let cell_corners = [
        (cell.min_x, cell.min_y),
        (cell.max_x, cell.min_y),
        (cell.max_x, cell.max_y),
        (cell.min_x, cell.max_y),
    ];

    let tolerance = 1e-9;
    for (cx, cy) in &cell_corners {
        let found = exterior
            .iter()
            .take(4)
            .any(|coord| (coord.x - cx).abs() < tolerance && (coord.y - cy).abs() < tolerance);
        if !found {
            return false;
        }
    }

    true
}

/// Clip a 2D polygon against an AABB using Sutherland-Hodgman algorithm
fn clip_polygon_2d(polygon: &Polygon2D<f64>, cell: &GridCell) -> Option<Polygon2D<f64>> {
    if let Some(poly_bounds) = polygon.bounding_rect() {
        let cell_bounds = Rect2D::new(
            Coordinate2D::new_(cell.min_x, cell.min_y),
            Coordinate2D::new_(cell.max_x, cell.max_y),
        );
        if !poly_bounds.overlap(&cell_bounds) {
            return None;
        }
    }

    let exterior_coords: Vec<Coordinate2D<f64>> = polygon.exterior().0.to_vec();
    let clipped_exterior = clip_polygon_coords_2d(&exterior_coords, cell)?;

    if clipped_exterior.len() < 3 {
        return None;
    }

    let clipped_interiors: Vec<LineString2D<f64>> = polygon
        .interiors()
        .iter()
        .filter_map(|interior| {
            let coords: Vec<Coordinate2D<f64>> = interior.0.to_vec();
            clip_polygon_coords_2d(&coords, cell)
                .filter(|clipped| clipped.len() >= 3)
                .map(LineString2D::new)
        })
        .collect();

    Some(Polygon2D::new(
        LineString2D::new(clipped_exterior),
        clipped_interiors,
    ))
}

/// Clip a 3D polygon against an AABB using Sutherland-Hodgman algorithm (XY plane clipping with Z interpolation)
fn clip_polygon_3d(polygon: &Polygon3D<f64>, cell: &GridCell) -> Option<Polygon3D<f64>> {
    if let Some(poly_bounds) = polygon.bounding_rect() {
        let poly_bounds_2d = Rect2D::from(poly_bounds);
        let cell_bounds = Rect2D::new(
            Coordinate2D::new_(cell.min_x, cell.min_y),
            Coordinate2D::new_(cell.max_x, cell.max_y),
        );
        if !poly_bounds_2d.overlap(&cell_bounds) {
            return None;
        }
    }

    let exterior_coords: Vec<Coordinate3D<f64>> = polygon.exterior().0.to_vec();
    let clipped_exterior = clip_polygon_coords_3d(&exterior_coords, cell)?;

    if clipped_exterior.len() < 3 {
        return None;
    }

    let clipped_interiors: Vec<LineString3D<f64>> = polygon
        .interiors()
        .iter()
        .filter_map(|interior| {
            let coords: Vec<Coordinate3D<f64>> = interior.0.to_vec();
            clip_polygon_coords_3d(&coords, cell)
                .filter(|clipped| clipped.len() >= 3)
                .map(LineString3D::new)
        })
        .collect();

    Some(Polygon3D::new(
        LineString3D::new(clipped_exterior),
        clipped_interiors,
    ))
}

/// Sutherland-Hodgman clipping for 2D coordinates
fn clip_polygon_coords_2d(
    coords: &[Coordinate2D<f64>],
    cell: &GridCell,
) -> Option<Vec<Coordinate2D<f64>>> {
    if coords.is_empty() {
        return None;
    }

    let mut output = coords.to_vec();

    // Remove last point if it duplicates the first (closing point)
    if output.len() > 1 && coords_equal_2d(&output[0], output.last().unwrap()) {
        output.pop();
    }

    if output.is_empty() {
        return None;
    }

    // Clip against each edge: left, right, bottom, top
    output = clip_against_edge_2d(output, Edge::Left(cell.min_x));
    if output.is_empty() {
        return None;
    }

    output = clip_against_edge_2d(output, Edge::Right(cell.max_x));
    if output.is_empty() {
        return None;
    }

    output = clip_against_edge_2d(output, Edge::Bottom(cell.min_y));
    if output.is_empty() {
        return None;
    }

    output = clip_against_edge_2d(output, Edge::Top(cell.max_y));
    if output.is_empty() {
        return None;
    }

    // Close the polygon
    if output.len() >= 3 && !coords_equal_2d(&output[0], output.last().unwrap()) {
        output.push(output[0]);
    }

    if output.len() < 4 {
        return None;
    }

    Some(output)
}

/// Sutherland-Hodgman clipping for 3D coordinates (XY clipping with Z interpolation)
fn clip_polygon_coords_3d(
    coords: &[Coordinate3D<f64>],
    cell: &GridCell,
) -> Option<Vec<Coordinate3D<f64>>> {
    if coords.is_empty() {
        return None;
    }

    let mut output = coords.to_vec();

    if output.len() > 1 && coords_equal_3d(&output[0], output.last().unwrap()) {
        output.pop();
    }

    if output.is_empty() {
        return None;
    }

    output = clip_against_edge_3d(output, Edge::Left(cell.min_x));
    if output.is_empty() {
        return None;
    }

    output = clip_against_edge_3d(output, Edge::Right(cell.max_x));
    if output.is_empty() {
        return None;
    }

    output = clip_against_edge_3d(output, Edge::Bottom(cell.min_y));
    if output.is_empty() {
        return None;
    }

    output = clip_against_edge_3d(output, Edge::Top(cell.max_y));
    if output.is_empty() {
        return None;
    }

    if output.len() >= 3 && !coords_equal_3d(&output[0], output.last().unwrap()) {
        output.push(output[0]);
    }

    if output.len() < 4 {
        return None;
    }

    Some(output)
}

#[derive(Debug, Clone, Copy)]
enum Edge {
    Left(f64),
    Right(f64),
    Bottom(f64),
    Top(f64),
}

fn is_inside_2d(coord: &Coordinate2D<f64>, edge: Edge) -> bool {
    match edge {
        Edge::Left(x) => coord.x >= x,
        Edge::Right(x) => coord.x <= x,
        Edge::Bottom(y) => coord.y >= y,
        Edge::Top(y) => coord.y <= y,
    }
}

fn is_inside_3d(coord: &Coordinate3D<f64>, edge: Edge) -> bool {
    match edge {
        Edge::Left(x) => coord.x >= x,
        Edge::Right(x) => coord.x <= x,
        Edge::Bottom(y) => coord.y >= y,
        Edge::Top(y) => coord.y <= y,
    }
}

fn intersect_2d(p1: &Coordinate2D<f64>, p2: &Coordinate2D<f64>, edge: Edge) -> Coordinate2D<f64> {
    let t = compute_t_2d(p1, p2, edge);
    Coordinate2D::new_(p1.x + t * (p2.x - p1.x), p1.y + t * (p2.y - p1.y))
}

fn intersect_3d(p1: &Coordinate3D<f64>, p2: &Coordinate3D<f64>, edge: Edge) -> Coordinate3D<f64> {
    let t = compute_t_3d(p1, p2, edge);
    Coordinate3D {
        x: p1.x + t * (p2.x - p1.x),
        y: p1.y + t * (p2.y - p1.y),
        z: p1.z + t * (p2.z - p1.z), // Interpolate Z
    }
}

fn compute_t_2d(p1: &Coordinate2D<f64>, p2: &Coordinate2D<f64>, edge: Edge) -> f64 {
    match edge {
        Edge::Left(x) | Edge::Right(x) => {
            if (p2.x - p1.x).abs() < f64::EPSILON {
                0.5
            } else {
                (x - p1.x) / (p2.x - p1.x)
            }
        }
        Edge::Bottom(y) | Edge::Top(y) => {
            if (p2.y - p1.y).abs() < f64::EPSILON {
                0.5
            } else {
                (y - p1.y) / (p2.y - p1.y)
            }
        }
    }
}

fn compute_t_3d(p1: &Coordinate3D<f64>, p2: &Coordinate3D<f64>, edge: Edge) -> f64 {
    match edge {
        Edge::Left(x) | Edge::Right(x) => {
            if (p2.x - p1.x).abs() < f64::EPSILON {
                0.5
            } else {
                (x - p1.x) / (p2.x - p1.x)
            }
        }
        Edge::Bottom(y) | Edge::Top(y) => {
            if (p2.y - p1.y).abs() < f64::EPSILON {
                0.5
            } else {
                (y - p1.y) / (p2.y - p1.y)
            }
        }
    }
}

fn clip_against_edge_2d(polygon: Vec<Coordinate2D<f64>>, edge: Edge) -> Vec<Coordinate2D<f64>> {
    if polygon.is_empty() {
        return vec![];
    }

    let mut output = Vec::new();
    let n = polygon.len();

    for i in 0..n {
        let current = &polygon[i];
        let next = &polygon[(i + 1) % n];

        let current_inside = is_inside_2d(current, edge);
        let next_inside = is_inside_2d(next, edge);

        match (current_inside, next_inside) {
            (true, true) => {
                output.push(*next);
            }
            (true, false) => {
                output.push(intersect_2d(current, next, edge));
            }
            (false, true) => {
                output.push(intersect_2d(current, next, edge));
                output.push(*next);
            }
            (false, false) => {}
        }
    }

    output
}

fn clip_against_edge_3d(polygon: Vec<Coordinate3D<f64>>, edge: Edge) -> Vec<Coordinate3D<f64>> {
    if polygon.is_empty() {
        return vec![];
    }

    let mut output = Vec::new();
    let n = polygon.len();

    for i in 0..n {
        let current = &polygon[i];
        let next = &polygon[(i + 1) % n];

        let current_inside = is_inside_3d(current, edge);
        let next_inside = is_inside_3d(next, edge);

        match (current_inside, next_inside) {
            (true, true) => {
                output.push(*next);
            }
            (true, false) => {
                output.push(intersect_3d(current, next, edge));
            }
            (false, true) => {
                output.push(intersect_3d(current, next, edge));
                output.push(*next);
            }
            (false, false) => {}
        }
    }

    output
}

fn coords_equal_2d(a: &Coordinate2D<f64>, b: &Coordinate2D<f64>) -> bool {
    (a.x - b.x).abs() < f64::EPSILON && (a.y - b.y).abs() < f64::EPSILON
}

fn coords_equal_3d(a: &Coordinate3D<f64>, b: &Coordinate3D<f64>) -> bool {
    (a.x - b.x).abs() < f64::EPSILON
        && (a.y - b.y).abs() < f64::EPSILON
        && (a.z - b.z).abs() < f64::EPSILON
}

/// Create output feature with all original attributes and grid metadata
fn create_output_feature(
    original: &Feature,
    clipped_geometry: GeometryValue,
    cell: &GridCell,
    _group_by: &Option<Vec<Attribute>>,
) -> Feature {
    let new_geometry = Geometry {
        epsg: original.geometry.epsg,
        value: clipped_geometry,
    };
    let mut new_feature = Feature::new_with_attributes_and_geometry(
        (*original.attributes).clone(),
        new_geometry,
        original.metadata.clone(),
    );

    new_feature.metadata = original.metadata.clone();

    new_feature.insert(
        "_grid_row",
        AttributeValue::Number(serde_json::Number::from(cell.row as i64)),
    );
    new_feature.insert(
        "_grid_col",
        AttributeValue::Number(serde_json::Number::from(cell.col as i64)),
    );

    new_feature
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::Attributes;

    fn create_test_polygon_2d() -> Polygon2D<f64> {
        let exterior = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(10.0, 0.0),
            Coordinate2D::new_(10.0, 10.0),
            Coordinate2D::new_(0.0, 10.0),
            Coordinate2D::new_(0.0, 0.0),
        ]);
        Polygon2D::new(exterior, vec![])
    }

    fn create_test_polygon_3d() -> Polygon3D<f64> {
        let exterior = LineString3D::new(vec![
            Coordinate3D {
                x: 0.0,
                y: 0.0,
                z: 5.0,
            },
            Coordinate3D {
                x: 10.0,
                y: 0.0,
                z: 5.0,
            },
            Coordinate3D {
                x: 10.0,
                y: 10.0,
                z: 15.0,
            },
            Coordinate3D {
                x: 0.0,
                y: 10.0,
                z: 15.0,
            },
            Coordinate3D {
                x: 0.0,
                y: 0.0,
                z: 5.0,
            },
        ]);
        Polygon3D::new(exterior, vec![])
    }

    #[test]
    fn test_clip_polygon_2d_fully_inside() {
        let polygon = create_test_polygon_2d();
        let cell = GridCell {
            min_x: -5.0,
            min_y: -5.0,
            max_x: 15.0,
            max_y: 15.0,
            row: 0,
            col: 0,
        };

        let clipped = clip_polygon_2d(&polygon, &cell);
        assert!(clipped.is_some());

        let clipped = clipped.unwrap();
        assert_eq!(clipped.exterior().0.len(), 5);
    }

    #[test]
    fn test_clip_polygon_2d_partial() {
        let polygon = create_test_polygon_2d();
        let cell = GridCell {
            min_x: 2.0,
            min_y: 2.0,
            max_x: 8.0,
            max_y: 8.0,
            row: 0,
            col: 0,
        };

        let clipped = clip_polygon_2d(&polygon, &cell);
        assert!(clipped.is_some());

        let clipped = clipped.unwrap();
        assert_eq!(clipped.exterior().0.len(), 5);
    }

    #[test]
    fn test_clip_polygon_2d_outside() {
        let polygon = create_test_polygon_2d();
        let cell = GridCell {
            min_x: 20.0,
            min_y: 20.0,
            max_x: 30.0,
            max_y: 30.0,
            row: 0,
            col: 0,
        };

        let clipped = clip_polygon_2d(&polygon, &cell);
        assert!(clipped.is_none());
    }

    #[test]
    fn test_clip_polygon_3d_with_z_interpolation() {
        let polygon = create_test_polygon_3d();
        let cell = GridCell {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 5.0,
            max_y: 5.0,
            row: 0,
            col: 0,
        };

        let clipped = clip_polygon_3d(&polygon, &cell);
        assert!(clipped.is_some());

        let clipped = clipped.unwrap();
        let z_values: Vec<f64> = clipped.exterior().0.iter().map(|c| c.z).collect();

        assert!(!z_values.is_empty());
    }

    #[test]
    fn test_clip_polygon_with_hole() {
        let exterior = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(20.0, 0.0),
            Coordinate2D::new_(20.0, 20.0),
            Coordinate2D::new_(0.0, 20.0),
            Coordinate2D::new_(0.0, 0.0),
        ]);
        let hole = LineString2D::new(vec![
            Coordinate2D::new_(5.0, 5.0),
            Coordinate2D::new_(15.0, 5.0),
            Coordinate2D::new_(15.0, 15.0),
            Coordinate2D::new_(5.0, 15.0),
            Coordinate2D::new_(5.0, 5.0),
        ]);
        let polygon = Polygon2D::new(exterior, vec![hole]);

        let cell = GridCell {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 10.0,
            max_y: 10.0,
            row: 0,
            col: 0,
        };

        let clipped = clip_polygon_2d(&polygon, &cell);
        assert!(clipped.is_some());

        let clipped = clipped.unwrap();
        assert!(!clipped.exterior().0.is_empty());
    }

    #[test]
    fn test_is_complete_square() {
        let cell = GridCell {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 5.0,
            max_y: 5.0,
            row: 0,
            col: 0,
        };

        let complete_poly = Polygon2D::new(
            LineString2D::new(vec![
                Coordinate2D::new_(0.0, 0.0),
                Coordinate2D::new_(5.0, 0.0),
                Coordinate2D::new_(5.0, 5.0),
                Coordinate2D::new_(0.0, 5.0),
                Coordinate2D::new_(0.0, 0.0),
            ]),
            vec![],
        );
        assert!(is_complete_square_2d_polygon(&complete_poly, &cell));

        let partial_poly = Polygon2D::new(
            LineString2D::new(vec![
                Coordinate2D::new_(0.0, 0.0),
                Coordinate2D::new_(3.0, 0.0),
                Coordinate2D::new_(5.0, 5.0),
                Coordinate2D::new_(0.0, 5.0),
                Coordinate2D::new_(0.0, 0.0),
            ]),
            vec![],
        );
        assert!(!is_complete_square_2d_polygon(&partial_poly, &cell));

        let triangle = Polygon2D::new(
            LineString2D::new(vec![
                Coordinate2D::new_(0.0, 0.0),
                Coordinate2D::new_(5.0, 0.0),
                Coordinate2D::new_(5.0, 5.0),
                Coordinate2D::new_(0.0, 0.0),
            ]),
            vec![],
        );
        assert!(!is_complete_square_2d_polygon(&triangle, &cell));
    }

    #[test]
    fn test_get_geometry_bounds_2d() {
        let polygon = create_test_polygon_2d();
        let geo = GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon));

        let bounds = get_geometry_bounds_2d(&geo);
        assert!(bounds.is_some());

        let bounds = bounds.unwrap();
        assert!((bounds.min().x - 0.0).abs() < f64::EPSILON);
        assert!((bounds.min().y - 0.0).abs() < f64::EPSILON);
        assert!((bounds.max().x - 10.0).abs() < f64::EPSILON);
        assert!((bounds.max().y - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_geometry_bounds_3d() {
        let polygon = create_test_polygon_3d();
        let geo = GeometryValue::FlowGeometry3D(Geometry3D::Polygon(polygon));

        let bounds = get_geometry_bounds_2d(&geo);
        assert!(bounds.is_some());

        let bounds = bounds.unwrap();
        assert!((bounds.min().x - 0.0).abs() < f64::EPSILON);
        assert!((bounds.min().y - 0.0).abs() < f64::EPSILON);
        assert!((bounds.max().x - 10.0).abs() < f64::EPSILON);
        assert!((bounds.max().y - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_create_output_feature() {
        let mut original = Feature::new_with_attributes(Attributes::default());
        original.insert("group_attr", AttributeValue::String("test".to_string()));
        original.insert(
            "other_attr",
            AttributeValue::String("also_kept".to_string()),
        );

        let cell = GridCell {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 5.0,
            max_y: 5.0,
            row: 1,
            col: 2,
        };

        let group_by = Some(vec![Attribute::new("group_attr")]);

        let output = create_output_feature(&original, GeometryValue::None, &cell, &group_by);

        assert!(output
            .attributes
            .contains_key(&Attribute::new("group_attr")));
        assert!(output
            .attributes
            .contains_key(&Attribute::new("other_attr")));
        assert!(output.attributes.contains_key(&Attribute::new("_grid_row")));
        assert!(output.attributes.contains_key(&Attribute::new("_grid_col")));

        assert_eq!(
            output.attributes.get(&Attribute::new("_grid_row")),
            Some(&AttributeValue::Number(serde_json::Number::from(1)))
        );
        assert_eq!(
            output.attributes.get(&Attribute::new("_grid_col")),
            Some(&AttributeValue::Number(serde_json::Number::from(2)))
        );
    }
}
