use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::Feature;

static PROCESS_COUNT: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Default)]
pub struct MemAccumulatorDebugSinkFactory;

impl SinkFactory for MemAccumulatorDebugSinkFactory {
    fn name(&self) -> &str {
        "MemAccumulatorDebugSink"
    }

    fn description(&self) -> &str {
        "DEBUG ONLY: retains every incoming feature in memory (no disk output) to measure raw retention footprint"
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
        Ok(Box::new(MemAccumulatorDebugSink { buffer: Vec::new() }))
    }
}

#[derive(Debug, Clone, Default)]
pub struct MemAccumulatorDebugSink {
    buffer: Vec<Feature>,
}

impl Sink for MemAccumulatorDebugSink {
    fn name(&self) -> &str {
        "MemAccumulatorDebugSink"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        self.buffer.push(ctx.feature);
        let n = PROCESS_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        if n % 10_000 == 0 {
            eprintln!(
                "[MEASURE mem_accumulator] features_retained={n} buffer_len={}",
                self.buffer.len()
            );
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext) -> Result<(), BoxedError> {
        eprintln!(
            "[MEASURE mem_accumulator] FINISH features_retained={}",
            self.buffer.len()
        );
        Ok(())
    }
}
