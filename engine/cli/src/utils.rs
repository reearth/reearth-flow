use std::collections::{BTreeMap, HashMap};

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
    pub tags: Vec<String>,
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
        tags: Vec<String>,
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
            tags,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub(crate) struct PropertyI18n {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    /// Overrides for the sub-parameters a `oneOf` variant carries. Without this
    /// the variant's own label translates but the fields belonging to it do not,
    /// which is most of the surface on any mode that takes settings of its own.
    /// Keys are alphabetical (BTreeMap) for stable diffs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<BTreeMap<String, PropertyI18n>>,
}

fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(value.and_then(|s| if s.is_empty() { None } else { Some(s) }))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct I18nSchema {
    pub(crate) name: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "empty_string_as_none"
    )]
    pub(crate) description: Option<String>,
    /// Legacy JSON Merge Patch for parameters (unused, kept for compatibility).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parameter: Option<serde_json::Value>,
    /// Flat map of top-level parameter property names to their i18n overrides.
    /// Use the empty string key `""` to override the root parameter object's title/description.
    /// Keys are always stored in alphabetical order (BTreeMap) for stable diffs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parameter_i18n: Option<BTreeMap<String, PropertyI18n>>,
    /// Map of definition name → property name → i18n overrides.
    /// Both levels are always stored in alphabetical order for stable diffs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) definition_i18n: Option<BTreeMap<String, BTreeMap<String, PropertyI18n>>>,
    /// Map of definition name → enum value → i18n overrides for oneOf/anyOf enum variants.
    /// Keyed by the variant's enum value (e.g. "max", "min") rather than array index,
    /// so adding new variants never invalidates existing translations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) enum_i18n: Option<BTreeMap<String, BTreeMap<String, PropertyI18n>>>,
}

/// Stamps translated `title` / `description` values onto a JSON Schema node.
fn patch_text(node: &mut serde_json::Value, i18n: &PropertyI18n) {
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

/// Stamps overrides onto the children of a node's own `properties`.
fn patch_properties(node: &mut serde_json::Value, nested: &BTreeMap<String, PropertyI18n>) {
    if let Some(properties) = node.get_mut("properties").and_then(|p| p.as_object_mut()) {
        for (name, child) in nested {
            if let Some(target) = properties.get_mut(name) {
                patch_node(target, child);
            }
        }
    }
}

/// Stamps translated `title` / `description` values onto a JSON Schema node, and
/// recurses into the node's own `properties` for any nested overrides.
fn patch_node(node: &mut serde_json::Value, i18n: &PropertyI18n) {
    patch_text(node, i18n);
    if let Some(nested) = &i18n.properties {
        patch_properties(node, nested);
    }
}

/// Stamps the overrides of one `oneOf`/`anyOf` variant keyed `key`: its own text
/// onto the variant, its sub-parameter overrides onto whichever node carries them
/// (see [`variant_sub_parameters`]).
fn patch_variant(variant: &mut serde_json::Value, key: &str, i18n: &PropertyI18n) {
    patch_text(variant, i18n);
    let Some(nested) = &i18n.properties else {
        return;
    };
    if variant_sub_parameters(variant, key).is_some() {
        if let Some(payload) = variant.get_mut("properties").and_then(|p| p.get_mut(key)) {
            patch_properties(payload, nested);
        }
    } else {
        patch_properties(variant, nested);
    }
}

/// The node whose `properties` are a variant's sub-parameters, when the variant is
/// externally tagged and so carries them one level down, inside the object named
/// for the variant itself. `None` for every other shape, which carries them
/// directly (see [`enum_variant_key`]).
///
/// A payload counts only when at least one of its properties carries user-facing
/// text, which is the same condition the scaffolder seeds an entry on. A newtype
/// variant wrapping a struct inlines that struct's own fields here, and those are
/// not the variant's sub-parameters.
pub(crate) fn variant_sub_parameters<'a>(
    variant: &'a serde_json::Value,
    key: &str,
) -> Option<&'a serde_json::Value> {
    let payload = variant.get("properties")?.get(key)?;
    let properties = payload.get("properties")?.as_object()?;
    properties
        .values()
        .any(|schema| schema.get("title").is_some() || schema.get("description").is_some())
        .then_some(payload)
}

/// The key identifying one `oneOf`/`anyOf` variant of an enum definition.
///
/// schemars renders a variant three different ways depending on how the Rust
/// enum is tagged, and only the first carries a top-level `enum`:
///
/// - a unit variant of an untagged or externally-tagged enum — `{"enum": ["csv"]}`
/// - an internally-tagged variant — `{"properties": {"type": {"enum": ["csv"]}}}`,
///   with the tag named in `required`
/// - an externally-tagged variant carrying fields — `{"required": ["csv"],
///   "properties": {"csv": {...}}}`
///
/// Keying only off the first shape is why every variant that carries
/// sub-parameters used to be invisible to both the scaffold and the applier, so
/// the labels a user picks between stayed English however the enum was written.
pub(crate) fn enum_variant_key(variant: &serde_json::Value) -> Option<String> {
    if let Some(value) = variant
        .get("enum")
        .and_then(|e| e.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
    {
        return Some(value.to_string());
    }

    let properties = variant.get("properties").and_then(|p| p.as_object())?;
    let required = variant.get("required").and_then(|r| r.as_array())?;

    // Internally tagged: exactly one required property pins a single-valued
    // string enum, and that value is the variant's name.
    let tagged = required.iter().filter_map(|r| r.as_str()).find_map(|name| {
        let values = properties.get(name)?.get("enum")?.as_array()?;
        match values.as_slice() {
            [serde_json::Value::String(value)] => Some(value.clone()),
            _ => None,
        }
    });
    if tagged.is_some() {
        return tagged;
    }

    // Externally tagged: the variant is an object holding exactly one required
    // property, named for the variant itself.
    match required.as_slice() {
        [serde_json::Value::String(name)] if properties.contains_key(name) => Some(name.clone()),
        _ => None,
    }
}

/// Applies flat property-path i18n overrides to a parameter JSON Schema.
///
/// - `param_i18n` keys map to `schema["properties"][key]`.
///   The special key `""` targets the root schema object itself.
/// - `def_i18n` keys map to `schema["definitions"][def_name]["properties"][prop_name]`.
/// - `enum_i18n` keys map to `schema["definitions"][def_name]["oneOf"|"anyOf"]` variants,
///   looked up by the variant's key (see [`enum_variant_key`]).
///
/// Missing keys are silently skipped — if a property was renamed or removed in
/// the Rust struct the i18n entry simply has no effect.
pub(crate) fn apply_parameter_i18n(
    schema: &mut serde_json::Value,
    param_i18n: &BTreeMap<String, PropertyI18n>,
    def_i18n: &BTreeMap<String, BTreeMap<String, PropertyI18n>>,
    enum_i18n: &BTreeMap<String, BTreeMap<String, PropertyI18n>>,
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

    if let Some(definitions) = schema
        .get_mut("definitions")
        .and_then(|d| d.as_object_mut())
    {
        // def_i18n → schema["definitions"][def_name]["properties"][prop_name]
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

        // enum_i18n → schema["definitions"][def_name]["oneOf"|"anyOf"][variant with enum value]
        for (def_name, variants) in enum_i18n {
            if let Some(def_schema) = definitions.get_mut(def_name) {
                for keyword in &["oneOf", "anyOf"] {
                    if let Some(arr) = def_schema.get_mut(*keyword).and_then(|v| v.as_array_mut()) {
                        for variant in arr.iter_mut() {
                            let enum_val = enum_variant_key(variant);
                            if let Some(val) = enum_val {
                                if let Some(i18n) = variants.get(&val) {
                                    patch_variant(variant, &val, i18n);
                                }
                            }
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
fn apply_i18n_to_parameter(
    parameter_schema: &mut serde_json::Value,
    i18n_schema: Option<&I18nSchema>,
) {
    let Some(i18n) = i18n_schema else { return };

    if let Some(patch) = &i18n.parameter {
        reearth_flow_common::json::json_merge_patch(parameter_schema, patch);
    }

    let param_i18n = i18n.parameter_i18n.as_ref();
    let def_i18n = i18n.definition_i18n.as_ref();
    let enum_i18n = i18n.enum_i18n.as_ref();

    if param_i18n.is_some() || def_i18n.is_some() || enum_i18n.is_some() {
        let empty_param = BTreeMap::new();
        let empty_def = BTreeMap::new();
        let empty_enum = BTreeMap::new();
        apply_parameter_i18n(
            parameter_schema,
            param_i18n.unwrap_or(&empty_param),
            def_i18n.unwrap_or(&empty_def),
            enum_i18n.unwrap_or(&empty_enum),
        );
    }
}

pub(crate) fn create_action_schema(
    kind: &NodeKind,
    builtin: bool,
    i18n: &HashMap<String, I18nSchema>,
) -> ActionSchema {
    let (name, description, parameter, input_ports, output_ports, categories, tags) = match kind {
        NodeKind::Source(factory) => {
            let i18n_schema = i18n.get(&factory.name().to_string());
            (
                factory.name().to_string(),
                i18n_schema
                    .and_then(|schema| schema.description.clone())
                    .unwrap_or_else(|| factory.description().to_string()),
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
                factory.tags().iter().map(|t| t.to_string()).collect(),
            )
        }
        NodeKind::Processor(factory) => {
            let i18n_schema = i18n.get(&factory.name().to_string());
            (
                factory.name().to_string(),
                i18n_schema
                    .and_then(|schema| schema.description.clone())
                    .unwrap_or_else(|| factory.description().to_string()),
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
                factory.tags().iter().map(|t| t.to_string()).collect(),
            )
        }
        NodeKind::Sink(factory) => {
            let i18n_schema = i18n.get(&factory.name().to_string());
            (
                factory.name().to_string(),
                i18n_schema
                    .and_then(|schema| schema.description.clone())
                    .unwrap_or_else(|| factory.description().to_string()),
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
                factory.tags().iter().map(|t| t.to_string()).collect(),
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
        tags,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An externally tagged struct variant, as schemars renders
    /// `Enabled { quantization_error }`.
    fn struct_variant() -> serde_json::Value {
        serde_json::json!({
            "title": "Enabled",
            "type": "object",
            "required": ["enabled"],
            "properties": {
                "enabled": {
                    "type": "object",
                    "properties": {
                        "quantizationError": {
                            "title": "Quantization Error",
                            "description": "Upper bound in meters.",
                            "type": ["number", "null"]
                        }
                    },
                    "additionalProperties": false
                }
            },
            "additionalProperties": false
        })
    }

    /// An externally tagged newtype variant, as schemars renders `Crs(Code)`:
    /// the wrapped type's own fields are inlined into the payload.
    fn newtype_variant() -> serde_json::Value {
        serde_json::json!({
            "title": "CRS",
            "type": "object",
            "required": ["crs"],
            "properties": {
                "crs": {
                    "type": "object",
                    "format": "code",
                    "required": ["type", "value"],
                    "properties": {
                        "type": {"type": "string", "enum": ["flowExpr"]},
                        "value": {"type": "string"}
                    }
                }
            },
            "additionalProperties": false
        })
    }

    #[test]
    fn struct_variant_carries_its_sub_parameters_one_level_down() {
        let variant = struct_variant();
        let payload = variant_sub_parameters(&variant, "enabled").expect("payload");
        assert_eq!(payload, &variant["properties"]["enabled"]);
    }

    #[test]
    fn newtype_variant_has_no_sub_parameters() {
        assert!(variant_sub_parameters(&newtype_variant(), "crs").is_none());
    }

    #[test]
    fn sub_parameter_overrides_land_inside_the_payload() {
        let mut variant = struct_variant();
        let i18n = PropertyI18n {
            title: Some("有効".to_string()),
            description: Some("Draco で圧縮する。".to_string()),
            properties: Some(BTreeMap::from([(
                "quantizationError".to_string(),
                PropertyI18n {
                    title: Some("量子化誤差".to_string()),
                    description: None,
                    properties: None,
                },
            )])),
        };

        patch_variant(&mut variant, "enabled", &i18n);

        assert_eq!(variant["title"], "有効");
        assert_eq!(
            variant["properties"]["enabled"]["properties"]["quantizationError"]["title"],
            "量子化誤差"
        );
    }

    #[test]
    fn a_newtype_payloads_internals_are_left_alone() {
        let mut variant = newtype_variant();
        let i18n = PropertyI18n {
            title: Some("CRS".to_string()),
            description: None,
            properties: Some(BTreeMap::from([(
                "value".to_string(),
                PropertyI18n {
                    title: Some("should not be applied".to_string()),
                    description: None,
                    properties: None,
                },
            )])),
        };

        patch_variant(&mut variant, "crs", &i18n);

        assert!(variant["properties"]["crs"]["properties"]["value"]
            .get("title")
            .is_none());
    }
}
