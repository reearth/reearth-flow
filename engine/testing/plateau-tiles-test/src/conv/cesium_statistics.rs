use crate::align_cesium::collect_geometries_by_ident;
use crate::geom_stats::{compute_area_weighted_winding, compute_bbox, compute_centroid};
use indexmap::IndexMap;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn compute_cesium_statistics(tileset_dir: &Path) -> Result<Value, String> {
    let collector = collect_geometries_by_ident(tileset_dir, &HashMap::new(), false)?;

    let mut stats: IndexMap<String, Value> = IndexMap::new();
    let mut keys: Vec<_> = collector.detail_levels.keys().cloned().collect();
    keys.sort();

    for key in keys {
        let levels = &collector.detail_levels[&key];
        let level = match levels.last() {
            Some(l) => l,
            None => continue,
        };
        let triangles = &level.triangles;
        let positions = &collector.vertex_positions;

        let texture_presence = level.source_idx.is_some();
        let bbox = compute_bbox(triangles, positions)?;
        let mass_center = compute_centroid(triangles, positions)?;
        let average_winding = compute_area_weighted_winding(triangles, positions)?;

        stats.insert(
            key,
            json!({
                "texture_presence": texture_presence,
                "bbox_min": [bbox.0.x, bbox.0.y, bbox.0.z],
                "bbox_max": [bbox.1.x, bbox.1.y, bbox.1.z],
                "mass_center": [mass_center.x, mass_center.y, mass_center.z],
                "average_winding": average_winding,
            }),
        );
    }

    serde_json::to_value(stats).map_err(|e| e.to_string())
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
