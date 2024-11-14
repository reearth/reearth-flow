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
