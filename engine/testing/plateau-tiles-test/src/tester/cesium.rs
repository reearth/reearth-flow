use crate::align_cesium::{
    collect_geometries_by_ident, find_cesium_tile_directories, DetailLevel, GeometryCollector,
};
use crate::cast_config::{convert_casts, CastConfigValue};
use crate::compare_attributes::{analyze_attributes, CastConfig};
use crate::geom_stats::{
    compute_area_weighted_winding, compute_bbox, compute_centroid, compute_total_area,
};
use reearth_flow_geometry::types::coordinate::Coordinate;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometryTest {
    TexturePresence,
    MonotonicGeometricError,
    BoundingBox,
    MassCenter,
    AverageColor,
    AverageWinding,
}

#[derive(Debug, Deserialize)]
pub struct CesiumConfig {
    pub casts: Option<HashMap<String, CastConfigValue>>,
    #[serde(default)]
    pub skip_geometry_tests: Vec<GeometryTest>,
    #[serde(default)]
    pub skip_all_geometry_tests: bool,
}

pub fn test_cesium(truth_path: &Path, flow_path: &Path, config: &CesiumConfig) -> Result<(), String> {
    let casts = if let Some(casts_cfg) = &config.casts {
        convert_casts(casts_cfg)?
    } else {
        HashMap::new()
    };

    let truth_dirs = find_cesium_tile_directories(truth_path)?;
    let flow_dirs = find_cesium_tile_directories(flow_path)?;

    if truth_dirs.is_empty() || flow_dirs.is_empty() {
        return Err("No 3D Tiles directories found".to_string());
    }
    if truth_dirs != flow_dirs {
        return Err(format!(
            "3D Tiles directories differ: Truth={:?}, Flow={:?}",
            truth_dirs, flow_dirs
        ));
    }

    for dir_name in &truth_dirs {
        let truth_dir = truth_path.join(dir_name);
        let flow_dir = flow_path.join(dir_name);

        tracing::debug!("Comparing Cesium in directory: {}", dir_name);

        let truth_collector = collect_geometries_by_ident(&truth_dir, &casts, false)?;
        let flow_collector = collect_geometries_by_ident(&flow_dir, &casts, true)?;

        // Compare attributes
        compare_attributes(&truth_collector, &flow_collector, &casts)?;

        // Compare statistics
        if !config.skip_all_geometry_tests {
            compare_geometry(&truth_collector, &flow_collector, config)?;
        }
    }

    Ok(())
}

fn compare_attributes(
    truth: &GeometryCollector,
    flow: &GeometryCollector,
    casts: &HashMap<String, CastConfig>,
) -> Result<(), String> {
    let all_keys: HashSet<_> = truth
        .feature_attributes
        .keys()
        .chain(flow.feature_attributes.keys())
        .collect();
    for ident in all_keys {
        let attr1 = truth
            .feature_attributes
            .get(ident)
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let attr2 = flow
            .feature_attributes
            .get(ident)
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        analyze_attributes(ident, &attr1, &attr2, casts.clone(), Default::default())?;
    }
    Ok(())
}

fn test_texture_presence(
    ident: &str,
    truth_detail_levels: &[DetailLevel],
    flow_detail_levels: &[DetailLevel],
) -> Result<(), String> {
    // all detail levels must have same texture presence (indicated by source_idx)
    let truth_has_texture = truth_detail_levels
        .first()
        .ok_or_else(|| format!("No detail levels for ident '{}' in Truth", ident))?
        .source_idx
        .is_some();
    for level in truth_detail_levels.iter() {
        if level.source_idx.is_some() != truth_has_texture {
            return Err(format!(
                "ident '{}': inconsistent texture presence in Truth detail levels",
                ident
            ));
        }
    }
    let flow_has_texture = flow_detail_levels
        .first()
        .ok_or_else(|| format!("No detail levels for ident '{}' in Flow", ident))?
        .source_idx
        .is_some();
    for level in flow_detail_levels.iter() {
        if level.source_idx.is_some() != flow_has_texture {
            return Err(format!(
                "ident '{}': inconsistent texture presence in Flow detail levels",
                ident
            ));
        }
    }
    if truth_has_texture != flow_has_texture {
        return Err(format!(
            "ident '{}': texture presence differs between Truth ({}) and Flow ({})",
            ident, truth_has_texture, flow_has_texture
        ));
    }
    Ok(())
}

fn compare_geometry(
    truth_geometries: &GeometryCollector,
    flow_geometries: &GeometryCollector,
    config: &CesiumConfig,
) -> Result<(), String> {
    let skip: HashSet<GeometryTest> = config.skip_geometry_tests.iter().copied().collect();
    let truth_detail_levels = &truth_geometries.detail_levels;
    let flow_detail_levels = &flow_geometries.detail_levels;
    let truth_keys: std::collections::HashSet<_> = truth_detail_levels.keys().collect();
    let flow_keys: std::collections::HashSet<_> = flow_detail_levels.keys().collect();

    if truth_keys != flow_keys {
        let missing_in_flow: Vec<_> = truth_keys.difference(&flow_keys).collect();
        let missing_in_truth: Vec<_> = flow_keys.difference(&truth_keys).collect();

        let mut error_msg = String::new();
        if !missing_in_flow.is_empty() {
            error_msg.push_str(&format!("Missing in Flow: {:?}\n", missing_in_flow));
        }
        if !missing_in_truth.is_empty() {
            error_msg.push_str(&format!("Missing in Truth: {:?}\n", missing_in_truth));
        }
        panic!("ident mismatch between Truth and Flow:\n{}", error_msg);
    }

    for ident in truth_keys {
        let truth_detail_levels = &truth_detail_levels[ident];
        let flow_detail_levels = &flow_detail_levels[ident];
        if !skip.contains(&GeometryTest::TexturePresence) {
            test_texture_presence(ident, truth_detail_levels, flow_detail_levels)?;
        }

        // Assert geometric error decreases monotonically
        if !skip.contains(&GeometryTest::MonotonicGeometricError) {
            verify_monotonic_geometric_error(ident, truth_detail_levels, "Truth")?;
            verify_monotonic_geometric_error(ident, flow_detail_levels, "Flow")?;
        }

        // compare each Flow detail level to the highest-detail Truth level
        let truth_highest_level = truth_detail_levels
            .last()
            .ok_or_else(|| format!("No detail levels for ident '{}' in Truth", ident))?;
        for (idx, level) in flow_detail_levels.iter().enumerate() {
            let result = compare_detail_level(
                ident,
                truth_highest_level,
                truth_geometries,
                level,
                flow_geometries,
                &skip,
            )?;
            tracing::debug!(
                "{}: level {}, bbox:{:.6}, center:{:.6}, color:{:.6}, winding:{:.6}",
                result.ident,
                idx,
                result.bounding_box_error,
                result.mass_center_error,
                result.average_color_error,
                result.average_winding_error
            );
        }
    }

    Ok(())
}

fn verify_monotonic_geometric_error(
    ident: &str,
    detail_levels: &[DetailLevel],
    source: &str,
) -> Result<(), String> {
    let mut prev_error = f64::INFINITY;

    for (i, level) in detail_levels.iter().enumerate() {
        if level.geometric_error < 0.0 || !level.geometric_error.is_finite() {
            return Err(format!(
                "{} ident '{}': invalid geometric error {} at level {}",
                source, ident, level.geometric_error, i
            ));
        }

        // 1mm tolerance, Truth is observed to produce slight non-monotonic errors
        if level.geometric_error > prev_error + 1e-3 {
            return Err(format!(
                "{} ident '{}': geometric error is not monotonically decreasing at level {} \
                 (previous: {}, current: {})",
                source, ident, i, prev_error, level.geometric_error
            ));
        }

        prev_error = level.geometric_error;
    }
    Ok(())
}

#[derive(Default, Debug)]
pub struct DetailLevelComparisonResult {
    ident: String,
    bounding_box_error: f64,
    mass_center_error: f64,
    average_color_error: f32,
    average_winding_error: f64,
}

impl std::fmt::Display for DetailLevelComparisonResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} bounding_box:{:.6} mass_center:{:.6} color:{:.6} winding:{:.6}",
            self.ident,
            self.bounding_box_error,
            self.mass_center_error,
            self.average_color_error,
            self.average_winding_error,
        )
    }
}

impl DetailLevelComparisonResult {
    fn new(ident: String) -> Self {
        Self {
            ident,
            ..Default::default()
        }
    }
}

fn compare_detail_level(
    ident: &str,
    truth_level: &DetailLevel,
    truth_geometries: &GeometryCollector,
    flow_level: &DetailLevel,
    flow_geometries: &GeometryCollector,
    skip: &HashSet<GeometryTest>,
) -> Result<DetailLevelComparisonResult, String> {
    let mut result = DetailLevelComparisonResult::new(ident.to_string());
    let truth_error = truth_level.geometric_error;
    let flow_error = flow_level.geometric_error;

    // If flow geometry is degenerate (all zero-area triangles), the object may have been
    // simplified away. This is acceptable if the Truth object's size is within the flow's
    // geometric error — i.e. the object is smaller than the LOD resolution.
    let flow_total_area =
        compute_total_area(&flow_level.triangles, &flow_geometries.vertex_positions);
    if flow_total_area == 0.0 {
        let truth_bbox = compute_bbox(&truth_level.triangles, &truth_geometries.vertex_positions)?;
        let bbox_diagonal = ((truth_bbox.1.x - truth_bbox.0.x).powi(2)
            + (truth_bbox.1.y - truth_bbox.0.y).powi(2)
            + (truth_bbox.1.z - truth_bbox.0.z).powi(2))
        .sqrt();
        if bbox_diagonal <= flow_error {
            tracing::debug!(
                "ident '{}': skipping geometry tests — flow geometry is degenerate and Truth \
                 object size ({:.6}) is within flow geometric error ({:.6})",
                ident,
                bbox_diagonal,
                flow_error
            );
            return Ok(result);
        }
        return Err(format!(
            "ident '{}': flow geometry is degenerate (zero total area) but Truth geometry \
             size ({:.6}) exceeds flow geometric error ({:.6})",
            ident, bbox_diagonal, flow_error
        ));
    }

    if !skip.contains(&GeometryTest::BoundingBox) {
        // Compute bounding boxes directly from vertex positions
        let truth_bbox = compute_bbox(&truth_level.triangles, &truth_geometries.vertex_positions)?;
        let flow_bbox = compute_bbox(&flow_level.triangles, &flow_geometries.vertex_positions)?;

        // if vertices have max error r, the bounding boxes can differ by at most r in each direction
        // thus the bounding box error is at most sqrt(3) * r. We simply use 2 * r as a safe upper bound.
        let bbox_error = 2.0 * (truth_error + flow_error);
        let error_min = ((truth_bbox.0.x - flow_bbox.0.x).powi(2)
            + (truth_bbox.0.y - flow_bbox.0.y).powi(2)
            + (truth_bbox.0.z - flow_bbox.0.z).powi(2))
        .sqrt()
            / bbox_error;
        let error_max = ((truth_bbox.1.x - flow_bbox.1.x).powi(2)
            + (truth_bbox.1.y - flow_bbox.1.y).powi(2)
            + (truth_bbox.1.z - flow_bbox.1.z).powi(2))
        .sqrt()
            / bbox_error;
        result.bounding_box_error = error_min.max(error_max);
        if result.bounding_box_error > 1.0 {
            return Err(format!(
                "ident '{}': bounding box mismatch exceeds max error: {}/{}: min:({}, {}, {}), max:({}, {}, {})",
                ident, truth_error, flow_error,
                truth_bbox.0.x - flow_bbox.0.x,
                truth_bbox.0.y - flow_bbox.0.y,
                truth_bbox.0.z - flow_bbox.0.z,
                truth_bbox.1.x - flow_bbox.1.x,
                truth_bbox.1.y - flow_bbox.1.y,
                truth_bbox.1.z - flow_bbox.1.z,
            ));
        }
    }

    if !skip.contains(&GeometryTest::MassCenter) {
        // Compute centroids directly from vertex positions
        let truth_centroid = compute_centroid(&truth_level.triangles, &truth_geometries.vertex_positions)
            .map_err(|e| format!("ident '{}': failed to compute Truth centroid: {}", ident, e))?;
        let flow_centroid =
            compute_centroid(&flow_level.triangles, &flow_geometries.vertex_positions).map_err(
                |e| format!("ident '{}': failed to compute Flow centroid: {}", ident, e),
            )?;

        // if vertices have max error r, centroids can differ by at most r
        let centroid_error_bound = truth_error + flow_error;
        let centroid_diff = ((truth_centroid.x - flow_centroid.x).powi(2)
            + (truth_centroid.y - flow_centroid.y).powi(2)
            + (truth_centroid.z - flow_centroid.z).powi(2))
        .sqrt();
        result.mass_center_error = centroid_diff / centroid_error_bound;
        if result.mass_center_error > 1.0 {
            return Err(format!(
                "ident '{}': mass center mismatch exceeds error bound: {}",
                ident, result.mass_center_error
            ));
        }
    }

    // Skip color comparison if textures are present
    // NOTE: (probably) diffuseColor in X3D material is used as base color, overriden by texture if present.
    // Therefore, we ignore color comparison when textures exist.
    if !skip.contains(&GeometryTest::AverageColor) {
        let has_texture = truth_level.source_idx.is_some() || flow_level.source_idx.is_some();
        if !has_texture {
            // Test face-weighted average color (material base color × vertex color)
            result.average_color_error = test_face_weighted_average_color(
                ident,
                truth_level,
                truth_geometries,
                flow_level,
                flow_geometries,
            )?;
        }
    }

    if !skip.contains(&GeometryTest::AverageWinding) {
        result.average_winding_error = test_area_weighted_average_winding(
            ident,
            truth_level,
            truth_geometries,
            flow_level,
            flow_geometries,
        )?;
    }

    Ok(result)
}

/// Test face-weighted average color by comparing material base color × vertex color
/// Returns normalized color error (0.0 = perfect match, 1.0 = at tolerance threshold)
fn test_face_weighted_average_color(
    ident: &str,
    truth_level: &DetailLevel,
    truth_geometries: &GeometryCollector,
    flow_level: &DetailLevel,
    flow_geometries: &GeometryCollector,
) -> Result<f32, String> {
    // Compute face-weighted average color for Truth
    let truth_avg_color = compute_face_weighted_average_color(
        &truth_level.triangles,
        &truth_geometries.vertex_positions,
        truth_geometries.vertex_colors.as_deref(),
        truth_geometries.vertex_materials.as_deref(),
        &truth_geometries.materials,
    )?;

    // Compute face-weighted average color for Flow
    let flow_avg_color = compute_face_weighted_average_color(
        &flow_level.triangles,
        &flow_geometries.vertex_positions,
        flow_geometries.vertex_colors.as_deref(),
        flow_geometries.vertex_materials.as_deref(),
        &flow_geometries.materials,
    )?;

    // Check if colors should be treated as equivalent defaults
    // Flow default: ~(0.7, 0.7, 0.7, 1.0), Truth default: (1.0, 1.0, 1.0, 1.0)
    let is_truth_default = is_near_default_color(&truth_avg_color, &[1.0, 1.0, 1.0, 1.0]);
    let is_flow_default = is_near_default_color(&flow_avg_color, &[0.7, 0.7, 0.7, 1.0]);

    if is_truth_default && is_flow_default {
        // Both are default colors, treat as equivalent
        return Ok(0.0);
    }

    // Compare average colors with tolerance
    let color_tolerance = 0.1; // Allow 10% difference per channel (0-1 range)
    let color_diff = [
        (truth_avg_color[0] - flow_avg_color[0]).abs(),
        (truth_avg_color[1] - flow_avg_color[1]).abs(),
        (truth_avg_color[2] - flow_avg_color[2]).abs(),
        (truth_avg_color[3] - flow_avg_color[3]).abs(),
    ];

    let max_diff = color_diff.iter().copied().fold(0.0f32, f32::max);
    let normalized_error = max_diff / color_tolerance;

    if max_diff > color_tolerance {
        return Err(format!(
            "ident '{}': face-weighted average color differs by {:.4} (max allowed: {:.4})\n\
             Truth: [{:.4}, {:.4}, {:.4}, {:.4}]\n\
             Flow: [{:.4}, {:.4}, {:.4}, {:.4}]",
            ident,
            max_diff,
            color_tolerance,
            truth_avg_color[0],
            truth_avg_color[1],
            truth_avg_color[2],
            truth_avg_color[3],
            flow_avg_color[0],
            flow_avg_color[1],
            flow_avg_color[2],
            flow_avg_color[3],
        ));
    }

    Ok(normalized_error)
}

/// Compute area-weighted average color from triangles
/// Each face color = material base color × vertex color (averaged across vertices)
fn compute_face_weighted_average_color(
    triangles: &[[usize; 3]],
    positions: &[Coordinate],
    vertex_colors: Option<&[[f32; 4]]>,
    vertex_materials: Option<&[u32]>,
    materials: &[reearth_flow_types::material::Material],
) -> Result<[f32; 4], String> {
    if triangles.is_empty() {
        return Err("Cannot compute average color: no triangles".to_string());
    }

    let mut weighted_color = [0.0f32; 4];
    let mut total_area = 0.0f32;

    for triangle in triangles {
        let p0 = positions[triangle[0]];
        let p1 = positions[triangle[1]];
        let p2 = positions[triangle[2]];

        // Compute triangle area
        let v1 = p1 - p0;
        let v2 = p2 - p0;
        let cross = v1.cross(&v2);
        let area = (cross.norm() / 2.0) as f32;

        if area == 0.0 {
            continue; // Skip degenerate triangles
        }

        // Get face color by averaging vertex colors and multiplying by material base color
        let face_color = compute_face_color(triangle, vertex_colors, vertex_materials, materials)?;

        // Accumulate area-weighted color
        weighted_color[0] += face_color[0] * area;
        weighted_color[1] += face_color[1] * area;
        weighted_color[2] += face_color[2] * area;
        weighted_color[3] += face_color[3] * area;
        total_area += area;
    }

    if total_area == 0.0 {
        return Err("Cannot compute average color: total area is zero".to_string());
    }

    // Normalize by total area
    Ok([
        weighted_color[0] / total_area,
        weighted_color[1] / total_area,
        weighted_color[2] / total_area,
        weighted_color[3] / total_area,
    ])
}

/// Check if a color is close to a default color (within 5% tolerance per channel)
fn is_near_default_color(color: &[f32; 4], default: &[f32; 4]) -> bool {
    const DEFAULT_TOLERANCE: f32 = 0.05;
    (color[0] - default[0]).abs() <= DEFAULT_TOLERANCE
        && (color[1] - default[1]).abs() <= DEFAULT_TOLERANCE
        && (color[2] - default[2]).abs() <= DEFAULT_TOLERANCE
        && (color[3] - default[3]).abs() <= DEFAULT_TOLERANCE
}

/// Compare area-weighted average winding vectors between Truth and Flow in Euclidean space.
/// Both vectors are dimensionless with magnitude in [0, 1].
/// Returns the error normalized by the tolerance (0.0 = perfect match, 1.0 = at threshold).
fn test_area_weighted_average_winding(
    ident: &str,
    truth_level: &DetailLevel,
    truth_geometries: &GeometryCollector,
    flow_level: &DetailLevel,
    flow_geometries: &GeometryCollector,
) -> Result<f64, String> {
    let truth_vec =
        compute_area_weighted_winding(&truth_level.triangles, &truth_geometries.vertex_positions)?;
    let flow_vec =
        compute_area_weighted_winding(&flow_level.triangles, &flow_geometries.vertex_positions)?;

    let diff_magnitude = ((truth_vec[0] - flow_vec[0]).powi(2)
        + (truth_vec[1] - flow_vec[1]).powi(2)
        + (truth_vec[2] - flow_vec[2]).powi(2))
    .sqrt();

    const TOLERANCE: f64 = 0.25;
    if diff_magnitude > TOLERANCE {
        return Err(format!(
            "ident '{}': area-weighted average winding differs by {:.4} (max allowed: {:.4})\n\
             Truth:  [{:.6}, {:.6}, {:.6}]\n\
             Flow: [{:.6}, {:.6}, {:.6}]",
            ident,
            diff_magnitude,
            TOLERANCE,
            truth_vec[0],
            truth_vec[1],
            truth_vec[2],
            flow_vec[0],
            flow_vec[1],
            flow_vec[2],
        ));
    }

    Ok(diff_magnitude / TOLERANCE)
}

/// Compute face color as material base color × average vertex color
fn compute_face_color(
    triangle: &[usize; 3],
    vertex_colors: Option<&[[f32; 4]]>,
    vertex_materials: Option<&[u32]>,
    materials: &[reearth_flow_types::material::Material],
) -> Result<[f32; 4], String> {
    // Get material base color for this face
    let base_color = if let Some(vertex_mats) = vertex_materials {
        let mat_idx = vertex_mats[triangle[0]] as usize;
        materials
            .get(mat_idx)
            .map(|m| m.base_color)
            .ok_or_else(|| format!("Invalid material index {}", mat_idx))?
    } else {
        // Use first material or default white
        materials
            .first()
            .map(|m| m.base_color)
            .unwrap_or([1.0, 1.0, 1.0, 1.0])
    };

    // Get average vertex color for the triangle
    let avg_vertex_color = if let Some(colors) = vertex_colors {
        let c0 = colors[triangle[0]];
        let c1 = colors[triangle[1]];
        let c2 = colors[triangle[2]];
        [
            (c0[0] + c1[0] + c2[0]) / 3.0,
            (c0[1] + c1[1] + c2[1]) / 3.0,
            (c0[2] + c1[2] + c2[2]) / 3.0,
            (c0[3] + c1[3] + c2[3]) / 3.0,
        ]
    } else {
        // Default to white if no vertex colors
        [1.0, 1.0, 1.0, 1.0]
    };

    // Multiply material base color × vertex color
    Ok([
        base_color[0] * avg_vertex_color[0],
        base_color[1] * avg_vertex_color[1],
        base_color[2] * avg_vertex_color[2],
        base_color[3] * avg_vertex_color[3],
    ])
}
