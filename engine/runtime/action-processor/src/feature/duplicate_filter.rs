use std::collections::{HashMap, HashSet};

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::Feature;
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureDuplicateFilterFactory;

impl ProcessorFactory for FeatureDuplicateFilterFactory {
    fn name(&self) -> &str {
        "FeatureDuplicateFilter"
    }

    fn description(&self) -> &str {
        "Filter Out Duplicate Features"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
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
        let process = FeatureDuplicateFilter {
            buffer: HashSet::new(),
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
struct FeatureDuplicateFilter {
    buffer: HashSet<Feature>,
}

impl Processor for FeatureDuplicateFilter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        self.buffer.insert(ctx.feature);
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        for feature in self.buffer.iter() {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature.clone(),
                DEFAULT_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureDuplicateFilter"
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;

    use crate::tests::utils::create_default_execute_context;

    use super::*;

    #[test]
    fn test_filter() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let feature = Feature::default();
        let ctx = create_default_execute_context(&feature);
        let mut filter = FeatureDuplicateFilter {
            buffer: HashSet::new(),
        };
        filter.process(ctx, &fw).unwrap();
        let ctx = create_default_execute_context(&feature);
        filter.process(ctx, &fw).unwrap();
        let feature = Feature::default();
        let ctx = create_default_execute_context(&feature);
        filter.process(ctx, &fw).unwrap();
        filter.finish(NodeContext::default(), &fw).unwrap();
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(noop.send_ports.lock().unwrap().len(), 2);
            assert_eq!(
                noop.send_ports.lock().unwrap().first().cloned(),
                Some(DEFAULT_PORT.clone())
            );
        }
    }
}
