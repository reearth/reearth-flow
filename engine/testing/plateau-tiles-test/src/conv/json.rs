use crate::cast_config::{convert_casts, CastConfigValue};
use crate::compare_attributes::apply_casts_to_value;
use crate::conv::cesium::load_cesium_attr;
use crate::conv::mvt::load_mvt_attr;
use crate::legacy_keys::GroupLabels;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Collects the group labels from the 3D Tiles or MVT features in `dir`.
pub fn sibling_labels(dir: &Path) -> Result<GroupLabels, String> {
    let features = if dir.join("tileset.json").exists() {
        load_cesium_attr(dir, false)?
    } else {
        load_mvt_attr(dir, false)?
    };
    let mut labels = GroupLabels::default();
    for props in features.values() {
        if let Value::Object(props) = props {
            labels.add_feature(props);
        }
    }
    Ok(labels)
}

/// Writes the part of the JSON `source` at `json_path` to `output`, applying `casts`.
/// When `legacy_keys` is set, the keys are converted to the legacy keys first, labelling the risk
/// and generic attribute columns by the tiles next to `source`.
pub fn write_json(
    source: &Path,
    output: &Path,
    json_path: Option<&str>,
    casts: &HashMap<String, CastConfigValue>,
    legacy_keys: bool,
) -> Result<(), String> {
    let content = fs::read(source).map_err(|e| format!("Failed to read {:?}: {}", source, e))?;
    let mut data: Value = serde_json::from_slice(&content)
        .map_err(|e| format!("Failed to parse JSON {:?}: {}", source, e))?;

    if let Some(json_path) = json_path {
        data = json_path
            .split('.')
            .filter(|t| !t.is_empty())
            .try_fold(data, |cur, token| {
                cur.get(token)
                    .cloned()
                    .ok_or_else(|| format!("json_path '{}' not found in {:?}", json_path, source))
            })?;
    }

    if legacy_keys {
        let dir = source
            .parent()
            .ok_or_else(|| format!("{:?} has no parent directory", source))?;
        data = sibling_labels(dir)?
            .legacy_value(data)
            .map_err(|e| format!("Failed to convert keys in {:?}: {}", source, e))?;
    }

    if !casts.is_empty() {
        let cast_map = convert_casts(casts)?;
        data = apply_casts_to_value(data, "", &cast_map);
    }

    let out =
        serde_json::to_vec_pretty(&data).map_err(|e| format!("Failed to serialize JSON: {}", e))?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{:?}: {}", parent, e))?;
    }
    fs::write(output, out).map_err(|e| format!("{:?}: {}", output, e))?;
    Ok(())
}
