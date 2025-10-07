use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;

use super::excel::{write_excel, ExcelWriterParam as OldExcelWriterParam};

#[derive(Debug, Clone, Default)]
pub(crate) struct ExcelWriterFactory;

impl SinkFactory for ExcelWriterFactory {
    fn name(&self) -> &str {
        "ExcelWriter"
    }

    fn description(&self) -> &str {
        "Writes features to Microsoft Excel format (.xlsx files)."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ExcelWriterParam))
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
                SinkError::ExcelWriterFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::ExcelWriterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SinkError::ExcelWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let sink = ExcelWriter {
            params,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(super) struct ExcelWriter {
    pub(super) params: ExcelWriterParam,
    pub(super) buffer: HashMap<Uri, Vec<Feature>>,
}

/// # ExcelWriter Parameters
///
/// Configuration for writing features to Microsoft Excel format.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExcelWriterParam {
    /// Output path or expression for the Excel file to create
    pub(super) output: Expr,
    /// Sheet name (defaults to "Sheet1")
    pub(super) sheet_name: Option<String>,
}

impl Sink for ExcelWriter {
    fn name(&self) -> &str {
        "ExcelWriter"
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
        for (uri, features) in &self.buffer {
            let old_params = OldExcelWriterParam {
                sheet_name: self.params.sheet_name.clone(),
            };
            write_excel(uri, &old_params, features, &storage_resolver)?;
        }
        Ok(())
    }
}
