use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct ListExploderFactory;

impl ProcessorFactory for ListExploderFactory {
    fn name(&self) -> &str {
        "List Exploder"
    }

    fn description(&self) -> &str {
        "Creates one feature per element of a list attribute, merging the element's key-value pairs into the feature's attributes and removing the source attribute. A feature whose attribute is missing, empty, or not a list of key-value pairs produces nothing and is rejected instead."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ListExploder))
    }

    fn categories(&self) -> &[&'static str] {
        &["Transform"]
    }

    fn tags(&self) -> &[&'static str] {
        &["list"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let process: ListExploder = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::TransformerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::TransformerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::TransformerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(process))
    }
}

/// # List Exploder Parameters
///
/// Configures which list attribute is expanded into individual features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ListExploder {
    /// # Source Attribute
    /// Attribute holding a list of key-value pairs, one entry per feature to create.
    source_attribute: Attribute,
}

impl Processor for ListExploder {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let Some(AttributeValue::Array(elements)) = feature.attributes.get(&self.source_attribute)
        else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        // All or nothing: the whole list is checked before anything is emitted. Deciding per
        // element as they were emitted meant a list mixing key-value maps with other values
        // emitted the maps it had reached AND the original feature, duplicating data.
        let elements: Option<Vec<_>> = elements
            .iter()
            .map(|element| match element {
                AttributeValue::Map(attributes) => Some(attributes),
                _ => None,
            })
            .collect();
        // An empty list is rejected along with the unusable ones: it produces no features, and
        // keeping it on `features` would mean that port carrying two shapes of feature — exploded
        // ones, and untouched ones still holding the list.
        let Some(elements) = elements.filter(|elements| !elements.is_empty()) else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        for attributes in elements {
            let mut exploded = feature.clone();
            exploded.refresh_id();
            exploded.remove(&self.source_attribute);
            exploded.extend_attributes(attributes.clone());
            fw.send(ctx.new_with_feature_and_port(exploded, FEATURES_PORT.clone()));
        }
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
        "List Exploder"
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use reearth_flow_runtime::forwarder::{NoopChannelForwarder, ProcessorChannelForwarder};
    use reearth_flow_types::Feature;

    use super::*;
    use crate::tests::utils;

    /// Runs the processor over one feature and returns the ports and features it emitted.
    fn run(source_attribute: &str, list: AttributeValue) -> (Vec<Port>, Vec<Feature>) {
        let mut attributes = IndexMap::new();
        attributes.insert(Attribute::new("gmlId"), AttributeValue::String("b1".into()));
        attributes.insert(Attribute::new(source_attribute), list);
        let feature = Feature::new_with_attributes(attributes);

        let mut exploder = ListExploder {
            source_attribute: Attribute::new(source_attribute),
        };
        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        exploder.process(ctx, &fw).expect("process should succeed");

        match fw {
            ProcessorChannelForwarder::Noop(noop) => (
                noop.send_ports.lock().unwrap().clone(),
                noop.send_features.lock().unwrap().clone(),
            ),
            _ => unreachable!(),
        }
    }

    fn element(key: &str, value: &str) -> AttributeValue {
        let mut map = HashMap::new();
        map.insert(key.to_string(), AttributeValue::String(value.to_string()));
        AttributeValue::Map(map)
    }

    #[test]
    fn explodes_one_feature_per_element() {
        let (ports, features) = run(
            "overlaps",
            AttributeValue::Array(vec![element("with", "b2"), element("with", "b3")]),
        );

        assert_eq!(ports, vec![FEATURES_PORT.clone(), FEATURES_PORT.clone()]);
        assert_eq!(features.len(), 2);
        for (feature, expected) in features.iter().zip(["b2", "b3"]) {
            assert_eq!(
                feature.get(Attribute::new("with")),
                Some(&AttributeValue::String(expected.to_string())),
                "the element's attributes should be merged in"
            );
            assert_eq!(
                feature.get(Attribute::new("gmlId")),
                Some(&AttributeValue::String("b1".to_string())),
                "attributes of the incoming feature should be kept"
            );
            assert!(
                feature.get(Attribute::new("overlaps")).is_none(),
                "the source attribute should be removed"
            );
        }
        assert_ne!(
            features[0].id, features[1].id,
            "exploded features should have distinct ids"
        );
    }

    /// Regression: a list mixing maps with other values used to emit the maps reached so far
    /// *and* the original feature on `features`, duplicating data (and dropping the rest of the
    /// list). Nothing is explodable here, so the whole feature is rejected.
    #[test]
    fn rejects_a_list_that_is_not_all_key_value_pairs() {
        let (ports, features) = run(
            "overlaps",
            AttributeValue::Array(vec![
                element("with", "b2"),
                AttributeValue::String("not a map".into()),
            ]),
        );

        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
        assert_eq!(features.len(), 1, "the feature should be emitted once");
        assert!(
            features[0].get(Attribute::new("overlaps")).is_some(),
            "the rejected feature should keep its list"
        );
        assert!(
            features[0].get(Attribute::new("with")).is_none(),
            "no element should have been exploded"
        );
    }

    #[test]
    fn rejects_an_empty_list() {
        let (ports, features) = run("overlaps", AttributeValue::Array(vec![]));

        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
        assert_eq!(
            features[0].get(Attribute::new("overlaps")),
            Some(&AttributeValue::Array(vec![]))
        );
    }

    #[test]
    fn rejects_an_attribute_that_is_not_a_list() {
        let (ports, features) = run("overlaps", AttributeValue::String("b2".into()));

        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
        assert_eq!(
            features[0].get(Attribute::new("overlaps")),
            Some(&AttributeValue::String("b2".to_string()))
        );
    }

    #[test]
    fn rejects_a_feature_without_the_attribute() {
        let feature = Feature::new_with_attributes(IndexMap::new());
        let mut exploder = ListExploder {
            source_attribute: Attribute::new("overlaps"),
        };
        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        exploder.process(ctx, &fw).expect("process should succeed");

        match fw {
            ProcessorChannelForwarder::Noop(noop) => {
                assert_eq!(
                    *noop.send_ports.lock().unwrap(),
                    vec![REJECTED_PORT.clone()]
                );
            }
            _ => unreachable!(),
        }
    }
}
