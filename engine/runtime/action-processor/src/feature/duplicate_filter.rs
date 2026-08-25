use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Attribute, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

/// Features that repeat one already forwarded.
pub static DUPLICATE_PORT: Lazy<Port> = Lazy::new(|| Port::new("duplicate"));

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureDuplicateFilterFactory;

impl ProcessorFactory for FeatureDuplicateFilterFactory {
    fn name(&self) -> &str {
        "Feature Duplicate Filter"
    }

    fn description(&self) -> &str {
        "Forwards the first feature carrying each distinct value and separates out the ones \
         that repeat it. Features are compared on their whole content unless the attributes \
         to compare are named."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureDuplicateFilterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), DUPLICATE_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        // Every parameter is optional, so a node with no `with` block is valid and compares
        // features on their whole content.
        let params: FeatureDuplicateFilterParam = match with {
            Some(with) => {
                let value: Value = serde_json::to_value(with).map_err(|e| {
                    FeatureProcessorError::DuplicateFilterFactory(format!(
                        "Failed to serialize `with` parameter: {e}"
                    ))
                })?;
                serde_json::from_value(value).map_err(|e| {
                    FeatureProcessorError::DuplicateFilterFactory(format!(
                        "Failed to deserialize `with` parameter: {e}"
                    ))
                })?
            }
            None => FeatureDuplicateFilterParam::default(),
        };

        Ok(Box::new(FeatureDuplicateFilter {
            filter_by: params.filter_by,
            seen: HashSet::new(),
        }))
    }
}

/// # Feature Duplicate Filter Parameters
///
/// Which features count as repeats of one another.
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct FeatureDuplicateFilterParam {
    /// # Filter Attributes
    /// Attributes whose combined values identify a repeat: the first feature carrying a given
    /// combination is forwarded and later ones are separated out. An attribute that is absent
    /// counts as part of the combination, so it is not the same as one holding an empty value.
    /// When omitted, features are compared on their whole content instead — every attribute
    /// and their geometry.
    #[serde(default)]
    filter_by: Option<Vec<Attribute>>,
}

#[derive(Debug, Clone)]
struct FeatureDuplicateFilter {
    filter_by: Option<Vec<Attribute>>,
    /// Comparison keys already forwarded. Keys are held rather than whole features, so the
    /// memory cost is one key per distinct value rather than one entry per feature.
    seen: HashSet<String>,
}

impl FeatureDuplicateFilter {
    /// The key a feature is compared on.
    ///
    /// Serializing rather than joining the values keeps the key unambiguous: a feature
    /// carrying `{a: "1", b: "2"}` cannot collide with one carrying `{a: "1,2"}`, which
    /// joining on a separator would allow.
    fn key(&self, feature: &Feature) -> Result<String, serde_json::Error> {
        let value = match &self.filter_by {
            Some(attributes) => serde_json::to_value(
                attributes
                    .iter()
                    .map(|attribute| feature.get(attribute))
                    .collect::<Vec<_>>(),
            )?,
            // `Feature`'s own equality is its id, which is unique per feature, so the content
            // has to be compared explicitly.
            None => serde_json::to_value((&*feature.attributes, &*feature.geometry))?,
        };
        serde_json::to_string(&canonical(value))
    }
}

/// The same value with every object's keys in a fixed order, so that equal content always
/// serializes to an equal string.
///
/// Two containers make this necessary. `Attributes` is an `IndexMap`, so its order is the order
/// the attributes were added, and `AttributeValue::Map` is a `std::collections::HashMap`, whose
/// iteration order differs between instances holding identical entries. Without this, two
/// features with the same content can produce different keys and escape comparison. Sorting is
/// applied to objects only — array order is part of the value, not an artifact of the container.
fn canonical(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries: Vec<_> = object.into_iter().collect();
            entries.sort_by(|(a, _), (b, _)| a.cmp(b));
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, canonical(value)))
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.into_iter().map(canonical).collect()),
        other => other,
    }
}

impl Processor for FeatureDuplicateFilter {
    /// Forward a feature the first time its value is seen, and route later features carrying
    /// that same value to `duplicate`.
    ///
    /// Features leave as they arrive rather than being held until the end of the stream, so
    /// input order is preserved and only the comparison keys are kept in memory.
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let key = self.key(&ctx.feature).map_err(|e| {
            FeatureProcessorError::DuplicateFilter(format!("Failed to build a comparison key: {e}"))
        })?;
        let port = if self.seen.insert(key) {
            FEATURES_PORT.clone()
        } else {
            DUPLICATE_PORT.clone()
        };
        fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), port));
        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Feature Duplicate Filter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::create_default_execute_context;
    use pretty_assertions::assert_eq;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::AttributeValue;

    /// A feature carrying `attributes`, with an id of its own.
    fn feature(attributes: &[(&str, &str)]) -> Feature {
        let mut feature = Feature::new_with_attributes(Default::default());
        for (key, value) in attributes {
            feature.insert(*key, AttributeValue::String((*value).to_string()));
        }
        feature
    }

    /// Run `features` through the filter, returning the port each one left by.
    fn ports(filter_by: Option<Vec<&str>>, features: &[Feature]) -> Vec<String> {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let mut filter = FeatureDuplicateFilter {
            filter_by: filter_by
                .map(|attrs| attrs.into_iter().map(Attribute::new).collect::<Vec<_>>()),
            seen: HashSet::new(),
        };
        for feature in features {
            filter
                .process(create_default_execute_context(feature), &fw)
                .unwrap();
        }
        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("built as a noop forwarder");
        };
        let sent = noop.send_ports.lock().unwrap().clone();
        sent.into_iter().map(|port| port.to_string()).collect()
    }

    #[test]
    fn features_with_the_same_content_but_different_ids_are_duplicates() {
        let sent = ports(
            None,
            &[feature(&[("gmlId", "a")]), feature(&[("gmlId", "a")])],
        );
        assert_eq!(sent, ["features", "duplicate"]);
    }

    #[test]
    fn features_with_different_content_all_pass() {
        let sent = ports(
            None,
            &[feature(&[("gmlId", "a")]), feature(&[("gmlId", "b")])],
        );
        assert_eq!(sent, ["features", "features"]);
    }

    #[test]
    fn only_the_named_attributes_are_compared() {
        let sent = ports(
            Some(vec!["gmlId"]),
            &[
                feature(&[("gmlId", "a"), ("lod", "1")]),
                feature(&[("gmlId", "a"), ("lod", "2")]),
            ],
        );
        assert_eq!(
            sent,
            ["features", "duplicate"],
            "lod differs, but only gmlId is compared"
        );
    }

    #[test]
    fn an_attribute_outside_the_comparison_does_not_split_a_duplicate() {
        let sent = ports(
            Some(vec!["gmlId"]),
            &[
                feature(&[("gmlId", "a")]),
                feature(&[("gmlId", "a"), ("extra", "x")]),
            ],
        );
        assert_eq!(sent, ["features", "duplicate"]);
    }

    /// The sibling Attribute Duplicate Filter joins its key values on a comma, which lets
    /// `{a: "1", b: "2"}` collide with `{a: "1,2"}`. Serializing the values must not.
    #[test]
    fn values_containing_the_separator_do_not_collide() {
        let sent = ports(
            Some(vec!["a", "b"]),
            &[feature(&[("a", "1"), ("b", "2")]), feature(&[("a", "1,2")])],
        );
        assert_eq!(sent, ["features", "features"]);
    }

    /// An absent attribute is part of the combination, so it must not be silently dropped the
    /// way a `flat_map` over lookups would drop it.
    #[test]
    fn an_absent_attribute_is_distinct_from_a_present_one() {
        let sent = ports(
            Some(vec!["a", "b"]),
            &[feature(&[("a", "1")]), feature(&[("b", "1")])],
        );
        assert_eq!(sent, ["features", "features"]);
    }

    /// `Attributes` is an `IndexMap`, so it preserves the order attributes were added. Two
    /// features holding the same pairs added in different orders are still the same feature.
    #[test]
    fn attribute_insertion_order_does_not_change_the_key() {
        let sent = ports(
            None,
            &[
                feature(&[("a", "1"), ("b", "2")]),
                feature(&[("b", "2"), ("a", "1")]),
            ],
        );
        assert_eq!(sent, ["features", "duplicate"]);
    }

    /// `AttributeValue::Map` is a `std::collections::HashMap`, whose iteration order differs
    /// between instances holding identical entries. Enough keys are used here that an
    /// uncanonicalized key would be very unlikely to match.
    #[test]
    fn a_nested_map_compares_by_content_not_by_iteration_order() {
        let nested = |pairs: &[(&str, &str)]| {
            let mut feature = Feature::new_with_attributes(Default::default());
            let map = pairs
                .iter()
                .map(|(k, v)| ((*k).to_string(), AttributeValue::String((*v).to_string())))
                .collect::<std::collections::HashMap<_, _>>();
            feature.insert("nested", AttributeValue::Map(map));
            feature
        };
        let pairs: Vec<(&str, &str)> = vec![
            ("a", "1"),
            ("b", "2"),
            ("c", "3"),
            ("d", "4"),
            ("e", "5"),
            ("f", "6"),
        ];
        let reversed: Vec<(&str, &str)> = pairs.iter().rev().copied().collect();

        let sent = ports(None, &[nested(&pairs), nested(&reversed)]);
        assert_eq!(sent, ["features", "duplicate"]);
    }

    /// Array order IS part of the value, so it must not be sorted away.
    #[test]
    fn array_order_still_distinguishes_features() {
        let listed = |values: &[&str]| {
            let mut feature = Feature::new_with_attributes(Default::default());
            feature.insert(
                "list",
                AttributeValue::Array(
                    values
                        .iter()
                        .map(|v| AttributeValue::String((*v).to_string()))
                        .collect(),
                ),
            );
            feature
        };
        let sent = ports(None, &[listed(&["a", "b"]), listed(&["b", "a"])]);
        assert_eq!(sent, ["features", "features"]);
    }

    #[test]
    fn the_first_of_a_run_is_the_one_forwarded_and_order_is_preserved() {
        let sent = ports(
            Some(vec!["gmlId"]),
            &[
                feature(&[("gmlId", "a")]),
                feature(&[("gmlId", "b")]),
                feature(&[("gmlId", "a")]),
                feature(&[("gmlId", "b")]),
            ],
        );
        assert_eq!(sent, ["features", "features", "duplicate", "duplicate"]);
    }
}
