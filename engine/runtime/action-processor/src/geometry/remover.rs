use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::Geometry;
use serde_json::Value;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "analyzer", derive(reearth_flow_analyzer_core::DataSize))]
pub struct GeometryRemoverFactory;

impl ProcessorFactory for GeometryRemoverFactory {
    fn name(&self) -> &str {
        "GeometryRemover"
    }

    fn description(&self) -> &str {
        "Removes geometry from a feature"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
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
        Ok(Box::new(GeometryRemover))
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "analyzer", derive(reearth_flow_analyzer_core::DataSize))]
pub struct GeometryRemover;

impl Processor for GeometryRemover {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        feature.geometry = Geometry::default();
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "GeometryRemover"
    }
}
