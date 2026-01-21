use crate::cast_config::{convert_casts, CastConfigValue};
use crate::compare_attributes::{analyze_attributes, make_feature_key};
use prost::Message;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use tinymvt::tag::TagsDecoder;
use tinymvt::vector_tile::Tile;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct MvtAttributesConfig {
    pub casts: Option<HashMap<String, CastConfigValue>>,
}

/// Loads all MVT attributes from a directory, keyed by ident (or composite key for DM features)
fn load_mvt_attr(dir: &Path) -> Result<HashMap<String, Value>, String> {
    let mut ret = HashMap::new();
    let mut rel = HashMap::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "mvt"))
    {
        let path = entry.path();
        let data = fs::read(path).map_err(|e| format!("Failed to read MVT file: {}", e))?;

        let tile =
            Tile::decode(&data[..]).map_err(|e| format!("Failed to decode MVT protobuf: {}", e))?;

        for layer in tile.layers {
            let tags_decoder = TagsDecoder::new(&layer.keys, &layer.values);

            for feature in layer.features {
                // Decode tags to get properties
                let props = match tags_decoder.decode(&feature.tags) {
                    Ok(tags) => {
                        let mut map = serde_json::Map::new();
                        for (key, value) in tags {
                            let json_value = tinymvt_value_to_json(&value);
                            map.insert(key.to_string(), json_value);
                        }
                        Value::Object(map)
                    }
                    Err(e) => {
                        return Err(format!("Failed to decode tags: {}", e));
                    }
                };

                // Create composite key for DM features to handle multiple DmGeometricAttribute per parent
                let feature_key = make_feature_key(&props, None);

                if let Some(existing) = ret.get(&feature_key) {
                    if existing != &props {
                        let existing_path = rel.get(&feature_key).unwrap();
                        return Err(format!(
                            "Conflicting feature_key {}: properties differ between {:?} and {:?}",
                            feature_key, existing_path, path
                        ));
                    }
                } else {
                    ret.insert(feature_key.clone(), props);
                    rel.insert(feature_key, path.to_path_buf());
                }
            }
        }
    }

    Ok(ret)
}

/// Converts tinymvt::tag::Value to serde_json::Value
fn tinymvt_value_to_json(value: &tinymvt::tag::Value) -> Value {
    use tinymvt::tag::Value as TValue;
    match value {
        TValue::String(s) => Value::String(s.clone()),
        TValue::Float(bytes) => {
            let f = f32::from_ne_bytes(*bytes);
            json!(f)
        }
        TValue::Double(bytes) => {
            let d = f64::from_ne_bytes(*bytes);
            json!(d)
        }
        TValue::Int(i) => json!(i),
        TValue::Uint(u) => json!(u),
        TValue::SInt(s) => json!(s),
        TValue::Bool(b) => json!(b),
    }
}

/// Aligns MVT attributes from two directories by ident
fn align_mvt_attr(dir1: &Path, dir2: &Path) -> Result<Vec<(String, Value, Value)>, String> {
    let map1 = load_mvt_attr(dir1)?;
    let map2 = load_mvt_attr(dir2)?;

    tracing::debug!(
        "Loaded MVT attributes: {} from {:?}, {} from {:?}",
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

/// Find top-level MVT tile directories (directories containing tilejson.json)
fn find_mvt_tile_directories(base_path: &Path) -> Result<Vec<String>, String> {
    let mut dirs = HashSet::new();

    for entry in WalkDir::new(base_path)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.file_name() == "tilejson.json")
    {
        if let Ok(rel) = entry.path().parent().unwrap().strip_prefix(base_path) {
            if let Some(first_component) = rel.iter().next() {
                dirs.insert(first_component.to_string_lossy().to_string());
            }
        }
    }

    let mut result: Vec<_> = dirs.into_iter().collect();
    result.sort();
    Ok(result)
}

/// Compares tilejson.json files between FME and Flow outputs
/// Currently compare only names
fn compare_tilejson(fme_dir: &Path, flow_dir: &Path) -> Result<(), String> {
    let fme_tilejson_path = fme_dir.join("tilejson.json");
    let flow_tilejson_path = flow_dir.join("tilejson.json");

    // Read both tilejson.json
    let fme_content = fs::read_to_string(&fme_tilejson_path).unwrap();
    let flow_content = fs::read_to_string(&flow_tilejson_path).unwrap();
    let fme_json: Value = serde_json::from_str(&fme_content).unwrap();
    let flow_json: Value = serde_json::from_str(&flow_content).unwrap();

    // Compare "name" field
    let fme_name = fme_json.get("name");
    let flow_name = flow_json.get("name");

    if fme_name != flow_name {
        return Err(format!(
            "tilejson.json 'name' field mismatch: FME={:?}, Flow={:?}",
            fme_name, flow_name
        ));
    }

    Ok(())
}

/// Tests MVT attributes between FME and Flow outputs
pub fn test_mvt_attributes(
    fme_path: &Path,
    flow_path: &Path,
    config: &MvtAttributesConfig,
) -> Result<(), String> {
    let casts = if let Some(casts_cfg) = &config.casts {
        convert_casts(casts_cfg)?
    } else {
        HashMap::new()
    };

    // Find top-level MVT tile directories
    let fme_dirs = find_mvt_tile_directories(fme_path)?;
    let flow_dirs = find_mvt_tile_directories(flow_path)?;

    if fme_dirs.is_empty() || flow_dirs.is_empty() {
        return Err("No MVT tile directories found".to_string());
    }
    if fme_dirs != flow_dirs {
        return Err(format!(
            "MVT tile directories differ: FME={:?}, Flow={:?}",
            fme_dirs, flow_dirs
        ));
    }

    // Compare each directory pair
    for dir_name in &fme_dirs {
        let fme_dir = fme_path.join(dir_name);
        let flow_dir = flow_path.join(dir_name);

        tracing::debug!("Comparing MVT attributes in directory: {}", dir_name);

        // Compare tilejson.json files
        compare_tilejson(&fme_dir, &flow_dir)?;

        // Compare MVT attributes
        for (ident, attr1, attr2) in align_mvt_attr(&fme_dir, &flow_dir)? {
            analyze_attributes(&ident, &attr1, &attr2, casts.clone())?;
        }
    }

    Ok(())
}
