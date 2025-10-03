use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use reearth_flow_common::csv::Delimiter;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{AttributeValue, Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::writer::write_csv;
use crate::errors::SinkError;

#[derive(Debug, Clone, Default)]
pub(crate) struct CsvWriterFactory;

impl SinkFactory for CsvWriterFactory {
    fn name(&self) -> &str {
        "CsvWriter"
    }

    fn description(&self) -> &str {
        "Writes features to CSV or TSV files."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CsvWriterParam))
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
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::CsvWriterFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::CsvWriterFactory(format!("Failed to deserialize `with` parameter: {e}"))
            })?
        } else {
            return Err(SinkError::CsvWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let sink = CsvWriter {
            params,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(super) struct CsvWriter {
    pub(super) params: CsvWriterParam,
    pub(super) buffer: HashMap<AttributeValue, Vec<Feature>>,
}

/// # CsvWriter Parameters
///
/// Configuration for writing features to CSV/TSV files.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct CsvWriterParam {
    /// Output path or expression for the CSV/TSV file to create
    pub(super) output: Expr,
    /// Optional attributes to group features by, creating separate files for each group
    pub(super) group_by: Option<Vec<String>>,
    /// File format: csv (comma) or tsv (tab)
    format: CsvFormat,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CsvFormat {
    /// # CSV (Comma-Separated Values)
    /// File with comma-separated values
    Csv,
    /// # TSV (Tab-Separated Values)
    /// File with tab-separated values
    Tsv,
}

impl CsvFormat {
    fn delimiter(&self) -> Delimiter {
        match self {
            CsvFormat::Csv => Delimiter::Comma,
            CsvFormat::Tsv => Delimiter::Tab,
        }
    }
}

impl Sink for CsvWriter {
    fn name(&self) -> &str {
        "CsvWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let key = if let Some(group_by) = &self.params.group_by {
            if group_by.is_empty() {
                AttributeValue::Null
            } else {
                let key = group_by
                    .iter()
                    .map(|k| feature.get(k).cloned().unwrap_or(AttributeValue::Null))
                    .collect::<Vec<_>>();
                AttributeValue::Array(key)
            }
        } else {
            AttributeValue::Null
        };
        self.buffer.entry(key).or_default().push(feature.clone());
        Ok(())
    }
    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let output = self.params.output.clone();
        let scope = expr_engine.new_scope();
        let path = scope
            .eval::<String>(output.as_ref())
            .unwrap_or_else(|_| output.as_ref().to_string());
        let output = Uri::from_str(path.as_str())?;
        let delimiter = self.params.format.delimiter();
        for (key, features) in self.buffer.iter() {
            let file_path = if *key == AttributeValue::Null {
                output.clone()
            } else {
                // Use .csv or .tsv extension based on format
                let ext = match self.params.format {
                    CsvFormat::Csv => "csv",
                    CsvFormat::Tsv => "tsv",
                };
                output.join(format!(
                    "{}.{}",
                    reearth_flow_common::str::to_hash(key.to_string().as_str()),
                    ext
                ))?
            };
            write_csv(&file_path, features, delimiter.clone(), &storage_resolver)?;
        }
        Ok(())
    }
}
