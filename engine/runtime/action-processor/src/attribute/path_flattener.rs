use std::{collections::HashMap, str::FromStr};

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Code, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::AttributeProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct AttributePathFlattenerFactory;

impl ProcessorFactory for AttributePathFlattenerFactory {
    fn name(&self) -> &str {
        "Attribute Path Flattener"
    }

    fn description(&self) -> &str {
        "Extracts values from nested map or list attributes into new top-level attributes, following a table of paths keyed by a feature type attribute, optionally also writing every resolved value as one JSON summary attribute."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AttributePathFlattenerParam))
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
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: AttributePathFlattenerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::PathFlattenerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::PathFlattenerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(AttributeProcessorError::PathFlattenerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let table: HashMap<String, Vec<FlattenRule>> = if let Some(dataset) = &params.dataset {
            let storage_resolver = &ctx.storage_resolver;
            let input_path = dataset
                .compile()
                .map_err(|e| {
                    AttributeProcessorError::PathFlattenerFactory(format!(
                        "Failed to compile dataset expression: {e:?}"
                    ))
                })?
                .eval_string_env_only(ctx.env_vars.clone())
                .map_err(|e| {
                    AttributeProcessorError::PathFlattenerFactory(format!(
                        "Failed to evaluate dataset expression: {e}"
                    ))
                })?;
            let input_path = Uri::from_str(input_path.as_str()).map_err(|e| {
                AttributeProcessorError::PathFlattenerFactory(format!("{e:?}"))
            })?;
            let storage = storage_resolver.resolve(&input_path).map_err(|e| {
                AttributeProcessorError::PathFlattenerFactory(format!("{e:?}"))
            })?;
            let bytes: Bytes = storage
                .get_sync(input_path.path().as_path())
                .map_err(|e| AttributeProcessorError::PathFlattenerFactory(format!("{e:?}")))?;
            serde_json::from_slice(&bytes).map_err(|e| {
                AttributeProcessorError::PathFlattenerFactory(format!(
                    "Failed to parse flatten table: {e}"
                ))
            })?
        } else if let Some(inline) = params.inline.clone() {
            serde_json::from_value(inline).map_err(|e| {
                AttributeProcessorError::PathFlattenerFactory(format!(
                    "Failed to parse flatten table: {e}"
                ))
            })?
        } else {
            return Err(AttributeProcessorError::PathFlattenerFactory(
                "Missing required parameter `dataset` or `inline`".to_string(),
            )
            .into());
        };

        let process = AttributePathFlattener {
            type_attribute: params
                .type_attribute
                .unwrap_or_else(|| "__citygml_feature_type".to_string()),
            summary_attribute: params.summary_attribute,
            table,
        };
        Ok(Box::new(process))
    }
}

/// # Attribute Path Flattener Parameters
/// Configures the table of paths used to pull nested attribute values to the top level.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AttributePathFlattenerParam {
    /// # Dataset URI
    /// Path or URI of the flatten table file. Provide either this or inline data.
    dataset: Option<Code>,
    /// # Inline Table
    /// Flatten table content provided directly as JSON. Used when no dataset URI is given.
    inline: Option<Value>,
    /// # Feature Type Attribute
    /// Attribute whose value selects which rule set in the table applies to the feature. Defaults to `__citygml_feature_type`.
    type_attribute: Option<String>,
    /// # Summary Attribute
    /// When set, also writes a JSON-serialized object of every attribute/value pair this run resolved to this attribute name.
    summary_attribute: Option<String>,
}

/// One extraction rule for a single feature type in the flatten table.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct FlattenRule {
    /// Name of the top-level attribute the extracted value is written to.
    attribute: String,
    /// Space-separated chain of nested keys to walk from the feature's top level down to the value.
    json_path: String,
    /// Optional coercion applied to the extracted value before it is written.
    #[serde(default)]
    data_type: Option<FlattenDataType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum FlattenDataType {
    /// # Integer
    /// Parses a string value as an integer.
    Int,
    /// # Float
    /// Parses a string value as a floating point number.
    Float,
}

#[derive(Debug, Clone)]
struct AttributePathFlattener {
    type_attribute: String,
    summary_attribute: Option<String>,
    table: HashMap<String, Vec<FlattenRule>>,
}

impl Processor for AttributePathFlattener {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        let feature_type = feature
            .get(&self.type_attribute)
            .and_then(|v| v.as_string());

        let mut resolved: HashMap<String, AttributeValue> = HashMap::new();

        if let Some(feature_type) = feature_type {
            if let Some(rules) = self.table.get(&feature_type) {
                for rule in rules {
                    let segments: Vec<&str> = rule.json_path.split(' ').collect();
                    if let Some(value) = resolve_path(&feature, &segments) {
                        let value = coerce(value, rule.data_type);
                        feature.insert(Attribute::new(rule.attribute.clone()), value.clone());
                        if self.summary_attribute.is_some() {
                            resolved.insert(rule.attribute.clone(), value);
                        }
                    }
                }
            }
        }

        if let Some(summary_attribute) = &self.summary_attribute {
            let json = serde_json::to_string(&serde_json::Value::from(AttributeValue::Map(
                resolved,
            )))
            .unwrap_or_else(|_| "{}".to_string());
            feature.insert(
                Attribute::new(summary_attribute.clone()),
                AttributeValue::String(json),
            );
        }

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
        "Attribute Path Flattener"
    }
}

/// Walks a dot-free, space-separated path of nested keys starting from the
/// feature's top-level attributes. Each segment tolerates the value at that
/// point being either a map (direct key lookup) or a list (searched
/// element-by-element for the first match) — CityGML ADE wrapper elements
/// may appear as either depending on cardinality.
fn resolve_path(feature: &Feature, segments: &[&str]) -> Option<AttributeValue> {
    let (first, rest) = segments.split_first()?;
    let mut current = feature.get(*first)?.clone();
    for segment in rest {
        current = get_from_value(&current, segment)?;
    }
    Some(current)
}

fn get_from_value(value: &AttributeValue, key: &str) -> Option<AttributeValue> {
    match value {
        AttributeValue::Map(map) => map.get(key).cloned(),
        AttributeValue::Array(list) => list.iter().find_map(|item| get_from_value(item, key)),
        _ => None,
    }
}

fn coerce(value: AttributeValue, data_type: Option<FlattenDataType>) -> AttributeValue {
    let AttributeValue::String(s) = &value else {
        return value;
    };
    match data_type {
        Some(FlattenDataType::Int) => s
            .trim()
            .parse::<i64>()
            .map(|n| AttributeValue::Number(n.into()))
            .unwrap_or(value),
        Some(FlattenDataType::Float) => s
            .trim()
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(AttributeValue::Number)
            .unwrap_or(value),
        None => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feature_with(attrs: HashMap<String, AttributeValue>) -> Feature {
        Feature::from(attrs.into_iter().collect::<indexmap::IndexMap<_, _>>())
    }

    #[test]
    fn resolves_through_list_wrapper() {
        let inner = HashMap::from([(
            "uro:BuildingIDAttribute".to_string(),
            AttributeValue::Map(HashMap::from([(
                "uro:city".to_string(),
                AttributeValue::String("Tokyo".to_string()),
            )])),
        )]);
        let feature = feature_with(HashMap::from([(
            "bldg:adeOfAbstractBuilding".to_string(),
            AttributeValue::Array(vec![AttributeValue::Map(inner)]),
        )]));
        let segments = [
            "bldg:adeOfAbstractBuilding",
            "uro:BuildingIDAttribute",
            "uro:city",
        ];
        assert_eq!(
            resolve_path(&feature, &segments),
            Some(AttributeValue::String("Tokyo".to_string()))
        );
    }

    #[test]
    fn resolves_through_direct_map() {
        let inner = HashMap::from([(
            "uro:BuildingIDAttribute".to_string(),
            AttributeValue::Map(HashMap::from([(
                "uro:city".to_string(),
                AttributeValue::String("Osaka".to_string()),
            )])),
        )]);
        let feature = feature_with(HashMap::from([(
            "bldg:adeOfAbstractBuilding".to_string(),
            AttributeValue::Map(inner),
        )]));
        let segments = [
            "bldg:adeOfAbstractBuilding",
            "uro:BuildingIDAttribute",
            "uro:city",
        ];
        assert_eq!(
            resolve_path(&feature, &segments),
            Some(AttributeValue::String("Osaka".to_string()))
        );
    }

    #[test]
    fn missing_path_yields_none() {
        let feature = feature_with(HashMap::new());
        assert_eq!(resolve_path(&feature, &["bldg:class"]), None);
    }
}
