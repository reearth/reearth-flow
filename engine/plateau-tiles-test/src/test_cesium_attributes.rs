use crate::align_cesium::{collect_geometries_by_ident, find_cesium_tile_directories};
use crate::cast_config::{convert_casts, CastConfigValue};
use crate::compare_attributes::analyze_attributes;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct CesiumAttributesConfig {
    pub casts: Option<HashMap<String, CastConfigValue>>,
}

/// Load tileset.json properties from a directory
fn load_tileset_properties(dir: &Path) -> Result<Value, String> {
    let tileset_path = dir.join("tileset.json");
    let content = std::fs::read_to_string(&tileset_path)
        .map_err(|e| format!("Failed to read tileset.json at {:?}: {}", tileset_path, e))?;
    let tileset: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse tileset.json at {:?}: {}", tileset_path, e))?;

    tileset
        .get("properties")
        .cloned()
        .ok_or_else(|| format!("No 'properties' field in tileset.json at {:?}", tileset_path))
}

/// Load all GLB attributes from a directory using the GeometryCollector
fn load_glb_attr(dir: &Path) -> Result<HashMap<String, Value>, String> {
    let collector = collect_geometries_by_ident(dir)?;
    Ok(collector.feature_attributes)
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

    for ident in all_keys {
        let attr1 = map1.get(ident).cloned().unwrap_or(Value::Null);
        let attr2 = map2.get(ident).cloned().unwrap_or(Value::Null);
        result.push((ident.clone(), attr1, attr2));
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

        // Compare tileset.json properties
        tracing::debug!("Comparing tileset.json properties for: {}", dir_name);
        let fme_properties = load_tileset_properties(&fme_dir)?;
        let flow_properties = load_tileset_properties(&flow_dir)?;
        analyze_attributes(
            &format!("{}/tileset.json", dir_name),
            &fme_properties,
            &flow_properties,
            casts.clone(),
        )?;

        // Compare GLB attributes
        for (ident, attr1, attr2) in align_glb_attr(&fme_dir, &flow_dir)? {
            analyze_attributes(&ident, &attr1, &attr2, casts.clone())?;
        }
    }

    Ok(())
}
