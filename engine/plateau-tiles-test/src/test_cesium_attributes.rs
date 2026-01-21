use crate::align_cesium::{collect_glb_paths_from_tileset, find_cesium_tile_directories};
use crate::cast_config::{convert_casts, CastConfigValue};
use crate::compare_attributes::analyze_attributes;
use reearth_flow_gltf::{extract_feature_properties, parse_gltf};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct CesiumAttributesConfig {
    pub casts: Option<HashMap<String, CastConfigValue>>,
}

/// Load all GLB attributes from a directory using tileset.json, keyed by gml_id
fn load_glb_attr(dir: &Path) -> Result<HashMap<String, Value>, String> {
    let mut ret = HashMap::new();
    let mut rel = HashMap::new();

    // Collect GLB paths from tileset.json
    let glb_paths = collect_glb_paths_from_tileset(dir)?;

    for path in glb_paths {
        let content = fs::read(&path).map_err(|e| format!("Failed to read GLB: {}", e))?;
        let gltf = parse_gltf(&bytes::Bytes::from(content))
            .map_err(|e| format!("Failed to parse GLB: {}", e))?;

        // Extract feature properties using the gltf module
        let features = extract_feature_properties(&gltf)
            .map_err(|e| format!("Failed to extract features from {:?}: {}", path, e))?;

        for (gml_id, props) in features {
            if let Some(existing) = ret.get(&gml_id) {
                if existing != &Value::Object(props.clone()) {
                    let existing_path = rel.get(&gml_id).unwrap();
                    tracing::debug!(
                        "Conflict for gml_id {}: {:?} vs {:?}",
                        gml_id,
                        existing,
                        props
                    );
                    return Err(format!(
                        "Conflicting gml_id {}: properties differ between {:?} and {:?}",
                        gml_id, existing_path, path
                    ));
                }
            } else {
                ret.insert(gml_id.clone(), Value::Object(props));
                rel.insert(gml_id, path.clone());
            }
        }
    }

    Ok(ret)
}

/// Align attributes from two GLB directories
fn align_glb_attr(dir1: &Path, dir2: &Path) -> Result<Vec<(String, Value, Value)>, String> {
    let map1 = load_glb_attr(dir1)?;
    let map2 = load_glb_attr(dir2)?;

    tracing::debug!(
        "Loaded GLB attributes: {} from {:?}, {} from {:?}",
        map1.len(),
        dir1,
        map2.len(),
        dir2
    );

    let mut result = Vec::new();

    let all_keys: HashSet<_> = map1.keys().chain(map2.keys()).collect();

    for gml_id in all_keys {
        let attr1 = map1.get(gml_id).cloned().unwrap_or(Value::Null);
        let attr2 = map2.get(gml_id).cloned().unwrap_or(Value::Null);
        result.push((gml_id.clone(), attr1, attr2));
    }

    Ok(result)
}

/// Tests Cesium 3D Tiles GLB attributes between FME and Flow outputs
pub fn test_cesium_attributes(
    fme_path: &Path,
    flow_path: &Path,
    config: &CesiumAttributesConfig,
) -> Result<(), String> {
    let casts = if let Some(casts_cfg) = &config.casts {
        convert_casts(casts_cfg)?
    } else {
        HashMap::new()
    };

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

    // Compare each directory pair
    for dir_name in &fme_dirs {
        let fme_dir = fme_path.join(dir_name);
        let flow_dir = flow_path.join(dir_name);

        tracing::debug!("Comparing Cesium attributes in directory: {}", dir_name);

        for (gml_id, attr1, attr2) in align_glb_attr(&fme_dir, &flow_dir)? {
            analyze_attributes(&gml_id, &attr1, &attr2, casts.clone())?;
        }
    }

    Ok(())
}
