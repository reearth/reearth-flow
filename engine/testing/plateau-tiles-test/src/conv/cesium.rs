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
                if existing != &props_value {
                    let existing_path = rel.get(&feature_key).unwrap();
                    return Err(format!(
                        "Conflicting feature_key {}: properties differ between {:?} and {:?}",
                        feature_key, existing_path, glb_path
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
