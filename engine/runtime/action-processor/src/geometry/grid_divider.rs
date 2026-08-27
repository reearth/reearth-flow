use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use rayon::prelude::*;
#[cfg(feature = "new-geometry")]
use reearth_flow_diagnostics::{DiagnosticDraft, ErrorCode};
use reearth_flow_geometry::algorithm::bounding_rect::BoundingRect;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::coordinate::UnitKind;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::{
    Aabb, BoundingBox, CellCoverage, DivideByGrid, GridDivideError, GridSpec,
};
use reearth_flow_geometry::types::coordinate::{Coordinate2D, Coordinate3D};
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_geometry::types::line_string::{LineString2D, LineString3D};
use reearth_flow_geometry::types::multi_polygon::{MultiPolygon2D, MultiPolygon3D};
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_geometry::types::rect::Rect2D;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::Geometry;
use reearth_flow_runtime::cache::executor_cache_subdir;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
#[cfg(feature = "new-geometry")]
use reearth_flow_types::Attributes;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::Geometry;
use reearth_flow_types::{
    Attribute, AttributeValue, CityGmlGeometry, Feature, GeometryValue, GmlGeometry,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;
use crate::ACCUMULATOR_BUFFER_BYTE_THRESHOLD;

/// The most cells one grid may span before the run stops.
///
/// Bounds a feature's own extent on the streaming path and a group's combined
/// extent on the accumulating one; both are "how many cells did this cell size
/// just ask for", which is the question worth refusing.
///
/// A cell size is typed by hand, and a slip of three decimal places turns a
/// city-sized extent into more cells than can ever be produced. Failing here
/// with the number in hand beats grinding to a halt with no explanation.
#[cfg(feature = "new-geometry")]
pub(super) const MAX_CELLS_PER_GRID: u128 = 50_000_000;

/// The group a feature belongs to.
///
/// Every attribute in `group_by` contributes a slot, with `Null` where the
/// feature does not carry it, so a feature missing an attribute cannot collapse
/// into a group it does not belong to.
#[cfg(feature = "new-geometry")]
fn group_key(attributes: &Attributes, group_by: &Option<Vec<Attribute>>) -> AttributeValue {
    match group_by {
        None => AttributeValue::Null,
        Some(attrs) => AttributeValue::Array(
            attrs
                .iter()
                .map(|a| attributes.get(a).cloned().unwrap_or(AttributeValue::Null))
                .collect(),
        ),
    }
}

#[derive(Debug, Clone, Default)]
pub struct GridDividerFactory;

impl ProcessorFactory for GridDividerFactory {
    fn name(&self) -> &str {
        "Grid Divider"
    }

    fn description(&self) -> &str {
        "Divides polygon geometries into a regular grid of equal-sized cells."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GridDividerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["spatial"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), REJECTED_PORT.clone()]
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

        let cell_size = param.cell_size;

        if cell_size <= 0.0 {
            return Err(GeometryProcessorError::GridDividerFactory(format!(
                "cell_size must be positive, got: {}",
                cell_size
            ))
            .into());
        }

        let processor = GridDivider {
            cell_size,
            complete_cells_only: param.complete_cells_only.unwrap_or(false),
            group_by: param.group_by,
            origin: param.origin,
            angular_warned: false,
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

/// # Grid Divider Parameters
/// Configure the size of the grid cells and how features are grouped onto a shared grid.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GridDividerParam {
    /// # Cell Size
    /// Side length of each grid cell, in the same units as the geometry coordinates.
    /// Must be greater than zero.
    pub cell_size: f64,
    /// # Complete Cells Only
    /// Whether to emit only cells that are whole, discarding the partial cells left
    /// where the grid meets the edge of a geometry. Defaults to false.
    pub complete_cells_only: Option<bool>,
    /// # Group By Attributes
    /// Attributes whose values group features together. Each group is divided on
    /// its own grid origin, derived from that group's combined bounds.
    pub group_by: Option<Vec<Attribute>>,
    /// # Grid Origin
    /// The point the grid is anchored at, as `[x, y]` in the same coordinate
    /// system as the geometry. When set, cells line up with this point, so
    /// separate Grid Dividers can share a lattice and repeat runs place features
    /// in the same cells. When left out, each group's grid starts at the corner
    /// of that group's own extent, which shifts if the input changes.
    pub origin: Option<[f64; 2]>,
}

pub struct GridDivider {
    cell_size: f64,
    complete_cells_only: bool,
    group_by: Option<Vec<Attribute>>,
    origin: Option<[f64; 2]>,
    // Whether the angular-frame warning has already fired for this processor,
    // so a stream of features on a degree-based grid warns once rather than
    // once per feature. Read only on the streaming (explicit-origin) path.
    #[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
    angular_warned: bool,

    // Disk-backed state
    bounds_per_group: HashMap<AttributeValue, Rect2D<f64>>,
    group_map: HashMap<AttributeValue, usize>,
    group_keys: Vec<AttributeValue>,
    group_count: usize,
    // In-memory buffers: group_idx -> compressed zstd bytes (concatenated frames)
    buffer: HashMap<usize, Vec<u8>>,
    buffer_bytes: usize,
    temp_dir: Option<PathBuf>,
    executor_id: Option<uuid::Uuid>,
}

impl fmt::Debug for GridDivider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Grid Divider")
            .field("cell_size", &self.cell_size)
            .field("group_count", &self.group_count)
            .field("buffer_bytes", &self.buffer_bytes)
            .field("temp_dir", &self.temp_dir)
            .finish()
    }
}

impl Clone for GridDivider {
    fn clone(&self) -> Self {
        Self {
            cell_size: self.cell_size,
            complete_cells_only: self.complete_cells_only,
            group_by: self.group_by.clone(),
            origin: self.origin,
            angular_warned: false,
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

#[cfg(test)]
impl GridDivider {
    /// A `GridDivider` with every field at its zero value, for tests that only
    /// care about a few fields and would otherwise have to repeat the rest.
    ///
    /// Only the new-geometry tests use this today (Task 8's accumulating tests
    /// are expected to as well), so it is unused — not an error — when the
    /// crate is tested without the `new-geometry` feature.
    #[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
    fn empty() -> Self {
        Self {
            cell_size: 1.0,
            complete_cells_only: false,
            group_by: None,
            origin: None,
            angular_warned: false,
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

        for (group_idx, bytes) in self.buffer.drain() {
            let path = dir.join(format!("group_{group_idx:06}.jsonl.zst"));
            let mut file = File::options().create(true).append(true).open(&path)?;
            file.write_all(&bytes)?;
        }

        self.buffer_bytes = 0;
        Ok(())
    }
}

impl Processor for GridDivider {
    #[cfg(not(feature = "new-geometry"))]
    fn is_accumulating(&self) -> bool {
        true
    }

    #[cfg(feature = "new-geometry")]
    fn is_accumulating(&self) -> bool {
        // With an origin in hand there is nothing to learn from the whole
        // stream, so features can be divided as they arrive.
        self.origin.is_none()
    }

    #[cfg(not(feature = "new-geometry"))]
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

                let json = serde_json::to_string(&feature).map_err(|e| {
                    GeometryProcessorError::GridDivider(format!("Failed to serialize feature: {e}"))
                })?;
                self.buffer_bytes += json.len();
                let mut src = json.into_bytes();
                src.push(b'\n');
                let frame = zstd::encode_all(src.as_slice(), 1)?;
                self.buffer.entry(group_idx).or_default().extend(frame);

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

    #[cfg(not(feature = "new-geometry"))]
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

        let output_path = dir.join("output.jsonl.zst");
        let mut output_writer = BufWriter::new(zstd::Encoder::new(File::create(&output_path)?, 1)?);
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

            let group_path = dir.join(format!("group_{group_idx:06}.jsonl.zst"));
            if !group_path.exists() {
                continue;
            }

            // Compute grid parameters from group bounds
            let grid_origin_x = bounds.min().x;
            let grid_origin_y = bounds.min().y;

            let file = File::open(&group_path)?;
            let reader = BufReader::new(zstd::Decoder::new(file)?);

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

                let unit_size = self.cell_size;
                let complete_cells_only = self.complete_cells_only;
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
                                    if complete_cells_only && !clip_result.is_complete_square {
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

        output_writer
            .into_inner()
            .map_err(|e| e.into_error())?
            .finish()?;

        if total_output > 0 {
            fw.send_file(output_path, FEATURES_PORT.clone(), ctx.as_context());
        }

        Ok(())
    }

    /// Divide the feature onto the grid and send one feature per cell.
    ///
    /// Only the streaming half lives here. Without an explicit origin the grid
    /// is not known until every feature's extent has been seen, so `process`
    /// spools instead and `finish` does the dividing.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let Some(origin) = self.origin else {
            return self.spool(ctx, fw);
        };

        let grid = GridSpec::new(origin, self.cell_size)
            .map_err(|e| GeometryProcessorError::GridDivider(e.to_string()))?;

        self.warn_if_angular(&ctx);
        self.guard_cell_count(&ctx.feature.geometry, &grid)?;

        let complete_only = self.complete_cells_only;
        let feature = &ctx.feature;
        let mut sent = 0usize;
        let result = feature
            .geometry
            .divide_by_grid(&grid, &mut |cell, coverage, piece| {
                if complete_only && coverage != CellCoverage::Full {
                    return;
                }
                let mut out = feature.clone();
                out.set_geometry(piece);
                out.insert(
                    "_grid_row",
                    AttributeValue::Number(serde_json::Number::from(cell.row)),
                );
                out.insert(
                    "_grid_col",
                    AttributeValue::Number(serde_json::Number::from(cell.col)),
                );
                fw.send(ctx.new_with_feature_and_port(out, FEATURES_PORT.clone()));
                sent += 1;
            });

        match result {
            Ok(()) if sent > 0 => Ok(()),
            // Divided into nothing, or had nothing to divide: the feature
            // leaves with a reason attached rather than vanishing.
            Ok(()) => {
                self.reject(&ctx, fw, "geometry produced no cells");
                Ok(())
            }
            Err(e) => {
                self.reject_with(&ctx, fw, e);
                Ok(())
            }
        }
    }

    /// The streaming path (explicit origin) divides and emits everything in
    /// `process`, so there is nothing left to do at end of stream. The
    /// accumulating path (Task 8) will do its work here instead.
    #[cfg(feature = "new-geometry")]
    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Grid Divider"
    }
}

#[cfg(feature = "new-geometry")]
impl GridDivider {
    /// Accumulate features until every group's extent has been seen, so a grid
    /// can be derived per group without an explicit origin.
    ///
    /// Stub: the accumulating path is Task 8's to build. Until then, a Grid
    /// Divider run with no `origin` on the new-geometry path fails loudly here
    /// rather than silently passing features through unchanged or hanging.
    fn spool(
        &mut self,
        _ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Err(GeometryProcessorError::GridDivider(
            "Grid Divider without an explicit `origin` is not yet implemented on the new \
             geometry path"
                .to_string(),
        )
        .into())
    }

    /// Stop the run when a cell size implies more cells than could be meant.
    fn guard_cell_count(&self, geometry: &Geometry, grid: &GridSpec) -> Result<(), BoxedError> {
        let Ok(aabb) = geometry.bounding_box() else {
            return Ok(());
        };
        let (min, max) = match aabb {
            Aabb::D2 { min, max } => (min, max),
            Aabb::D3 { min, max } => ([min[0], min[1]], [max[0], max[1]]),
        };
        let count = grid.cell_count(min, max);
        if count > MAX_CELLS_PER_GRID {
            return Err(GeometryProcessorError::GridDivider(format!(
                "cell size {} over an extent of {:.1} x {:.1} implies {} cells, \
                 more than the limit of {}. Check the cell size is in the units \
                 of the data's coordinate system.",
                self.cell_size,
                max[0] - min[0],
                max[1] - min[1],
                count,
                MAX_CELLS_PER_GRID,
            ))
            .into());
        }
        Ok(())
    }

    /// Say so once when the grid is measured in degrees rather than metres.
    ///
    /// A feature whose frame is not angular, or whose frame cannot be
    /// determined at all (`Geometry::frame()` returns `None` on disagreement
    /// or when nothing exposes one), leaves the check armed: only a feature
    /// that is actually found angular latches `angular_warned`, so an early
    /// non-angular or indeterminate feature can never permanently silence a
    /// later, genuinely angular one.
    fn warn_if_angular(&mut self, ctx: &ExecutorContext) {
        if self.angular_warned {
            return;
        }
        if let Some(frame) = ctx.feature.geometry.frame() {
            if matches!(frame.unit_kind(), UnitKind::Angular) {
                self.angular_warned = true;
                ctx.warn(DiagnosticDraft::new(ErrorCode::GridAngularFrame));
            }
        }
    }

    /// Send `ctx`'s feature to `rejected` with `why` recorded on `_grid_error`.
    fn reject(&self, ctx: &ExecutorContext, fw: &ProcessorChannelForwarder, why: &str) {
        let mut out = ctx.feature.clone();
        out.insert("_grid_error", AttributeValue::String(why.to_string()));
        fw.send(ctx.new_with_feature_and_port(out, REJECTED_PORT.clone()));
    }

    /// As [`reject`](Self::reject), plus the diagnostic code a `GridDivideError`
    /// implies. Uses `ctx.warn`, not `ctx.report`: the feature is already being
    /// routed to `rejected` explicitly here, so a disposition-driven drop would
    /// be redundant at best and double-count at worst.
    fn reject_with(
        &self,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
        e: GridDivideError,
    ) {
        match &e {
            GridDivideError::MixedFrames => {
                ctx.warn(DiagnosticDraft::new(ErrorCode::GridMixedFrames))
            }
            _ => ctx.warn(DiagnosticDraft::new(ErrorCode::GridUnsupportedGeometry)),
        }
        self.reject(ctx, fw, &e.to_string());
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
#[cfg(not(feature = "new-geometry"))]
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
    let mut new_feature =
        Feature::new_with_attributes_and_geometry((*original.attributes).clone(), new_geometry);

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

    #[cfg(not(feature = "new-geometry"))]
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

#[cfg(all(test, feature = "new-geometry"))]
mod new_geometry_tests {
    use super::*;

    #[test]
    fn explicit_origin_makes_the_processor_streaming() {
        // Built via `empty()` then direct field assignment, not struct-update
        // syntax (`..base`): `GridDivider` implements `Drop`, and Rust refuses
        // to move fields out of a `Drop` type via `..base` (E0509). Direct
        // assignment on an owned value has no such restriction and keeps
        // `Drop` unconditional.
        let mut streaming = GridDivider::empty();
        streaming.cell_size = 1.0;
        streaming.complete_cells_only = false;
        streaming.group_by = None;
        streaming.origin = Some([0.0, 0.0]);
        assert!(!streaming.is_accumulating());

        let mut accumulating = streaming.clone();
        accumulating.origin = None;
        assert!(accumulating.is_accumulating());
    }

    #[test]
    fn cell_count_guard_rejects_an_absurd_cell_size() {
        // 4 km by 3 km at 1 mm cells is 1.2e13 cells.
        let grid = GridSpec::new([0.0, 0.0], 0.001).expect("valid spec");
        let count = grid.cell_count([0.0, 0.0], [4000.0, 3000.0]);
        assert!(count > MAX_CELLS_PER_GRID);
    }

    #[test]
    fn a_sane_cell_size_passes_the_guard() {
        let grid = GridSpec::new([0.0, 0.0], 1.0).expect("valid spec");
        let count = grid.cell_count([0.0, 0.0], [4000.0, 3000.0]);
        assert!(count <= MAX_CELLS_PER_GRID);
    }

    #[test]
    fn group_key_distinguishes_a_missing_attribute_from_a_present_one() {
        // B7: with filter_map, {region: "north"} and {zone: "north"} both keyed
        // to ["north"] and shared a grid origin.
        let group_by = vec![
            Attribute::new("region".to_string()),
            Attribute::new("zone".to_string()),
        ];

        let mut only_region = Attributes::default();
        only_region.insert(
            Attribute::new("region".to_string()),
            AttributeValue::String("north".to_string()),
        );

        let mut only_zone = Attributes::default();
        only_zone.insert(
            Attribute::new("zone".to_string()),
            AttributeValue::String("north".to_string()),
        );

        assert_ne!(
            group_key(&only_region, &Some(group_by.clone())),
            group_key(&only_zone, &Some(group_by)),
        );
    }

    // End-to-end coverage of `process` itself: the tests above pin the pieces
    // (the guard's math, the streaming/accumulating switch, the group key),
    // but none of them actually runs a feature through the processor. These
    // do, using the same `Noop` forwarder + broadcast-hub harness
    // `area_calculator`'s tests use.
    mod process {
        use std::sync::Arc;

        use reearth_flow_common::uri::Uri;
        use reearth_flow_geometry::collection::Collection3D;
        use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
        use reearth_flow_geometry::point::Point3D;
        use reearth_flow_geometry::polygon::Polygon3D;
        use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
        use reearth_flow_runtime::event::Event;
        use reearth_flow_runtime::forwarder::NoopChannelForwarder;
        use reearth_flow_runtime::kvs;
        use reearth_flow_storage::resolve::StorageResolver;
        use serde_json::json;

        use super::*;

        /// A flat rectangle from `min` to `max` at `z = 0`, in `frame`.
        fn rect_leaf(min: [f64; 2], max: [f64; 2], frame: CoordinateFrame) -> Euclidean3DGeometry {
            Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
                frame,
                vec![
                    [min[0], min[1], 0.0],
                    [max[0], min[1], 0.0],
                    [max[0], max[1], 0.0],
                    [min[0], max[1], 0.0],
                    [min[0], min[1], 0.0],
                ],
                Vec::<Vec<[f64; 3]>>::new(),
            )))
        }

        /// As [`rect_leaf`], in `CoordinateFrame::Euclidean`, wrapped as a
        /// top-level [`Geometry`].
        fn rect(min: [f64; 2], max: [f64; 2]) -> Geometry {
            Geometry::Euclidean3D(rect_leaf(min, max, CoordinateFrame::Euclidean))
        }

        fn build(with: Value) -> Box<dyn Processor> {
            GridDividerFactory
                .build(
                    NodeContext::default(),
                    EventHub::new(1),
                    "Grid Divider".to_string(),
                    Some(serde_json::from_value(with).unwrap()),
                )
                .unwrap()
        }

        fn row_col(feature: &Feature) -> (i64, i64) {
            let get = |name: &str| match feature.attributes.get(&Attribute::new(name.to_string())) {
                Some(AttributeValue::Number(n)) => n.as_i64().unwrap(),
                other => panic!("expected `{name}` on {feature:?}, got {other:?}"),
            };
            (get("_grid_row"), get("_grid_col"))
        }

        fn grid_error(feature: &Feature) -> Option<&str> {
            match feature
                .attributes
                .get(&Attribute::new("_grid_error".to_string()))
            {
                Some(AttributeValue::String(s)) => Some(s.as_str()),
                _ => None,
            }
        }

        /// Run every `geometry` through one processor built from `with`, and
        /// return every feature it sent alongside the diagnostic code of every
        /// warning it raised. Mirrors `area_calculator`'s `process_many`.
        fn process_many(with: Value, geometries: Vec<Geometry>) -> (Vec<Feature>, Vec<ErrorCode>) {
            let hub = EventHub::new(64);
            let mut rx = hub.receiver.resubscribe();
            let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
            let mut processor = build(with);

            for geometry in geometries {
                let ctx = ExecutorContext::new(
                    Feature::from(geometry),
                    FEATURES_PORT.clone(),
                    Arc::new(serde_json::Map::new()),
                    Arc::new(StorageResolver::new()),
                    Arc::new(kvs::create_kv_store()),
                    hub.clone(),
                    Uri::for_test("file:///"),
                );
                processor.process(ctx, &fw).unwrap();
            }

            let ProcessorChannelForwarder::Noop(noop) = &fw else {
                unreachable!("built as a noop forwarder");
            };
            let features = noop.send_features.lock().unwrap().clone();

            let mut warnings = Vec::new();
            while let Ok(event) = rx.try_recv() {
                if let Event::Diagnostic(diagnostic) = event {
                    warnings.push(diagnostic.code);
                }
            }
            (features, warnings)
        }

        /// A square exactly two cells wide divides into all four cells it
        /// touches, each reported full, tagged with its own row and column.
        #[test]
        fn a_two_by_two_square_divides_into_four_full_cells() {
            let (features, _) = process_many(
                json!({"cellSize": 1.0, "origin": [0.0, 0.0]}),
                vec![rect([0.0, 0.0], [2.0, 2.0])],
            );

            let mut cells: Vec<(i64, i64)> = features.iter().map(row_col).collect();
            cells.sort();
            assert_eq!(cells, vec![(0, 0), (0, 1), (1, 0), (1, 1)]);
            assert!(
                features.iter().all(|f| grid_error(f).is_none()),
                "a successful division carries no `_grid_error`"
            );
        }

        /// A rectangle one cell tall and half a cell into the next touches one
        /// full cell and one partial cell. `completeCellsOnly` keeps the full
        /// one and drops the partial one, rather than rejecting the feature
        /// outright.
        #[test]
        fn complete_cells_only_drops_the_partial_cell_and_keeps_the_full_one() {
            let geometry = rect([0.0, 0.0], [1.0, 1.5]);

            let (both, _) = process_many(
                json!({"cellSize": 1.0, "origin": [0.0, 0.0]}),
                vec![geometry.clone()],
            );
            assert_eq!(both.len(), 2, "one full cell, one partial: {both:?}");

            let (complete_only, _) = process_many(
                json!({"cellSize": 1.0, "origin": [0.0, 0.0], "completeCellsOnly": true}),
                vec![geometry],
            );
            assert_eq!(
                complete_only.len(),
                1,
                "only the full cell survives: {complete_only:?}"
            );
            assert_eq!(row_col(&complete_only[0]), (0, 0));
        }

        /// A bare point has no area to divide, so `divide_by_grid` reports it
        /// `Unsupported`. The feature is not dropped silently: it leaves via
        /// `rejected` with a reason attached, and the run is told why via the
        /// registered diagnostic code.
        #[test]
        fn unsupported_geometry_is_rejected_with_a_reason_and_a_diagnostic() {
            let point = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                CoordinateFrame::Euclidean,
                [0.0, 0.0, 0.0],
            )));

            let (features, warnings) =
                process_many(json!({"cellSize": 1.0, "origin": [0.0, 0.0]}), vec![point]);

            assert_eq!(features.len(), 1);
            assert!(
                grid_error(&features[0]).is_some(),
                "expected `_grid_error` on {:?}",
                features[0]
            );
            assert_eq!(
                warnings
                    .iter()
                    .filter(|&&c| c == ErrorCode::GridUnsupportedGeometry)
                    .count(),
                1,
                "{warnings:?}"
            );
        }

        /// A feature whose parts sit in different coordinate frames cannot be
        /// covered by one grid. It is rejected with the mixed-frames code, not
        /// the generic unsupported one.
        #[test]
        fn mixed_frames_are_rejected_with_the_mixed_frames_diagnostic() {
            let mixed =
                Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new(vec![
                    rect_leaf([0.0, 0.0], [1.0, 1.0], CoordinateFrame::Euclidean),
                    rect_leaf(
                        [0.0, 0.0],
                        [1.0, 1.0],
                        CoordinateFrame::Crs(EpsgCode::from(3857)),
                    ),
                ])));

            let (features, warnings) =
                process_many(json!({"cellSize": 1.0, "origin": [0.0, 0.0]}), vec![mixed]);

            assert_eq!(features.len(), 1);
            assert!(grid_error(&features[0]).is_some());
            assert_eq!(
                warnings
                    .iter()
                    .filter(|&&c| c == ErrorCode::GridMixedFrames)
                    .count(),
                1,
                "{warnings:?}"
            );
        }

        /// Once a feature is found angular, the warning stays quiet for the
        /// rest of the run (`GridDivider::warn_if_angular` latches
        /// `angular_warned` only on that path) — one grid is one origin and
        /// one cell size for the whole stream, so a second look never tells
        /// the run anything new. Two angular features in, one warning out.
        #[test]
        fn the_angular_frame_warning_fires_once_per_run_not_once_per_feature() {
            let angular = CoordinateFrame::Crs(EpsgCode::from(4269));
            let (_, warnings) = process_many(
                json!({"cellSize": 1.0, "origin": [0.0, 0.0]}),
                vec![
                    Geometry::Euclidean3D(rect_leaf([0.0, 0.0], [1.0, 1.0], angular.clone())),
                    Geometry::Euclidean3D(rect_leaf([0.0, 0.0], [1.0, 1.0], angular)),
                ],
            );

            assert_eq!(
                warnings
                    .iter()
                    .filter(|&&c| c == ErrorCode::GridAngularFrame)
                    .count(),
                1,
                "{warnings:?}"
            );
        }

        /// The check must stay armed past a feature that turns out not to be
        /// angular (or whose frame can't be determined at all): only a
        /// feature that is actually found angular may latch `angular_warned`.
        /// A non-angular first feature followed by a genuinely angular one
        /// must still warn — the bug this guards against is the flag being
        /// set unconditionally on the first call, which would permanently
        /// silence every angular feature behind a non-angular first one.
        #[test]
        fn a_non_angular_first_feature_does_not_suppress_a_later_angular_one() {
            let euclidean = CoordinateFrame::Euclidean;
            let angular = CoordinateFrame::Crs(EpsgCode::from(4269));
            let (_, warnings) = process_many(
                json!({"cellSize": 1.0, "origin": [0.0, 0.0]}),
                vec![
                    Geometry::Euclidean3D(rect_leaf([0.0, 0.0], [1.0, 1.0], euclidean)),
                    Geometry::Euclidean3D(rect_leaf([0.0, 0.0], [1.0, 1.0], angular)),
                ],
            );

            assert_eq!(
                warnings
                    .iter()
                    .filter(|&&c| c == ErrorCode::GridAngularFrame)
                    .count(),
                1,
                "a non-angular first feature must not permanently silence the check: {warnings:?}"
            );
        }
    }
}
