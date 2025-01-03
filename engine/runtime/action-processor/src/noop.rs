use std::{collections::HashMap, fmt::Debug};

use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct NoopProcessorFactory;

impl ProcessorFactory for NoopProcessorFactory {
    fn name(&self) -> &str {
        "NoopProcessor"
    }

    fn description(&self) -> &str {
        "Noop features"
    }

    fn parameter_schema(&self) -> Option<schemars::Schema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Noop"]
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
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(NoopProcessor))
    }
}

#[derive(Debug, Clone)]
pub struct NoopProcessor;

impl Processor for NoopProcessor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        fw.send(ctx);
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
        "NoopProcessor"
    }
}
