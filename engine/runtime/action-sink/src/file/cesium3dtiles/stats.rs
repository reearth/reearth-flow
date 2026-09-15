use indexmap::IndexMap;
use reearth_flow_gltf::tiles::metadata::{flatten_attributes, MetadataOptions};
use reearth_flow_types::{AttributeValue, Feature};
use serde_json::Number;

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
    options: MetadataOptions,
) -> IndexMap<String, PropertyStats> {
    let mut stats: IndexMap<String, PropertyStats> = IndexMap::new();
    for feature in features {
        for (path, value) in flatten_attributes(feature, options) {
            let entry = stats.entry(path).or_default();
            if let AttributeValue::Number(n) = value {
                entry.update(&n);
            }
        }
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feature(height: f64, name: &str) -> Feature {
        let mut attrs: indexmap::IndexMap<String, AttributeValue> = indexmap::IndexMap::new();
        attrs.insert(
            "height".to_string(),
            AttributeValue::Number(Number::from_f64(height).unwrap()),
        );
        attrs.insert("name".to_string(), AttributeValue::String(name.to_string()));
        Feature::from(attrs)
    }

    #[test]
    fn numeric_attribute_gets_whole_dataset_min_max() {
        let features = vec![feature(3.4, "a"), feature(11.4, "b"), feature(5.0, "c")];
        let stats = collect(&features, MetadataOptions::default());
        let height = &stats["height"];
        assert_eq!(height.minimum.as_ref().unwrap().as_f64(), Some(3.4));
        assert_eq!(height.maximum.as_ref().unwrap().as_f64(), Some(11.4));
    }

    #[test]
    fn non_numeric_attribute_has_no_min_max() {
        let features = vec![feature(1.0, "a")];
        let stats = collect(&features, MetadataOptions::default());
        let name = &stats["name"];
        assert!(name.minimum.is_none());
        assert!(name.maximum.is_none());
    }
}
