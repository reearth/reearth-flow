use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::{BoxedError, ExecutionError},
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory},
};
use reearth_flow_types::{Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

static REQUESTOR_PORT: Lazy<Port> = Lazy::new(|| Port::new("requestor"));
static SUPPLIER_PORT: Lazy<Port> = Lazy::new(|| Port::new("supplier"));
static MERGED_PORT: Lazy<Port> = Lazy::new(|| Port::new("merged"));
static UNMERGED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unmerged"));

#[derive(Debug, Clone, Default)]
pub struct FeatureMergerFactory;

impl ProcessorFactory for FeatureMergerFactory {
    fn name(&self) -> &str {
        "FeatureMerger"
    }

    fn description(&self) -> &str {
        "Merges features by attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureMergerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![REQUESTOR_PORT.clone(), SUPPLIER_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![MERGED_PORT.clone(), UNMERGED_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureMergerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::MergerFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::MergerFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(FeatureProcessorError::MergerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let requestor_attribute = expr_engine
            .compile(params.requestor_attribute.as_ref())
            .map_err(|e| {
                FeatureProcessorError::MergerFactory(format!(
                    "Failed to compile requestor attribute: {}",
                    e
                ))
            })?;
        let supplier_attribute = expr_engine
            .compile(params.supplier_attribute.as_ref())
            .map_err(|e| {
                FeatureProcessorError::MergerFactory(format!(
                    "Failed to compile supplier attribute: {}",
                    e
                ))
            })?;

        let process = FeatureMerger {
            params: CompliledParam {
                requestor_attribute,
                supplier_attribute,
            },
            request_features: vec![],
            supplier_buffer: HashMap::new(),
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct FeatureMerger {
    params: CompliledParam,
    request_features: Vec<Feature>,
    supplier_buffer: HashMap<String, Vec<Feature>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureMergerParam {
    requestor_attribute: Expr,
    supplier_attribute: Expr,
}

#[derive(Debug, Clone)]
struct CompliledParam {
    requestor_attribute: rhai::AST,
    supplier_attribute: rhai::AST,
}

impl Processor for FeatureMerger {
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
                let expr_engine = Arc::clone(&ctx.expr_engine);
                let scope = feature.new_scope(expr_engine.clone());
                let value = scope
                    .eval_ast::<String>(&self.params.supplier_attribute)
                    .map_err(|e| {
                        FeatureProcessorError::Merger(format!(
                            "Failed to evaluate supplier attribute: {}",
                            e
                        ))
                    })?;
                match self.supplier_buffer.entry(value) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().push(feature);
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(vec![feature]);
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
        let expr_engine = Arc::clone(&ctx.expr_engine);
        for request_feature in self.request_features.iter() {
            let scope = request_feature.new_scope(expr_engine.clone());
            let request_value = scope
                .eval_ast::<String>(&self.params.requestor_attribute)
                .map_err(|e| {
                    FeatureProcessorError::Merger(format!(
                        "Failed to evaluate requestor attribute: {}",
                        e
                    ))
                })?;

            let Some(supplier_features) = self.supplier_buffer.get(&request_value) else {
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    request_feature.clone(),
                    UNMERGED_PORT.clone(),
                ));
                continue;
            };

            for (idx, supplier_feature) in supplier_features.iter().enumerate() {
                let mut merged_feature = request_feature.clone();
                if idx > 0 {
                    merged_feature.refresh_id();
                }
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
