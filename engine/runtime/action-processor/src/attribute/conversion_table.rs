use std::{collections::HashMap, io::Cursor, str::FromStr, sync::Arc};

use bytes::Bytes;
use reearth_flow_common::{csv::Delimiter, uri::Uri};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::AttributeProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct AttributeConversionTableFactory;

impl ProcessorFactory for AttributeConversionTableFactory {
    fn name(&self) -> &str {
        "AttributeConversionTable"
    }

    fn description(&self) -> &str {
        "Converts attributes from conversion table"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AttributeConversionTableParam))
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
        let params: AttributeConversionTableParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::ConversionTableFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::ConversionTableFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(AttributeProcessorError::ConversionTableFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let storage_resolver = &ctx.storage_resolver;
        let scope = expr_engine.new_scope();
        if let Some(with) = with {
            for (k, v) in with.iter() {
                scope.set(k, v.clone());
            }
        }
        let bytes = if let Some(dataset) = params.dataset {
            let input_path = scope
                .eval::<String>(dataset.to_string().as_str())
                .map_err(|e| {
                    super::errors::AttributeProcessorError::ConversionTable(format!(
                        "Failed to evaluate expr: {}",
                        e
                    ))
                })?;
            let input_path = Uri::from_str(input_path.as_str()).map_err(|e| {
                super::errors::AttributeProcessorError::ConversionTable(format!("{:?}", e))
            })?;
            let storage = storage_resolver.resolve(&input_path).map_err(|e| {
                super::errors::AttributeProcessorError::ConversionTable(format!("{:?}", e))
            })?;
            storage.get_sync(input_path.path().as_path()).map_err(|e| {
                super::errors::AttributeProcessorError::ConversionTable(format!("{:?}", e))
            })?
        } else if let Some(inline) = params.inline {
            Bytes::from(inline.into_bytes())
        } else {
            return Err(AttributeProcessorError::ConversionTableFactory(
                "Missing required parameter `dataset` or `inline`".to_string(),
            )
            .into());
        };

        match params.format {
            ConversionTableFormat::Csv => {
                let conversion_table = read_csv(Delimiter::Comma, bytes)?;
                let conversion_table_indexes =
                    generate_indexes_from_rules(params.rules.clone(), conversion_table.clone());
                let process = AttributeConversionTable {
                    rules: params.rules,
                    conversion_table,
                    conversion_table_indexes,
                };
                Ok(Box::new(process))
            }
            ConversionTableFormat::Tsv => {
                let conversion_table = read_csv(Delimiter::Tab, bytes)?;
                let conversion_table_indexes =
                    generate_indexes_from_rules(params.rules.clone(), conversion_table.clone());
                let process = AttributeConversionTable {
                    rules: params.rules,
                    conversion_table,
                    conversion_table_indexes,
                };
                Ok(Box::new(process))
            }
            ConversionTableFormat::Json => {
                let conversion_table = read_json(bytes)?;
                let conversion_table_indexes =
                    generate_indexes_from_rules(params.rules.clone(), conversion_table.clone());
                let process = AttributeConversionTable {
                    rules: params.rules,
                    conversion_table,
                    conversion_table_indexes,
                };
                Ok(Box::new(process))
            }
        }
    }
}

#[derive(Debug, Clone)]
struct AttributeConversionTable {
    rules: Vec<AttributeConversionTableRule>,
    conversion_table: HashMap<uuid::Uuid, HashMap<String, AttributeValue>>,
    conversion_table_indexes: HashMap<String, HashMap<AttributeValue, uuid::Uuid>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AttributeConversionTableParam {
    /// # Rules to convert attributes
    rules: Vec<AttributeConversionTableRule>,
    /// # Dataset URI
    dataset: Option<Expr>,
    /// # Inline conversion table
    inline: Option<String>,
    /// # Format of conversion table
    format: ConversionTableFormat,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ConversionTableFormat {
    Csv,
    Tsv,
    Json,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AttributeConversionTableRule {
    /// # Attributes to convert from
    feature_froms: Vec<Attribute>,
    /// # Attribute to convert to
    feature_to: Attribute,
    /// # Keys to match in conversion table
    conversion_table_keys: Vec<String>,
    /// # Attribute to convert to
    conversion_table_to: String,
}

impl Processor for AttributeConversionTable {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        for rule in &self.rules {
            let value = if let Some(AttributeValue::Array(value)) =
                feature.get_by_keys(&rule.feature_froms)
            {
                if let Some(id) = self
                    .conversion_table_indexes
                    .get(&generate_index_key(&rule.conversion_table_keys))
                    .and_then(|index| index.get(&generate_index_value_by_attribute_value(&value)))
                {
                    if let Some(row) = self.conversion_table.get(id) {
                        row.get(&rule.conversion_table_to)
                            .cloned()
                            .unwrap_or(AttributeValue::Null)
                    } else {
                        AttributeValue::Null
                    }
                } else {
                    AttributeValue::Null
                }
            } else {
                AttributeValue::Null
            };
            feature.attributes.insert(rule.feature_to.clone(), value);
        }
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeConversionTable"
    }
}

fn read_csv(
    delimiter: Delimiter,
    byte: Bytes,
) -> Result<
    HashMap<uuid::Uuid, HashMap<String, AttributeValue>>,
    super::errors::AttributeProcessorError,
> {
    let cursor = Cursor::new(byte);
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .delimiter(delimiter.into())
        .from_reader(cursor);
    let header = rdr
        .deserialize()
        .next()
        .unwrap_or(Ok(Vec::<String>::new()))
        .map_err(|e| super::errors::AttributeProcessorError::ConversionTable(format!("{:?}", e)))?;
    let mut rows = HashMap::new();
    for rd in rdr.deserialize() {
        let record: Vec<String> = rd.map_err(|e| {
            super::errors::AttributeProcessorError::ConversionTable(format!("{:?}", e))
        })?;
        if record.len() < header.len() {
            return Err(super::errors::AttributeProcessorError::ConversionTable(
                format!(
                    "CSV row has fewer columns ({}) than header ({})",
                    record.len(),
                    header.len()
                ),
            ));
        }
        let row = record
            .iter()
            .enumerate()
            .map(|(i, value)| (header[i].clone(), AttributeValue::String(value.clone())))
            .collect::<HashMap<String, AttributeValue>>();
        rows.insert(uuid::Uuid::new_v4(), row);
    }
    Ok(rows)
}

fn read_json(
    byte: Bytes,
) -> Result<
    HashMap<uuid::Uuid, HashMap<String, AttributeValue>>,
    super::errors::AttributeProcessorError,
> {
    let value: serde_json::Value =
        serde_json::from_str(std::str::from_utf8(&byte).map_err(|e| {
            super::errors::AttributeProcessorError::ConversionTable(format!("{:?}", e))
        })?)
        .map_err(|e| super::errors::AttributeProcessorError::ConversionTable(format!("{:?}", e)))?;
    let mut rows = HashMap::new();
    match value {
        serde_json::Value::Array(arr) => {
            for v in arr {
                match v {
                    serde_json::Value::Object(obj) => {
                        let row = obj
                            .iter()
                            .map(|(k, v)| (k.clone(), AttributeValue::from(v.clone())))
                            .collect::<HashMap<String, AttributeValue>>();
                        rows.insert(uuid::Uuid::new_v4(), row);
                    }
                    _ => {
                        return Err(super::errors::AttributeProcessorError::ConversionTable(
                            "Invalid JSON format".to_string(),
                        ));
                    }
                }
            }
        }
        serde_json::Value::Object(obj) => {
            let row = obj
                .iter()
                .map(|(k, v)| (k.clone(), AttributeValue::from(v.clone())))
                .collect::<HashMap<String, AttributeValue>>();
            rows.insert(uuid::Uuid::new_v4(), row);
        }
        _ => {
            return Err(super::errors::AttributeProcessorError::ConversionTable(
                "Invalid JSON format".to_string(),
            ));
        }
    }
    Ok(rows)
}

fn generate_indexes_from_rules(
    rules: Vec<AttributeConversionTableRule>,
    conversion_table: HashMap<uuid::Uuid, HashMap<String, AttributeValue>>,
) -> HashMap<String, HashMap<AttributeValue, uuid::Uuid>> {
    let mut indexes = HashMap::new();
    for rule in rules {
        let keys = &rule.conversion_table_keys;
        for (id, row) in conversion_table.iter() {
            let values = generate_index_value(row, keys);
            let index = indexes
                .entry(generate_index_key(keys))
                .or_insert(HashMap::new());
            index.insert(AttributeValue::String(values), *id);
        }
    }
    indexes
}

fn generate_index_key(keys: &[String]) -> String {
    keys.join("")
}

fn generate_index_value(row: &HashMap<String, AttributeValue>, keys: &[String]) -> String {
    keys.iter().fold("".to_string(), |acc, key| {
        if let Some(AttributeValue::String(value)) = row.get(key) {
            format!("{}{}", acc, value)
        } else {
            acc
        }
    })
}

fn generate_index_value_by_attribute_value(values: &[AttributeValue]) -> AttributeValue {
    let value = values.iter().fold("".to_string(), |acc, value| {
        if let AttributeValue::String(value) = value {
            format!("{}{}", acc, value)
        } else {
            acc
        }
    });
    AttributeValue::String(value)
}
