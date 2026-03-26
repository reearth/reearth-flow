use std::collections::HashMap;

use reearth_flow_runtime::node::NodeKind;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActionSchema {
    pub name: String,
    pub r#type: String,
    pub description: String,
    pub parameter: serde_json::Value,
    pub builtin: bool,
    pub input_ports: Vec<String>,
    pub output_ports: Vec<String>,
    pub categories: Vec<String>,
}

impl ActionSchema {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        r#type: String,
        description: String,
        parameter: serde_json::Value,
        builtin: bool,
        input_ports: Vec<String>,
        output_ports: Vec<String>,
        categories: Vec<String>,
    ) -> Self {
        Self {
            name,
            r#type,
            description,
            parameter,
            builtin,
            input_ports,
            output_ports,
            categories,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct I18nSchema {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) parameter: Option<serde_json::Value>,
}

/// Extracts top-level default values from a JSON schema's properties.
/// Fields with an explicit `"default"` key are included directly.
/// Nullable fields (anyOf with null) without an explicit default get `null`.
fn extract_defaults_from_schema(schema: &serde_json::Value) -> HashMap<String, serde_json::Value> {
    let mut defaults = HashMap::new();
    if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
        for (key, prop_schema) in props {
            if let Some(default) = prop_schema.get("default") {
                defaults.insert(key.clone(), default.clone());
            } else if is_nullable(prop_schema) {
                defaults.insert(key.clone(), serde_json::Value::Null);
            }
        }
    }
    defaults
}

/// Returns true if the schema represents a nullable type.
/// Checks for: anyOf/oneOf containing {type: "null"}, or type array containing "null".
fn is_nullable(schema: &serde_json::Value) -> bool {
    // Check anyOf/oneOf containing {type: "null"}
    for key in &["anyOf", "oneOf"] {
        if let Some(variants) = schema.get(key).and_then(|v| v.as_array()) {
            if variants
                .iter()
                .any(|v| v.get("type").and_then(|t| t.as_str()) == Some("null"))
            {
                return true;
            }
        }
    }
    // Check type array containing "null" (e.g., "type": ["string", "null"])
    if let Some(types) = schema.get("type").and_then(|t| t.as_array()) {
        if types.iter().any(|t| t.as_str() == Some("null")) {
            return true;
        }
    }
    false
}

/// Returns true if the schema has no type constraints (e.g., serde_json::Value).
/// These are implicitly any-type and can be treated as having a null default.
fn is_unconstrained(schema: &serde_json::Value) -> bool {
    !schema.as_object().is_some_and(|obj| {
        obj.contains_key("type")
            || obj.contains_key("anyOf")
            || obj.contains_key("oneOf")
            || obj.contains_key("$ref")
            || obj.contains_key("allOf")
    })
}

/// Validates that all top-level properties in an action's schema have defaults.
/// Panics if any property is missing both an explicit `"default"` and nullable/unconstrained status.
/// Called during schema generation (which runs in CI) to catch missing `#[serde(default)]`.
pub(crate) fn validate_schema_defaults(action_name: &str, schema: &serde_json::Value) {
    if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
        for (key, prop_schema) in props {
            if prop_schema.get("default").is_none()
                && !is_nullable(prop_schema)
                && !is_unconstrained(prop_schema)
            {
                panic!(
                    "Action '{}': property '{}' is missing #[serde(default)] — \
                     all param fields must have defaults for port computation",
                    action_name, key
                );
            }
        }
    }
}

pub(crate) fn create_action_schema(
    kind: &NodeKind,
    builtin: bool,
    i18n: &HashMap<String, I18nSchema>,
) -> ActionSchema {
    let (name, description, parameter, input_ports, output_ports, categories) = match kind {
        NodeKind::Source(factory) => {
            let i18n_schema = i18n.get(&factory.name().to_string());
            let parameter_schema =
                factory
                    .parameter_schema()
                    .map_or(serde_json::Value::Null, |schema| {
                        let mut parameter_schema: serde_json::Value =
                            serde_json::from_str(serde_json::to_string(&schema).unwrap().as_str())
                                .unwrap();
                        if let Some(parameter) =
                            i18n_schema.and_then(|schema| schema.parameter.clone())
                        {
                            reearth_flow_common::json::json_merge_patch(
                                &mut parameter_schema,
                                &parameter,
                            );
                        }
                        parameter_schema
                    });
            let default_with = extract_defaults_from_schema(&parameter_schema);
            (
                factory.name().to_string(),
                i18n_schema
                    .map(|schema| schema.description.clone())
                    .unwrap_or(factory.description().to_string()),
                parameter_schema,
                vec![],
                factory
                    .get_output_ports(&default_with)
                    .iter()
                    .map(|p| p.to_string())
                    .collect(),
                factory.categories().iter().map(|c| c.to_string()).collect(),
            )
        }
        NodeKind::Processor(factory) => {
            let i18n_schema = i18n.get(&factory.name().to_string());
            let parameter_schema =
                factory
                    .parameter_schema()
                    .map_or(serde_json::Value::Null, |schema| {
                        let mut parameter_schema: serde_json::Value =
                            serde_json::from_str(serde_json::to_string(&schema).unwrap().as_str())
                                .unwrap();
                        if let Some(parameter) =
                            i18n_schema.and_then(|schema| schema.parameter.clone())
                        {
                            reearth_flow_common::json::json_merge_patch(
                                &mut parameter_schema,
                                &parameter,
                            );
                        }
                        parameter_schema
                    });
            let default_with = extract_defaults_from_schema(&parameter_schema);
            (
                factory.name().to_string(),
                i18n_schema
                    .map(|schema| schema.description.clone())
                    .unwrap_or(factory.description().to_string()),
                parameter_schema,
                factory
                    .get_input_ports(&default_with)
                    .iter()
                    .map(|p| p.to_string())
                    .collect(),
                factory
                    .get_output_ports(&default_with)
                    .iter()
                    .map(|p| p.to_string())
                    .collect(),
                factory.categories().iter().map(|c| c.to_string()).collect(),
            )
        }
        NodeKind::Sink(factory) => {
            let i18n_schema = i18n.get(&factory.name().to_string());
            let parameter_schema =
                factory
                    .parameter_schema()
                    .map_or(serde_json::Value::Null, |schema| {
                        let mut parameter_schema: serde_json::Value =
                            serde_json::from_str(serde_json::to_string(&schema).unwrap().as_str())
                                .unwrap();
                        if let Some(parameter) =
                            i18n_schema.and_then(|schema| schema.parameter.clone())
                        {
                            reearth_flow_common::json::json_merge_patch(
                                &mut parameter_schema,
                                &parameter,
                            );
                        }
                        parameter_schema
                    });
            let default_with = extract_defaults_from_schema(&parameter_schema);
            (
                factory.name().to_string(),
                i18n_schema
                    .map(|schema| schema.description.clone())
                    .unwrap_or(factory.description().to_string()),
                parameter_schema,
                factory
                    .get_input_ports(&default_with)
                    .iter()
                    .map(|p| p.to_string())
                    .collect(),
                vec![],
                factory.categories().iter().map(|c| c.to_string()).collect(),
            )
        }
    };

    validate_schema_defaults(&name, &parameter);

    ActionSchema::new(
        name,
        match kind {
            NodeKind::Source(_) => "source".to_string(),
            NodeKind::Processor(_) => "processor".to_string(),
            NodeKind::Sink(_) => "sink".to_string(),
        },
        description,
        parameter,
        builtin,
        input_ports,
        output_ports,
        categories,
    )
}
