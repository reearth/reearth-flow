use crate::align_cesium::load_tileset;
use crate::cast_config::{convert_casts, CastConfigValue};
use crate::compare_attributes::{apply_casts_to_value, make_feature_key};
use crate::tileset::collect_tile_contents;
use indexmap::IndexMap;
use reearth_flow_gltf::{extract_feature_properties, parse_gltf};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Whether two copies of a feature's properties (from different tiles) agree.
/// A tile whose values for a column are all integral may store it as an integer
/// column while another tile stores the same column as float, so numbers compare
/// by value (`53` equals `53.0`).
fn same_properties(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y || x.as_f64() == y.as_f64(),
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| same_properties(v, w)))
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(v, w)| same_properties(v, w))
        }
        _ => a == b,
    }
}

/// Loads all Cesium 3D Tiles feature attributes from a tileset directory,
/// keyed by feature identifier (via `make_feature_key`).
/// Returns an IndexMap to preserve stable insertion order across runs.
pub fn load_cesium_attr(tileset_dir: &Path) -> Result<IndexMap<String, Value>, String> {
    let tileset_info = load_tileset(tileset_dir)?;
    let dir_name = tileset_dir.file_name().and_then(|n| n.to_str());

    let mut glb_paths: Vec<PathBuf> = match tileset_info.content.get("root") {
        Some(root) => collect_tile_contents(tileset_dir, root)?
            .into_iter()
            .map(|c| c.path)
            .collect(),
        None => Vec::new(),
    };
    glb_paths.sort();

    let mut ret: IndexMap<String, Value> = IndexMap::new();
    let mut rel: IndexMap<String, PathBuf> = IndexMap::new();

    for glb_path in &glb_paths {
        let content = fs::read(glb_path)
            .map_err(|e| format!("Failed to read GLB file {:?}: {}", glb_path, e))?;
        let gltf = parse_gltf(&bytes::Bytes::from(content))
            .map_err(|e| format!("Failed to parse GLB {:?}: {}", glb_path, e))?;

        let features = extract_feature_properties(&gltf)
            .map_err(|e| format!("Failed to extract features from {:?}: {}", glb_path, e))?;

        for props in features {
            let props_value = Value::Object(props);
            let feature_key = make_feature_key(&props_value, dir_name);

            if let Some(existing) = ret.get(&feature_key) {
                if !same_properties(existing, &props_value) {
                    let existing_path = rel.get(&feature_key).unwrap();
                    let differing: Vec<String> = match (existing, &props_value) {
                        (Value::Object(a), Value::Object(b)) => {
                            let mut keys: Vec<&String> = a
                                .keys()
                                .chain(b.keys())
                                .filter(|k| a.get(*k) != b.get(*k))
                                .collect();
                            keys.dedup();
                            keys.into_iter()
                                .map(|k| format!("{k}: {:?} vs {:?}", a.get(k), b.get(k)))
                                .collect()
                        }
                        _ => Vec::new(),
                    };
                    return Err(format!(
                        "Conflicting feature_key {}: properties differ between {:?} and {:?} (differing keys: {:?})",
                        feature_key, existing_path, glb_path, differing
                    ));
                }
            } else {
                ret.insert(feature_key.clone(), props_value);
                rel.insert(feature_key, glb_path.clone());
            }
        }
    }

    Ok(ret)
}

/// Converts Cesium 3D Tiles attributes to a JSON file at `output_path`, applying optional casts.
pub fn write_cesium_json(
    tileset_dir: &Path,
    output_path: &Path,
    casts_cfg: Option<&HashMap<String, CastConfigValue>>,
) -> Result<(), String> {
    let casts = if let Some(cfg) = casts_cfg {
        convert_casts(cfg)?
    } else {
        HashMap::new()
    };

    let raw = load_cesium_attr(tileset_dir)?;

    let normalized: serde_json::Map<String, Value> = raw
        .into_iter()
        .map(|(feature_key, props)| {
            let props = apply_casts_to_value(props, "", &casts);
            (feature_key, props)
        })
        .collect();

    let json = serde_json::to_string_pretty(&Value::Object(normalized))
        .map_err(|e| format!("Failed to serialize: {}", e))?;

    fs::write(output_path, &json)
        .map_err(|e| format!("Failed to write {}: {}", output_path.display(), e))?;

    Ok(())
}
