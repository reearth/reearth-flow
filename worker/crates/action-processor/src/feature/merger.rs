use std::collections::{hash_map::Entry, HashMap};

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::{BoxedError, ExecutionError},
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

static REQUESTOR_PORT: Lazy<Port> = Lazy::new(|| Port::new("requestor"));
static SUPPLIER_PORT: Lazy<Port> = Lazy::new(|| Port::new("supplier"));
static MERGED_PORT: Lazy<Port> = Lazy::new(|| Port::new("merged"));
static UNMERGED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unmerged"));

#[derive(Debug, Clone, Default)]
pub struct FeatureMergerFactory;

#[async_trait::async_trait]
impl ProcessorFactory for FeatureMergerFactory {
    fn get_input_ports(&self) -> Vec<Port> {
        vec![REQUESTOR_PORT.clone(), SUPPLIER_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![MERGED_PORT.clone(), UNMERGED_PORT.clone()]
    }

    async fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureMergerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::MergerFactory(format!("Failed to serialize with: {}", e))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::MergerFactory(format!("Failed to deserialize with: {}", e))
            })?
        } else {
            return Err(FeatureProcessorError::MergerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = FeatureMerger {
            params,
            request_features: vec![],
            supplier_buffer: HashMap::new(),
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct FeatureMerger {
    params: FeatureMergerParam,
    request_features: Vec<Feature>,
    supplier_buffer: HashMap<AttributeValue, Vec<Feature>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FeatureMergerParam {
    join: Join,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Join {
    requestor: String,
    supplier: String,
}

impl Processor for FeatureMerger {
    fn initialize(&mut self, _ctx: NodeContext) {}

    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        match ctx.port {
            port if port == REQUESTOR_PORT.clone() => {
                let feature = ctx.feature;
                self.request_features.push(feature);
            }
            port if port == SUPPLIER_PORT.clone() => {
                let feature = ctx.feature;
                if let Some(value) = feature
                    .attributes
                    .get(&Attribute::new(&self.params.join.supplier))
                {
                    match self.supplier_buffer.entry(value.clone()) {
                        Entry::Occupied(entry) => {
                            entry.into_mut().push(feature);
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(vec![feature]);
                        }
                    }
                }
            }
            port => return Err(ExecutionError::InvalidPortHandle(port).into()),
        }
        Ok(())
    }

    fn finish(
        &self,
        ctx: NodeContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        for request_feature in self.request_features.iter() {
            let request_value = request_feature
                .attributes
                .get(&Attribute::new(&self.params.join.requestor))
                .ok_or(FeatureProcessorError::Merger(
                    "No Requestor Value".to_string(),
                ))?;
            let Some(supplier_features) = self.supplier_buffer.get(request_value) else {
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    request_feature.clone(),
                    UNMERGED_PORT.clone(),
                ));
                continue;
            };

            for supplier_feature in supplier_features {
                let mut merged_feature = request_feature.clone();
                merged_feature
                    .attributes
                    .extend(supplier_feature.attributes.clone());
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    merged_feature,
                    MERGED_PORT.clone(),
                ));
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureMerger"
    }
}
