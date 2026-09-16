use indexmap::IndexMap;
use nusamai_citygml::schema::{Schema, TypeDef, TypeRef};
use reearth_flow_gltf::tiles::metadata::{flatten_attributes, MetadataOptions};
use reearth_flow_types::{AttributeValue, Feature};
use serde_json::Number;

use crate::schema::schema_attributes;

#[derive(Default)]
pub(super) struct PropertyStats {
    pub(super) minimum: Option<Number>,
    pub(super) maximum: Option<Number>,
}

impl PropertyStats {
    fn update(&mut self, n: &Number) {
        let Some(v) = n.as_f64() else { return };
        if self
            .minimum
            .as_ref()
            .and_then(Number::as_f64)
            .is_none_or(|m| v < m)
        {
            self.minimum = Some(n.clone());
        }
        if self
            .maximum
            .as_ref()
            .and_then(Number::as_f64)
            .is_none_or(|m| v > m)
        {
            self.maximum = Some(n.clone());
        }
    }
}

pub(super) fn collect<'a>(
    features: impl IntoIterator<Item = &'a Feature>,
    schema: &Schema,
    options: MetadataOptions,
) -> IndexMap<String, PropertyStats> {
    // Pre-initialize from the schema so every declared attribute is listed, in
    // schema order, even when no feature carries a value for it.
    let mut stats: IndexMap<String, PropertyStats> = IndexMap::new();
    for typedef in schema.types.values() {
        if let TypeDef::Feature(fdef) = typedef {
            for key in fdef.attributes.keys() {
                stats.entry(key.clone()).or_default();
            }
        }
    }

    for feature in features {
        let Some(feature_type) = feature.feature_type() else {
            continue;
        };
        let Some(schema_attrs) = schema_attributes(&feature_type, schema) else {
            continue;
        };
        for (path, value) in flatten_attributes(feature, options) {
            let Some(attr_def) = schema_attrs.get(path.as_str()) else {
                continue;
            };
            let numeric = numeric_value(&attr_def.type_ref, &value);
            let entry = stats.entry(path).or_default();
            if let Some(n) = numeric {
                entry.update(&n);
            }
        }
    }
    stats
}

fn numeric_value(type_ref: &TypeRef, value: &AttributeValue) -> Option<Number> {
    match type_ref {
        TypeRef::Integer => match value {
            AttributeValue::Number(n) => n.as_i64().map(Number::from),
            AttributeValue::String(s) => s.parse::<i64>().ok().map(Number::from),
            _ => None,
        },
        TypeRef::NonNegativeInteger => match value {
            AttributeValue::Number(n) => n.as_u64().map(Number::from),
            AttributeValue::String(s) => s.parse::<u64>().ok().map(Number::from),
            _ => None,
        },
        TypeRef::Double | TypeRef::Measure => match value {
            AttributeValue::Number(n) => Some(n.clone()),
            AttributeValue::String(s) => serde_json::from_str::<Number>(s).ok(),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use nusamai_citygml::schema::{FeatureTypeDef, TypeDef};

    use super::*;

    const FEATURE_TYPE: &str = "Feature";

    fn schema() -> Schema {
        let mut attributes = nusamai_citygml::schema::Map::default();
        attributes.insert(
            "height".to_string(),
            nusamai_citygml::schema::Attribute {
                type_ref: TypeRef::Double,
                ..Default::default()
            },
        );
        attributes.insert(
            "name".to_string(),
            nusamai_citygml::schema::Attribute {
                type_ref: TypeRef::String,
                ..Default::default()
            },
        );
        let mut schema = Schema::default();
        schema.types.insert(
            FEATURE_TYPE.to_string(),
            TypeDef::Feature(FeatureTypeDef {
                attributes,
                additional_attributes: true,
            }),
        );
        schema
    }

    fn feature(height: f64, name: &str) -> Feature {
        let mut attrs: indexmap::IndexMap<String, AttributeValue> = indexmap::IndexMap::new();
        attrs.insert(
            "height".to_string(),
            AttributeValue::Number(Number::from_f64(height).unwrap()),
        );
        attrs.insert("name".to_string(), AttributeValue::String(name.to_string()));
        let mut feature = Feature::from(attrs);
        feature.update_feature_type(FEATURE_TYPE.to_string());
        feature
    }

    #[test]
    fn numeric_attribute_gets_whole_dataset_min_max() {
        let features = vec![feature(3.4, "a"), feature(11.4, "b"), feature(5.0, "c")];
        let stats = collect(&features, &schema(), MetadataOptions::default());
        let height = &stats["height"];
        assert_eq!(height.minimum.as_ref().unwrap().as_f64(), Some(3.4));
        assert_eq!(height.maximum.as_ref().unwrap().as_f64(), Some(11.4));
    }

    #[test]
    fn non_numeric_attribute_has_no_min_max() {
        let features = vec![feature(1.0, "a")];
        let stats = collect(&features, &schema(), MetadataOptions::default());
        let name = &stats["name"];
        assert!(name.minimum.is_none());
        assert!(name.maximum.is_none());
    }

    #[test]
    fn feature_type_with_no_schema_entry_contributes_nothing() {
        let features = vec![feature(1.0, "a")];
        let stats = collect(&features, &Schema::default(), MetadataOptions::default());
        assert!(stats.is_empty());
    }

    #[test]
    fn schema_attributes_are_listed_even_without_features() {
        let features: Vec<Feature> = Vec::new();
        let stats = collect(&features, &schema(), MetadataOptions::default());
        assert_eq!(
            stats.keys().collect::<Vec<_>>(),
            vec!["height", "name"],
            "schema order is preserved"
        );
        assert!(stats["height"].minimum.is_none());
    }

    #[test]
    fn numeric_attribute_stored_as_string_is_parsed() {
        let mut attrs: indexmap::IndexMap<String, AttributeValue> = indexmap::IndexMap::new();
        attrs.insert(
            "height".to_string(),
            AttributeValue::String("7.5".to_string()),
        );
        let mut feature = Feature::from(attrs);
        feature.update_feature_type(FEATURE_TYPE.to_string());
        let stats = collect(&[feature], &schema(), MetadataOptions::default());
        assert_eq!(
            stats["height"].minimum.as_ref().unwrap().as_f64(),
            Some(7.5)
        );
    }
}
