use std::{collections::HashMap, sync::Arc};

use reearth_flow_diagnostics::{DiagnosticDraft, ErrorCode};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
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
        "Attribute Manager"
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
        use reearth_flow_types::attr_schema::{AttrField, AttrSchema, AttrType, Presence};
        use reearth_flow_types::Attribute;

        let params = parse_params(with)?;

        let mut out = inputs
            .get(&FEATURES_PORT.clone())
            .cloned()
            .unwrap_or_else(AttrSchema::open);

        // Inference must NOT over-claim presence relative to the conditional
        // runtime in `process_feature`: Create/Convert/Rename are all applied
        // conditionally (eval failure, missing source, destination collision),
        // so we stay conservative about whether a key is present.
        for op in &params.operations {
            let attr = Attribute::new(op.attribute.clone());
            match op.method {
                // Create/Convert derive the value from an expression whose type
                // we can't analyze statically -> Unknown.
                // Runtime: Create overwrites on success (else warns + skips);
                // Convert requires the source present and keeps the original on
                // failure. So an already-present key stays present (downgrade is
                // unwarranted), but a key that wasn't there only appears when the
                // expression succeeds -> Maybe.
                Method::Create | Method::Convert => {
                    if let Some(existing) = out.fields.get_mut(&attr) {
                        existing.ty = AttrType::Unknown;
                    } else {
                        out.insert(attr, AttrField::maybe(AttrType::Unknown));
                    }
                }
                // Rename's destination name is an expression -> not statically
                // knowable, and the runtime keeps the source when eval fails or
                // the destination already exists. So downgrade the source to
                // Maybe (it might be removed, might not) rather than dropping it,
                // and mark the schema open (an unknown-named attr may appear).
                Method::Rename => {
                    if let Some(existing) = out.fields.get_mut(&attr) {
                        existing.presence = Presence::Maybe;
                    }
                    out.open = true;
                }
                // Remove deletes the key when present (and is a no-op when absent),
                // so dropping it from the schema is correct.
                Method::Remove => {
                    out.fields.shift_remove(&attr);
                }
            }
        }

        Some(HashMap::from([(FEATURES_PORT.clone(), out)]))
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
        let env_vars = ctx.env_vars.clone();
        let feature = process_feature(&ctx, &ctx.feature, &self.operations, env_vars);
        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
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
        "Attribute Manager"
    }
}

fn process_feature(
    ctx: &ExecutorContext,
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
                            ctx.warn(
                                DiagnosticDraft::new(ErrorCode::ExprAttributeOperationFailed)
                                    .with_message(format!("convert error with: {e:?}")),
                            );
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
                            ctx.warn(
                                DiagnosticDraft::new(ErrorCode::ExprAttributeOperationFailed)
                                    .with_message(format!("create error with: {e:?}")),
                            );
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
                        ctx.warn(
                            DiagnosticDraft::new(ErrorCode::ExprAttributeOperationFailed)
                                .with_message(format!("rename error with: {e:?}")),
                        );
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
        inputs.insert(FEATURES_PORT.clone(), input);

        let out = AttributeManagerFactory
            .infer_output_schema(&inputs, &with)
            .expect("inference should succeed");
        let schema = out
            .get(&FEATURES_PORT.clone())
            .expect("default port present");

        assert_eq!(
            schema.fields.get(&Attribute::new("bar".to_string())),
            Some(&AttrField::always(AttrType::String))
        );
        // "foo" was not present on input, so Create only produces it when the
        // expression succeeds -> Maybe, not Always.
        assert_eq!(
            schema.fields.get(&Attribute::new("foo".to_string())),
            Some(&AttrField::maybe(AttrType::Unknown))
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
        inputs.insert(FEATURES_PORT.clone(), input);

        let out = AttributeManagerFactory
            .infer_output_schema(&inputs, &with)
            .expect("inference should succeed");
        let schema = out
            .get(&FEATURES_PORT.clone())
            .expect("default port present");

        assert!(!schema.fields.contains_key(&Attribute::new("a".to_string())));
        assert_eq!(
            schema.fields.get(&Attribute::new("b".to_string())),
            Some(&AttrField::always(AttrType::Number))
        );
    }

    #[test]
    fn infer_rename_downgrades_source_and_sets_open() {
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
        inputs.insert(FEATURES_PORT.clone(), input);

        let out = AttributeManagerFactory
            .infer_output_schema(&inputs, &with)
            .expect("inference should succeed");
        let schema = out
            .get(&FEATURES_PORT.clone())
            .expect("default port present");

        // The runtime keeps the source key when eval fails or the destination
        // already exists, so the source is downgraded to Maybe rather than
        // dropped. Its type is unchanged.
        assert_eq!(
            schema.fields.get(&Attribute::new("a".to_string())),
            Some(&AttrField::maybe(AttrType::String))
        );
        // The destination name is an expression, so the schema is open.
        assert!(schema.open);
    }
}

#[cfg(test)]
mod diagnostics_tests {
    use std::sync::Arc;

    use indexmap::IndexMap;
    use reearth_flow_runtime::diagnostics::NodeDiagnosticsHandle;
    use reearth_flow_runtime::node::NodeHandle;
    use reearth_flow_types::{AttributeValue, CodeType};

    use super::*;

    /// `NodeId` is `pub(super)` inside `reearth_flow_runtime` (crate-private),
    /// so an external crate cannot name it directly. Its `Deserialize` impl
    /// is still public, so we build one via inference through `NodeHandle`'s
    /// public `id` field instead of naming the type.
    fn test_node_handle(id: &str) -> NodeHandle {
        NodeHandle {
            id: serde_json::from_value(serde_json::Value::String(id.to_string())).unwrap(),
        }
    }

    fn failing_code() -> CompiledCode {
        // "division by zero" is a deterministic eval-time failure that
        // still compiles (parsing never inspects operand values), so this
        // exercises the eval `Err` branch without depending on env/attribute
        // wiring.
        let code: Code = Code {
            ty: CodeType::FlowExpr,
            value: "1 / 0".to_string(),
        };
        code.compile().unwrap()
    }

    #[test]
    fn convert_eval_failure_warns_and_keeps_the_feature_flowing_unchanged() {
        let handle = Arc::new(NodeDiagnosticsHandle::new(
            "n1".to_string(),
            test_node_handle("n1"),
            "processor".into(),
            "Attribute Manager".into(),
            Arc::default(),
            Arc::new(reearth_flow_diagnostics::DispositionPolicy::default()),
            false,
        ));
        let node_ctx = NodeContext::default();
        let mut feature = Feature::from(IndexMap::<String, AttributeValue>::new());
        feature.insert("src", AttributeValue::String("v".into()));
        let mut ctx = ExecutorContext::new_with_node_context_feature_and_port(
            &node_ctx,
            feature.clone(),
            FEATURES_PORT.clone(),
        );
        ctx.diagnostics = Some(handle.clone());

        let operations = vec![Operate::Convert {
            code: Some(failing_code()),
            attribute: "src".to_string(),
        }];

        let result = process_feature(&ctx, &feature, &operations, ctx.env_vars.clone());

        // control-flow preservation: the feature keeps flowing, unchanged.
        assert_eq!(result.attributes, feature.attributes);

        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].aggregated.as_ref().unwrap().count, 1);
        // warn-and-continue never resolves an effective disposition.
        assert!(summaries[0].effective_disposition.is_none());
        assert!(summaries[0]
            .message
            .contains("expr.attribute_operation_failed"));
    }

    #[test]
    fn create_eval_failure_warns_and_keeps_the_feature_flowing_unchanged() {
        let handle = Arc::new(NodeDiagnosticsHandle::new(
            "n1".to_string(),
            test_node_handle("n1"),
            "processor".into(),
            "Attribute Manager".into(),
            Arc::default(),
            Arc::new(reearth_flow_diagnostics::DispositionPolicy::default()),
            false,
        ));
        let node_ctx = NodeContext::default();
        let feature = Feature::from(IndexMap::<String, AttributeValue>::new());
        let mut ctx = ExecutorContext::new_with_node_context_feature_and_port(
            &node_ctx,
            feature.clone(),
            FEATURES_PORT.clone(),
        );
        ctx.diagnostics = Some(handle.clone());

        let operations = vec![Operate::Create {
            code: Some(failing_code()),
            attribute: "computed".to_string(),
        }];

        let result = process_feature(&ctx, &feature, &operations, ctx.env_vars.clone());

        assert_eq!(result.attributes, feature.attributes);

        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].aggregated.as_ref().unwrap().count, 1);
        assert!(summaries[0].effective_disposition.is_none());
        assert!(summaries[0]
            .message
            .contains("expr.attribute_operation_failed"));
    }

    #[test]
    fn rename_eval_failure_warns_and_keeps_the_feature_flowing_unchanged() {
        let handle = Arc::new(NodeDiagnosticsHandle::new(
            "n1".to_string(),
            test_node_handle("n1"),
            "processor".into(),
            "Attribute Manager".into(),
            Arc::default(),
            Arc::new(reearth_flow_diagnostics::DispositionPolicy::default()),
            false,
        ));
        let node_ctx = NodeContext::default();
        let mut feature = Feature::from(IndexMap::<String, AttributeValue>::new());
        feature.insert("src", AttributeValue::String("v".into()));
        let mut ctx = ExecutorContext::new_with_node_context_feature_and_port(
            &node_ctx,
            feature.clone(),
            FEATURES_PORT.clone(),
        );
        ctx.diagnostics = Some(handle.clone());

        let operations = vec![Operate::Rename {
            new_key: failing_code(),
            attribute: "src".to_string(),
        }];

        let result = process_feature(&ctx, &feature, &operations, ctx.env_vars.clone());

        // rename never applied: source key is untouched.
        assert_eq!(result.attributes, feature.attributes);

        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].aggregated.as_ref().unwrap().count, 1);
        assert!(summaries[0].effective_disposition.is_none());
        assert!(summaries[0]
            .message
            .contains("expr.attribute_operation_failed"));
    }
}
