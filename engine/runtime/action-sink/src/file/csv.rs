use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use reearth_flow_common::csv::Delimiter;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{Expr, Feature};
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
    pub(super) buffer: HashMap<Uri, Vec<Feature>>,
}

/// # CsvWriter Parameters
///
/// Configuration for writing features to CSV/TSV files.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct CsvWriterParam {
    /// Output path or expression for the CSV/TSV file to create
    pub(super) output: Expr,
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
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = expr_engine.new_scope();
        let output = &self.params.output;
        let path = scope
            .eval::<String>(output.as_ref())
            .unwrap_or_else(|_| output.as_ref().to_string());
        let uri = Uri::from_str(&path)?;
        self.buffer.entry(uri).or_default().push(ctx.feature);
        Ok(())
    }

    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let delimiter = self.params.format.delimiter();
        for (uri, features) in &self.buffer {
            write_csv(&uri, features, delimiter.clone(), &storage_resolver)?;
        }
        Ok(())
    }
}
