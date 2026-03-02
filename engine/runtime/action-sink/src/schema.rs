use nusamai_citygml::schema::{Schema, TypeDef, TypeRef};
use reearth_flow_types::{Attribute, AttributeValue, Attributes, Feature};
// FIXME: remove this module and disable auto fallback to __citygml_feature_type
use reearth_flow_types::CitygmlFeatureExt;

/// Get the attribute definitions map for a feature type from the schema.
/// Returns None if the feature type is not found or is a Property type.
pub fn schema_attributes<'a>(
    feature_type: &str,
    schema: &'a Schema,
) -> Option<&'a nusamai_citygml::schema::Map> {
    schema.types.get(feature_type).and_then(|td| match td {
        TypeDef::Feature(ft) => Some(&ft.attributes),
        TypeDef::Data(dt) => Some(&dt.attributes),
        TypeDef::Property(_) => None,
    })
}

/// Cast an AttributeValue to match the expected schema TypeRef.
/// Returns the value unchanged if it already matches or conversion is not possible.
pub fn cast_attribute_value(value: &AttributeValue, type_ref: &TypeRef) -> AttributeValue {
    match type_ref {
        TypeRef::Integer => match value {
            AttributeValue::Number(_) => value.clone(),
            AttributeValue::String(s) => s
                .parse::<i64>()
                .ok()
                .map(|n| AttributeValue::Number(serde_json::Number::from(n)))
                .unwrap_or_else(|| value.clone()),
            AttributeValue::Bool(b) => {
                AttributeValue::Number(serde_json::Number::from(if *b { 1i64 } else { 0i64 }))
            }
            _ => value.clone(),
        },
        TypeRef::NonNegativeInteger => match value {
            AttributeValue::Number(_) => value.clone(),
            AttributeValue::String(s) => s
                .parse::<u64>()
                .ok()
                .map(|n| AttributeValue::Number(serde_json::Number::from(n)))
                .unwrap_or_else(|| value.clone()),
            _ => value.clone(),
        },
        TypeRef::Double | TypeRef::Measure => match value {
            AttributeValue::Number(_) => value.clone(),
            AttributeValue::String(s) => s
                .parse::<f64>()
                .ok()
                .and_then(serde_json::Number::from_f64)
                .map(AttributeValue::Number)
                .unwrap_or_else(|| value.clone()),
            _ => value.clone(),
        },
        TypeRef::Boolean => match value {
            AttributeValue::Bool(_) => value.clone(),
            AttributeValue::String(s) => match s.to_lowercase().as_str() {
                "true" | "1" => AttributeValue::Bool(true),
                "false" | "0" => AttributeValue::Bool(false),
                _ => value.clone(),
            },
            AttributeValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    AttributeValue::Bool(i != 0)
                } else {
                    value.clone()
                }
            }
            _ => value.clone(),
        },
        TypeRef::String | TypeRef::Code | TypeRef::URI | TypeRef::Date | TypeRef::DateTime => {
            match value {
                AttributeValue::String(_) => value.clone(),
                AttributeValue::Number(n) => AttributeValue::String(n.to_string()),
                AttributeValue::Bool(b) => AttributeValue::String(b.to_string()),
                AttributeValue::DateTime(dt) => AttributeValue::String(dt.to_string()),
                _ => value.clone(),
            }
        }
        // Unknown, JsonString, Point, Named — pass through unchanged
        _ => value.clone(),
    }
}

/// Filter feature attributes by schema and cast values to match schema types.
/// `schema_key` is an optional attribute key whose value is used to look up the schema type;
/// falls back to `feature_type()` when absent or when the attribute is not found.
/// If no schema is found, returns attributes unchanged.
pub fn filter_and_cast_attributes(
    feature: &Feature,
    schema: &Schema,
    schema_key: Option<&str>,
) -> Attributes {
    let schema_type = schema_key
        .and_then(|key| feature.get(key))
        .and_then(|v| v.as_string())
        .or_else(|| feature.feature_type());
    let Some(schema_attrs) = schema_type
        .as_ref()
        .and_then(|ft| schema_attributes(ft, schema))
    else {
        return feature.attributes.as_ref().clone();
    };

    schema_attrs
        .iter()
        .filter_map(|(schema_key, attr_def)| {
            let value = feature.get(schema_key)?;
            Some((
                Attribute::new(schema_key.clone()),
                cast_attribute_value(value, &attr_def.type_ref),
            ))
        })
        .collect()
}
