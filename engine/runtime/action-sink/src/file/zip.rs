use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Cursor};
use std::path::Path;
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

        let sink = ZipFileWriter {
            output: params.output,
            base_path: params.base_path,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
struct ZipFileWriter {
    output: Expr,
    base_path: Option<Expr>,
    buffer: Vec<Uri>,
}

/// # ZipFileWriter Parameters
///
/// Configuration for creating ZIP archive files from features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ZipFileWriterParam {
    /// Output path
    output: Expr,
    /// Base path for computing relative paths. When set, directory structure
    /// relative to this base will be preserved in the zip file.
    /// If not set, all files are placed at the root of the zip (flattened).
    base_path: Option<Expr>,
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
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = expr_engine.new_scope();

        let output_path = scope
            .eval::<String>(self.output.as_ref())
            .unwrap_or_else(|_| self.output.as_ref().to_string());
        let output = Uri::from_str(output_path.as_str())?;

        let base_path = self.base_path.as_ref().map(|bp| {
            let evaluated = scope
                .eval::<String>(bp.as_ref())
                .unwrap_or_else(|_| bp.as_ref().to_string());
            Uri::from_str(&evaluated).ok()
        });
        let base_path = base_path.flatten();

        let temp_dir_path = dir::project_temp_dir(uuid::Uuid::new_v4().to_string().as_str())?;

        if let Some(base) = &base_path {
            // Preserve directory structure relative to base_path
            let base_path_str = base.path();
            for file_uri in &self.buffer {
                let file_path = file_uri.path();
                let relative_path = file_path.strip_prefix(&base_path_str).unwrap_or_else(|_| {
                    // If file is not under base_path, use just the filename
                    file_path
                        .file_name()
                        .map(Path::new)
                        .unwrap_or(file_path.as_path())
                });
                let dest_path = temp_dir_path.join(relative_path);
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        SinkError::ZipFileWriter(format!(
                            "Failed to create directory {}: {}",
                            parent.display(),
                            e
                        ))
                    })?;
                }
                fs::rename(file_uri.path(), &dest_path).map_err(|e| {
                    SinkError::ZipFileWriter(format!(
                        "Failed to move file {} to {}: {}",
                        file_uri.path().display(),
                        dest_path.display(),
                        e
                    ))
                })?;
            }
        } else {
            // Flatten all files to root (original behavior)
            dir::move_files(&temp_dir_path, &self.buffer)?;
        }

        let buffer = Vec::new();
        let mut cursor = Cursor::new(buffer);
        let writer = BufWriter::new(&mut cursor);
        zip::write(writer, temp_dir_path.as_path())
            .map_err(|e| crate::errors::SinkError::ZipFileWriter(e.to_string()))?;
        let storage = storage_resolver.resolve(&output).map_err(|e| {
            crate::errors::SinkError::ZipFileWriter(format!(
                "Failed to resolve storage for {output}: {e}"
            ))
        })?;
        storage.put_sync(
            output.path().as_path(),
            bytes::Bytes::from(cursor.into_inner()),
        )?;
        Ok(())
    }
}
