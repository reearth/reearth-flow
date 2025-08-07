use serde_json::Value;
use std::collections::HashMap;

use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};

#[derive(Debug, Clone, Default)]
pub struct EchoSinkFactory;

impl SinkFactory for EchoSinkFactory {
    fn name(&self) -> &str {
        "EchoSink"
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

    fn prepare(&self) -> Result<(), BoxedError> {
        Ok(())
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        Ok(Box::new(EchoSink))
    }
}

#[derive(Debug, Clone)]
pub struct EchoSink;

impl Sink for EchoSink {
    fn name(&self) -> &str {
        "EchoSink"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let span = ctx.info_span();
        let feature: serde_json::Value = ctx.feature.clone().into();
        ctx.event_hub.info_log(
            Some(span.clone()),
            format!(
                "echo with feature = {:?}",
                serde_json::to_string(&feature).unwrap_or_default()
            ),
        );
        Ok(())
    }
    fn finish(&self, _ctx: NodeContext) -> Result<(), BoxedError> {
        Ok(())
    }
}
