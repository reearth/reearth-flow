use std::collections::HashMap;
use std::{str::FromStr, sync::Arc};

use bytes::Bytes;
use reearth_flow_common::csv::Delimiter;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{AttributeValue, Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use reearth_flow_common::uri::Uri;
use serde_json::Value;

use crate::errors::SinkError;

use super::excel::write_excel;

#[derive(Debug, Clone, Default)]
pub struct FileWriterSinkFactory;

impl SinkFactory for FileWriterSinkFactory {
    fn name(&self) -> &str {
        "FileWriter"
    }

    fn description(&self) -> &str {
        "Writes features to a file"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FileWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["File"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn prepare(&self) -> Result<(), BoxedError> {
        Ok(())
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params = if let Some(with) = with {
            let value: Value = serde_json::to_value(with)
                .map_err(|e| SinkError::BuildFactory(format!("Failed to serialize with: {}", e)))?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::BuildFactory(format!("Failed to deserialize with: {}", e))
            })?
        } else {
            return Err(
                SinkError::BuildFactory("Missing required parameter `with`".to_string()).into(),
            );
        };

        let sink = FileWriter {
            params,
            buffer: Vec::new(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileWriterParam {
    format: Format,
    pub(super) output: Expr,
}

#[derive(Debug, Clone)]
pub struct FileWriter {
    pub(super) params: FileWriterParam,
    pub(super) buffer: Vec<Feature>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
enum Format {
    #[serde(rename = "csv")]
    Csv,
    #[serde(rename = "tsv")]
    Tsv,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "excel")]
    Excel,
}

impl Sink for FileWriter {
    fn initialize(&self, _ctx: NodeContext) {}
    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        self.buffer.push(ctx.feature);
        Ok(())
    }
    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let scope = ctx.expr_engine.new_scope();
        let path = ctx
            .expr_engine
            .eval_scope::<String>(self.params.output.as_ref(), &scope)
            .unwrap_or_else(|_| self.params.output.as_ref().to_string());
        let output = Uri::from_str(path.as_str())?;
        let result = match self.params.format {
            Format::Json => write_json(&output, &self.buffer, storage_resolver),
            Format::Csv => write_csv(&output, &self.buffer, Delimiter::Comma, storage_resolver),
            Format::Tsv => write_csv(&output, &self.buffer, Delimiter::Tab, storage_resolver),
            Format::Excel => write_excel(&output, &self.buffer, storage_resolver),
        };
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
}

fn write_json(
    output: &Uri,
    features: &[Feature],
    storage_resolver: Arc<StorageResolver>,
) -> Result<(), crate::errors::SinkError> {
    let json_value: serde_json::Value = features.into();
    let storage = storage_resolver
        .resolve(output)
        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?;
    storage
        .put_sync(output.path().as_path(), Bytes::from(json_value.to_string()))
        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?;
    Ok(())
}

fn write_csv(
    output: &Uri,
    features: &[Feature],
    delimiter: Delimiter,
    storage_resolver: Arc<StorageResolver>,
) -> Result<(), crate::errors::SinkError> {
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter.into())
        .quote_style(csv::QuoteStyle::NonNumeric)
        .from_writer(vec![]);
    let rows: Vec<AttributeValue> = features.iter().map(|f| f.clone().into()).collect();
    let fields = get_fields(rows.first().unwrap());
    for row in rows {
        match fields {
            Some(ref fields) if !fields.is_empty() => {
                let values = get_row_values(&row, &fields.clone())?;
                wtr.write_record(values)
                    .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?;
            }
            _ => match row {
                AttributeValue::String(s) => wtr
                    .write_record(vec![s])
                    .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?,
                AttributeValue::Array(s) => {
                    let values = s
                        .into_iter()
                        .map(|v| match v {
                            AttributeValue::String(s) => s,
                            _ => "".to_string(),
                        })
                        .collect::<Vec<_>>();
                    wtr.write_record(values)
                        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?
                }
                _ => {
                    return Err(crate::errors::SinkError::FileWriter(
                        "Unsupported input".to_string(),
                    ))
                }
            },
        }
    }
    wtr.flush()
        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?;
    let data = String::from_utf8(
        wtr.into_inner()
            .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?,
    )
    .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?;
    let storage = storage_resolver
        .resolve(output)
        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?;
    storage
        .put_sync(output.path().as_path(), Bytes::from(data))
        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?;
    Ok(())
}

fn get_fields(row: &AttributeValue) -> Option<Vec<String>> {
    match row {
        AttributeValue::Map(row) => Some(row.keys().cloned().collect::<Vec<_>>()),
        _ => None,
    }
}

fn get_row_values(
    row: &AttributeValue,
    fields: &[String],
) -> Result<Vec<String>, crate::errors::SinkError> {
    fields
        .iter()
        .map(|field| match row {
            AttributeValue::Map(row) => row.get(field).map(|v| v.to_string()).ok_or_else(|| {
                crate::errors::SinkError::FileWriter(format!("Field not found: {}", field))
            }),
            _ => Err(crate::errors::SinkError::FileWriter(
                "Unsupported input".to_string(),
            )),
        })
        .collect()
}
