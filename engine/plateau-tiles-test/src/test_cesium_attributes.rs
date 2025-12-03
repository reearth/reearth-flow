use crate::cast_config::{convert_casts, CastConfigValue};
use crate::compare_attributes::analyze_attributes;
use reearth_flow_gltf::parse_gltf;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct CesiumAttributesConfig {
    pub casts: Option<HashMap<String, CastConfigValue>>,
}

/// Load all GLB attributes from a directory, keyed by feature ID
fn load_glb_attr(dir: &Path) -> Result<HashMap<String, Value>, String> {
    let mut ret = HashMap::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "glb"))
    {
        let path = entry.path();
        let content = fs::read(path).map_err(|e| format!("Failed to read GLB: {}", e))?;
        let gltf = parse_gltf(&bytes::Bytes::from(content))
            .map_err(|e| format!("Failed to parse GLB: {}", e))?;

        // Extract attributes from EXT_structural_metadata if present
        if let Some(_metadata) = gltf.extension_value("EXT_structural_metadata") {
            // TODO: Parse property tables and extract attributes
            // For now, store minimal info
            tracing::debug!("Found EXT_structural_metadata in {:?}", path);
        }

        // For now, create a simple placeholder based on mesh names
        for (mesh_idx, mesh) in gltf.meshes().enumerate() {
            let mut props = serde_json::Map::new();
            if let Some(name) = mesh.name() {
                props.insert("mesh_name".to_string(), Value::String(name.to_string()));
            }
            props.insert("mesh_index".to_string(), Value::Number(mesh_idx.into()));

            let key = format!("{}_{}", path.file_stem().unwrap().to_string_lossy(), mesh_idx);
            ret.insert(key, Value::Object(props));
        }
    }

    Ok(ret)
}

/// Align GLB attributes from two directories
fn align_glb_attr(
    dir1: &Path,
    dir2: &Path,
) -> Result<Vec<(String, Value, Value)>, String> {
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

    for key in all_keys {
        let attr1 = map1.get(key).cloned().unwrap_or(Value::Null);
        let attr2 = map2.get(key).cloned().unwrap_or(Value::Null);
        result.push((key.clone(), attr1, attr2));
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

    for (key, attr1, attr2) in align_glb_attr(fme_path, flow_path)? {
        analyze_attributes(&key, &attr1, &attr2, casts.clone())?;
    }

    Ok(())
}
