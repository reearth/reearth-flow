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
pub(super) struct AttributeTableExtractorFactory;

impl ProcessorFactory for AttributeTableExtractorFactory {
    fn name(&self) -> &str {
        "Attribute Table Extractor"
    }

    fn description(&self) -> &str {
        "Moves values between nested map/list attribute paths, following a table of source/destination path pairs keyed by a feature type attribute. A destination path with more than one segment creates nested maps as needed."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AttributeTableExtractorParam))
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
        let params: AttributeTableExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::TableExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::TableExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(AttributeProcessorError::TableExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let table: HashMap<String, Vec<ExtractRule>> = if let Some(dataset) = &params.dataset {
            let storage_resolver = &ctx.storage_resolver;
            let input_path = dataset
                .compile()
                .map_err(|e| {
                    AttributeProcessorError::TableExtractorFactory(format!(
                        "Failed to compile dataset expression: {e:?}"
                    ))
                })?
                .eval_string_env_only(ctx.env_vars.clone())
                .map_err(|e| {
                    AttributeProcessorError::TableExtractorFactory(format!(
                        "Failed to evaluate dataset expression: {e}"
                    ))
                })?;
            let input_path = Uri::from_str(input_path.as_str())
                .map_err(|e| AttributeProcessorError::TableExtractorFactory(format!("{e:?}")))?;
            let storage = storage_resolver
                .resolve(&input_path)
                .map_err(|e| AttributeProcessorError::TableExtractorFactory(format!("{e:?}")))?;
            let bytes: Bytes = storage
                .get_sync(input_path.path().as_path())
                .map_err(|e| AttributeProcessorError::TableExtractorFactory(format!("{e:?}")))?;
            serde_json::from_slice(&bytes).map_err(|e| {
                AttributeProcessorError::TableExtractorFactory(format!(
                    "Failed to parse extraction table: {e}"
                ))
            })?
        } else if let Some(inline) = params.inline.clone() {
            serde_json::from_value(inline).map_err(|e| {
                AttributeProcessorError::TableExtractorFactory(format!(
                    "Failed to parse extraction table: {e}"
                ))
            })?
        } else {
            return Err(AttributeProcessorError::TableExtractorFactory(
                "Missing required parameter `dataset` or `inline`".to_string(),
            )
            .into());
        };

        let process = AttributeTableExtractor {
            type_attribute: params
                .type_attribute
                .unwrap_or_else(|| "__citygml_feature_type".to_string()),
            table,
        };
        Ok(Box::new(process))
    }
}

/// # Attribute Table Extractor Parameters
/// Configures the table of source/destination path pairs used to move nested attribute values.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AttributeTableExtractorParam {
    /// # Dataset URI
    /// Path or URI of the extraction table file. Provide either this or inline data.
    dataset: Option<Code>,
    /// # Inline Table
    /// Extraction table content provided directly as JSON. Used when no dataset URI is given.
    inline: Option<Value>,
    /// # Feature Type Attribute
    /// Attribute whose value selects which rule set in the table applies to the feature. Defaults to `__citygml_feature_type`.
    type_attribute: Option<String>,
}

/// One extraction rule for a single feature type in the extraction table.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ExtractRule {
    /// Space-separated chain of keys naming where the extracted value is written. A single
    /// segment writes a top-level attribute; multiple segments write into a nested map,
    /// creating it (or any missing intermediate map) as needed.
    attribute: String,
    /// Space-separated chain of nested keys to walk from the feature's top level down to the value.
    json_path: String,
    /// Optional coercion applied to the extracted value before it is written.
    #[serde(default)]
    data_type: Option<ExtractDataType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum ExtractDataType {
    /// # Integer
    /// Parses a string value as an integer.
    Int,
    /// # Float
    /// Parses a string value as a floating point number.
    Float,
}

#[derive(Debug, Clone)]
struct AttributeTableExtractor {
    type_attribute: String,
    table: HashMap<String, Vec<ExtractRule>>,
}

impl Processor for AttributeTableExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        let feature_type = feature
            .get(&self.type_attribute)
            .and_then(|v| v.as_string());

        if let Some(feature_type) = feature_type {
            if let Some(rules) = self.table.get(&feature_type) {
                for rule in rules {
                    let src_segments: Vec<&str> = rule.json_path.split_whitespace().collect();
                    if let Some(value) = resolve_path(&feature, &src_segments) {
                        let value = coerce(value, rule.data_type);
                        let dst_segments: Vec<&str> = rule.attribute.split_whitespace().collect();
                        if !dst_segments.is_empty() {
                            write_path(&mut feature, &dst_segments, value);
                        }
                    }
                }
            }
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
        "Attribute Table Extractor"
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

/// Writes `value` at a space-separated chain of keys under the feature's top level, creating
/// (or replacing, if not already a map) any missing intermediate map along the way.
fn write_path(feature: &mut Feature, segments: &[&str], value: AttributeValue) {
    let (first, rest) = segments
        .split_first()
        .expect("segments is never empty: caller only calls with a non-empty path");
    if rest.is_empty() {
        feature.insert(Attribute::new(*first), value);
        return;
    }
    let mut top = feature
        .get(*first)
        .cloned()
        .unwrap_or_else(|| AttributeValue::Map(HashMap::new()));
    set_nested(&mut top, rest, value);
    feature.insert(Attribute::new(*first), top);
}

fn set_nested(current: &mut AttributeValue, segments: &[&str], value: AttributeValue) {
    let (first, rest) = segments
        .split_first()
        .expect("segments is never empty: caller only recurses while rest is non-empty");
    if !matches!(current, AttributeValue::Map(_)) {
        *current = AttributeValue::Map(HashMap::new());
    }
    let AttributeValue::Map(map) = current else {
        unreachable!()
    };
    if rest.is_empty() {
        map.insert((*first).to_string(), value);
    } else {
        let child = map
            .entry((*first).to_string())
            .or_insert_with(|| AttributeValue::Map(HashMap::new()));
        set_nested(child, rest, value);
    }
}

fn coerce(value: AttributeValue, data_type: Option<ExtractDataType>) -> AttributeValue {
    let AttributeValue::String(s) = &value else {
        return value;
    };
    match data_type {
        Some(ExtractDataType::Int) => s
            .trim()
            .parse::<i64>()
            .map(|n| AttributeValue::Number(n.into()))
            .unwrap_or(value),
        Some(ExtractDataType::Float) => s
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

    #[test]
    fn write_path_creates_nested_map() {
        let mut feature = feature_with(HashMap::new());
        write_path(
            &mut feature,
            &["attributes", "bldg:usage"],
            AttributeValue::String("office".to_string()),
        );
        write_path(
            &mut feature,
            &["attributes", "bldg:class"],
            AttributeValue::String("residential".to_string()),
        );
        let AttributeValue::Map(nested) = feature.get("attributes").unwrap() else {
            panic!("expected a map");
        };
        assert_eq!(
            nested.get("bldg:usage"),
            Some(&AttributeValue::String("office".to_string()))
        );
        assert_eq!(
            nested.get("bldg:class"),
            Some(&AttributeValue::String("residential".to_string()))
        );
    }

    #[test]
    fn write_path_single_segment_writes_top_level() {
        let mut feature = feature_with(HashMap::new());
        write_path(
            &mut feature,
            &["bldg:class"],
            AttributeValue::String("residential".to_string()),
        );
        assert_eq!(
            feature.get("bldg:class"),
            Some(&AttributeValue::String("residential".to_string()))
        );
    }
}
