use std::collections::HashMap;

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
    pub(super) buffer: HashMap<Uri, (crate::SinkOutput, Vec<Feature>)>,
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
        let node_ctx: NodeContext = ctx.clone().into();
        let (path, uri) = crate::SinkOutput::evaluate_uri(&node_ctx, &self.params.output)
            .map_err(|e| SinkError::ExcelWriterFactory(e.to_string()))?;
        let feature = ctx.feature.clone();
        use std::collections::hash_map::Entry;
        match self.buffer.entry(uri) {
            Entry::Occupied(mut e) => {
                e.get_mut().1.push(feature);
            }
            Entry::Vacant(e) => {
                let out = crate::SinkOutput::from_path(&node_ctx, &path)
                    .map_err(|e| SinkError::ExcelWriterFactory(e.to_string()))?;
                e.insert((out, vec![feature]));
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext) -> Result<(), BoxedError> {
        for (out, features) in self.buffer.values() {
            let old_params = OldExcelWriterParam {
                sheet_name: self.params.sheet_name.clone(),
            };
            write_excel(out, &old_params, features)?;
        }
        Ok(())
    }
}
