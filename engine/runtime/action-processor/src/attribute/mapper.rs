use std::{collections::HashMap, sync::Arc};

use indexmap::IndexMap;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Code, CodeType, CompiledCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::AttributeProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct AttributeMapperFactory;

impl ProcessorFactory for AttributeMapperFactory {
    fn name(&self) -> &str {
        "AttributeMapper"
    }

    fn description(&self) -> &str {
        "Transform Feature Attributes Using Expressions and Mappings"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AttributeMapperParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Attribute"]
    }

    fn tags(&self) -> &[&'static str] {
        &["mapping"]
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
        let params: AttributeMapperParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::MapperFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::MapperFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(AttributeProcessorError::MapperFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let mut mappers = Vec::<CompiledMapper>::new();
        for mapper in &params.mappers {
            let expr = if let Some(expr) = &mapper.expr {
                Some(
                    expr.compile()
                        .map_err(|e| AttributeProcessorError::MapperFactory(format!("{e:?}")))?,
                )
            } else {
                None
            };
            let multiple_expr = if let Some(multiple_expr) = &mapper.multiple_expr {
                Some(
                    multiple_expr
                        .compile()
                        .map_err(|e| AttributeProcessorError::MapperFactory(format!("{e:?}")))?,
                )
            } else {
                None
            };
            mappers.push(CompiledMapper {
                attribute: mapper.attribute.clone(),
                value_attribute: mapper.value_attribute.clone(),
                parent_attribute: mapper.parent_attribute.clone(),
                child_attribute: mapper.child_attribute.clone(),
                expr,
                multiple_expr,
            });
        }

        let processor = AttributeMapper {
            mapper: CompiledAttributeMapperParam { mappers },
        };
        Ok(Box::new(processor))
    }

    fn infer_output_schema(
        &self,
        _inputs: &HashMap<Port, reearth_flow_types::attr_schema::AttrSchema>,
        with: &Option<HashMap<String, Value>>,
    ) -> Option<HashMap<Port, reearth_flow_types::attr_schema::AttrSchema>> {
        use reearth_flow_types::attr_schema::{AttrField, AttrSchema, AttrType};
        use reearth_flow_types::Attribute;

        let params = parse_params(with)?;

        // AttributeMapper REPLACES the whole attribute set (it builds a fresh
        // IndexMap and calls `with_attributes`). Input attributes do NOT pass
        // through, so the seed is an empty, CLOSED schema.
        let mut out = AttrSchema::empty();

        for mapper in &params.mappers {
            match &mapper.attribute {
                Some(name) => {
                    let attr = Attribute::new(name.clone());
                    if mapper.expr.is_some() || mapper.value_attribute.is_some() {
                        // Expression/copy-derived value: key is always inserted,
                        // type can't be analyzed statically -> Unknown, Always.
                        out.insert(attr, AttrField::always(AttrType::Unknown));
                    } else if mapper.parent_attribute.is_some() && mapper.child_attribute.is_some()
                    {
                        // Parent/child copy: the key is only inserted when the
                        // parent map contains the child -> conditional -> Maybe.
                        out.insert(attr, AttrField::maybe(AttrType::Unknown));
                    }
                    // Otherwise the runtime inserts nothing for this mapper; skip.
                }
                None => {
                    // `multipleExpr` returns a Map with unpredictable keys; we
                    // can't enumerate them, so mark the schema open.
                    if mapper.multiple_expr.is_some() {
                        out.open = true;
                    }
                    // No `multipleExpr` -> no-op; skip.
                }
            }
        }

        Some(HashMap::from([(FEATURES_PORT.clone(), out)]))
    }
}

/// Deserialize the `AttributeMapperParam` from the node's `with` params,
/// mirroring the deserialization done in `build`. Returns `None` when `with`
/// is absent or the params don't deserialize (inference not possible).
fn parse_params(with: &Option<HashMap<String, Value>>) -> Option<AttributeMapperParam> {
    let with = with.as_ref()?;
    let value = serde_json::to_value(with).ok()?;
    serde_json::from_value::<AttributeMapperParam>(value).ok()
}

/// # AttributeMapper Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AttributeMapperParam {
    /// # Attribute Mappers
    /// List of mapping rules to transform attributes using expressions or value copying
    mappers: Vec<Mapper>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Mapper {
    /// # Attribute name
    attribute: Option<String>,
    /// # Expression to evaluate
    expr: Option<Code>,
    /// # Attribute name to get value from
    value_attribute: Option<String>,
    /// # Parent attribute name
    parent_attribute: Option<String>,
    /// # Child attribute name
    child_attribute: Option<String>,
    /// # Expression to evaluate multiple attributes
    multiple_expr: Option<Code<{ CodeType::FlowExpr as u32 }>>,
}

#[derive(Debug, Clone)]
struct CompiledAttributeMapperParam {
    mappers: Vec<CompiledMapper>,
}

#[derive(Debug, Clone)]
struct CompiledMapper {
    attribute: Option<String>,
    expr: Option<CompiledCode>,
    value_attribute: Option<String>,
    parent_attribute: Option<String>,
    child_attribute: Option<String>,
    multiple_expr: Option<CompiledCode>,
}

#[derive(Debug, Clone)]
pub struct AttributeMapper {
    mapper: CompiledAttributeMapperParam,
}

impl Processor for AttributeMapper {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let env_vars = ctx.env_vars.clone();
        let mut attributes = IndexMap::<Attribute, AttributeValue>::new();
        for mapper in &self.mapper.mappers {
            match &mapper.attribute {
                Some(attribute) => {
                    if let Some(expr) = &mapper.expr {
                        let new_value = match expr.eval(feature, Arc::clone(&env_vars)) {
                            Ok(v) => v,
                            Err(e) => {
                                tracing::error!(
                                    "Failed to evaluate expr for attribute `{attribute}`: {e:?}"
                                );
                                AttributeValue::Null
                            }
                        };
                        attributes.insert(Attribute::new(attribute.clone()), new_value);
                        continue;
                    } else if let Some(value_attribute) = &mapper.value_attribute {
                        if let Some(value) = feature.get(value_attribute) {
                            attributes.insert(Attribute::new(attribute.clone()), value.clone());
                        } else {
                            // Missing attribute is defined behavior. Do not log an error here.
                            attributes
                                .insert(Attribute::new(attribute.clone()), AttributeValue::Null);
                        }
                        continue;
                    } else if let (Some(parent_attribute), Some(child_attribute)) =
                        (&mapper.parent_attribute, &mapper.child_attribute)
                    {
                        if let Some(AttributeValue::Map(parent)) = feature.get(parent_attribute) {
                            if let Some(child) = parent.get(child_attribute) {
                                attributes.insert(Attribute::new(attribute.clone()), child.clone());
                            }
                        }
                    }
                }
                None => {
                    if let Some(multiple_expr) = &mapper.multiple_expr {
                        match multiple_expr.eval(feature, Arc::clone(&env_vars)) {
                            Err(e) => {
                                tracing::error!("Failed to evaluate multiple_expr: {e:?}");
                            }
                            Ok(AttributeValue::Map(new_value)) => {
                                attributes.extend(
                                    new_value
                                        .iter()
                                        .map(|(k, v)| (Attribute::new(k.clone()), v.clone())),
                                );
                            }
                            Ok(other) => {
                                tracing::error!(
                                    "multiple_expr did not produce a Map, got: {other:?}"
                                );
                            }
                        }
                    }
                }
            }
        }
        fw.send(
            ctx.new_with_feature_and_port(
                feature.with_attributes(attributes),
                FEATURES_PORT.clone(),
            ),
        );
        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeMapper"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::attr_schema::{AttrField, AttrSchema, AttrType, Presence};
    use reearth_flow_types::Attribute;
    use serde_json::json;

    fn with_from(value: Value) -> Option<HashMap<String, Value>> {
        Some(serde_json::from_value(value).unwrap())
    }

    #[test]
    fn infer_replaces_with_mapped_attrs_only() {
        let with = with_from(json!({
            "mappers": [
                { "attribute": "a", "valueAttribute": "src" }
            ]
        }));

        let mut input = AttrSchema::empty();
        input.insert(
            Attribute::new("keep_me".to_string()),
            AttrField::always(AttrType::String),
        );
        let mut inputs = HashMap::new();
        inputs.insert(FEATURES_PORT.clone(), input);

        let out = AttributeMapperFactory
            .infer_output_schema(&inputs, &with)
            .expect("inference should succeed");
        let schema = out
            .get(&FEATURES_PORT.clone())
            .expect("default port present");

        // "a" is present, Unknown + Always.
        assert_eq!(
            schema.fields.get(&Attribute::new("a".to_string())),
            Some(&AttrField::always(AttrType::Unknown))
        );
        // Input attribute does NOT pass through (replace semantics).
        assert!(!schema
            .fields
            .contains_key(&Attribute::new("keep_me".to_string())));
        // Only one field, not open.
        assert_eq!(schema.fields.len(), 1);
        assert!(!schema.open);
    }

    #[test]
    fn infer_multiple_expr_sets_open() {
        let with = with_from(json!({
            "mappers": [
                { "multipleExpr": { "type": "flowExpr", "value": "#{ x: 1 }" } }
            ]
        }));

        let inputs = HashMap::new();

        let out = AttributeMapperFactory
            .infer_output_schema(&inputs, &with)
            .expect("inference should succeed");
        let schema = out
            .get(&FEATURES_PORT.clone())
            .expect("default port present");

        assert!(schema.open);
        // No named fields can be enumerated for multipleExpr.
        assert!(schema.fields.is_empty());
    }

    #[test]
    fn infer_parent_child_is_maybe() {
        let with = with_from(json!({
            "mappers": [
                { "attribute": "a", "parentAttribute": "p", "childAttribute": "c" }
            ]
        }));

        let inputs = HashMap::new();

        let out = AttributeMapperFactory
            .infer_output_schema(&inputs, &with)
            .expect("inference should succeed");
        let schema = out
            .get(&FEATURES_PORT.clone())
            .expect("default port present");

        let field = schema
            .fields
            .get(&Attribute::new("a".to_string()))
            .expect("a present");
        assert_eq!(field.presence, Presence::Maybe);
        assert_eq!(field.ty, AttrType::Unknown);
    }
}
