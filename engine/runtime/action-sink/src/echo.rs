use serde_json::Value;
use std::collections::HashMap;

use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, FEATURES_PORT};

#[derive(Debug, Clone, Default)]
pub struct EchoSinkFactory;

impl SinkFactory for EchoSinkFactory {
    fn name(&self) -> &str {
        "Echo Sink"
    }

    fn description(&self) -> &str {
        "Echoes features to logs and discards them."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Debug"]
    }

    fn tags(&self) -> &[&'static str] {
        &["logging"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
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
        "Echo Sink"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let span = ctx.info_span();
        let feature =
            serde_json::to_value(&ctx.feature).unwrap_or_else(|_| serde_json::Value::Null);
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
