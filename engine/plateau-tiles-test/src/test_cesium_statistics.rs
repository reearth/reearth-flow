use crate::align_cesium::{
    collect_geometries_by_ident, find_cesium_tile_directories, DetailLevel, GeometryCollector,
};
use reearth_flow_geometry::types::coordinate::Coordinate;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct CesiumStatisticsConfig {}

/// Tests Cesium 3D Tiles statistics between FME and Flow outputs
pub fn test_cesium_statistics(
    fme_path: &Path,
    flow_path: &Path,
    _config: &CesiumStatisticsConfig,
) -> Result<(), String> {
    tracing::debug!("Testing Cesium statistics");

    // Find top-level 3D Tiles directories
    let fme_dirs = find_cesium_tile_directories(fme_path)?;
    let flow_dirs = find_cesium_tile_directories(flow_path)?;

    if fme_dirs.is_empty() || flow_dirs.is_empty() {
        return Err("No 3D Tiles directories found".to_string());
    }
    if fme_dirs != flow_dirs {
        return Err(format!(
            "3D Tiles directories differ: FME={:?}, Flow={:?}",
            fme_dirs, flow_dirs
        ));
    }

    for dir_name in &fme_dirs {
        let fme_dir = fme_path.join(dir_name);
        let flow_dir = flow_path.join(dir_name);

        tracing::debug!("Collecting geometries from directory: {}", dir_name);

        let fme_geometries = collect_geometries_by_ident(&fme_dir)?;
        let flow_geometries = collect_geometries_by_ident(&flow_dir)?;

        align_and_compare(&fme_geometries, &flow_geometries)?;
    }

    Ok(())
}

fn test_texture_presence(
    ident: &str,
    fme_detail_levels: &[DetailLevel],
    flow_detail_levels: &[DetailLevel],
) -> Result<(), String> {
    // all detail levels must have same texture presence (indicated by source_idx)
    let fme_has_texture = fme_detail_levels
        .first()
        .ok_or_else(|| format!("No detail levels for ident '{}' in FME", ident))?
        .source_idx
        .is_some();
    for level in fme_detail_levels.iter() {
        if level.source_idx.is_some() != fme_has_texture {
            return Err(format!(
                "ident '{}': inconsistent texture presence in FME detail levels",
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
    if fme_has_texture != flow_has_texture {
        return Err(format!(
            "ident '{}': texture presence differs between FME ({}) and Flow ({})",
            ident, fme_has_texture, flow_has_texture
        ));
    }
    Ok(())
}

fn align_and_compare(
    fme_geometries: &GeometryCollector,
    flow_geometries: &GeometryCollector,
) -> Result<(), String> {
    let fme_detail_levels = &fme_geometries.detail_levels;
    let flow_detail_levels = &flow_geometries.detail_levels;
    let fme_keys: std::collections::HashSet<_> = fme_detail_levels.keys().collect();
    let flow_keys: std::collections::HashSet<_> = flow_detail_levels.keys().collect();

    if fme_keys != flow_keys {
        let missing_in_flow: Vec<_> = fme_keys.difference(&flow_keys).collect();
        let missing_in_fme: Vec<_> = flow_keys.difference(&fme_keys).collect();

        let mut error_msg = String::new();
        if !missing_in_flow.is_empty() {
            error_msg.push_str(&format!("Missing in Flow: {:?}\n", missing_in_flow));
        }
        if !missing_in_fme.is_empty() {
            error_msg.push_str(&format!("Missing in FME: {:?}\n", missing_in_fme));
        }
        panic!("ident mismatch between FME and Flow:\n{}", error_msg);
    }

    for ident in fme_keys {
        let fme_detail_levels = &fme_detail_levels[ident];
        let flow_detail_levels = &flow_detail_levels[ident];
        test_texture_presence(ident, fme_detail_levels, flow_detail_levels)?;

        // Assert geometric error decreases monotonically
        verify_monotonic_geometric_error(ident, fme_detail_levels, "FME")?;
        verify_monotonic_geometric_error(ident, flow_detail_levels, "Flow")?;

        // compare each Flow detail level to the highest-detail FME level
        let fme_highest_level = fme_detail_levels
            .last()
            .ok_or_else(|| format!("No detail levels for ident '{}' in FME", ident))?;
        for (idx, level) in flow_detail_levels.iter().enumerate() {
            let result = compare_detail_level(
                ident,
                fme_highest_level,
                fme_geometries,
                level,
                flow_geometries,
            )?;
            tracing::debug!(
                "{}: level {}, bbox:{:.6}, center:{:.6}, color:{:.6}",
                result.ident,
                idx,
                result.bounding_box_error,
                result.mass_center_error,
                result.average_color_error
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

        // 1mm tolerance, FME is observed to produce slight non-monotonic errors
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
}

impl std::fmt::Display for DetailLevelComparisonResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} bounding_box:{:.6} mass_center:{:.6} color:{:.6}",
            self.ident, self.bounding_box_error, self.mass_center_error, self.average_color_error,
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
    fme_level: &DetailLevel,
    fme_geometries: &GeometryCollector,
    flow_level: &DetailLevel,
    flow_geometries: &GeometryCollector,
) -> Result<DetailLevelComparisonResult, String> {
    let mut result = DetailLevelComparisonResult::new(ident.to_string());
    let fme_error = fme_level.geometric_error;
    let flow_error = flow_level.geometric_error;

    // Compute bounding boxes directly from vertex positions
    let fme_bbox = compute_bbox(&fme_level.triangles, &fme_geometries.vertex_positions)?;
    let flow_bbox = compute_bbox(&flow_level.triangles, &flow_geometries.vertex_positions)?;

    // if vertices have max error r, the bounding boxes can differ by at most r in each direction
    // thus the bounding box error is at most sqrt(3) * r. We simply use 2 * r as a safe upper bound.
    let bbox_error = 2.0 * (fme_error + flow_error);
    let error_min = ((fme_bbox.0.x - flow_bbox.0.x).powi(2)
        + (fme_bbox.0.y - flow_bbox.0.y).powi(2)
        + (fme_bbox.0.z - flow_bbox.0.z).powi(2))
    .sqrt()
        / bbox_error;
    let error_max = ((fme_bbox.1.x - flow_bbox.1.x).powi(2)
        + (fme_bbox.1.y - flow_bbox.1.y).powi(2)
        + (fme_bbox.1.z - flow_bbox.1.z).powi(2))
    .sqrt()
        / bbox_error;
    result.bounding_box_error = error_min.max(error_max);
    if result.bounding_box_error > 1.0 {
        return Err(format!(
            "ident '{}': bounding box mismatch exceeds max error: {}: min:({}, {}, {}), max:({}, {}, {})",
            ident, bbox_error,
            fme_bbox.0.x - flow_bbox.0.x,
            fme_bbox.0.y - flow_bbox.0.y,
            fme_bbox.0.z - flow_bbox.0.z,
            fme_bbox.1.x - flow_bbox.1.x,
            fme_bbox.1.y - flow_bbox.1.y,
            fme_bbox.1.z - flow_bbox.1.z,
        ));
    }

    // Compute centroids directly from vertex positions
    let fme_centroid = compute_centroid(&fme_level.triangles, &fme_geometries.vertex_positions)?;
    let flow_centroid = compute_centroid(&flow_level.triangles, &flow_geometries.vertex_positions)?;

    // if vertices have max error r, centroids can differ by at most r
    let centroid_error_bound = fme_error + flow_error;
    let centroid_diff = ((fme_centroid.x - flow_centroid.x).powi(2)
        + (fme_centroid.y - flow_centroid.y).powi(2)
        + (fme_centroid.z - flow_centroid.z).powi(2))
    .sqrt();
    result.mass_center_error = centroid_diff / centroid_error_bound;
    if result.mass_center_error > 1.0 {
        return Err(format!(
            "ident '{}': mass center mismatch exceeds error bound: {}",
            ident, result.mass_center_error
        ));
    }

    // Skip color comparison if textures are present
    // NOTE: (probably) diffuseColor in X3D material is used as base color, overriden by texture if present.
    // Therefore, we ignore color comparison when textures exist.
    let has_texture = fme_level.source_idx.is_some() || flow_level.source_idx.is_some();
    if !has_texture {
        // Test face-weighted average color (material base color × vertex color)
        result.average_color_error = test_face_weighted_average_color(
            ident,
            fme_level,
            fme_geometries,
            flow_level,
            flow_geometries,
        )?;
    }

    Ok(result)
}

/// Compute axis-aligned bounding box from triangles and vertex positions
fn compute_bbox(
    triangles: &[[usize; 3]],
    positions: &[Coordinate],
) -> Result<(Coordinate, Coordinate), String> {
    if triangles.is_empty() {
        return Err("Cannot compute bbox: no triangles".to_string());
    }

    let mut min = Coordinate::from((f64::INFINITY, f64::INFINITY, f64::INFINITY));
    let mut max = Coordinate::from((f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY));

    for triangle in triangles {
        for &idx in triangle {
            let pos = positions
                .get(idx)
                .ok_or_else(|| format!("Invalid vertex index {}", idx))?;
            min.x = min.x.min(pos.x);
            min.y = min.y.min(pos.y);
            min.z = min.z.min(pos.z);
            max.x = max.x.max(pos.x);
            max.y = max.y.max(pos.y);
            max.z = max.z.max(pos.z);
        }
    }

    Ok((min, max))
}

/// Compute area-weighted centroid from triangles and vertex positions
fn compute_centroid(
    triangles: &[[usize; 3]],
    positions: &[Coordinate],
) -> Result<Coordinate, String> {
    if triangles.is_empty() {
        return Err("Cannot compute centroid: no triangles".to_string());
    }

    let mut weighted_sum = Coordinate::from((0.0, 0.0, 0.0));
    let mut total_area = 0.0;

    for triangle in triangles {
        let p0 = positions[triangle[0]];
        let p1 = positions[triangle[1]];
        let p2 = positions[triangle[2]];

        // Compute triangle centroid (average of three vertices)
        let tri_centroid = Coordinate::from((
            (p0.x + p1.x + p2.x) / 3.0,
            (p0.y + p1.y + p2.y) / 3.0,
            (p0.z + p1.z + p2.z) / 3.0,
        ));

        // Compute triangle area using cross product: ||(p1-p0) × (p2-p0)|| / 2
        let v1 = p1 - p0;
        let v2 = p2 - p0;
        let cross = v1.cross(&v2);
        let area = cross.norm() / 2.0;

        // Accumulate area-weighted centroid
        weighted_sum.x += tri_centroid.x * area;
        weighted_sum.y += tri_centroid.y * area;
        weighted_sum.z += tri_centroid.z * area;
        total_area += area;
    }

    if total_area == 0.0 {
        return Err("Cannot compute centroid: total area is zero".to_string());
    }

    Ok(Coordinate::from((
        weighted_sum.x / total_area,
        weighted_sum.y / total_area,
        weighted_sum.z / total_area,
    )))
}

/// Test face-weighted average color by comparing material base color × vertex color
/// Returns normalized color error (0.0 = perfect match, 1.0 = at tolerance threshold)
fn test_face_weighted_average_color(
    ident: &str,
    fme_level: &DetailLevel,
    fme_geometries: &GeometryCollector,
    flow_level: &DetailLevel,
    flow_geometries: &GeometryCollector,
) -> Result<f32, String> {
    // Compute face-weighted average color for FME
    let fme_avg_color = compute_face_weighted_average_color(
        &fme_level.triangles,
        &fme_geometries.vertex_positions,
        fme_geometries.vertex_colors.as_deref(),
        fme_geometries.vertex_materials.as_deref(),
        &fme_geometries.materials,
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
    // Flow default: ~(0.7, 0.7, 0.7, 1.0), FME default: (1.0, 1.0, 1.0, 1.0)
    let is_fme_default = is_near_default_color(&fme_avg_color, &[1.0, 1.0, 1.0, 1.0]);
    let is_flow_default = is_near_default_color(&flow_avg_color, &[0.7, 0.7, 0.7, 1.0]);

    if is_fme_default && is_flow_default {
        // Both are default colors, treat as equivalent
        return Ok(0.0);
    }

    // Compare average colors with tolerance
    let color_tolerance = 0.02; // Allow 2% difference per channel (0-1 range)
    let color_diff = [
        (fme_avg_color[0] - flow_avg_color[0]).abs(),
        (fme_avg_color[1] - flow_avg_color[1]).abs(),
        (fme_avg_color[2] - flow_avg_color[2]).abs(),
        (fme_avg_color[3] - flow_avg_color[3]).abs(),
    ];

    let max_diff = color_diff.iter().copied().fold(0.0f32, f32::max);
    let normalized_error = max_diff / color_tolerance;

    if max_diff > color_tolerance {
        return Err(format!(
            "ident '{}': face-weighted average color differs by {:.4} (max allowed: {:.4})\n\
             FME: [{:.4}, {:.4}, {:.4}, {:.4}]\n\
             Flow: [{:.4}, {:.4}, {:.4}, {:.4}]",
            ident,
            max_diff,
            color_tolerance,
            fme_avg_color[0],
            fme_avg_color[1],
            fme_avg_color[2],
            fme_avg_color[3],
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
