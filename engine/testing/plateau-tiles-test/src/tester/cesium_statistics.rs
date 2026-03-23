use crate::conv::cesium_statistics::compute_cesium_statistics;
use crate::profile_config::ConvCesiumStatisticsEntry;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn get_bool(
    obj: &serde_json::Map<String, Value>,
    field: &str,
    ident: &str,
) -> Result<bool, String> {
    obj.get(field)
        .and_then(|v| v.as_bool())
        .ok_or_else(|| format!("ident '{}': field '{}' missing or not bool", ident, field))
}

fn get_f64_arr3(
    obj: &serde_json::Map<String, Value>,
    field: &str,
    ident: &str,
) -> Result<[f64; 3], String> {
    let arr = obj
        .get(field)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("ident '{}': field '{}' missing or not array", ident, field))?;
    if arr.len() != 3 {
        return Err(format!(
            "ident '{}': field '{}' must have 3 elements",
            ident, field
        ));
    }
    Ok([
        arr[0]
            .as_f64()
            .ok_or_else(|| format!("ident '{}': {}[0] not f64", ident, field))?,
        arr[1]
            .as_f64()
            .ok_or_else(|| format!("ident '{}': {}[1] not f64", ident, field))?,
        arr[2]
            .as_f64()
            .ok_or_else(|| format!("ident '{}': {}[2] not f64", ident, field))?,
    ])
}

fn assert_f64_arr3_close(
    ident: &str,
    field: &str,
    truth: [f64; 3],
    actual: [f64; 3],
    tol: f64,
) -> Result<(), String> {
    for i in 0..3 {
        let diff = (truth[i] - actual[i]).abs();
        if diff > tol {
            return Err(format!(
                "ident '{}': {}[{}] mismatch: truth={:.6}, actual={:.6}, diff={:.2e} (tol={:.2e})",
                ident, field, i, truth[i], actual[i], diff, tol
            ));
        }
    }
    Ok(())
}

fn compare_feature_stats(ident: &str, truth: &Value, actual: &Value) -> Result<(), String> {
    let t = truth
        .as_object()
        .ok_or_else(|| format!("ident '{}': truth not an object", ident))?;
    let a = actual
        .as_object()
        .ok_or_else(|| format!("ident '{}': actual not an object", ident))?;

    // texture_presence: exact bool
    let t_tex = get_bool(t, "texture_presence", ident)?;
    let a_tex = get_bool(a, "texture_presence", ident)?;
    if t_tex != a_tex {
        return Err(format!(
            "ident '{}': texture_presence: truth={}, actual={}",
            ident, t_tex, a_tex
        ));
    }

    // bbox: world coordinates — 1mm absolute tolerance
    let t_bbox_min = get_f64_arr3(t, "bbox_min", ident)?;
    let a_bbox_min = get_f64_arr3(a, "bbox_min", ident)?;
    assert_f64_arr3_close(ident, "bbox_min", t_bbox_min, a_bbox_min, 1e-3)?;

    let t_bbox_max = get_f64_arr3(t, "bbox_max", ident)?;
    let a_bbox_max = get_f64_arr3(a, "bbox_max", ident)?;
    assert_f64_arr3_close(ident, "bbox_max", t_bbox_max, a_bbox_max, 1e-3)?;

    // mass_center: world coordinates — 1mm absolute tolerance
    let t_mc = get_f64_arr3(t, "mass_center", ident)?;
    let a_mc = get_f64_arr3(a, "mass_center", ident)?;
    assert_f64_arr3_close(ident, "mass_center", t_mc, a_mc, 1e-3)?;

    // average_winding: dimensionless unit-ish vector — matches cesium.rs tolerance of 0.25
    let t_aw = get_f64_arr3(t, "average_winding", ident)?;
    let a_aw = get_f64_arr3(a, "average_winding", ident)?;
    assert_f64_arr3_close(ident, "average_winding", t_aw, a_aw, 0.25)?;

    Ok(())
}

pub fn test_cesium_statistics(
    truth_dir: &Path,
    flow_extracted_dir: &Path,
    entries: &HashMap<String, ConvCesiumStatisticsEntry>,
) -> Result<(), String> {
    for (id, entry) in entries {
        let flow_tileset_dir = flow_extracted_dir.join(&entry.path);
        let truth_path = truth_dir.join(&entry.truth_path);

        let actual = compute_cesium_statistics(&flow_tileset_dir)?;
        let truth: Value = serde_json::from_slice(
            &fs::read(&truth_path)
                .map_err(|e| format!("cesium_statistics/{}: truth not found: {}", id, e))?,
        )
        .map_err(|e| e.to_string())?;

        let truth_map = truth
            .as_object()
            .ok_or_else(|| format!("cesium_statistics/{}: truth not an object", id))?;
        let actual_map = actual
            .as_object()
            .ok_or_else(|| format!("cesium_statistics/{}: actual not an object", id))?;

        for (ident, truth_stats) in truth_map {
            let actual_stats = actual_map.get(ident).ok_or_else(|| {
                format!(
                    "cesium_statistics/{}: missing feature '{}' in actual",
                    id, ident
                )
            })?;
            compare_feature_stats(ident, truth_stats, actual_stats)?;
        }

        tracing::debug!("OK: cesium_statistics '{}'", id);
    }
    Ok(())
}
