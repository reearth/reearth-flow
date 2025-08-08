use std::{collections::HashMap, fmt::Debug};

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct EchoProcessorFactory;

impl ProcessorFactory for EchoProcessorFactory {
    fn name(&self) -> &str {
        "EchoProcessor"
    }

    fn description(&self) -> &str {
        "Debug Echo Features to Logs"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Debug"]
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
        Ok(Box::new(EchoProcessor))
    }
}

#[derive(Debug, Clone)]
pub struct EchoProcessor;

impl Processor for EchoProcessor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let span = ctx.info_span();
        let feature: serde_json::Value = ctx.feature.clone().into();
        ctx.event_hub.info_log(
            Some(span.clone()),
            format!(
                "echo with feature = {:?}",
                serde_json::to_string(&feature).unwrap_or_default()
            ),
        );
        fw.send(ctx);
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "EchoProcessor"
    }
}
