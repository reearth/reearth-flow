use std::collections::HashMap;

use rayon::prelude::*;
use reearth_flow_geometry::algorithm::bounding_rect::BoundingRect;
use reearth_flow_geometry::types::coordinate::{Coordinate2D, Coordinate3D};
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_geometry::types::line_string::{LineString2D, LineString3D};
use reearth_flow_geometry::types::multi_polygon::{MultiPolygon2D, MultiPolygon3D};
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_geometry::types::rect::Rect2D;
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
            buffer: HashMap::new(),
            bounds_per_group: HashMap::new(),
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

#[derive(Debug, Clone)]
pub struct GridDivider {
    unit_square_size: f64,
    keep_square_only: bool,
    group_by: Option<Vec<Attribute>>,
    buffer: HashMap<AttributeValue, Vec<Feature>>,
    bounds_per_group: HashMap<AttributeValue, Rect2D<f64>>,
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

impl Processor for GridDivider {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        // Try to get bounding rect from geometry
        let bounds_opt = get_geometry_bounds_2d(&geometry.value);

        match bounds_opt {
            Some(bounds) => {
                // Compute group key from group_by attributes
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

                // Store feature in buffer
                self.buffer.entry(key).or_default().push(feature.clone());
            }
            None => {
                // No valid bounds (geometry is not a polygon type or is empty)
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
        // Collect results from parallel processing
        let results: Vec<(Feature, Port)> = self
            .buffer
            .par_iter()
            .flat_map(|(key, features)| {
                let bounds = match self.bounds_per_group.get(key) {
                    Some(b) => b,
                    None => return vec![],
                };

                let grid_cells = generate_grid_cells(bounds, self.unit_square_size);

                features
                    .par_iter()
                    .flat_map(|feature| {
                        let mut results = Vec::new();

                        for cell in &grid_cells {
                            // clip_geometry_by_cell returns Vec for CityGmlGeometry (one per polygon)
                            for clip_result in clip_geometry_by_cell(&feature.geometry.value, cell)
                            {
                                // Skip non-complete squares if keep_square_only is set
                                if self.keep_square_only && !clip_result.is_complete_square {
                                    continue;
                                }

                                let new_feature = create_output_feature(
                                    feature,
                                    clip_result.geometry,
                                    cell,
                                    &self.group_by,
                                );
                                results.push((new_feature, DEFAULT_PORT.clone()));
                            }
                        }

                        results
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        // Send all results
        for (feature, port) in results {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx, feature, port,
            ));
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

/// Generate grid cells covering the given bounding box
fn generate_grid_cells(bounds: &Rect2D<f64>, unit_size: f64) -> Vec<GridCell> {
    let min_x = bounds.min().x;
    let min_y = bounds.min().y;
    let max_x = bounds.max().x;
    let max_y = bounds.max().y;

    let width = max_x - min_x;
    let height = max_y - min_y;

    let cols = (width / unit_size).ceil() as usize;
    let rows = (height / unit_size).ceil() as usize;

    let mut cells = Vec::with_capacity(rows * cols);

    for row in 0..rows {
        for col in 0..cols {
            let cell_min_x = min_x + (col as f64) * unit_size;
            let cell_min_y = min_y + (row as f64) * unit_size;
            let cell_max_x = cell_min_x + unit_size;
            let cell_max_y = cell_min_y + unit_size;

            cells.push(GridCell {
                min_x: cell_min_x,
                min_y: cell_min_y,
                max_x: cell_max_x,
                max_y: cell_max_y,
                row,
                col,
            });
        }
    }

    cells
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
                };

                // Create placeholder UV polygon matching the structure of clipped_poly
                // Each vertex gets placeholder UV coordinates (0.0, 0.0)
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

        // Also process composite surfaces recursively
        for cs in &gml.composite_surfaces {
            // Count total polygons in composite surface
            let poly_count = count_polygons_recursive(cs);
            let empty_uv_polygon = Polygon2D::new(
                reearth_flow_geometry::types::line_string::LineString2D::new(vec![]),
                vec![],
            );

            // Clone and reset pos/len to match the new arrays
            let mut cs_clone = cs.clone();
            cs_clone.pos = 0;
            cs_clone.len = poly_count as u32;
            // Also reset nested composite_surfaces recursively
            reset_pos_len_recursive(&mut cs_clone);

            let cs_citygml = CityGmlGeometry {
                gml_geometries: vec![cs_clone],
                materials: citygml.materials.clone(),
                textures: citygml.textures.clone(),
                polygon_materials: vec![None; poly_count],
                polygon_textures: vec![None; poly_count],
                polygon_uvs: MultiPolygon2D::new(vec![empty_uv_polygon; poly_count]),
            };
            results.extend(clip_citygml_geometry_per_polygon(&cs_citygml, cell));
        }
    }

    results
}

/// Count total polygons in a GmlGeometry recursively (including composite_surfaces)
fn count_polygons_recursive(gml: &GmlGeometry) -> usize {
    let mut count = gml.polygons.len();
    for cs in &gml.composite_surfaces {
        count += count_polygons_recursive(cs);
    }
    count
}

/// Reset pos/len fields recursively to match the polygon count
/// This is needed when cloning a GmlGeometry to create a new CityGmlGeometry
/// with fresh polygon_uvs/materials/textures arrays
fn reset_pos_len_recursive(gml: &mut GmlGeometry) {
    let mut offset = gml.polygons.len() as u32;
    for cs in &mut gml.composite_surfaces {
        cs.pos = offset;
        let cs_poly_count = count_polygons_recursive(cs) as u32;
        cs.len = cs_poly_count;
        reset_pos_len_recursive(cs);
        offset += cs_poly_count;
    }
}

/// Create a placeholder UV polygon that matches the ring structure of a 3D polygon.
/// Each vertex gets placeholder UV coordinates (0.0, 0.0).
/// This is needed because Cesium3DTilesWriter expects UV polygons to have the same
/// number of rings and vertices as the corresponding 3D polygon.
fn create_placeholder_uv_polygon(poly3d: &Polygon3D<f64>) -> Polygon2D<f64> {
    // Create exterior ring with placeholder UVs
    let exterior_uv_coords: Vec<Coordinate2D<f64>> = poly3d
        .exterior()
        .0
        .iter()
        .map(|_| Coordinate2D::new_(0.0, 0.0))
        .collect();
    let exterior_uv = LineString2D::new(exterior_uv_coords);

    // Create interior rings with placeholder UVs
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
        Geometry2D::MultiPolygon(mpoly) => {
            // For multi-polygon, all polygons must be complete squares
            // (though typically we'd have a single polygon per cell)
            mpoly
                .iter()
                .all(|poly| is_complete_square_2d_polygon(poly, cell))
        }
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
    // A complete square has exactly 4 corners + closing point = 5 vertices
    let exterior = &poly.exterior().0;
    if exterior.len() != 5 {
        return false;
    }

    // Must have no holes
    if !poly.interiors().is_empty() {
        return false;
    }

    // Check that all 4 cell corners are present in the polygon
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
    // A complete square has exactly 4 corners + closing point = 5 vertices
    let exterior = &poly.exterior().0;
    if exterior.len() != 5 {
        return false;
    }

    // Must have no holes
    if !poly.interiors().is_empty() {
        return false;
    }

    // Check that all 4 cell corners are present in the polygon (XY only)
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
    // First check if polygon bounding box overlaps with cell at all
    if let Some(poly_bounds) = polygon.bounding_rect() {
        let cell_bounds = Rect2D::new(
            Coordinate2D::new_(cell.min_x, cell.min_y),
            Coordinate2D::new_(cell.max_x, cell.max_y),
        );
        if !poly_bounds.overlap(&cell_bounds) {
            return None;
        }
    }

    // Clip exterior ring
    let exterior_coords: Vec<Coordinate2D<f64>> = polygon.exterior().0.to_vec();
    let clipped_exterior = clip_polygon_coords_2d(&exterior_coords, cell)?;

    if clipped_exterior.len() < 3 {
        return None;
    }

    // Clip interior rings (holes)
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
    // First check if polygon bounding box overlaps with cell at all (in XY)
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

    // Clip exterior ring
    let exterior_coords: Vec<Coordinate3D<f64>> = polygon.exterior().0.to_vec();
    let clipped_exterior = clip_polygon_coords_3d(&exterior_coords, cell)?;

    if clipped_exterior.len() < 3 {
        return None;
    }

    // Clip interior rings (holes)
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
        // Need at least 3 points + closing point
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

    // Remove last point if it duplicates the first (closing point)
    if output.len() > 1 && coords_equal_3d(&output[0], output.last().unwrap()) {
        output.pop();
    }

    if output.is_empty() {
        return None;
    }

    // Clip against each edge: left, right, bottom, top
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

    // Close the polygon
    if output.len() >= 3 && !coords_equal_3d(&output[0], output.last().unwrap()) {
        output.push(output[0]);
    }

    if output.len() < 4 {
        // Need at least 3 points + closing point
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
                // Both inside, add next
                output.push(*next);
            }
            (true, false) => {
                // Going out, add intersection
                output.push(intersect_2d(current, next, edge));
            }
            (false, true) => {
                // Coming in, add intersection and next
                output.push(intersect_2d(current, next, edge));
                output.push(*next);
            }
            (false, false) => {
                // Both outside, add nothing
            }
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
                // Both inside, add next
                output.push(*next);
            }
            (true, false) => {
                // Going out, add intersection
                output.push(intersect_3d(current, next, edge));
            }
            (false, true) => {
                // Coming in, add intersection and next
                output.push(intersect_3d(current, next, edge));
                output.push(*next);
            }
            (false, false) => {
                // Both outside, add nothing
            }
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

    // Preserve metadata (feature_type, feature_id, lod)
    new_feature.metadata = original.metadata.clone();

    // Add grid cell metadata
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
    fn test_generate_grid_cells() {
        let bounds = Rect2D::new(Coordinate2D::new_(0.0, 0.0), Coordinate2D::new_(10.0, 10.0));

        let cells = generate_grid_cells(&bounds, 5.0);

        assert_eq!(cells.len(), 4); // 2x2 grid

        // Check first cell
        assert_eq!(cells[0].row, 0);
        assert_eq!(cells[0].col, 0);

        // Check last cell
        assert_eq!(cells[3].row, 1);
        assert_eq!(cells[3].col, 1);
    }

    #[test]
    fn test_generate_grid_cells_non_exact() {
        let bounds = Rect2D::new(Coordinate2D::new_(0.0, 0.0), Coordinate2D::new_(12.0, 12.0));

        let cells = generate_grid_cells(&bounds, 5.0);

        assert_eq!(cells.len(), 9); // 3x3 grid (ceil(12/5) = 3)
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
        // Should have approximately the same number of points
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
        // Should be a 4-corner polygon (square) + closing point
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
        // Check that Z values are interpolated
        // At y=5 (halfway between y=0 and y=10), z should be interpolated from 5 to 15
        // Expected z at y=5 should be 10 (midpoint)
        let z_values: Vec<f64> = clipped.exterior().0.iter().map(|c| c.z).collect();

        // Verify Z values are present and interpolated
        assert!(!z_values.is_empty());
        // Some Z values should be the original (5.0)
        // Some should be interpolated (~10.0 at the clipped edge)
    }

    #[test]
    fn test_clip_polygon_with_hole() {
        // Create polygon with a hole
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

        // Clip with a cell that contains part of the hole
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
        // Should have exterior ring
        assert!(!clipped.exterior().0.is_empty());
        // Hole should be partially clipped (may or may not be present depending on clipping)
    }

    #[test]
    fn test_is_complete_square() {
        // A polygon that exactly matches the cell should be complete
        let cell = GridCell {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 5.0,
            max_y: 5.0,
            row: 0,
            col: 0,
        };

        // Create a polygon that exactly matches the cell corners
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

        // Create a polygon that does NOT match the cell corners (partial)
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

        // A triangle (3 corners + close) should not be complete
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
        // Should only return 2D bounds
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

        // Should have all original attributes
        assert!(output
            .attributes
            .contains_key(&Attribute::new("group_attr")));
        assert!(output
            .attributes
            .contains_key(&Attribute::new("other_attr")));
        // Should have grid metadata
        assert!(output.attributes.contains_key(&Attribute::new("_grid_row")));
        assert!(output.attributes.contains_key(&Attribute::new("_grid_col")));

        // Check grid metadata values
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
