use std::{collections::HashMap, fs, path::PathBuf, str::FromStr, sync::Arc};

use reearth_flow_common::{dir::project_temp_dir, uri::Uri};
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::utils::decompressor::extract_archive;

#[derive(Debug, Clone, Default)]
pub struct DirectoryDecompressorFactory;

impl ProcessorFactory for DirectoryDecompressorFactory {
    fn name(&self) -> &str {
        "DirectoryDecompressor"
    }

    fn description(&self) -> &str {
        "Decompresses a directory"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(DirectoryDecompressorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["File"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: DirectoryDecompressorParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                super::errors::FileProcessorError::DirectoryDecompressorFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                super::errors::FileProcessorError::DirectoryDecompressorFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(
                super::errors::FileProcessorError::DirectoryDecompressorFactory(
                    "Missing required parameter `with`".to_string(),
                )
                .into(),
            );
        };
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let archive_path = expr_engine
            .compile(param.archive_path.as_ref())
            .map_err(|e| {
                super::errors::FileProcessorError::DirectoryDecompressorFactory(format!(
                    "Failed to compile `archive_path` expression: {}",
                    e
                ))
            })?;
        let process = DirectoryDecompressor {
            params: DirectoryDecompressorCompiledParam {
                archive_path,
                output_path_attribute: param.output_path_attribute,
            },
            with,
        };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryDecompressorParam {
    archive_path: Expr,
    output_path_attribute: Attribute,
}

#[derive(Debug, Clone)]
pub struct DirectoryDecompressorCompiledParam {
    archive_path: rhai::AST,
    output_path_attribute: Attribute,
}

#[derive(Debug, Clone)]
pub struct DirectoryDecompressor {
    params: DirectoryDecompressorCompiledParam,
    with: Option<HashMap<String, Value>>,
}

impl Processor for DirectoryDecompressor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = feature.new_scope(expr_engine, &self.with);
        let source_dataset = scope
            .eval_ast::<String>(&self.params.archive_path)
            .map_err(|e| {
                super::errors::FileProcessorError::DirectoryDecompressor(format!(
                    "Failed to evaluate `source_dataset` expression: {}",
                    e
                ))
            })?;
        let source_dataset = Uri::from_str(source_dataset.as_str()).map_err(|_| {
            super::errors::FileProcessorError::DirectoryDecompressor("Invalid path".to_string())
        })?;
        let root_output_path = project_temp_dir(uuid::Uuid::new_v4().to_string().as_str())?;
        let root_output_path = Uri::from_str(root_output_path.to_str().ok_or(
            super::errors::FileProcessorError::DirectoryDecompressor("Invalid path".to_string()),
        )?)
        .map_err(|e| {
            super::errors::FileProcessorError::DirectoryDecompressor(format!(
                "Failed to convert `root_output_path` to URI: {}",
                e
            ))
        })?;
        let _ = extract_archive(
            &source_dataset,
            &root_output_path,
            ctx.storage_resolver.clone(),
        )
        .map_err(|e| {
            super::errors::FileProcessorError::DirectoryDecompressor(format!(
                "Failed to extract archive: {}",
                e
            ))
        })?;
        let mut feature = ctx.feature.clone();
        let root_output_path = get_single_subfolder_or_self(&root_output_path)?;
        feature.insert(
            self.params.output_path_attribute.clone(),
            AttributeValue::String(root_output_path.to_string()),
        );
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(
        &self,
        _ctx: NodeContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "DirectoryDecompressor"
    }
}

fn get_single_subfolder_or_self(parent_dir: &Uri) -> super::errors::Result<Uri> {
    let subfolders: Vec<PathBuf> = fs::read_dir(parent_dir.path())
        .map_err(|e| super::errors::FileProcessorError::DirectoryDecompressor(format!("{:?}", e)))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            Some(path)
        })
        .collect();

    if subfolders.len() == 1 && subfolders[0].is_dir() {
        Ok(Uri::from_str(subfolders[0].to_str().ok_or(
            super::errors::FileProcessorError::DirectoryDecompressor("Invalid path".to_string()),
        )?)
        .map_err(|e| {
            super::errors::FileProcessorError::DirectoryDecompressor(format!(
                "Failed to convert `subfolders[0]` to URI: {}",
                e
            ))
        })?)
    } else {
        Ok(parent_dir.clone())
    }
}
