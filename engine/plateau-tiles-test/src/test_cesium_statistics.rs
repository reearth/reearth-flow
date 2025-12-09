use crate::align_cesium::{collect_geometries_by_gmlid, find_cesium_tile_directories, DetailLevel};
use serde::Deserialize;
use std::collections::HashMap;
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

        let fme_geometries = collect_geometries_by_gmlid(&fme_dir)?;
        let flow_geometries = collect_geometries_by_gmlid(&flow_dir)?;

        align_and_compare(&fme_geometries, &flow_geometries)?;
    }

    Ok(())
}

fn align_and_compare(
    fme_geometries: &HashMap<String, Vec<DetailLevel>>,
    flow_geometries: &HashMap<String, Vec<DetailLevel>>,
) -> Result<(), String> {
    let fme_keys: std::collections::HashSet<_> = fme_geometries.keys().collect();
    let flow_keys: std::collections::HashSet<_> = flow_geometries.keys().collect();

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
        panic!("gml_id mismatch between FME and Flow:\n{}", error_msg);
    }

    for gml_id in fme_keys {
        let fme_detail_levels = &fme_geometries[gml_id];
        let flow_detail_levels = &flow_geometries[gml_id];

        // Assert geometric error decreases monotonically
        verify_monotonic_geometric_error(gml_id, fme_detail_levels, "FME")?;
        verify_monotonic_geometric_error(gml_id, flow_detail_levels, "Flow")?;

        let fme_highest_level = fme_detail_levels
            .last()
            .ok_or_else(|| format!("No detail levels for gml_id '{}' in FME", gml_id))?;
        for level in flow_detail_levels.iter() {
            compare_detail_level(gml_id, fme_highest_level, level)?;
        }
    }

    Ok(())
}

fn verify_monotonic_geometric_error(
    gml_id: &str,
    detail_levels: &[DetailLevel],
    source: &str,
) -> Result<(), String> {
    let mut prev_error = f64::INFINITY;

    for (i, level) in detail_levels.iter().enumerate() {
        if level.geometric_error < 0.0 || !level.geometric_error.is_finite() {
            return Err(format!(
                "{} gml_id '{}': invalid geometric error {} at level {}",
                source, gml_id, level.geometric_error, i
            ));
        }

        if level.geometric_error > prev_error {
            return Err(format!(
                "{} gml_id '{}': geometric error is not monotonically decreasing at level {} \
                 (previous: {}, current: {})",
                source, gml_id, i, prev_error, level.geometric_error
            ));
        }

        prev_error = level.geometric_error;
    }
    Ok(())
}

#[derive(Debug, Default)]
struct DetailLevelComparisonResult {
    gml_id: String,
    bounding_box_error: f64,
}

impl DetailLevelComparisonResult {
    fn new(gml_id: String) -> Self {
        Self {
            gml_id,
            ..Default::default()
        }
    }
}

fn compare_detail_level(
    gml_id: &str,
    fme_level: &DetailLevel,
    flow_level: &DetailLevel,
) -> Result<(), String> {
    let result = DetailLevelComparisonResult::new(gml_id.to_string());
    let fme_error = fme_level.geometric_error;
    let fme_geometry = &fme_level.multipolygon;
    let flow_error = flow_level.geometric_error;
    let flow_geometry = &flow_level.multipolygon;

    // compare bounding boxes
    let fme_bbox = fme_geometry.bounding_box();
    let flow_bbox = flow_geometry.bounding_box();
    eprintln!("{:?} - FME BBox: {:?}, Flow BBox: {:?}", result.gml_id, fme_bbox, flow_bbox);

    Ok(())
}
