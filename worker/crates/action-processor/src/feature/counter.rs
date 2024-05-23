use std::{
    collections::HashMap,
    fmt::Debug,
    sync::atomic::{AtomicUsize, Ordering},
};

use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::AttributeValue;
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
pub struct FeatureCounterFactory;

#[async_trait::async_trait]
impl ProcessorFactory for FeatureCounterFactory {
    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![REJECTED_PORT.clone()]
    }

    async fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureCounterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::CounterFactory(format!("Failed to serialize with: {}", e))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::CounterFactory(format!("Failed to deserialize with: {}", e))
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
pub struct FeatureCounter {
    counter: AtomicCounterMap,
    params: FeatureCounterParam,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCounterParam {
    count_start: i64,
    group_by: Option<Vec<String>>,
    output_attribute: String,
}

impl Processor for FeatureCounter {
    fn initialize(&mut self, _ctx: NodeContext) {}

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        if self.params.group_by.is_none() {
            let count = self.counter.increment("_all");
            let mut new_row = feature.clone();
            new_row.insert(
                self.params.output_attribute.clone(),
                AttributeValue::Number(serde_json::Number::from(count)),
            );
            fw.send(ctx.new_with_feature_and_port(new_row, DEFAULT_PORT.clone()));
        } else {
            let group_by = self.params.group_by.as_ref().unwrap();
            let key = group_by
                .iter()
                .map(|k| feature.get(k).unwrap().to_string())
                .collect::<Vec<_>>()
                .join(",");
            let count = self.counter.increment(&key);
            let mut new_row = feature.clone();
            new_row.insert(
                self.params.output_attribute.clone(),
                AttributeValue::Number(serde_json::Number::from(count)),
            );
            fw.send(ctx.new_with_feature_and_port(new_row, DEFAULT_PORT.clone()));
        }
        Ok(())
    }

    fn finish(
        &self,
        _ctx: NodeContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureCounter"
    }
}
