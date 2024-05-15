use std::{collections::HashMap, fmt::Debug};

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Sink, SinkFactory, DEFAULT_PORT},
};
use serde_json::Value;

use crate::errors::SinkError;

#[typetag::serde(tag = "action", content = "with")]
pub trait UniversalSink: Send + Debug + UniversalSinkClone {
    fn initialize(&self, ctx: NodeContext);
    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError>;

    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError>;
    fn set_source_state(&mut self, _source_state: &[u8]) -> Result<(), BoxedError> {
        Ok(())
    }
    fn get_source_state(&mut self) -> Result<Option<Vec<u8>>, BoxedError> {
        Ok(None)
    }

    fn preferred_batch_size(&self) -> Option<u64> {
        None
    }

    fn max_batch_duration_ms(&self) -> Option<u64> {
        None
    }

    fn flush_batch(&mut self) -> Result<(), BoxedError> {
        Ok(())
    }

    fn supports_batching(&self) -> bool {
        false
    }
}

pub trait UniversalSinkClone {
    fn clone_box(&self) -> Box<dyn UniversalSink>;
}

impl<T> UniversalSinkClone for T
where
    T: 'static + UniversalSink + Clone,
{
    fn clone_box(&self) -> Box<dyn UniversalSink> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn UniversalSink> {
    fn clone(&self) -> Box<dyn UniversalSink> {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct UniversalOperator(Box<dyn UniversalSink>);

impl Sink for UniversalOperator {
    fn initialize(&self, ctx: NodeContext) {
        self.0.initialize(ctx)
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        self.0.process(ctx)
    }

    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        self.0.finish(ctx)
    }
    fn set_source_state(&mut self, source_state: &[u8]) -> Result<(), BoxedError> {
        self.0.set_source_state(source_state)
    }
    fn get_source_state(&mut self) -> Result<Option<Vec<u8>>, BoxedError> {
        self.0.get_source_state()
    }

    fn preferred_batch_size(&self) -> Option<u64> {
        self.0.preferred_batch_size()
    }

    fn max_batch_duration_ms(&self) -> Option<u64> {
        self.0.max_batch_duration_ms()
    }

    fn flush_batch(&mut self) -> Result<(), BoxedError> {
        self.0.flush_batch()
    }

    fn supports_batching(&self) -> bool {
        self.0.supports_batching()
    }
}

#[derive(Debug, Clone, Default)]
pub struct UniversalSinkFactory {}

#[async_trait::async_trait]
impl SinkFactory for UniversalSinkFactory {
    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn prepare(&self) -> Result<(), BoxedError> {
        Ok(())
    }

    async fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let mut params = vec![(
            "action".to_owned(),
            serde_json::Value::String(action.to_string()),
        )];

        if let Some(with) = with {
            let value = serde_json::to_value(with)
                .map_err(|e| SinkError::BuildFactory(format!("Failed to serialize with: {}", e)))?;
            params.push(("with".to_owned(), value));
        }

        let sink: serde_json::Result<Box<dyn UniversalSink>> =
            serde_json::from_value(serde_json::Value::Object(
                params
                    .clone()
                    .into_iter()
                    .collect::<serde_json::Map<_, _>>(),
            ));
        let sink = sink
            .map_err(|e| SinkError::BuildFactory(format!("Failed to deserialize sink: {}", e)))?;
        Ok(Box::new(UniversalOperator(sink.clone())))
    }
}
