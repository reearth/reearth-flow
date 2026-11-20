use crate::align_cesium::find_cesium_tile_directories;
use crate::conv::cesium_statistics::compute_cesium_statistics;
use crate::profile_config::{ConvCesiumStatisticsEntry, GeometryTest};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;

/// Statistics of one detail level of a feature (see [`compute_cesium_statistics`]).
#[derive(Debug)]
struct Level {
    geometric_error: f64,
    texture_presence: bool,
    total_area: f64,
    bbox: Option<([f64; 3], [f64; 3])>,
    mass_center: Option<[f64; 3]>,
    average_color: Option<[f64; 4]>,
    average_winding: Option<[f64; 3]>,
}

fn f64_array<const N: usize>(value: &Value) -> Option<[f64; N]> {
    let arr = value.as_array()?;
    if arr.len() != N {
        return None;
    }
    let mut out = [0.0; N];
    for (slot, v) in out.iter_mut().zip(arr) {
        *slot = v.as_f64()?;
    }
    Some(out)
}

fn parse_levels(ident: &str, value: &Value) -> Result<Vec<Level>, String> {
    let levels = value
        .get("levels")
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("ident '{}': 'levels' missing or not an array", ident))?;
    levels
        .iter()
        .enumerate()
        .map(|(i, level)| {
            let field = |name: &str| {
                level
                    .get(name)
                    .ok_or_else(|| format!("ident '{}': level {}: '{}' missing", ident, i, name))
            };
            let number = |name: &str| -> Result<f64, String> {
                field(name)?.as_f64().ok_or_else(|| {
                    format!("ident '{}': level {}: '{}' not a number", ident, i, name)
                })
            };
            let bbox = match (
                f64_array::<3>(field("bbox_min")?),
                f64_array::<3>(field("bbox_max")?),
            ) {
                (Some(min), Some(max)) => Some((min, max)),
                _ => None,
            };
            Ok(Level {
                geometric_error: number("geometric_error")?,
                texture_presence: field("texture_presence")?.as_bool().ok_or_else(|| {
                    format!(
                        "ident '{}': level {}: 'texture_presence' not bool",
                        ident, i
                    )
                })?,
                total_area: number("total_area")?,
                bbox,
                mass_center: f64_array::<3>(field("mass_center")?),
                average_color: f64_array::<4>(field("average_color")?),
                average_winding: f64_array::<3>(field("average_winding")?),
            })
        })
        .collect()
}

fn test_texture_presence(
    ident: &str,
    truth_levels: &[Level],
    flow_levels: &[Level],
) -> Result<(), String> {
    // all detail levels must have same texture presence
    let consistent = |levels: &[Level], source: &str| -> Result<bool, String> {
        let first = levels
            .first()
            .ok_or_else(|| format!("No detail levels for ident '{}' in {}", ident, source))?
            .texture_presence;
        if levels.iter().any(|l| l.texture_presence != first) {
            return Err(format!(
                "ident '{}': inconsistent texture presence in {} detail levels",
                ident, source
            ));
        }
        Ok(first)
    };
    let truth_has_texture = consistent(truth_levels, "Truth")?;
    let flow_has_texture = consistent(flow_levels, "Flow")?;
    if truth_has_texture != flow_has_texture {
        return Err(format!(
            "ident '{}': texture presence differs between Truth ({}) and Flow ({})",
            ident, truth_has_texture, flow_has_texture
        ));
    }
    Ok(())
}

/// Asserts geometric error decreases monotonically.
fn verify_monotonic_geometric_error(
    ident: &str,
    levels: &[Level],
    source: &str,
) -> Result<(), String> {
    let mut prev_error = f64::INFINITY;

    for (i, level) in levels.iter().enumerate() {
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

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Compares one Flow detail level to the highest-detail Truth level.
fn compare_detail_level(
    ident: &str,
    truth_level: &Level,
    flow_level: &Level,
    entry: &ConvCesiumStatisticsEntry,
) -> Result<(), String> {
    let truth_error = truth_level.geometric_error;
    let flow_error = flow_level.geometric_error;

    // If flow geometry is degenerate (all zero-area triangles), the object may have been
    // simplified away. This is acceptable if the Truth object's size is within the flow's
    // geometric error — i.e. the object is smaller than the LOD resolution.
    if flow_level.total_area == 0.0 {
        let (truth_min, truth_max) = truth_level
            .bbox
            .ok_or_else(|| format!("ident '{}': Cannot compute bbox: no triangles", ident))?;
        let bbox_diagonal = distance(truth_min, truth_max);
        if bbox_diagonal <= flow_error {
            tracing::debug!(
                "ident '{}': skipping geometry tests — flow geometry is degenerate and Truth \
                 object size ({:.6}) is within flow geometric error ({:.6})",
                ident,
                bbox_diagonal,
                flow_error
            );
            return Ok(());
        }
        return Err(format!(
            "ident '{}': flow geometry is degenerate (zero total area) but Truth geometry \
             size ({:.6}) exceeds flow geometric error ({:.6})",
            ident, bbox_diagonal, flow_error
        ));
    }

    if entry.enabled(GeometryTest::BoundingBox) {
        let (truth_min, truth_max) = truth_level
            .bbox
            .ok_or_else(|| format!("ident '{}': Cannot compute Truth bbox", ident))?;
        let (flow_min, flow_max) = flow_level
            .bbox
            .ok_or_else(|| format!("ident '{}': Cannot compute Flow bbox", ident))?;

        // if vertices have max error r, the bounding boxes can differ by at most r in each direction
        // thus the bounding box error is at most sqrt(3) * r. We simply use 2 * r as a safe upper bound.
        let bbox_error = 2.0 * (truth_error + flow_error);
        let error_min = distance(truth_min, flow_min) / bbox_error;
        let error_max = distance(truth_max, flow_max) / bbox_error;
        let bounding_box_error = error_min.max(error_max);
        if bounding_box_error > 1.0 {
            return Err(format!(
                "ident '{}': bounding box mismatch exceeds max error: {}/{}: min:({}, {}, {}), max:({}, {}, {})",
                ident,
                truth_error,
                flow_error,
                truth_min[0] - flow_min[0],
                truth_min[1] - flow_min[1],
                truth_min[2] - flow_min[2],
                truth_max[0] - flow_max[0],
                truth_max[1] - flow_max[1],
                truth_max[2] - flow_max[2],
            ));
        }
    }

    if entry.enabled(GeometryTest::MassCenter) {
        let truth_centroid = truth_level
            .mass_center
            .ok_or_else(|| format!("ident '{}': failed to compute Truth centroid", ident))?;
        let flow_centroid = flow_level
            .mass_center
            .ok_or_else(|| format!("ident '{}': failed to compute Flow centroid", ident))?;

        // if vertices have max error r, centroids can differ by at most r
        let centroid_error_bound = truth_error + flow_error;
        let mass_center_error = distance(truth_centroid, flow_centroid) / centroid_error_bound;
        if mass_center_error > 1.0 {
            return Err(format!(
                "ident '{}': mass center mismatch exceeds error bound: {}",
                ident, mass_center_error
            ));
        }
    }

    // Skip color comparison if textures are present
    // NOTE: (probably) diffuseColor in X3D material is used as base color, overriden by texture if present.
    // Therefore, we ignore color comparison when textures exist.
    if entry.enabled(GeometryTest::AverageColor) {
        let has_texture = truth_level.texture_presence || flow_level.texture_presence;
        if !has_texture {
            test_face_weighted_average_color(ident, truth_level, flow_level)?;
        }
    }

    if entry.enabled(GeometryTest::AverageWinding) {
        test_area_weighted_average_winding(ident, truth_level, flow_level)?;
    }

    Ok(())
}

/// Test face-weighted average color by comparing material base color × vertex color
fn test_face_weighted_average_color(
    ident: &str,
    truth_level: &Level,
    flow_level: &Level,
) -> Result<(), String> {
    let truth_avg_color = truth_level
        .average_color
        .ok_or_else(|| format!("ident '{}': Cannot compute Truth average color", ident))?;
    let flow_avg_color = flow_level
        .average_color
        .ok_or_else(|| format!("ident '{}': Cannot compute Flow average color", ident))?;

    // Check if colors should be treated as equivalent defaults
    // Flow default: ~(0.7, 0.7, 0.7, 1.0), Truth default: (1.0, 1.0, 1.0, 1.0)
    let is_truth_default = is_near_default_color(&truth_avg_color, &[1.0, 1.0, 1.0, 1.0]);
    let is_flow_default = is_near_default_color(&flow_avg_color, &[0.7, 0.7, 0.7, 1.0]);

    if is_truth_default && is_flow_default {
        // Both are default colors, treat as equivalent
        return Ok(());
    }

    // Compare average colors with tolerance
    let color_tolerance = 0.1; // Allow 10% difference per channel (0-1 range)
    let max_diff = truth_avg_color
        .iter()
        .zip(flow_avg_color)
        .map(|(t, f)| (t - f).abs())
        .fold(0.0f64, f64::max);

    if max_diff > color_tolerance {
        return Err(format!(
            "ident '{}': face-weighted average color differs by {:.4} (max allowed: {:.4})\n\
             Truth: {:?}\n\
             Flow: {:?}",
            ident, max_diff, color_tolerance, truth_avg_color, flow_avg_color,
        ));
    }

    Ok(())
}

/// Check if a color is close to a default color (within 5% tolerance per channel)
fn is_near_default_color(color: &[f64; 4], default: &[f64; 4]) -> bool {
    const DEFAULT_TOLERANCE: f64 = 0.05;
    color
        .iter()
        .zip(default)
        .all(|(c, d)| (c - d).abs() <= DEFAULT_TOLERANCE)
}

/// Compare area-weighted average winding vectors between Truth and Flow in Euclidean space.
/// Both vectors are dimensionless with magnitude in [0, 1].
fn test_area_weighted_average_winding(
    ident: &str,
    truth_level: &Level,
    flow_level: &Level,
) -> Result<(), String> {
    let truth_vec = truth_level
        .average_winding
        .ok_or_else(|| format!("ident '{}': Cannot compute Truth average winding", ident))?;
    let flow_vec = flow_level
        .average_winding
        .ok_or_else(|| format!("ident '{}': Cannot compute Flow average winding", ident))?;

    let diff_magnitude = distance(truth_vec, flow_vec);

    const TOLERANCE: f64 = 0.25;
    if diff_magnitude > TOLERANCE {
        return Err(format!(
            "ident '{}': area-weighted average winding differs by {:.4} (max allowed: {:.4})\n\
             Truth:  {:?}\n\
             Flow: {:?}",
            ident, diff_magnitude, TOLERANCE, truth_vec, flow_vec,
        ));
    }

    Ok(())
}

/// Compares the statistics of one tileset (see [`compute_cesium_statistics`]) of Truth and Flow.
fn compare_statistics(
    truth: &Value,
    flow: &Value,
    entry: &ConvCesiumStatisticsEntry,
) -> Result<(), String> {
    let truth_map = truth
        .as_object()
        .ok_or_else(|| "truth statistics not an object".to_string())?;
    let flow_map = flow
        .as_object()
        .ok_or_else(|| "flow statistics not an object".to_string())?;

    let truth_keys: BTreeSet<_> = truth_map.keys().collect();
    let flow_keys: BTreeSet<_> = flow_map.keys().collect();
    if truth_keys != flow_keys {
        return Err(format!(
            "ident mismatch between Truth and Flow:\nMissing in Flow: {:?}\nMissing in Truth: {:?}",
            truth_keys.difference(&flow_keys).collect::<Vec<_>>(),
            flow_keys.difference(&truth_keys).collect::<Vec<_>>(),
        ));
    }

    for (ident, truth_stats) in truth_map {
        let truth_levels = parse_levels(ident, truth_stats)?;
        let flow_levels = parse_levels(ident, &flow_map[ident])?;

        if entry.enabled(GeometryTest::TexturePresence) {
            test_texture_presence(ident, &truth_levels, &flow_levels)?;
        }

        if entry.enabled(GeometryTest::MonotonicGeometricError) {
            verify_monotonic_geometric_error(ident, &truth_levels, "Truth")?;
            verify_monotonic_geometric_error(ident, &flow_levels, "Flow")?;
        }

        // compare each Flow detail level to the highest-detail Truth level
        let truth_highest_level = truth_levels
            .last()
            .ok_or_else(|| format!("No detail levels for ident '{}' in Truth", ident))?;
        for flow_level in &flow_levels {
            compare_detail_level(ident, truth_highest_level, flow_level, entry)?;
        }
    }
    Ok(())
}

pub fn test_cesium_statistics(
    truth_dir: &Path,
    truth_extracted_dir: &Path,
    flow_extracted_dir: &Path,
    entries: &HashMap<String, ConvCesiumStatisticsEntry>,
) -> Result<(), String> {
    // Truth and Flow must produce the same set of 3D Tiles directories, all covered by an entry.
    let truth_dirs = find_cesium_tile_directories(truth_extracted_dir)?;
    let flow_dirs = find_cesium_tile_directories(flow_extracted_dir)?;
    if truth_dirs.is_empty() || flow_dirs.is_empty() {
        return Err("No 3D Tiles directories found".to_string());
    }
    if truth_dirs != flow_dirs {
        return Err(format!(
            "3D Tiles directories differ: Truth={:?}, Flow={:?}",
            truth_dirs, flow_dirs
        ));
    }
    let mut covered: Vec<_> = entries.values().map(|e| e.path.clone()).collect();
    covered.sort();
    if covered != truth_dirs {
        return Err(format!(
            "cesium_statistics entries {:?} do not cover the 3D Tiles directories {:?}",
            covered, truth_dirs
        ));
    }

    for (id, entry) in entries {
        let flow_tileset_dir = flow_extracted_dir.join(&entry.path);
        let truth_path = truth_dir.join(&entry.truth_path);

        let actual = compute_cesium_statistics(&flow_tileset_dir)?;
        let truth: Value = serde_json::from_slice(
            &fs::read(&truth_path)
                .map_err(|e| format!("cesium_statistics/{}: truth not found: {}", id, e))?,
        )
        .map_err(|e| e.to_string())?;

        compare_statistics(&truth, &actual, entry)
            .map_err(|e| format!("cesium_statistics/{}: {}", id, e))?;

        tracing::debug!("OK: cesium_statistics '{}'", id);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn entry(geometry_tests: &[(GeometryTest, bool)]) -> ConvCesiumStatisticsEntry {
        ConvCesiumStatisticsEntry {
            path: String::new(),
            truth_path: String::new(),
            generate_truth: false,
            geometry_tests: geometry_tests.iter().copied().collect(),
        }
    }

    fn level(geometric_error: f64, x: f64) -> Value {
        json!({
            "geometric_error": geometric_error,
            "texture_presence": false,
            "total_area": 10.0,
            "bbox_min": [x, 0.0, 0.0],
            "bbox_max": [x + 1.0, 1.0, 1.0],
            "mass_center": [x + 0.5, 0.5, 0.5],
            "average_color": [0.5, 0.5, 0.5, 1.0],
            "average_winding": [0.0, 0.0, 1.0],
        })
    }

    fn stats(levels: Vec<Value>) -> Value {
        json!({ "f": { "levels": levels } })
    }

    #[test]
    fn identical_statistics_pass() {
        let s = stats(vec![level(5.0, 0.0), level(0.0, 0.0)]);
        compare_statistics(&s, &s, &entry(&[])).unwrap();
    }

    #[test]
    fn lower_flow_level_is_compared_with_error_tolerance() {
        let truth = stats(vec![level(0.0, 0.0)]);
        // flow's coarse level is 1m off but its geometric error is 5m: tolerated
        let flow = stats(vec![level(5.0, 1.0), level(0.0, 0.0)]);
        compare_statistics(&truth, &flow, &entry(&[])).unwrap();
        // the same offset at a level with zero error is not
        let flow = stats(vec![level(0.0, 1.0)]);
        let err = compare_statistics(&truth, &flow, &entry(&[])).unwrap_err();
        assert!(err.contains("bounding box mismatch"), "{err}");
    }

    #[test]
    fn feature_sets_must_match() {
        let truth = stats(vec![level(0.0, 0.0)]);
        let flow = json!({ "g": { "levels": [level(0.0, 0.0)] } });
        let err = compare_statistics(&truth, &flow, &entry(&[])).unwrap_err();
        assert!(err.contains("ident mismatch"), "{err}");
    }

    #[test]
    fn texture_presence_must_match_and_can_be_disabled() {
        let truth = stats(vec![level(0.0, 0.0)]);
        let mut textured = level(0.0, 0.0);
        textured["texture_presence"] = json!(true);
        let flow = stats(vec![textured]);
        let err = compare_statistics(&truth, &flow, &entry(&[])).unwrap_err();
        assert!(err.contains("texture presence differs"), "{err}");
        compare_statistics(
            &truth,
            &flow,
            &entry(&[(GeometryTest::TexturePresence, false)]),
        )
        .unwrap();
    }

    #[test]
    fn geometric_error_must_decrease() {
        let truth = stats(vec![level(0.0, 0.0)]);
        let flow = stats(vec![level(0.0, 0.0), level(5.0, 0.0)]);
        let err = compare_statistics(&truth, &flow, &entry(&[])).unwrap_err();
        assert!(err.contains("not monotonically decreasing"), "{err}");
    }

    #[test]
    fn average_color_is_compared_unless_textured() {
        let truth = stats(vec![level(0.0, 0.0)]);
        let mut other = level(0.0, 0.0);
        other["average_color"] = json!([0.1, 0.1, 0.1, 1.0]);
        let flow = stats(vec![other]);
        let err = compare_statistics(&truth, &flow, &entry(&[])).unwrap_err();
        assert!(err.contains("average color"), "{err}");
        compare_statistics(
            &truth,
            &flow,
            &entry(&[(GeometryTest::AverageColor, false)]),
        )
        .unwrap();
    }

    #[test]
    fn average_winding_is_off_by_default() {
        let truth = stats(vec![level(0.0, 0.0)]);
        let mut other = level(0.0, 0.0);
        other["average_winding"] = json!([0.0, 0.0, -1.0]);
        let flow = stats(vec![other]);
        compare_statistics(&truth, &flow, &entry(&[])).unwrap();
        let err = compare_statistics(
            &truth,
            &flow,
            &entry(&[(GeometryTest::AverageWinding, true)]),
        )
        .unwrap_err();
        assert!(err.contains("average winding"), "{err}");
    }

    #[test]
    fn degenerate_flow_level_is_tolerated_only_within_its_error() {
        let truth = stats(vec![level(0.0, 0.0)]);
        let mut degenerate = level(5.0, 0.0);
        degenerate["total_area"] = json!(0.0);
        // truth bbox diagonal is sqrt(3) < 5
        compare_statistics(
            &truth,
            &stats(vec![degenerate.clone(), level(0.0, 0.0)]),
            &entry(&[]),
        )
        .unwrap();
        degenerate["geometric_error"] = json!(1.0);
        let err = compare_statistics(&truth, &stats(vec![degenerate]), &entry(&[])).unwrap_err();
        assert!(err.contains("flow geometry is degenerate"), "{err}");
    }
}
