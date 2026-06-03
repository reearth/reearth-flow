use std::{collections::HashMap, sync::Arc};

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};

use reearth_flow_types::{Code, CompiledCode, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::AttributeProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct AttributeManagerFactory;

impl ProcessorFactory for AttributeManagerFactory {
    fn name(&self) -> &str {
        "AttributeManager"
    }

    fn description(&self) -> &str {
        "Create, Convert, Rename, and Remove Feature Attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AttributeManagerParam))
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
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: AttributeManagerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::ManagerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::ManagerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(AttributeProcessorError::ManagerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let operations = convert_single_operation(&params.operations)?;
        let process = AttributeManager { operations };
        Ok(Box::new(process))
    }

    fn infer_output_schema(
        &self,
        inputs: &HashMap<Port, reearth_flow_types::attr_schema::AttrSchema>,
        with: &Option<HashMap<String, Value>>,
    ) -> Option<HashMap<Port, reearth_flow_types::attr_schema::AttrSchema>> {
        use reearth_flow_types::attr_schema::{AttrField, AttrSchema, AttrType};
        use reearth_flow_types::Attribute;

        let params = parse_params(with)?;

        let mut out = inputs
            .get(&DEFAULT_PORT.clone())
            .cloned()
            .unwrap_or_else(AttrSchema::open);

        for op in &params.operations {
            let attr = Attribute::new(op.attribute.clone());
            match op.method {
                // Create/Convert both set the attribute to an expression-derived value,
                // whose type we can't analyze statically -> Unknown, Always present.
                Method::Create | Method::Convert => {
                    out.insert(attr, AttrField::always(AttrType::Unknown));
                }
                // Rename's destination name is an expression -> not statically knowable.
                // Drop the source key and mark the schema open (an unknown-named attr appears).
                Method::Rename => {
                    out.fields.shift_remove(&attr);
                    out.open = true;
                }
                Method::Remove => {
                    out.fields.shift_remove(&attr);
                }
            }
        }

        Some(HashMap::from([(DEFAULT_PORT.clone(), out)]))
    }
}

/// Deserialize the `AttributeManagerParam` from the node's `with` params,
/// mirroring the deserialization done in `build`. Returns `None` when `with`
/// is absent or the params don't deserialize (inference not possible).
fn parse_params(with: &Option<HashMap<String, Value>>) -> Option<AttributeManagerParam> {
    let with = with.as_ref()?;
    let value = serde_json::to_value(with).ok()?;
    serde_json::from_value::<AttributeManagerParam>(value).ok()
}

#[derive(Debug, Clone)]
struct AttributeManager {
    operations: Vec<Operate>,
}

/// # AttributeManager Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AttributeManagerParam {
    /// # Attribute Operations
    /// List of operations to perform on feature attributes (create, convert, rename, remove)
    operations: Vec<Operation>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Operation {
    /// # Attribute name
    attribute: String,
    /// # Operation to perform
    method: Method,
    /// # Value
    /// Value to use for the operation
    value: Option<Code>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum Method {
    Convert,
    Create,
    Rename,
    Remove,
}

#[derive(Debug, Clone)]
enum Operate {
    Convert {
        code: Option<CompiledCode>,
        attribute: String,
    },
    Create {
        code: Option<CompiledCode>,
        attribute: String,
    },
    Rename {
        new_key: CompiledCode,
        attribute: String,
    },
    Remove {
        attribute: String,
    },
}

impl Processor for AttributeManager {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let env_vars = ctx.expr_engine.vars();
        let feature = process_feature(ctx.as_context(), &ctx.feature, &self.operations, env_vars);
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
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
        "AttributeManager"
    }
}

fn process_feature(
    ctx: Context,
    feature: &Feature,
    operations: &[Operate],
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
) -> Feature {
    let mut result = feature.clone();
    for operation in operations {
        match operation {
            Operate::Convert { code, attribute } => {
                if feature.get(attribute).is_none() {
                    continue;
                }
                if let Some(code) = code {
                    match code.eval(feature, Arc::clone(&env_vars)) {
                        Ok(new_value) => {
                            result.insert(attribute.clone(), new_value);
                        }
                        Err(e) => {
                            ctx.event_hub
                                .warn_log(None, format!("convert error with: {e:?}"));
                        }
                    }
                }
            }
            Operate::Create { code, attribute } => {
                if let Some(code) = code {
                    match code.eval(feature, Arc::clone(&env_vars)) {
                        Ok(new_value) => {
                            result.insert(attribute.clone(), new_value);
                        }
                        Err(e) => {
                            ctx.event_hub
                                .warn_log(None, format!("create error with: {e:?}"));
                        }
                    }
                }
            }
            Operate::Rename { new_key, attribute } => {
                if !feature.contains_key(attribute) {
                    continue;
                }
                match new_key.eval_string(feature, Arc::clone(&env_vars)) {
                    Ok(new_key_str) => {
                        if feature.contains_key(&new_key_str) {
                            continue;
                        }
                        let value = feature.get(attribute);
                        result.remove(attribute);
                        result.insert(new_key_str, value.cloned().unwrap_or_default());
                    }
                    Err(e) => {
                        ctx.event_hub
                            .warn_log(None, format!("rename error with: {e:?}"));
                    }
                }
            }
            Operate::Remove { attribute } => {
                if !feature.contains_key(attribute) {
                    continue;
                }
                result.remove(attribute);
            }
        };
    }
    result
}

fn convert_single_operation(operations: &[Operation]) -> super::errors::Result<Vec<Operate>> {
    let mut result = Vec::new();
    for operation in operations.iter() {
        let method = &operation.method;
        let attribute = &operation.attribute;
        let code = if let Some(code) = operation
            .value
            .clone()
            .take_if(|_| matches!(method, Method::Convert | Method::Create))
        {
            Some(
                code.compile()
                    .map_err(|e| AttributeProcessorError::ManagerFactory(format!("{e:?}")))?,
            )
        } else {
            None
        };
        let value = match method {
            Method::Convert => Operate::Convert {
                code,
                attribute: attribute.clone(),
            },
            Method::Create => Operate::Create {
                code,
                attribute: attribute.clone(),
            },
            Method::Rename => {
                let new_key = operation
                    .value
                    .as_ref()
                    .ok_or_else(|| {
                        AttributeProcessorError::ManagerFactory(
                            "Rename requires a value".to_string(),
                        )
                    })?
                    .compile()
                    .map_err(|e| AttributeProcessorError::ManagerFactory(format!("{e:?}")))?;
                Operate::Rename {
                    new_key,
                    attribute: attribute.clone(),
                }
            }
            Method::Remove => Operate::Remove {
                attribute: attribute.clone(),
            },
        };
        result.push(value);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::attr_schema::{AttrField, AttrSchema, AttrType};
    use reearth_flow_types::Attribute;
    use serde_json::json;

    fn with_from(value: Value) -> Option<HashMap<String, Value>> {
        Some(serde_json::from_value(value).unwrap())
    }

    #[test]
    fn infer_create_adds_unknown_attribute() {
        let with = with_from(json!({
            "operations": [
                { "attribute": "foo", "method": "create", "value": null }
            ]
        }));

        let mut input = AttrSchema::empty();
        input.insert(
            Attribute::new("bar".to_string()),
            AttrField::always(AttrType::String),
        );
        let mut inputs = HashMap::new();
        inputs.insert(DEFAULT_PORT.clone(), input);

        let out = AttributeManagerFactory
            .infer_output_schema(&inputs, &with)
            .expect("inference should succeed");
        let schema = out
            .get(&DEFAULT_PORT.clone())
            .expect("default port present");

        assert_eq!(
            schema.fields.get(&Attribute::new("bar".to_string())),
            Some(&AttrField::always(AttrType::String))
        );
        assert_eq!(
            schema.fields.get(&Attribute::new("foo".to_string())),
            Some(&AttrField::always(AttrType::Unknown))
        );
    }

    #[test]
    fn infer_remove_drops_attribute() {
        let with = with_from(json!({
            "operations": [
                { "attribute": "a", "method": "remove", "value": null }
            ]
        }));

        let mut input = AttrSchema::empty();
        input.insert(
            Attribute::new("a".to_string()),
            AttrField::always(AttrType::String),
        );
        input.insert(
            Attribute::new("b".to_string()),
            AttrField::always(AttrType::Number),
        );
        let mut inputs = HashMap::new();
        inputs.insert(DEFAULT_PORT.clone(), input);

        let out = AttributeManagerFactory
            .infer_output_schema(&inputs, &with)
            .expect("inference should succeed");
        let schema = out
            .get(&DEFAULT_PORT.clone())
            .expect("default port present");

        assert!(!schema.fields.contains_key(&Attribute::new("a".to_string())));
        assert_eq!(
            schema.fields.get(&Attribute::new("b".to_string())),
            Some(&AttrField::always(AttrType::Number))
        );
    }

    #[test]
    fn infer_rename_sets_open_and_drops_source() {
        let with = with_from(json!({
            "operations": [
                { "attribute": "a", "method": "rename", "value": { "type": "string", "value": "new_name" } }
            ]
        }));

        let mut input = AttrSchema::empty();
        input.insert(
            Attribute::new("a".to_string()),
            AttrField::always(AttrType::String),
        );
        let mut inputs = HashMap::new();
        inputs.insert(DEFAULT_PORT.clone(), input);

        let out = AttributeManagerFactory
            .infer_output_schema(&inputs, &with)
            .expect("inference should succeed");
        let schema = out
            .get(&DEFAULT_PORT.clone())
            .expect("default port present");

        assert!(!schema.fields.contains_key(&Attribute::new("a".to_string())));
        assert!(schema.open);
    }
}
