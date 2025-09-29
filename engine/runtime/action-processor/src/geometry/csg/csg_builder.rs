use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, REJECTED_PORT},
};
use serde_json::Value;

use super::super::errors::GeometryProcessorError;

static LEFT_PORT: Lazy<Port> = Lazy::new(|| Port::new("left"));
static RIGHT_PORT: Lazy<Port> = Lazy::new(|| Port::new("right"));
static INTERSECTION_PORT: Lazy<Port> = Lazy::new(|| Port::new("intersection"));
static UNION_PORT: Lazy<Port> = Lazy::new(|| Port::new("union"));
static DIFFERENCE_PORT: Lazy<Port> = Lazy::new(|| Port::new("difference"));

#[derive(Debug, Clone, Default)]
pub struct CSGBuilderFactory;

impl ProcessorFactory for CSGBuilderFactory {
    fn name(&self) -> &str {
        "CSGBuilder"
    }

    fn description(&self) -> &str {
        "Constructs a Constructive Solid Geometry (CSG) representation from a pair (Left, Right) of solid geometries. It detects union, intersection, difference (Left - Right). \
        It however does not compute the resulting geometry, but outputs the CSG tree structure. To evaluate the CSG tree into a solid geometry, use CSGEvaluator."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![LEFT_PORT.clone(), RIGHT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            INTERSECTION_PORT.clone(),
            UNION_PORT.clone(),
            DIFFERENCE_PORT.clone(),
            REJECTED_PORT.clone(),
        ]
    }
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let processor = CSGBuilder {};
        Ok(Box::new(processor))
    }
}

/// # CSG Builder
/// Builds a CSG tree from two solid geometries. To create a mesh from the CSG tree, use CSGEvaluator.
#[derive(Debug, Clone)]
pub struct CSGBuilder {}

impl Processor for CSGBuilder {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "CSGBuilder"
    }
}