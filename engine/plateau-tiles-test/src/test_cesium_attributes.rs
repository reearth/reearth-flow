use crate::cast_config::{convert_casts, CastConfigValue};
use crate::compare_attributes::analyze_attributes;
use reearth_flow_gltf::{extract_feature_properties, parse_gltf};
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

/// Load attributes from FME JSON export, keyed by gml_id
fn load_json_attr(json_path: &Path) -> Result<HashMap<String, Value>, String> {
    let mut ret = HashMap::new();

    let content = fs::read_to_string(json_path)
        .map_err(|e| format!("Failed to read JSON file {:?}: {}", json_path, e))?;

    let features: Vec<Value> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON from {:?}: {}", json_path, e))?;

    for feature in features {
        if let Some(obj) = feature.as_object() {
            let gml_id = obj
                .get("gml_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("Feature missing gml_id: {:?}", feature))?;

            // Create a copy of the feature without geometry fields
            let mut props = obj.clone();
            props.remove("json_geometry");
            props.remove("json_ogc_wkt_crs");
            props.remove("json_featuretype");

            ret.insert(gml_id.to_string(), Value::Object(props));
        }
    }

    Ok(ret)
}

/// Load all GLB attributes from a directory, keyed by gml_id
fn load_glb_attr(dir: &Path) -> Result<HashMap<String, Value>, String> {
    let mut ret = HashMap::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "glb"))
    {
        let path = entry.path();
        let content = fs::read(path).map_err(|e| format!("Failed to read GLB: {}", e))?;
        let gltf = parse_gltf(&bytes::Bytes::from(content))
            .map_err(|e| format!("Failed to parse GLB: {}", e))?;

        // Extract feature properties using the gltf module
        let features = extract_feature_properties(&gltf)
            .map_err(|e| format!("Failed to extract features from {:?}: {}", path, e))?;

        for (gml_id, props) in features {
            ret.insert(gml_id, Value::Object(props));
        }
    }

    Ok(ret)
}

/// Align attributes from FME JSON export (dir1) and Flow GLB output (dir2)
fn align_glb_attr(fme_json: &Path, flow_dir: &Path) -> Result<Vec<(String, Value, Value)>, String> {
    // Load FME output from JSON export
    let map1 = load_json_attr(fme_json)?;

    // Load Flow output from GLB tiles
    let map2 = load_glb_attr(flow_dir)?;

    tracing::debug!(
        "Loaded attributes: {} from FME JSON {:?}, {} from Flow GLBs {:?}",
        map1.len(),
        fme_json,
        map2.len(),
        flow_dir
    );

    // Validate that keys match exactly
    let keys1: HashSet<_> = map1.keys().collect();
    let keys2: HashSet<_> = map2.keys().collect();

    if keys1 != keys2 {
        let only_in_fme: Vec<_> = keys1.difference(&keys2).collect();
        let only_in_flow: Vec<_> = keys2.difference(&keys1).collect();

        panic!(
            "FME: {} keys, flow: {} keys, Only in FME: {:?}, Only in Flow: {:?}",
            map1.len(),
            map2.len(),
            only_in_fme,
            only_in_flow
        );
    }

    let mut result = Vec::new();
    for key in keys1 {
        let attr1 = map1.get(key).cloned().unwrap();
        let attr2 = map2.get(key).cloned().unwrap();
        result.push((key.clone(), attr1, attr2));
    }

    Ok(result)
}

/// Tests Cesium 3D Tiles GLB attributes between FME and Flow outputs
///
/// FME output is expected to be a JSON export file (export.json) since
/// there's no plan to support FME's b3dm output with Draco decoding.
/// Flow output is a directory containing GLB tiles.
pub fn test_cesium_attributes(
    fme_json_path: &Path,
    flow_tiles_dir: &Path,
    config: &CesiumAttributesConfig,
) -> Result<(), String> {
    let casts = if let Some(casts_cfg) = &config.casts {
        convert_casts(casts_cfg)?
    } else {
        HashMap::new()
    };

    for (key, attr1, attr2) in align_glb_attr(fme_json_path, flow_tiles_dir)? {
        analyze_attributes(&key, &attr1, &attr2, casts.clone())?;
    }

    Ok(())
}
