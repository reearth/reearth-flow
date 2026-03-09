use crate::cast_config::{convert_casts, CastConfigValue};
use crate::compare_attributes::{apply_casts_to_value, make_feature_key};
use prost::Message;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tinymvt::tag::TagsDecoder;
use tinymvt::vector_tile::Tile;
use walkdir::WalkDir;

pub fn tinymvt_value_to_json(value: &tinymvt::tag::Value) -> Value {
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

/// Loads all MVT attributes from a directory, keyed by ident (or composite key for DM features)
pub fn load_mvt_attr(dir: &Path) -> Result<HashMap<String, Value>, String> {
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

/// Converts MVT tiles in `mvt_dir` to a JSON file at `output_path`, applying optional casts.
pub fn write_mvt_json(
    mvt_dir: &Path,
    output_path: &Path,
    casts_cfg: Option<&HashMap<String, CastConfigValue>>,
) -> Result<(), String> {
    let casts = if let Some(cfg) = casts_cfg {
        convert_casts(cfg)?
    } else {
        HashMap::new()
    };

    let raw = load_mvt_attr(mvt_dir)?;

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
