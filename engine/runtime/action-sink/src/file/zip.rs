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

/// # ZipFileWriter Parameters
///
/// Configuration for creating ZIP archive files from features.
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
        let output = self.output.clone();
        let scope = expr_engine.new_scope();
        let path = scope
            .eval::<String>(output.as_ref())
            .unwrap_or_else(|_| output.as_ref().to_string());
        let output = Uri::from_str(path.as_str())?;

        // - missing files are skipped with a warning
        let total = self.buffer.len();
        let mut missing = 0usize;
        let mut existing: Vec<Uri> = Vec::with_capacity(total);

        for u in &self.buffer {
            let p = u.path();
            let exists = p.exists();
            tracing::info!(
                "ZipFileWriter buffer file src={} exists={}",
                p.display(),
                exists
            );
            if !exists {
                missing += 1;
                tracing::warn!(
                    "ZipFileWriter: missing input file. skipping. src={}",
                    p.display()
                );
                continue;
            }
            existing.push(u.clone());
        }

        tracing::info!(
            "ZipFileWriter: input summary total={} used={} missing={}",
            total,
            existing.len(),
            missing
        );

        // If no files remain, do not create a zip and only issue a warning as per the specification.
        if existing.is_empty() {
            tracing::warn!("ZipFileWriter: no existing files to archive. skip creating zip.");
            return Ok(());
        }

        // Gather files directly under artifacts (overwrite if the same name exists)
        let artifact_root = dir::current_job_artifact_root()?;
        std::fs::create_dir_all(&artifact_root).map_err(reearth_flow_common::Error::dir)?;

        for u in &existing {
            let src = u.path();
            let Some(name) = u.file_name() else {
                tracing::warn!(
                    "ZipFileWriter: invalid input file path. skipping. src={}",
                    src.display()
                );
                missing += 1;
                continue;
            };
            let dest = artifact_root.join(name);

            // If the same name already exists, overwrite it (copy after deleting)
            if dest.exists() {
                if let Err(e) = std::fs::remove_file(&dest) {
                    // This failure may occur due to permissions or directory issues, so we raise an error (cannot overwrite).
                    return Err(crate::errors::SinkError::ZipFileWriter(format!(
                        "Failed to remove existing file before overwrite dest={} err={}",
                        dest.display(),
                        e
                    ))
                    .into());
                }
            }

            std::fs::copy(&src, &dest).map_err(|e| {
                crate::errors::SinkError::ZipFileWriter(format!(
                    "Failed to copy file src={} dest={} err={}",
                    src.display(),
                    dest.display(),
                    e
                ))
            })?;
        }

        tracing::info!(
            "ZipFileWriter: artifacts copy done total={} used={} missing={}",
            total,
            existing.len(),
            missing
        );

        let buffer = Vec::new();
        let mut cursor = Cursor::new(buffer);
        let writer = BufWriter::new(&mut cursor);

        // Create a zip archive directly under artifacts (including the files just gathered)
        zip::write(writer, artifact_root.as_path())
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
