use serde_json::Value;
use std::collections::HashMap;

use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};

#[derive(Debug, Clone, Default)]
pub struct NoopSinkFactory;

impl SinkFactory for NoopSinkFactory {
    fn name(&self) -> &str {
        "NoopSink"
    }

    fn description(&self) -> &str {
        "No-Operation Sink (Discard Features)"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Noop"]
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
        Ok(Box::new(NoopSink))
    }
}

#[derive(Debug, Clone)]
pub struct NoopSink;

impl Sink for NoopSink {
    fn name(&self) -> &str {
        "NoopSink"
    }

    fn process(&mut self, _ctx: ExecutorContext) -> Result<(), BoxedError> {
        Ok(())
    }
    fn finish(&self, _ctx: NodeContext) -> Result<(), BoxedError> {
        Ok(())
    }
}
