use std::{
    collections::HashMap,
    fmt::Debug,
    sync::atomic::{AtomicUsize, Ordering},
};

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

#[derive(Debug)]
struct AtomicCounterMap {
    start: i64,
    inner: parking_lot::Mutex<HashMap<String, AtomicUsize>>,
}

impl Clone for AtomicCounterMap {
    fn clone(&self) -> Self {
        let inner = self.inner.lock();
        let new_inner = inner
            .iter()
            .map(|(k, v)| (k.clone(), AtomicUsize::new(v.load(Ordering::SeqCst))))
            .collect();
        AtomicCounterMap {
            start: self.start,
            inner: parking_lot::Mutex::new(new_inner),
        }
    }
}

impl AtomicCounterMap {
    fn new(start: i64) -> Self {
        AtomicCounterMap {
            start,
            inner: parking_lot::Mutex::new(HashMap::new()),
        }
    }

    fn increment(&self, key: &str) -> i64 {
        let mut map = self.inner.lock();
        let counter = map
            .entry(key.to_string())
            .or_insert_with(|| AtomicUsize::new(self.start as usize));
        let result = counter.fetch_add(1, Ordering::SeqCst);
        result as i64
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureCounterFactory;

impl ProcessorFactory for FeatureCounterFactory {
    fn name(&self) -> &str {
        "FeatureCounter"
    }

    fn description(&self) -> &str {
        "Count Features and Add Counter to Attribute"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureCounterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureCounterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::CounterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::CounterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::CounterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let process = FeatureCounter {
            counter: AtomicCounterMap::new(params.count_start),
            params,
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
struct FeatureCounter {
    counter: AtomicCounterMap,
    params: FeatureCounterParam,
}

/// # Feature Counter Parameters
/// Configure how features are counted and grouped, and where to store the count
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct FeatureCounterParam {
    /// # Start Count
    /// Starting value for the counter
    count_start: i64,
    /// # Group By Attributes
    /// List of attribute names to group features by before counting
    group_by: Option<Vec<Attribute>>,
    /// # Output Attribute
    /// Name of the attribute where the count will be stored
    output_attribute: String,
}

impl Processor for FeatureCounter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        if let Some(group_by) = &self.params.group_by {
            let key = group_by
                .iter()
                .map(|k| feature.get(&k).map(|v| v.to_string()).unwrap_or_default())
                .collect::<Vec<_>>()
                .join(",");
            let count = self.counter.increment(&key);
            let mut new_row = feature.clone();
            new_row.insert(
                self.params.output_attribute.clone(),
                AttributeValue::Number(serde_json::Number::from(count)),
            );
            fw.send(ctx.new_with_feature_and_port(new_row, DEFAULT_PORT.clone()));
        } else {
            let count = self.counter.increment("_all");
            let mut new_row = feature.clone();
            new_row.insert(
                self.params.output_attribute.clone(),
                AttributeValue::Number(serde_json::Number::from(count)),
            );
            fw.send(ctx.new_with_feature_and_port(new_row, DEFAULT_PORT.clone()));
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureCounter"
    }
}
