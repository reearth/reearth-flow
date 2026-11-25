use std::{collections::HashMap, str::FromStr};

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_diagnostics::{DiagnosticDraft, ErrorCode};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{FEATURES_PORT, Port, Processor, ProcessorFactory},
};
use reearth_flow_types::{Attribute, AttributeValue, Attributes, Code, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::AttributeProcessorError;
use super::table_query::Query;

#[derive(Debug, Clone, Default)]
pub(super) struct AttributeTableExtractorFactory;

impl ProcessorFactory for AttributeTableExtractorFactory {
    fn name(&self) -> &str {
        "Attribute Table Extractor"
    }

    fn description(&self) -> &str {
        "Copies values between nested map/list attribute paths, following a table of source/destination path pairs keyed by a feature type attribute. A source path handles lists explicitly with steps such as `[]`, `[*]` and `[n]`. A destination path with more than one segment creates nested maps as needed."
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
                .eval_string_variables_only(ctx.variables.clone())
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
            inline
        } else {
            return Err(AttributeProcessorError::TableExtractorFactory(
                "Missing required parameter `dataset` or `inline`".to_string(),
            )
            .into());
        };

        let table = table
            .into_iter()
            .map(|(feature_type, rules)| {
                let rules = rules
                    .into_iter()
                    .map(CompiledRule::compile)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok((feature_type, rules))
            })
            .collect::<Result<HashMap<_, _>, AttributeProcessorError>>()?;

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
/// Supply the table either as a file, via Dataset URI, or directly, via Inline Table — one of
/// the two is required.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AttributeTableExtractorParam {
    /// # Dataset URI
    /// Path or URI of a JSON file holding the extraction table. Provide either this or Inline
    /// Table.
    dataset: Option<Code>,
    /// # Inline Table
    /// Extraction table given directly, keyed by feature type. Each key is a feature type name
    /// as it appears in the type attribute, and its value is the list of rules applied to
    /// features of that type. Provide either this or Dataset URI.
    inline: Option<HashMap<String, Vec<ExtractRule>>>,
    /// # Feature Type Attribute
    /// Attribute whose value selects which rule set in the table applies to the feature. Defaults to `__citygml_feature_type`.
    type_attribute: Option<String>,
}

/// # Extraction Rule
/// Moves one value from a source path to a destination path.
///
/// Both paths are chains of attribute keys separated by **spaces**, not dots — the keys
/// themselves are qualified XML names such as `bldg:measuredHeight`, which may contain colons
/// and dots but never whitespace.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ExtractRule {
    /// # Destination Path
    /// Where the value is written, as a space-separated chain of keys. A single segment writes a
    /// top-level attribute; several segments write into a nested map, creating it and any
    /// missing intermediate map as needed.
    destination_path: String,
    /// # Source Path
    /// Where the value is read from, as whitespace-separated steps walked from the feature's
    /// top level: `k` takes key `k` of a map, `[]` makes a single value a one-item list, `[*]`
    /// expands a list into its items, `[n]` takes item `n` of a list (negative counts from the
    /// end), and `[k]` gathers the `k` values of all results into one list. The value is
    /// written only when exactly one result remains, e.g.
    /// `__city_gml_attributes bldg:adeOfAbstractBuilding [] [*] [uro:BuildingIDAttribute] [0] uro:buildingID`.
    source_path: String,
    /// # Value Type
    /// Converts the extracted value before writing it. Leave unset to write it unchanged.
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
struct CompiledRule {
    source: Query,
    destination: Vec<String>,
    data_type: Option<ExtractDataType>,
}

impl CompiledRule {
    fn compile(rule: ExtractRule) -> Result<Self, AttributeProcessorError> {
        let source = Query::parse(&rule.source_path)
            .map_err(|e| AttributeProcessorError::TableExtractorFactory(e.to_string()))?;
        let destination: Vec<String> = rule
            .destination_path
            .split_whitespace()
            .map(str::to_string)
            .collect();
        if destination.is_empty() {
            return Err(AttributeProcessorError::TableExtractorFactory(format!(
                "empty destination path for source `{}`",
                rule.source_path
            )));
        }
        Ok(Self {
            source,
            destination,
            data_type: rule.data_type,
        })
    }
}

#[derive(Debug, Clone)]
struct AttributeTableExtractor {
    type_attribute: String,
    table: HashMap<String, Vec<CompiledRule>>,
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

        if let Some(feature_type) = feature_type
            && let Some(rules) = self.table.get(&feature_type)
        {
            for rule in rules {
                if let Some(value) = rule.source.evaluate(&feature.attributes) {
                    let value = coerce(&ctx, value, rule.data_type);
                    write_path(&mut feature, &rule.destination, value);
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

/// Writes `value` at a space-separated chain of keys under the feature's top level, creating
/// (or replacing, if not already a map) any missing intermediate map along the way.
fn write_path(feature: &mut Feature, segments: &[String], value: AttributeValue) {
    let (first, rest) = segments
        .split_first()
        .expect("segments is never empty: caller only calls with a non-empty path");
    if rest.is_empty() {
        feature.insert(Attribute::new(first.as_str()), value);
        return;
    }
    let mut top = feature
        .get(first)
        .cloned()
        .unwrap_or_else(|| AttributeValue::Map(Attributes::new()));
    set_nested(&mut top, rest, value);
    feature.insert(Attribute::new(first.as_str()), top);
}

fn set_nested(current: &mut AttributeValue, segments: &[String], value: AttributeValue) {
    let (first, rest) = segments
        .split_first()
        .expect("segments is never empty: caller only recurses while rest is non-empty");
    if !matches!(current, AttributeValue::Map(_)) {
        *current = AttributeValue::Map(Attributes::new());
    }
    let AttributeValue::Map(map) = current else {
        unreachable!()
    };
    if rest.is_empty() {
        map.insert(Attribute::new(first.as_str()), value);
    } else {
        let child = map
            .entry(Attribute::new(first.as_str()))
            .or_insert_with(|| AttributeValue::Map(Attributes::new()));
        set_nested(child, rest, value);
    }
}

fn coerce(
    ctx: &ExecutorContext,
    value: AttributeValue,
    data_type: Option<ExtractDataType>,
) -> AttributeValue {
    let Some(data_type) = data_type else {
        return value;
    };
    let AttributeValue::String(s) = &value else {
        if !matches!(value, AttributeValue::Number(_)) {
            ctx.warn(
                DiagnosticDraft::new(ErrorCode::TableExtractorNonStringValue)
                    .with_message(format!("data_type={data_type:?}, value={value:?}")),
            );
        }
        return value;
    };
    let coerced = match data_type {
        ExtractDataType::Int => s
            .trim()
            .parse::<i64>()
            .ok()
            .map(|n| AttributeValue::Number(n.into())),
        ExtractDataType::Float => s
            .trim()
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(AttributeValue::Number),
    };
    match coerced {
        Some(v) => v,
        None => {
            ctx.warn(
                DiagnosticDraft::new(ErrorCode::TableExtractorCoerceFailed)
                    .with_message(format!("data_type={data_type:?}, value={s}")),
            );
            value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feature_with(attrs: Attributes) -> Feature {
        Feature::from(attrs)
    }

    fn path(segments: &[&str]) -> Vec<String> {
        segments.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn write_path_creates_nested_map() {
        let mut feature = feature_with(Attributes::new());
        write_path(
            &mut feature,
            &path(&["attributes", "bldg:usage"]),
            AttributeValue::String("office".to_string()),
        );
        write_path(
            &mut feature,
            &path(&["attributes", "bldg:class"]),
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
        let mut feature = feature_with(Attributes::new());
        write_path(
            &mut feature,
            &path(&["bldg:class"]),
            AttributeValue::String("residential".to_string()),
        );
        assert_eq!(
            feature.get("bldg:class"),
            Some(&AttributeValue::String("residential".to_string()))
        );
    }
}
