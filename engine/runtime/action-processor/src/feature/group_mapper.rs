use std::collections::HashMap;
use std::sync::Arc;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Code, CodeType, CompiledCode, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureGroupMapperFactory;

impl ProcessorFactory for FeatureGroupMapperFactory {
    fn name(&self) -> &str {
        "Feature Group Mapper"
    }

    fn description(&self) -> &str {
        "Groups consecutive features sharing the same attribute value and replaces each feature's attributes with the corresponding entry returned by an expression over the whole group. Input must already be sorted by the grouping attribute."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureGroupMapperParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Transform"]
    }

    fn tags(&self) -> &[&'static str] {
        &["scripting", "attribute"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureGroupMapperParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::GroupMapperFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::GroupMapperFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::GroupMapperFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let expr = params
            .expr
            .compile()
            .map_err(|e| FeatureProcessorError::GroupMapperFactory(format!("{e:?}")))?;

        Ok(Box::new(FeatureGroupMapper {
            attribute: params.attribute,
            expr,
            current_key: None,
            group: Vec::new(),
        }))
    }
}

/// # Feature Group Mapper Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct FeatureGroupMapperParam {
    /// # Group By Attribute
    /// Attribute whose value groups consecutive features together.
    attribute: Attribute,
    /// # Expression
    /// Expression over `features` (one attributes entry per feature in the group) and `variables`, returning a list the same length as `features`. Each returned map replaces the corresponding feature's attributes.
    expr: Code<{ CodeType::FlowExpr as u32 }>,
}

#[derive(Debug, Clone)]
struct FeatureGroupMapper {
    attribute: Attribute,
    expr: CompiledCode,
    current_key: Option<AttributeValue>,
    group: Vec<Feature>,
}

impl FeatureGroupMapper {
    fn flush(
        &mut self,
        variables: Arc<serde_json::Map<String, serde_json::Value>>,
        mut emit: impl FnMut(Feature),
    ) -> Result<(), FeatureProcessorError> {
        if self.group.is_empty() {
            return Ok(());
        }
        let group = std::mem::take(&mut self.group);
        for feature in merge_group(&self.expr, &group, variables)? {
            emit(feature);
        }
        Ok(())
    }
}

impl Processor for FeatureGroupMapper {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = ctx.feature.clone();
        let key = feature
            .get(&self.attribute)
            .ok_or_else(|| {
                FeatureProcessorError::GroupMapper(format!(
                    "feature is missing grouping attribute `{}`",
                    self.attribute
                ))
            })?
            .clone();

        if self.current_key.as_ref().is_some_and(|k| *k != key) {
            self.flush(ctx.variables.clone(), |f| {
                fw.send(ctx.new_with_feature_and_port(f, FEATURES_PORT.clone()));
            })?;
        }
        self.current_key = Some(key);
        self.group.push(feature);
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let context = ctx.as_context();
        self.flush(ctx.variables.clone(), |f| {
            fw.send(ExecutorContext::new_with_context_feature_and_port(
                &context,
                f,
                FEATURES_PORT.clone(),
            ));
        })?;
        Ok(())
    }

    fn name(&self) -> &str {
        "Feature Group Mapper"
    }
}

fn merge_group(
    expr: &CompiledCode,
    group: &[Feature],
    variables: Arc<serde_json::Map<String, serde_json::Value>>,
) -> Result<Vec<Feature>, FeatureProcessorError> {
    let result = expr.eval_features(group, variables).map_err(|e| {
        FeatureProcessorError::GroupMapper(format!("Failed to evaluate expression: {e}"))
    })?;
    let AttributeValue::Array(items) = result else {
        return Err(FeatureProcessorError::GroupMapper(
            "expression must return a list of attribute maps".to_string(),
        ));
    };
    if items.len() != group.len() {
        return Err(FeatureProcessorError::GroupMapper(format!(
            "expression returned {} item(s), expected {} (one per feature in the group)",
            items.len(),
            group.len()
        )));
    }
    group
        .iter()
        .zip(items)
        .map(|(feature, item)| {
            let AttributeValue::Map(map) = item else {
                return Err(FeatureProcessorError::GroupMapper(
                    "expression must return a list of attribute maps".to_string(),
                ));
            };
            Ok(feature.with_attributes(
                map.into_iter()
                    .map(|(k, v)| (Attribute::new(k), v))
                    .collect(),
            ))
        })
        .collect()
}
