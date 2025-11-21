use std::collections::HashMap;

use itertools::Itertools;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureSorterFactory;

impl ProcessorFactory for FeatureSorterFactory {
    fn name(&self) -> &str {
        "FeatureSorter"
    }

    fn description(&self) -> &str {
        "Sorts features based on specified attributes in ascending or descending order"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureSorterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureSorterParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::SorterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::SorterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::SorterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let process = FeatureSorter {
            params,
            buffer: HashMap::new(),
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
struct FeatureSorter {
    params: FeatureSorterParam,
    buffer: HashMap<AttributeValue, Vec<Feature>>,
}

/// # FeatureSorter Parameters
///
/// Configuration for sorting features based on attribute values.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct FeatureSorterParam {
    /// Attributes to use for sorting features (sort order based on attribute order)
    attributes: Vec<Attribute>,
    /// Sorting order (ascending or descending)
    order: Order,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
enum Order {
    #[serde(rename = "ascending")]
    Asc,
    #[serde(rename = "descending")]
    Desc,
}

impl Processor for FeatureSorter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = ctx.feature;
        let key = self
            .params
            .attributes
            .iter()
            .flat_map(|attribute| feature.get(attribute))
            .cloned()
            .collect_vec();

        self.buffer
            .entry(AttributeValue::Array(key))
            .or_default()
            .push(feature.clone());
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        let mut sorted = self.buffer.keys().collect_vec();
        if self.params.order == Order::Desc {
            sorted.sort_by(|a, b| b.cmp(a));
        } else {
            sorted.sort();
        }
        for key in sorted {
            for feature in self.buffer.get(key).unwrap() {
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    feature.clone(),
                    DEFAULT_PORT.clone(),
                ));
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureSorter"
    }
}
