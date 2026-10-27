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
                convert_properties(tags_enc, key.as_str(), value);
            }
        }
        AttributeValue::DateTime(v) => {
            tags_enc.add(name, v.to_string());
        }
    }
}

/// Adds `leaf` as the tag `path`. Maps and arrays are dropped: a tag holds one
/// scalar.
#[cfg(feature = "new-geometry")]
pub fn convert_scalar_property(tags_enc: &mut TagsEncoder, path: &str, leaf: &AttributeValue) {
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
        AttributeValue::Bytes(_) | AttributeValue::Map(_) | AttributeValue::Array(_) => {
            // ignore
        }
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn maps_and_arrays_are_dropped_and_scalars_kept() {
        let mut tags_enc = TagsEncoder::default();
        let scalar: AttributeValue = json!(1).into();
        convert_scalar_property(&mut tags_enc, "a", &scalar);
        let map: AttributeValue = json!({"c": "x"}).into();
        convert_scalar_property(&mut tags_enc, "b", &map);
        let array: AttributeValue = json!([1, 2]).into();
        convert_scalar_property(&mut tags_enc, "d", &array);
        let (keys, _) = tags_enc.into_keys_and_values();
        assert_eq!(keys, vec!["a".to_string()]);
    }
}
