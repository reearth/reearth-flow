use std::collections::HashMap;

use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{Code, CompiledCode, Feature};
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
        let params: ExcelWriterParam = if let Some(with) = with {
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
        let output = params.output.compile().map_err(|e| {
            SinkError::ExcelWriterFactory(format!("Failed to compile `output`: {e:?}"))
        })?;
        let sink = ExcelWriter {
            output,
            sheet_name: params.sheet_name,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(super) struct ExcelWriter {
    output: CompiledCode,
    sheet_name: Option<String>,
    pub(super) buffer: HashMap<String, (crate::SinkOutput, Vec<Feature>)>,
}

/// # ExcelWriter Parameters
///
/// Configuration for writing features to Microsoft Excel format.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExcelWriterParam {
    /// Output path or expression for the Excel file to create
    pub(super) output: Code,
    /// Sheet name (defaults to "Sheet1")
    pub(super) sheet_name: Option<String>,
}

impl Sink for ExcelWriter {
    fn name(&self) -> &str {
        "ExcelWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let path = self
            .output
            .eval_string(&ctx.feature, ctx.expr_engine.vars())
            .map_err(|e| SinkError::ExcelWriterFactory(format!("{e:?}")))?;
        let feature = ctx.feature.clone();
        let node_ctx: NodeContext = ctx.into();
        use std::collections::hash_map::Entry;
        match self.buffer.entry(path.clone()) {
            Entry::Occupied(mut e) => {
                e.get_mut().1.push(feature);
            }
            Entry::Vacant(e) => {
                let out = crate::SinkOutput::new(
                    &node_ctx.sandbox_root,
                    &path,
                    &node_ctx.storage_resolver,
                )
                .map_err(|e| SinkError::ExcelWriterFactory(e.to_string()))?;
                e.insert((out, vec![feature]));
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext) -> Result<(), BoxedError> {
        for (out, features) in self.buffer.values() {
            let old_params = OldExcelWriterParam {
                sheet_name: self.sheet_name.clone(),
            };
            write_excel(out, &old_params, features)?;
        }
        Ok(())
    }
}
