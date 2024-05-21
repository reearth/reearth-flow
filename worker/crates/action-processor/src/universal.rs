use std::{collections::HashMap, fmt::Debug};

use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use serde_json::Value;

use crate::errors::ProcessorError;

#[typetag::serde(tag = "action", content = "with")]
pub trait UniversalProcessor: Send + Sync + Debug + UniversalProcessorClone {
    fn initialize(&mut self, ctx: NodeContext);
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError>;
    fn finish(
        &self,
        ctx: NodeContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError>;

    fn name(&self) -> &str;
}

pub trait UniversalProcessorClone {
    fn clone_box(&self) -> Box<dyn UniversalProcessor>;
}

impl<T> UniversalProcessorClone for T
where
    T: 'static + UniversalProcessor + Clone,
{
    fn clone_box(&self) -> Box<dyn UniversalProcessor> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn UniversalProcessor> {
    fn clone(&self) -> Box<dyn UniversalProcessor> {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct UniversalOperator(Box<dyn UniversalProcessor>);

impl Processor for UniversalOperator {
    fn initialize(&mut self, ctx: NodeContext) {
        self.0.initialize(ctx)
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        self.0.process(ctx, fw)
    }

    fn finish(
        &self,
        ctx: NodeContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        self.0.finish(ctx, fw)
    }

    fn name(&self) -> &str {
        self.0.name()
    }
}

#[derive(Debug, Clone, Default)]
pub struct UniversalProcessorFactory;

#[async_trait::async_trait]
impl ProcessorFactory for UniversalProcessorFactory {
    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }
    async fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let mut params = vec![(
            "action".to_owned(),
            serde_json::Value::String(action.to_string()),
        )];

        if let Some(with) = with {
            let value = serde_json::to_value(with).map_err(|e| {
                ProcessorError::UniversalProcessorFactory(format!(
                    "Failed to serialize with: {}",
                    e
                ))
            })?;
            params.push(("with".to_owned(), value));
        }

        let processor: serde_json::Result<Box<dyn UniversalProcessor>> =
            serde_json::from_value(serde_json::Value::Object(
                params
                    .clone()
                    .into_iter()
                    .collect::<serde_json::Map<_, _>>(),
            ));
        let processor = processor.map_err(|e| {
            ProcessorError::UniversalProcessorFactory(format!(
                "Failed to deserialize processor: {}",
                e
            ))
        })?;
        Ok(Box::new(UniversalOperator(processor.clone())))
    }
}
