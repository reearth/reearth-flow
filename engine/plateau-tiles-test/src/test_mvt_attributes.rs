use crate::cast_config::{convert_casts, CastConfigValue};
use crate::compare_attributes::analyze_attributes;
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

/// Loads all MVT attributes from a directory, keyed by gml_id
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

                // Extract gml_id
                let gml_id = props
                    .get("gml_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing gml_id in feature properties".to_string())?
                    .to_string();

                if let Some(existing) = ret.get(&gml_id) {
                    if existing != &props {
                        let existing_path = rel.get(&gml_id).unwrap();
                        return Err(format!(
                            "Conflicting gml_id {}: properties differ between {:?} and {:?}",
                            gml_id, existing_path, path
                        ));
                    }
                } else {
                    ret.insert(gml_id.clone(), props);
                    rel.insert(gml_id, path.to_path_buf());
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

/// Aligns MVT attributes from two directories by gml_id
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

    for gml_id in all_keys {
        let attr1 = map1.get(gml_id).cloned().unwrap_or(Value::Null);
        let attr2 = map2.get(gml_id).cloned().unwrap_or(Value::Null);
        result.push((gml_id.clone(), attr1, attr2));
    }

    Ok(result)
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

    // Find all top-level directories containing .mvt files
    let fme_tops: HashSet<_> = WalkDir::new(fme_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "mvt"))
        .filter_map(|e| {
            let rel = e.path().strip_prefix(fme_path).ok()?;
            let parts: Vec<_> = rel.iter().collect();
            if parts.len() > 1 {
                parts[0].to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();

    let flow_tops: HashSet<_> = WalkDir::new(flow_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "mvt"))
        .filter_map(|e| {
            let rel = e.path().strip_prefix(flow_path).ok()?;
            let parts: Vec<_> = rel.iter().collect();
            if parts.len() > 1 {
                parts[0].to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();

    if fme_tops.is_empty() || flow_tops.is_empty() {
        return Err("No MVT files found".to_string());
    }
    if fme_tops != flow_tops {
        return Err(format!(
            "MVT top-level directories differ: FME={:?}, Flow={:?}",
            fme_tops, flow_tops
        ));
    }

    let mut all_tops: Vec<_> = fme_tops.union(&flow_tops).cloned().collect();
    all_tops.sort();

    for top_dir in all_tops {
        let fme_dir = fme_path.join(&top_dir);
        let flow_dir = flow_path.join(&top_dir);

        for (gml_id, attr1, attr2) in align_mvt_attr(&fme_dir, &flow_dir)? {
            analyze_attributes(&gml_id, &attr1, &attr2, casts.clone())?;
        }
    }

    Ok(())
}
