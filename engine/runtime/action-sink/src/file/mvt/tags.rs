use nusamai_mvt::tag::TagsEncoder;
use reearth_flow_types::AttributeValue;

pub fn convert_properties(
    tags: &mut Vec<u32>,
    tags_enc: &mut TagsEncoder,
    name: &str,
    tree: &AttributeValue,
) {
    match &tree {
        AttributeValue::Null => {
            // ignore
        }
        AttributeValue::String(v) => {
            tags.extend(tags_enc.add(name, v.clone().into()));
        }
        AttributeValue::Bool(v) => {
            tags.extend(tags_enc.add(name, (*v).into()));
        }
        AttributeValue::Number(v) => {
            if let Some(v) = v.as_u64() {
                tags.extend(tags_enc.add(name, v.into()));
            } else if let Some(v) = v.as_i64() {
                tags.extend(tags_enc.add(name, v.into()));
            } else if let Some(v) = v.as_f64() {
                tags.extend(tags_enc.add(name, v.into()));
            } else {
                // Handle any remaining number types by converting to string
                tags.extend(tags_enc.add(name, v.to_string().into()));
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
                convert_properties(tags, tags_enc, key, value);
            }
        }
        AttributeValue::DateTime(v) => {
            tags.extend(tags_enc.add(name, v.to_string().into()));
        }
    }
}
