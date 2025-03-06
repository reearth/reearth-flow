use std::collections::HashMap;
use std::io::{BufWriter, Cursor};
use std::str::FromStr;
use std::sync::Arc;
use std::vec;

use reearth_flow_common::uri::Uri;
use reearth_flow_common::{dir, zip};
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{AttributeValue, Expr};
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
        let params: ZipFileWriterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::ZipFileWriterFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::ZipFileWriterFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(SinkError::ZipFileWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let sink = ZipFileWriter {
            output: params.output,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
struct ZipFileWriter {
    output: Expr,
    buffer: Vec<Uri>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ZipFileWriterParam {
    /// Output path
    output: Expr,
}

impl Sink for ZipFileWriter {
    fn name(&self) -> &str {
        "ZipFileWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let Some(AttributeValue::String(file_path)) = feature.get(&"filePath") else {
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
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let output = self.output.clone();
        let scope = expr_engine.new_scope();
        let path = scope
            .eval::<String>(output.as_ref())
            .unwrap_or_else(|_| output.as_ref().to_string());
        let output = Uri::from_str(path.as_str())?;
        let temp_dir_path = dir::project_temp_dir(uuid::Uuid::new_v4().to_string().as_str())?;
        dir::move_files(&temp_dir_path, &self.buffer)?;
        let buffer = Vec::new();
        let mut cursor = Cursor::new(buffer);
        let writer = BufWriter::new(&mut cursor);
        zip::write(writer, temp_dir_path.as_path())
            .map_err(|e| crate::errors::SinkError::ZipFileWriter(e.to_string()))?;
        let storage = storage_resolver.resolve(&output).map_err(|e| {
            crate::errors::SinkError::ZipFileWriter(format!(
                "Failed to resolve storage for {}: {}",
                output, e
            ))
        })?;
        storage.put_sync(
            output.path().as_path(),
            bytes::Bytes::from(cursor.into_inner()),
        )?;
        Ok(())
    }
}
