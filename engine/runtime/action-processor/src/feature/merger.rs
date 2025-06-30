use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::{BoxedError, ExecutionError},
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory},
};
use reearth_flow_types::{Attribute, Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::{self, FeatureProcessorError};

static REQUESTOR_PORT: Lazy<Port> = Lazy::new(|| Port::new("requestor"));
static SUPPLIER_PORT: Lazy<Port> = Lazy::new(|| Port::new("supplier"));
static MERGED_PORT: Lazy<Port> = Lazy::new(|| Port::new("merged"));
static UNMERGED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unmerged"));

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureMergerFactory;

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
        let params: FeatureMergerParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::MergerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::MergerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::MergerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let requestor_attribute_value =
            if let Some(requestor_attribute_value) = params.requestor_attribute_value {
                let result = expr_engine
                    .compile(requestor_attribute_value.as_ref())
                    .map_err(|e| {
                        FeatureProcessorError::MergerFactory(format!(
                            "Failed to compile requestor attribute value: {e}"
                        ))
                    })?;
                Some(result)
            } else {
                None
            };
        let supplier_attribute_value =
            if let Some(supplier_attribute_value) = params.supplier_attribute_value {
                let result = expr_engine
                    .compile(supplier_attribute_value.as_ref())
                    .map_err(|e| {
                        FeatureProcessorError::MergerFactory(format!(
                            "Failed to compile supplier attribute value: {e}"
                        ))
                    })?;
                Some(result)
            } else {
                None
            };
        if requestor_attribute_value.is_none() && params.requestor_attribute.is_none() {
            return Err(FeatureProcessorError::MergerFactory(
                "At least one of requestor_attribute_value or requestor_attribute must be provided"
                    .to_string(),
            )
            .into());
        }
        if supplier_attribute_value.is_none() && params.supplier_attribute.is_none() {
            return Err(FeatureProcessorError::MergerFactory(
                "At least one of supplier_attribute_value or supplier_attribute must be provided"
                    .to_string(),
            )
            .into());
        }
        let process = FeatureMerger {
            global_params: with,
            params: CompiledParam {
                requestor_attribute_value,
                supplier_attribute_value,
                requestor_attribute: params.requestor_attribute,
                supplier_attribute: params.supplier_attribute,
                complete_grouped: params.complete_grouped.unwrap_or(false),
            },
            requestor_buffer: HashMap::new(),
            supplier_buffer: HashMap::new(),
            requestor_before_value: None,
            supplier_before_value: None,
        };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureMergerParam {
    requestor_attribute: Option<Vec<Attribute>>,
    supplier_attribute: Option<Vec<Attribute>>,
    requestor_attribute_value: Option<Expr>,
    supplier_attribute_value: Option<Expr>,
    complete_grouped: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct FeatureMerger {
    global_params: Option<HashMap<String, serde_json::Value>>,
    params: CompiledParam,
    requestor_buffer: HashMap<String, (bool, Vec<Feature>)>, // (complete_grouped, features)
    supplier_buffer: HashMap<String, (bool, Vec<Feature>)>,  // (complete_grouped, features)
    requestor_before_value: Option<String>,
    supplier_before_value: Option<String>,
}

#[derive(Debug, Clone)]
struct CompiledParam {
    requestor_attribute: Option<Vec<Attribute>>,
    supplier_attribute: Option<Vec<Attribute>>,
    requestor_attribute_value: Option<rhai::AST>,
    supplier_attribute_value: Option<rhai::AST>,
    complete_grouped: bool,
}

impl Processor for FeatureMerger {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        match ctx.port {
            port if port == REQUESTOR_PORT.clone() => {
                let feature = &ctx.feature;
                let expr_engine = Arc::clone(&ctx.expr_engine);
                let requestor_attribute_value = feature.fetch_attribute_value(
                    expr_engine,
                    &self.global_params,
                    &self.params.requestor_attribute,
                    &self.params.requestor_attribute_value,
                );
                match self
                    .requestor_buffer
                    .entry(requestor_attribute_value.clone())
                {
                    Entry::Occupied(mut entry) => {
                        self.requestor_before_value = Some(requestor_attribute_value.clone());
                        let (_, buffer) = entry.get_mut();
                        buffer.push(feature.clone());
                    }
                    Entry::Vacant(entry) => {
                        entry.insert((false, vec![feature.clone()]));
                        if self.requestor_before_value.is_some() {
                            if let Entry::Occupied(mut entry) = self
                                .requestor_buffer
                                .entry(self.requestor_before_value.clone().unwrap())
                            {
                                let (complete_grouped, _) = entry.get_mut();
                                *complete_grouped = true;
                            }
                            self.change_group(
                                Context {
                                    expr_engine: ctx.expr_engine.clone(),
                                    storage_resolver: ctx.storage_resolver.clone(),
                                    kv_store: ctx.kv_store.clone(),
                                    event_hub: ctx.event_hub.clone(),
                                },
                                fw,
                            )?;
                        }
                        self.requestor_before_value = Some(requestor_attribute_value.clone());
                    }
                }
            }
            port if port == SUPPLIER_PORT.clone() => {
                let feature = &ctx.feature;
                let expr_engine = Arc::clone(&ctx.expr_engine);
                let supplier_attribute_value = feature.fetch_attribute_value(
                    expr_engine,
                    &self.global_params,
                    &self.params.supplier_attribute,
                    &self.params.supplier_attribute_value,
                );
                match self.supplier_buffer.entry(supplier_attribute_value.clone()) {
                    Entry::Occupied(mut entry) => {
                        self.supplier_before_value = Some(supplier_attribute_value.clone());
                        let (_, buffer) = entry.get_mut();
                        buffer.push(feature.clone());
                    }
                    Entry::Vacant(entry) => {
                        entry.insert((false, vec![feature.clone()]));
                        if self.supplier_before_value.is_some() {
                            if let Entry::Occupied(mut entry) = self
                                .supplier_buffer
                                .entry(self.supplier_before_value.clone().unwrap())
                            {
                                let (complete_grouped, _) = entry.get_mut();
                                *complete_grouped = true;
                            }
                            self.change_group(
                                Context {
                                    expr_engine: ctx.expr_engine.clone(),
                                    storage_resolver: ctx.storage_resolver.clone(),
                                    kv_store: ctx.kv_store.clone(),
                                    event_hub: ctx.event_hub.clone(),
                                },
                                fw,
                            )?;
                        }
                        self.supplier_before_value = Some(supplier_attribute_value.clone());
                    }
                }
            }
            port => return Err(ExecutionError::InvalidPortHandle(port).into()),
        }
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        for (request_value, (_, request_features)) in self.requestor_buffer.iter() {
            let Some((_, supplier_features)) = self.supplier_buffer.get(request_value) else {
                for request_feature in request_features.iter() {
                    fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                        &ctx,
                        request_feature.clone(),
                        UNMERGED_PORT.clone(),
                    ));
                }
                continue;
            };

            for request_feature in request_features.iter() {
                let mut merged_feature = request_feature.clone();
                for supplier_feature in supplier_features.iter() {
                    merged_feature
                        .attributes
                        .extend(supplier_feature.attributes.clone());
                }
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

impl FeatureMerger {
    fn change_group(&mut self, ctx: Context, fw: &ProcessorChannelForwarder) -> errors::Result<()> {
        if !self.params.complete_grouped {
            return Ok(());
        }
        let mut complete_keys = Vec::new();
        for (attribute, (complete_grouped, _)) in self.requestor_buffer.iter() {
            if !complete_grouped {
                continue;
            }
            let Some((supplier_complete, _)) = self.supplier_buffer.get(attribute) else {
                continue;
            };
            if !*supplier_complete {
                continue;
            }
            complete_keys.push(attribute.clone());
        }
        for attribute_value in complete_keys.iter() {
            let Some((_, requestor_features)) = self.requestor_buffer.remove(attribute_value)
            else {
                return Ok(());
            };
            let Some((_, supplier_features)) = self.supplier_buffer.remove(attribute_value) else {
                for request_feature in requestor_features.iter() {
                    fw.send(
                        ctx.as_executor_context(request_feature.clone(), UNMERGED_PORT.clone()),
                    );
                }
                return Ok(());
            };
            for request_feature in requestor_features.iter() {
                let mut merged_feature = request_feature.clone();
                for supplier_feature in supplier_features.iter() {
                    merged_feature
                        .attributes
                        .extend(supplier_feature.attributes.clone());
                }
                fw.send(ctx.as_executor_context(merged_feature, MERGED_PORT.clone()));
            }
        }
        Ok(())
    }
}
