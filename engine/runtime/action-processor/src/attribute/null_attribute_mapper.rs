use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::AttributeProcessorError;

static HAS_NULL_PORT: &str = "hasNull";
static REJECTED_PORT: &str = "rejected";

/// Defines what states count as "null"
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum NullKind {
    /// AttributeValue::Null
    #[default]
    Null,
    /// Attribute key absent from the feature
    Missing,
    /// AttributeValue::String("")
    EmptyString,
}

/// Scope of attributes to inspect
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub enum Scope {
    /// Only check attributes named in mappings
    #[default]
    Listed,
    /// Check all attributes on the feature
    All,
}

/// What to do when attribute is missing but not in nullDefinition
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub enum OnMissing {
    /// Leave unchanged
    #[default]
    Skip,
    /// Write replacement value, creating the attribute
    Create,
}

/// Per-attribute replacement mapping
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AttributeMapping {
    /// Name of the attribute to inspect
    pub attribute: String,
    /// Value to write when attribute is null-like
    /// null means remove the attribute
    pub replacement: Option<Value>,
    /// What to do when attribute is missing but not in nullDefinition
    #[serde(default)]
    pub on_missing: OnMissing,
}

/// NullAttributeMapper parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NullAttributeMapperParams {
    /// Per-attribute replacement rules
    #[serde(default)]
    pub mappings: Vec<AttributeMapping>,
    /// Fallback replacement for attributes not in mappings (when scope = "all")
    #[serde(default)]
    pub default_replacement: Option<Value>,
    /// Which states count as "null"
    #[serde(default = "default_null_definition")]
    pub null_definition: Vec<NullKind>,
    /// Emit original features with nulls to hasNull port
    #[serde(default)]
    pub route_null_features: bool,
    /// Which attributes to inspect
    #[serde(default)]
    pub scope: Scope,
}

fn default_null_definition() -> Vec<NullKind> {
    vec![NullKind::Null, NullKind::Missing]
}

impl Default for NullAttributeMapperParams {
    fn default() -> Self {
        Self {
            mappings: Vec::new(),
            default_replacement: None,
            null_definition: default_null_definition(),
            route_null_features: false,
            scope: Scope::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct NullAttributeMapperFactory;

impl ProcessorFactory for NullAttributeMapperFactory {
    fn name(&self) -> &str {
        "NullAttributeMapper"
    }

    fn description(&self) -> &str {
        "Replace null-like attribute values with configured defaults"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(NullAttributeMapperParams))
    }

    fn categories(&self) -> &[&'static str] {
        &["Attribute"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            DEFAULT_PORT.clone(),     // mapped
            Port::new(HAS_NULL_PORT), // hasNull
            Port::new(REJECTED_PORT), // rejected
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: NullAttributeMapperParams = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::NullAttributeMapperFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::NullAttributeMapperFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            NullAttributeMapperParams::default()
        };

        // Validate: null_definition must not be empty
        if params.null_definition.is_empty() {
            return Err(AttributeProcessorError::NullAttributeMapperFactory(
                "nullDefinition must include at least one value".to_string(),
            )
            .into());
        }

        let processor = NullAttributeMapper { params };
        Ok(Box::new(processor))
    }
}

#[derive(Debug, Clone)]
pub struct NullAttributeMapper {
    params: NullAttributeMapperParams,
}

/// Internal enum to track attribute state
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum AttributeState {
    Null,
    Missing,
    EmptyString,
    Present(AttributeValue),
}

impl Processor for NullAttributeMapper {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // If the feature arrives on a non-default port, reject it
        if ctx.port != *DEFAULT_PORT {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), Port::new(REJECTED_PORT)));
            return Ok(());
        }

        let feature = ctx.feature.clone();
        let mut had_null = false;

        // Clone attributes from Arc for modification
        let mut attributes: reearth_flow_types::Attributes = (*feature.attributes).clone();

        // Determine which attributes to inspect
        let keys_to_inspect: Vec<Attribute> = match &self.params.scope {
            Scope::Listed => self
                .params
                .mappings
                .iter()
                .map(|m| Attribute::new(&m.attribute))
                .collect(),
            Scope::All => {
                // All existing keys plus any keys in mappings that are missing
                let mut keys: Vec<Attribute> = attributes.keys().cloned().collect();
                for mapping in &self.params.mappings {
                    let attr = Attribute::new(&mapping.attribute);
                    if !keys.contains(&attr) {
                        keys.push(attr);
                    }
                }
                keys
            }
        };

        for attr_key in keys_to_inspect {
            let key_str = attr_key.to_string();
            let state = Self::determine_state(&attributes, &attr_key);
            let is_null = self.is_null_like(&state);

            if is_null {
                had_null = true;

                // Find replacement value (per-attribute mapping takes precedence)
                let mapping = self.params.mappings.iter().find(|m| m.attribute == key_str);

                // Distinguish between:
                // - mapping present with replacement Some(Value)
                // - mapping present with replacement None (explicit removal)
                // - mapping absent, in which case default_replacement (if any) applies
                let replacement = match mapping {
                    Some(m) => m.replacement.clone(),
                    None => self.params.default_replacement.clone(),
                };

                match (replacement, mapping.is_some()) {
                    (Some(value), _) => {
                        let attr_value = AttributeValue::from(value);
                        attributes.insert(attr_key, attr_value);
                    }
                    (None, true) => {
                        // Explicit removal requested by mapping (replacement: null)
                        attributes.swap_remove(&attr_key);
                    }
                    (None, false) => {
                        // No mapping and no default replacement: leave attribute unchanged
                    }
                }
            } else if matches!(state, AttributeState::Missing) {
                // Handle onMissing for non-null missing attributes
                if let Some(mapping) = self.params.mappings.iter().find(|m| m.attribute == key_str)
                {
                    if matches!(mapping.on_missing, OnMissing::Create) {
                        if let Some(ref replacement) = mapping.replacement {
                            let attr_value = AttributeValue::from(replacement.clone());
                            attributes.insert(attr_key, attr_value);
                        }
                    }
                }
            }
        }

        // Create modified feature with new attributes
        let modified_feature = reearth_flow_types::Feature {
            id: feature.id,
            attributes: std::sync::Arc::new(attributes),
            metadata: feature.metadata.clone(),
            geometry: feature.geometry.clone(),
        };

        // Emit to hasNull port if routeNullFeatures is true and feature had nulls
        if had_null && self.params.route_null_features {
            fw.send(ctx.new_with_feature_and_port(feature, Port::new(HAS_NULL_PORT)));
        }

        // Emit modified feature to mapped (default) port
        fw.send(ctx.new_with_feature_and_port(modified_feature, DEFAULT_PORT.clone()));

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
        "NullAttributeMapper"
    }
}

impl NullAttributeMapper {
    fn determine_state(
        attributes: &reearth_flow_types::Attributes,
        key: &Attribute,
    ) -> AttributeState {
        match attributes.get(key) {
            None => AttributeState::Missing,
            Some(AttributeValue::Null) => AttributeState::Null,
            Some(AttributeValue::String(s)) if s.is_empty() => AttributeState::EmptyString,
            Some(value) => AttributeState::Present(value.clone()),
        }
    }

    fn is_null_like(&self, state: &AttributeState) -> bool {
        match state {
            AttributeState::Null => self.params.null_definition.contains(&NullKind::Null),
            AttributeState::Missing => self.params.null_definition.contains(&NullKind::Missing),
            AttributeState::EmptyString => {
                self.params.null_definition.contains(&NullKind::EmptyString)
            }
            AttributeState::Present(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Attributes;
    use serde_json::Number;

    /// Helper to create a test feature with given attributes
    fn create_test_feature(
        attrs: Vec<(&str, Option<AttributeValue>)>,
    ) -> reearth_flow_types::Feature {
        let mut attributes = Attributes::new();
        for (key, value) in attrs {
            if let Some(v) = value {
                attributes.insert(Attribute::new(key), v);
            }
        }
        reearth_flow_types::Feature::new_with_attributes(attributes)
    }

    /// Helper to run processor and capture outputs
    fn run_processor(
        processor: &mut NullAttributeMapper,
        feature: &reearth_flow_types::Feature,
    ) -> (
        Vec<reearth_flow_types::Feature>,
        Vec<reearth_flow_types::Feature>,
    ) {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop.clone());

        // Use the test utility to create context
        let ctx = crate::tests::utils::create_default_execute_context(feature);

        processor.process(ctx, &fw).unwrap();

        // Get the features and ports from the NoopChannelForwarder
        let features = noop.send_features.lock().unwrap().clone();
        let ports = noop.send_ports.lock().unwrap().clone();

        // Separate by port
        let mut mapped = Vec::new();
        let mut has_null = Vec::new();

        for (i, feature) in features.iter().enumerate() {
            if i < ports.len() {
                if ports[i] == DEFAULT_PORT.clone() {
                    mapped.push(feature.clone());
                } else if ports[i] == Port::new(HAS_NULL_PORT) {
                    has_null.push(feature.clone());
                }
            }
        }

        (mapped, has_null)
    }

    #[test]
    fn test_explicit_null_replaced() {
        let feature = create_test_feature(vec![
            ("name", Some(AttributeValue::String("test".to_string()))),
            ("value", Some(AttributeValue::Null)),
        ]);

        let params = NullAttributeMapperParams {
            mappings: vec![AttributeMapping {
                attribute: "value".to_string(),
                replacement: Some(Value::Number(Number::from(0))),
                on_missing: OnMissing::Skip,
            }],
            null_definition: vec![NullKind::Null],
            ..Default::default()
        };

        let mut processor = NullAttributeMapper { params };
        let (mapped, has_null) = run_processor(&mut processor, &feature);

        assert_eq!(mapped.len(), 1);
        assert_eq!(has_null.len(), 0);
        assert_eq!(
            mapped[0].get("value"),
            Some(&AttributeValue::Number(Number::from(0)))
        );
        assert_eq!(
            mapped[0].get("name"),
            Some(&AttributeValue::String("test".to_string()))
        );
    }

    #[test]
    fn test_missing_attribute_created() {
        let feature = create_test_feature(vec![(
            "name",
            Some(AttributeValue::String("test".to_string())),
        )]);

        let params = NullAttributeMapperParams {
            mappings: vec![AttributeMapping {
                attribute: "value".to_string(),
                replacement: Some(Value::Number(Number::from(42))),
                on_missing: OnMissing::Create,
            }],
            null_definition: vec![NullKind::Missing],
            ..Default::default()
        };

        let mut processor = NullAttributeMapper { params };
        let (mapped, _) = run_processor(&mut processor, &feature);

        assert_eq!(mapped.len(), 1);
        assert_eq!(
            mapped[0].get("value"),
            Some(&AttributeValue::Number(Number::from(42)))
        );
    }

    #[test]
    fn test_empty_string_replaced() {
        let feature = create_test_feature(vec![
            ("name", Some(AttributeValue::String("".to_string()))),
            ("desc", Some(AttributeValue::String("test".to_string()))),
        ]);

        let params = NullAttributeMapperParams {
            mappings: vec![AttributeMapping {
                attribute: "name".to_string(),
                replacement: Some(Value::String("default".to_string())),
                on_missing: OnMissing::Skip,
            }],
            null_definition: vec![NullKind::EmptyString],
            ..Default::default()
        };

        let mut processor = NullAttributeMapper { params };
        let (mapped, _) = run_processor(&mut processor, &feature);

        assert_eq!(mapped.len(), 1);
        assert_eq!(
            mapped[0].get("name"),
            Some(&AttributeValue::String("default".to_string()))
        );
        assert_eq!(
            mapped[0].get("desc"),
            Some(&AttributeValue::String("test".to_string()))
        );
    }

    #[test]
    fn test_empty_string_not_replaced() {
        let feature =
            create_test_feature(vec![("name", Some(AttributeValue::String("".to_string())))]);

        let params = NullAttributeMapperParams {
            mappings: vec![AttributeMapping {
                attribute: "name".to_string(),
                replacement: Some(Value::String("default".to_string())),
                on_missing: OnMissing::Skip,
            }],
            // EmptyString NOT in null_definition
            null_definition: vec![NullKind::Null, NullKind::Missing],
            ..Default::default()
        };

        let mut processor = NullAttributeMapper { params };
        let (mapped, _) = run_processor(&mut processor, &feature);

        assert_eq!(mapped.len(), 1);
        // Should remain empty string
        assert_eq!(
            mapped[0].get("name"),
            Some(&AttributeValue::String("".to_string()))
        );
    }

    #[test]
    fn test_attribute_removed() {
        let feature = create_test_feature(vec![
            ("name", Some(AttributeValue::String("test".to_string()))),
            ("value", Some(AttributeValue::Null)),
        ]);

        let params = NullAttributeMapperParams {
            mappings: vec![AttributeMapping {
                attribute: "value".to_string(),
                replacement: None, // null means remove
                on_missing: OnMissing::Skip,
            }],
            null_definition: vec![NullKind::Null],
            ..Default::default()
        };

        let mut processor = NullAttributeMapper { params };
        let (mapped, _) = run_processor(&mut processor, &feature);

        assert_eq!(mapped.len(), 1);
        // Attribute should be removed
        assert_eq!(mapped[0].get("value"), None);
        assert_eq!(
            mapped[0].get("name"),
            Some(&AttributeValue::String("test".to_string()))
        );
    }

    #[test]
    fn test_route_null_features_emits_to_hasnull() {
        let feature = create_test_feature(vec![
            ("name", Some(AttributeValue::String("test".to_string()))),
            ("value", Some(AttributeValue::Null)),
        ]);

        let params = NullAttributeMapperParams {
            mappings: vec![AttributeMapping {
                attribute: "value".to_string(),
                replacement: Some(Value::Number(Number::from(0))),
                on_missing: OnMissing::Skip,
            }],
            null_definition: vec![NullKind::Null],
            route_null_features: true,
            ..Default::default()
        };

        let mut processor = NullAttributeMapper { params };
        let (mapped, has_null) = run_processor(&mut processor, &feature);

        // Both ports should receive a feature
        assert_eq!(mapped.len(), 1);
        assert_eq!(has_null.len(), 1);

        // hasNull should have original (pre-replacement) feature
        assert_eq!(has_null[0].get("value"), Some(&AttributeValue::Null));

        // mapped should have modified feature
        assert_eq!(
            mapped[0].get("value"),
            Some(&AttributeValue::Number(Number::from(0)))
        );
    }

    #[test]
    fn test_route_null_features_no_nulls() {
        let feature = create_test_feature(vec![
            ("name", Some(AttributeValue::String("test".to_string()))),
            ("value", Some(AttributeValue::Number(Number::from(42)))),
        ]);

        let params = NullAttributeMapperParams {
            mappings: vec![AttributeMapping {
                attribute: "value".to_string(),
                replacement: Some(Value::Number(Number::from(0))),
                on_missing: OnMissing::Skip,
            }],
            null_definition: vec![NullKind::Null],
            route_null_features: true,
            ..Default::default()
        };

        let mut processor = NullAttributeMapper { params };
        let (mapped, has_null) = run_processor(&mut processor, &feature);

        // Only mapped should receive feature (no nulls to route)
        assert_eq!(mapped.len(), 1);
        assert_eq!(has_null.len(), 0);
    }

    #[test]
    fn test_scope_all_with_default() {
        let feature = create_test_feature(vec![
            ("a", Some(AttributeValue::Null)),
            ("b", Some(AttributeValue::Null)),
            ("c", Some(AttributeValue::String("keep".to_string()))),
        ]);

        let params = NullAttributeMapperParams {
            mappings: vec![], // No per-attribute mappings
            default_replacement: Some(Value::String("replaced".to_string())),
            null_definition: vec![NullKind::Null],
            scope: Scope::All,
            ..Default::default()
        };

        let mut processor = NullAttributeMapper { params };
        let (mapped, _) = run_processor(&mut processor, &feature);

        assert_eq!(mapped.len(), 1);
        assert_eq!(
            mapped[0].get("a"),
            Some(&AttributeValue::String("replaced".to_string()))
        );
        assert_eq!(
            mapped[0].get("b"),
            Some(&AttributeValue::String("replaced".to_string()))
        );
        assert_eq!(
            mapped[0].get("c"),
            Some(&AttributeValue::String("keep".to_string()))
        );
    }

    #[test]
    fn test_per_attribute_override() {
        let feature = create_test_feature(vec![
            ("a", Some(AttributeValue::Null)),
            ("b", Some(AttributeValue::Null)),
        ]);

        let params = NullAttributeMapperParams {
            mappings: vec![AttributeMapping {
                attribute: "a".to_string(),
                replacement: Some(Value::String("special".to_string())),
                on_missing: OnMissing::Skip,
            }],
            default_replacement: Some(Value::String("default".to_string())),
            null_definition: vec![NullKind::Null],
            scope: Scope::All,
            ..Default::default()
        };

        let mut processor = NullAttributeMapper { params };
        let (mapped, _) = run_processor(&mut processor, &feature);

        assert_eq!(mapped.len(), 1);
        // Per-attribute mapping takes precedence
        assert_eq!(
            mapped[0].get("a"),
            Some(&AttributeValue::String("special".to_string()))
        );
        // Uses default
        assert_eq!(
            mapped[0].get("b"),
            Some(&AttributeValue::String("default".to_string()))
        );
    }

    #[test]
    fn test_no_nulls_passthrough() {
        let feature = create_test_feature(vec![
            ("name", Some(AttributeValue::String("test".to_string()))),
            ("value", Some(AttributeValue::Number(Number::from(42)))),
        ]);

        let params = NullAttributeMapperParams {
            mappings: vec![AttributeMapping {
                attribute: "value".to_string(),
                replacement: Some(Value::Number(Number::from(0))),
                on_missing: OnMissing::Skip,
            }],
            null_definition: vec![NullKind::Null],
            ..Default::default()
        };

        let mut processor = NullAttributeMapper { params };
        let (mapped, _) = run_processor(&mut processor, &feature);

        assert_eq!(mapped.len(), 1);
        // Feature should pass through unchanged
        assert_eq!(
            mapped[0].get("name"),
            Some(&AttributeValue::String("test".to_string()))
        );
        assert_eq!(
            mapped[0].get("value"),
            Some(&AttributeValue::Number(Number::from(42)))
        );
    }

    #[test]
    fn test_on_missing_skip() {
        let feature = create_test_feature(vec![(
            "name",
            Some(AttributeValue::String("test".to_string())),
        )]);

        let params = NullAttributeMapperParams {
            mappings: vec![AttributeMapping {
                attribute: "value".to_string(),
                replacement: Some(Value::Number(Number::from(42))),
                on_missing: OnMissing::Skip, // Should NOT create
            }],
            null_definition: vec![NullKind::Null], // Missing is NOT null
            ..Default::default()
        };

        let mut processor = NullAttributeMapper { params };
        let (mapped, _) = run_processor(&mut processor, &feature);

        assert_eq!(mapped.len(), 1);
        // Attribute should NOT be created
        assert_eq!(mapped[0].get("value"), None);
    }

    #[test]
    fn test_null_definition_validation() {
        // Test that empty null_definition fails at build time
        let params = NullAttributeMapperParams {
            null_definition: vec![], // Empty - should error
            ..Default::default()
        };

        let factory = NullAttributeMapperFactory;
        let result = factory.build(
            NodeContext::default(),
            EventHub::new(30),
            "NullAttributeMapper".to_string(),
            Some(serde_json::from_str(&serde_json::to_string(&params).unwrap()).unwrap()),
        );

        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("nullDefinition must include at least one value"));
    }
}
