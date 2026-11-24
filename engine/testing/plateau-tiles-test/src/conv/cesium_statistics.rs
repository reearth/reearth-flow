use crate::align_cesium::{collect_geometries_by_ident, DetailLevel, GeometryCollector};
use crate::geom_stats::{
    compute_area_weighted_winding, compute_bbox, compute_centroid, compute_total_area,
};
use indexmap::IndexMap;
use reearth_flow_geometry::types::coordinate::Coordinate;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Geometric statistics of every detail level of every feature of a 3D Tiles tileset.
///
/// ```json
/// { "<feature key>": { "levels": [ {
///     "geometric_error": 0.0, "texture_presence": false, "total_area": 12.5,
///     "bbox_min": [x, y, z], "bbox_max": [x, y, z], "mass_center": [x, y, z],
///     "average_color": [r, g, b, a], "average_winding": [x, y, z]
/// } ] } }
/// ```
///
/// `levels` keeps the tileset's detail-level order. A statistic that cannot be
/// computed for a level (for example no triangles, or zero total area) is `null`.
pub fn compute_cesium_statistics(tileset_dir: &Path) -> Result<Value, String> {
    let collector = collect_geometries_by_ident(tileset_dir, &HashMap::new(), false)?;

    let mut stats: IndexMap<String, Value> = IndexMap::new();
    let mut keys: Vec<_> = collector.detail_levels.keys().cloned().collect();
    keys.sort();

    for key in keys {
        let levels: Vec<Value> = collector.detail_levels[&key]
            .iter()
            .map(|level| level_statistics(&collector, level))
            .collect();
        stats.insert(key, json!({ "levels": levels }));
    }

    serde_json::to_value(stats).map_err(|e| e.to_string())
}

fn level_statistics(collector: &GeometryCollector, level: &DetailLevel) -> Value {
    let triangles = &level.triangles;
    let positions = &collector.vertex_positions;
    let arr3 = |c: &Coordinate| json!([c.x, c.y, c.z]);

    let bbox = compute_bbox(triangles, positions).ok();
    json!({
        "geometric_error": level.geometric_error,
        "texture_presence": level.source_idx.is_some(),
        "total_area": compute_total_area(triangles, positions),
        "bbox_min": bbox.as_ref().map(|b| arr3(&b.0)),
        "bbox_max": bbox.as_ref().map(|b| arr3(&b.1)),
        "mass_center": compute_centroid(triangles, positions).ok().map(|c| arr3(&c)),
        "average_color": compute_face_weighted_average_color(
            triangles,
            positions,
            collector.vertex_colors.as_deref(),
            collector.vertex_materials.as_deref(),
            &collector.materials,
        )
        .ok(),
        "average_winding": compute_area_weighted_winding(triangles, positions).ok(),
    })
}

pub fn write_cesium_statistics(tileset_dir: &Path, output_path: &Path) -> Result<(), String> {
    let stats = compute_cesium_statistics(tileset_dir)?;
    let out = serde_json::to_vec_pretty(&stats).map_err(|e| e.to_string())?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(output_path, out).map_err(|e| e.to_string())?;
    Ok(())
}

/// Area-weighted average face color, each face being material base color x average vertex color.
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

        let area = ((p1 - p0).cross(&(p2 - p0)).norm() / 2.0) as f32;
        if area == 0.0 {
            continue; // Skip degenerate triangles
        }

        let face_color = compute_face_color(triangle, vertex_colors, vertex_materials, materials)?;
        for (sum, c) in weighted_color.iter_mut().zip(face_color) {
            *sum += c * area;
        }
        total_area += area;
    }

    if total_area == 0.0 {
        return Err("Cannot compute average color: total area is zero".to_string());
    }

    Ok(weighted_color.map(|c| c / total_area))
}

/// Face color as material base color x average vertex color.
fn compute_face_color(
    triangle: &[usize; 3],
    vertex_colors: Option<&[[f32; 4]]>,
    vertex_materials: Option<&[u32]>,
    materials: &[reearth_flow_types::material::Material],
) -> Result<[f32; 4], String> {
    let base_color = if let Some(vertex_mats) = vertex_materials {
        let mat_idx = vertex_mats[triangle[0]] as usize;
        materials
            .get(mat_idx)
            .map(|m| m.base_color)
            .ok_or_else(|| format!("Invalid material index {}", mat_idx))?
    } else {
        materials
            .first()
            .map(|m| m.base_color)
            .unwrap_or([1.0, 1.0, 1.0, 1.0])
    };

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
        [1.0, 1.0, 1.0, 1.0]
    };

    Ok([
        base_color[0] * avg_vertex_color[0],
        base_color[1] * avg_vertex_color[1],
        base_color[2] * avg_vertex_color[2],
        base_color[3] * avg_vertex_color[3],
    ])
}
