use reearth_flow_types::AttributeValue;
use tinymvt::tag::TagsEncoder;

pub fn convert_properties(tags_enc: &mut TagsEncoder, name: &str, tree: &AttributeValue) {
    match &tree {
        AttributeValue::Null => {
            // ignore
        }
        AttributeValue::String(v) => {
            tags_enc.add(name, v.clone());
        }
        AttributeValue::Bool(v) => {
            tags_enc.add(name, *v);
        }
        AttributeValue::Number(v) => {
            if let Some(v) = v.as_u64() {
                tags_enc.add(name, v);
            } else if let Some(v) = v.as_i64() {
                tags_enc.add(name, v);
            } else if let Some(v) = v.as_f64() {
                tags_enc.add(name, v);
            } else {
                // Handle any remaining number types by converting to string
                tags_enc.add(name, v.to_string());
            }
        }
        AttributeValue::Array(_arr) => {
            // ignore non-root attributes
        }
        AttributeValue::Bytes(_v) => {
            // ignore non-root attributes
        }
        AttributeValue::Map(obj) => {
            for (key, value) in obj {
                convert_properties(tags_enc, key, value);
            }
        }
        AttributeValue::DateTime(v) => {
            tags_enc.add(name, v.to_string());
        }
    }
}

#[cfg(feature = "new-geometry")]
pub fn convert_properties_with_separator(
    tags_enc: &mut TagsEncoder,
    name: &str,
    tree: &AttributeValue,
    separator: Option<&str>,
) {
    match separator {
        Some(sep) => flatten(tags_enc, name.to_string(), tree, sep),
        None if matches!(tree, AttributeValue::Map(_) | AttributeValue::Array(_)) => {
            // Dropped, same as the Cesium 3D Tiles writer's array_map_separator: None.
        }
        None => insert_leaf(tags_enc, name, tree),
    }
}

#[cfg(feature = "new-geometry")]
fn flatten(tags_enc: &mut TagsEncoder, path: String, value: &AttributeValue, sep: &str) {
    match value {
        AttributeValue::Map(obj) => {
            for (key, child) in obj {
                flatten(tags_enc, format!("{path}{sep}{key}"), child, sep);
            }
        }
        AttributeValue::Array(items) => {
            for (i, child) in items.iter().enumerate() {
                flatten(tags_enc, format!("{path}{sep}{i}"), child, sep);
            }
        }
        leaf => insert_leaf(tags_enc, &path, leaf),
    }
}

#[cfg(feature = "new-geometry")]
fn insert_leaf(tags_enc: &mut TagsEncoder, path: &str, leaf: &AttributeValue) {
    match leaf {
        AttributeValue::Null => {
            // ignore
        }
        AttributeValue::String(v) => {
            tags_enc.add(path, v.clone());
        }
        AttributeValue::Bool(v) => {
            tags_enc.add(path, *v);
        }
        AttributeValue::Number(v) => {
            if let Some(v) = v.as_u64() {
                tags_enc.add(path, v);
            } else if let Some(v) = v.as_i64() {
                tags_enc.add(path, v);
            } else if let Some(v) = v.as_f64() {
                tags_enc.add(path, v);
            } else {
                tags_enc.add(path, v.to_string());
            }
        }
        AttributeValue::DateTime(v) => {
            tags_enc.add(path, v.to_string());
        }
        AttributeValue::Bytes(_) => {
            // ignore
        }
        AttributeValue::Map(_) | AttributeValue::Array(_) => {
            unreachable!("insert_leaf is only called on non-Map/Array values")
        }
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use serde_json::json;

    fn encode(
        f: impl FnOnce(&mut TagsEncoder),
    ) -> (Vec<String>, Vec<tinymvt::vector_tile::tile::Value>) {
        let mut tags_enc = TagsEncoder::default();
        f(&mut tags_enc);
        tags_enc.into_keys_and_values()
    }

    #[test]
    fn none_separator_drops_maps_and_arrays_keeps_scalars() {
        let (keys, _) = encode(|enc| {
            let scalar: AttributeValue = json!(1).into();
            convert_properties_with_separator(enc, "a", &scalar, None);
            let map: AttributeValue = json!({"c": "x"}).into();
            convert_properties_with_separator(enc, "b", &map, None);
            let array: AttributeValue = json!([1, 2]).into();
            convert_properties_with_separator(enc, "d", &array, None);
        });
        assert_eq!(keys, vec!["a".to_string()]);
    }

    #[test]
    fn separator_flattens_nested_maps_and_arrays() {
        let tree: AttributeValue = json!({"b": {"c": "x"}, "d": [1, 2]}).into();
        let (keys, _) =
            encode(|enc| convert_properties_with_separator(enc, "root", &tree, Some(".")));
        assert!(keys.contains(&"root.b.c".to_string()));
        assert!(keys.contains(&"root.d.0".to_string()));
        assert!(keys.contains(&"root.d.1".to_string()));
    }
}
