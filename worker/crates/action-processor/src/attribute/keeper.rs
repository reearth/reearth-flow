use std::collections::HashMap;

use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    executor_operation::{ExecutorContext, NodeContext},
    node::DEFAULT_PORT,
};
use serde::{Deserialize, Serialize};

use crate::universal::UniversalProcessor;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttributeKeeper {
    keep_attributes: Vec<String>,
}

#[typetag::serde(name = "AttributeKeeper")]
impl UniversalProcessor for AttributeKeeper {
    fn initialize(&mut self, _ctx: NodeContext) {}
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let processed_data = feature
            .iter()
            .filter(|(key, _)| self.keep_attributes.contains(&key.inner()))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<HashMap<_, _>>();
        fw.send(ctx.new_with_feature_and_port(
            feature.with_attributes(processed_data),
            DEFAULT_PORT.clone(),
        ));
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
        "AttributeKeeper"
    }
}
