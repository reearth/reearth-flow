use std::collections::HashMap;
use std::io::{BufWriter, Cursor};
use std::str::FromStr;
use std::vec;

use reearth_flow_common::uri::Uri;
use reearth_flow_common::{dir, zip};
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{AttributeValue, Code};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;

#[derive(Debug, Clone, Default)]
pub(crate) struct ZipFileWriterFactory;

impl SinkFactory for ZipFileWriterFactory {
    fn name(&self) -> &str {
        "ZipFileWriter"
    }

    fn description(&self) -> &str {
        "Writes features to a zip file"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ZipFileWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Output"]
    }

    fn tags(&self) -> &[&'static str] {
        &["file-system", "compression"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn prepare(&self) -> Result<(), BoxedError> {
        Ok(())
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params: ZipFileWriterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::ZipFileWriterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::ZipFileWriterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SinkError::ZipFileWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let output = params
            .output
            .compile()
            .map_err(|e| {
                SinkError::ZipFileWriterFactory(format!("Failed to compile `output`: {e:?}"))
            })?
            .eval_string_env_only(ctx.expr_engine.vars())
            .map_err(|e| {
                SinkError::ZipFileWriterFactory(format!("Failed to evaluate `output`: {e:?}"))
            })?;
        let sink = ZipFileWriter {
            output,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
struct ZipFileWriter {
    output: String,
    buffer: Vec<Uri>,
}

/// # ZipFileWriter Parameters
///
/// Configuration for creating ZIP archive files from features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ZipFileWriterParam {
    /// Output path
    output: Code,
}

impl Sink for ZipFileWriter {
    fn name(&self) -> &str {
        "ZipFileWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let Some(AttributeValue::String(file_path)) = feature.get("filePath") else {
            return Ok(());
        };
        let file_path = Uri::from_str(file_path.as_str())?;
        self.buffer.push(file_path);
        Ok(())
    }
    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        let path = self.output.as_str();
        let out = crate::SinkOutput::new(&ctx.sandbox_root, path, &ctx.storage_resolver)
            .map_err(|e| crate::errors::SinkError::ZipFileWriter(e.to_string()))?;
        let temp_dir_path = dir::project_temp_dir(uuid::Uuid::new_v4().to_string().as_str())?;
        dir::move_files_with_structure(&temp_dir_path, &self.buffer)?;
        let buffer = Vec::new();
        let mut cursor = Cursor::new(buffer);
        let writer = BufWriter::new(&mut cursor);
        zip::write(writer, temp_dir_path.as_path())
            .map_err(|e| crate::errors::SinkError::ZipFileWriter(e.to_string()))?;
        out.write(bytes::Bytes::from(cursor.into_inner()))
            .map_err(|e| crate::errors::SinkError::ZipFileWriter(e.to_string()))?;
        Ok(())
    }
}
