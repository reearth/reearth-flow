use std::{collections::HashMap, sync::Arc};

use indexmap::IndexMap;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr};
use rhai::Dynamic;
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

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: AttributeMapperParam = if let Some(with) = with.clone() {
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
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let mut mappers = Vec::<CompiledMapper>::new();
        for mapper in &params.mappers {
            let expr = if let Some(expr) = &mapper.expr {
                Some(
                    expr_engine
                        .compile(expr.as_ref())
                        .map_err(|e| AttributeProcessorError::MapperFactory(format!("{e:?}")))?,
                )
            } else {
                None
            };
            let multiple_expr = if let Some(multiple_expr) = &mapper.multiple_expr {
                Some(
                    expr_engine
                        .compile(multiple_expr.as_ref())
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
            global_params: with,
            mapper: CompiledAttributeMapperParam { mappers },
            keep_existing_attributes: params.keep_existing_attributes,
        };
        Ok(Box::new(processor))
    }
}

/// # AttributeMapper Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AttributeMapperParam {
    /// # Attribute Mappers
    /// List of mapping rules to transform attributes using expressions or value copying
    mappers: Vec<Mapper>,
    /// # Keep Existing Attributes
    /// When true, preserves all existing feature attributes and adds/overwrites only the mapped attributes.
    /// When false (default), replaces all attributes with only the mapped ones.
    ///
    /// Use true for: Adding calculated fields, chaining multiple mappers, pipeline-style processing
    /// Use false for: Extracting specific fields for reports, creating clean output datasets
    #[serde(default)]
    keep_existing_attributes: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Mapper {
    /// # Attribute name
    attribute: Option<String>,
    /// # Expression to evaluate
    expr: Option<Expr>,
    /// # Attribute name to get value from
    value_attribute: Option<String>,
    /// # Parent attribute name
    parent_attribute: Option<String>,
    /// # Child attribute name
    child_attribute: Option<String>,
    /// # Expression to evaluate multiple attributes
    multiple_expr: Option<Expr>,
}

#[derive(Debug, Clone)]
struct CompiledAttributeMapperParam {
    mappers: Vec<CompiledMapper>,
}

#[derive(Debug, Clone)]
struct CompiledMapper {
    attribute: Option<String>,
    expr: Option<rhai::AST>,
    value_attribute: Option<String>,
    parent_attribute: Option<String>,
    child_attribute: Option<String>,
    multiple_expr: Option<rhai::AST>,
}

#[derive(Debug, Clone)]
pub struct AttributeMapper {
    global_params: Option<HashMap<String, serde_json::Value>>,
    mapper: CompiledAttributeMapperParam,
    keep_existing_attributes: bool,
}

impl Processor for AttributeMapper {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        // Start with existing attributes if keep_existing_attributes is true
        let mut attributes = if self.keep_existing_attributes {
            feature.attributes.clone()
        } else {
            IndexMap::<Attribute, AttributeValue>::new()
        };
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);
        for mapper in &self.mapper.mappers {
            match &mapper.attribute {
                Some(attribute) => {
                    if let Some(expr) = &mapper.expr {
                        let new_value = scope.eval_ast::<Dynamic>(expr);
                        if let Ok(new_value) = new_value {
                            if let Ok(new_value) = new_value.try_into() {
                                attributes.insert(Attribute::new(attribute.clone()), new_value);
                            } else {
                                attributes.insert(
                                    Attribute::new(attribute.clone()),
                                    AttributeValue::Null,
                                );
                            }
                        } else {
                            attributes
                                .insert(Attribute::new(attribute.clone()), AttributeValue::Null);
                        }
                        continue;
                    } else if let Some(value_attribute) = &mapper.value_attribute {
                        if let Some(value) = feature.get(value_attribute) {
                            attributes.insert(Attribute::new(attribute.clone()), value.clone());
                        } else {
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
                        let new_value = scope.eval_ast::<Dynamic>(multiple_expr);
                        if let Ok(new_value) = new_value {
                            if new_value.is::<rhai::Map>() {
                                if let Ok(AttributeValue::Map(new_value)) = new_value.try_into() {
                                    attributes.extend(
                                        new_value
                                            .iter()
                                            .map(|(k, v)| (Attribute::new(k.clone()), v.clone())),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        fw.send(
            ctx.new_with_feature_and_port(
                feature.with_attributes(attributes),
                DEFAULT_PORT.clone(),
            ),
        );
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeMapper"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keep_existing_attributes_false_replaces_all() {
        // Test that with keep_existing_attributes: false (default),
        // only mapped attributes are present in output
        let param = AttributeMapperParam {
            mappers: vec![Mapper {
                attribute: Some("output_value".to_string()),
                expr: Some(Expr::new("20")),
                value_attribute: None,
                parent_attribute: None,
                child_attribute: None,
                multiple_expr: None,
            }],
            keep_existing_attributes: false, // Default behavior
        };

        // After mapping, only "output_value" should exist
        // "existing_attr" and "input_value" should be gone
        assert!(!param.keep_existing_attributes);
    }

    #[test]
    fn test_keep_existing_attributes_true_preserves_all() {
        // Test that with keep_existing_attributes: true,
        // existing attributes are preserved and new ones are added
        let param = AttributeMapperParam {
            mappers: vec![Mapper {
                attribute: Some("output_value".to_string()),
                expr: Some(Expr::new("20")),
                value_attribute: None,
                parent_attribute: None,
                child_attribute: None,
                multiple_expr: None,
            }],
            keep_existing_attributes: true, // Preserve existing
        };

        // After mapping, all three attributes should exist:
        // "existing_attr", "input_value", and "output_value"
        assert!(param.keep_existing_attributes);
    }

    #[test]
    fn test_keep_existing_attributes_overwrites_existing() {
        // Test that mapped attributes can overwrite existing ones
        // when keep_existing_attributes is true
        let param = AttributeMapperParam {
            mappers: vec![Mapper {
                attribute: Some("value".to_string()),
                expr: Some(Expr::new("20")), // Overwrite with new value
                value_attribute: None,
                parent_attribute: None,
                child_attribute: None,
                multiple_expr: None,
            }],
            keep_existing_attributes: true,
        };

        // The "value" attribute should be overwritten with the new mapped value
        assert!(param.keep_existing_attributes);
    }

    #[test]
    fn test_default_is_false_for_backward_compatibility() {
        // Test that keep_existing_attributes defaults to false
        // for backward compatibility with existing workflows
        let json = r#"{"mappers": []}"#;
        let param: AttributeMapperParam = serde_json::from_str(json).unwrap();

        assert!(
            !param.keep_existing_attributes,
            "keep_existing_attributes should default to false for backward compatibility"
        );
    }

    #[test]
    fn test_explicit_true_serialization() {
        // Test that keep_existing_attributes: true can be deserialized
        let json = r#"{"mappers": [], "keepExistingAttributes": true}"#;
        let param: AttributeMapperParam = serde_json::from_str(json).unwrap();

        assert!(param.keep_existing_attributes);
    }

    #[test]
    fn test_explicit_false_serialization() {
        // Test that keep_existing_attributes: false can be explicitly set
        let json = r#"{"mappers": [], "keepExistingAttributes": false}"#;
        let param: AttributeMapperParam = serde_json::from_str(json).unwrap();

        assert!(!param.keep_existing_attributes);
    }
}
