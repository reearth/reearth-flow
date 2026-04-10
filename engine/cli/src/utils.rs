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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub(crate) struct PropertyI18n {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct I18nSchema {
    pub(crate) name: String,
    pub(crate) description: String,
    /// Legacy JSON Merge Patch for parameters (unused, kept for compatibility).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parameter: Option<serde_json::Value>,
    /// Flat map of top-level parameter property names to their i18n overrides.
    /// Use the empty string key `""` to override the root parameter object's title/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parameter_i18n: Option<HashMap<String, PropertyI18n>>,
    /// Map of definition name → property name → i18n overrides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) definition_i18n: Option<HashMap<String, HashMap<String, PropertyI18n>>>,
}

/// Stamps translated `title` / `description` values onto a JSON Schema node.
fn patch_node(node: &mut serde_json::Value, i18n: &PropertyI18n) {
    if let Some(title) = &i18n.title {
        if !title.is_empty() {
            node["title"] = serde_json::Value::String(title.clone());
        }
    }
    if let Some(desc) = &i18n.description {
        if !desc.is_empty() {
            node["description"] = serde_json::Value::String(desc.clone());
        }
    }
}

/// Applies flat property-path i18n overrides to a parameter JSON Schema.
///
/// - `param_i18n` keys map to `schema["properties"][key]`.
///   The special key `""` targets the root schema object itself.
/// - `def_i18n` keys map to `schema["definitions"][def_name]["properties"][prop_name]`.
///
/// Missing keys are silently skipped — if a property was renamed or removed in
/// the Rust struct the i18n entry simply has no effect.
pub(crate) fn apply_parameter_i18n(
    schema: &mut serde_json::Value,
    param_i18n: &HashMap<String, PropertyI18n>,
    def_i18n: &HashMap<String, HashMap<String, PropertyI18n>>,
) {
    // "" key → root schema title/description
    if let Some(root_i18n) = param_i18n.get("") {
        patch_node(schema, root_i18n);
    }

    // all other keys → schema["properties"][key]
    if let Some(properties) = schema.get_mut("properties").and_then(|p| p.as_object_mut()) {
        for (key, i18n) in param_i18n.iter().filter(|(k, _)| !k.is_empty()) {
            if let Some(node) = properties.get_mut(key) {
                patch_node(node, i18n);
            }
        }
    }

    // def_i18n → schema["definitions"][def_name]["properties"][prop_name]
    if let Some(definitions) = schema.get_mut("definitions").and_then(|d| d.as_object_mut()) {
        for (def_name, props) in def_i18n {
            if let Some(def_schema) = definitions.get_mut(def_name) {
                if let Some(def_props) = def_schema
                    .get_mut("properties")
                    .and_then(|p| p.as_object_mut())
                {
                    for (prop_name, i18n) in props {
                        if let Some(node) = def_props.get_mut(prop_name) {
                            patch_node(node, i18n);
                        }
                    }
                }
            }
        }
    }
}

/// Applies all i18n overrides from an `I18nSchema` entry to an already-parsed
/// parameter JSON Schema value. Handles both the legacy merge-patch field and
/// the new flat property-path fields, in that order.
fn apply_i18n_to_parameter(parameter_schema: &mut serde_json::Value, i18n_schema: Option<&I18nSchema>) {
    let Some(i18n) = i18n_schema else { return };

    if let Some(patch) = &i18n.parameter {
        reearth_flow_common::json::json_merge_patch(parameter_schema, patch);
    }

    let param_i18n = i18n.parameter_i18n.as_ref();
    let def_i18n = i18n.definition_i18n.as_ref();

    if param_i18n.is_some() || def_i18n.is_some() {
        apply_parameter_i18n(
            parameter_schema,
            param_i18n.unwrap_or(&HashMap::new()),
            def_i18n.unwrap_or(&HashMap::new()),
        );
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
            (
                factory.name().to_string(),
                i18n_schema
                    .map(|schema| schema.description.clone())
                    .unwrap_or(factory.description().to_string()),
                factory
                    .parameter_schema()
                    .map_or(serde_json::Value::Null, |schema| {
                        let mut parameter_schema: serde_json::Value =
                            serde_json::from_str(serde_json::to_string(&schema).unwrap().as_str())
                                .unwrap();
                        apply_i18n_to_parameter(&mut parameter_schema, i18n_schema);
                        parameter_schema
                    }),
                vec![],
                factory
                    .get_output_ports()
                    .iter()
                    .map(|p| p.to_string())
                    .collect(),
                factory.categories().iter().map(|c| c.to_string()).collect(),
            )
        }
        NodeKind::Processor(factory) => {
            let i18n_schema = i18n.get(&factory.name().to_string());
            (
                factory.name().to_string(),
                i18n_schema
                    .map(|schema| schema.description.clone())
                    .unwrap_or(factory.description().to_string()),
                factory
                    .parameter_schema()
                    .map_or(serde_json::Value::Null, |schema| {
                        let mut parameter_schema: serde_json::Value =
                            serde_json::from_str(serde_json::to_string(&schema).unwrap().as_str())
                                .unwrap();
                        apply_i18n_to_parameter(&mut parameter_schema, i18n_schema);
                        parameter_schema
                    }),
                factory
                    .get_input_ports()
                    .iter()
                    .map(|p| p.to_string())
                    .collect(),
                factory
                    .get_output_ports()
                    .iter()
                    .map(|p| p.to_string())
                    .collect(),
                factory.categories().iter().map(|c| c.to_string()).collect(),
            )
        }
        NodeKind::Sink(factory) => {
            let i18n_schema = i18n.get(&factory.name().to_string());
            (
                factory.name().to_string(),
                i18n_schema
                    .map(|schema| schema.description.clone())
                    .unwrap_or(factory.description().to_string()),
                factory
                    .parameter_schema()
                    .map_or(serde_json::Value::Null, |schema| {
                        let mut parameter_schema: serde_json::Value =
                            serde_json::from_str(serde_json::to_string(&schema).unwrap().as_str())
                                .unwrap();
                        apply_i18n_to_parameter(&mut parameter_schema, i18n_schema);
                        parameter_schema
                    }),
                factory
                    .get_input_ports()
                    .iter()
                    .map(|p| p.to_string())
                    .collect(),
                vec![],
                factory.categories().iter().map(|c| c.to_string()).collect(),
            )
        }
    };

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
