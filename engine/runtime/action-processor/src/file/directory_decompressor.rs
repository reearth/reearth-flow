use std::{collections::HashMap, fs, path::PathBuf, str::FromStr, sync::Arc};

use reearth_flow_common::{dir::project_temp_dir, uri::Uri};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub(super) struct DirectoryDecompressorFactory;

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
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: DirectoryDecompressorParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                super::errors::FileProcessorError::DirectoryDecompressorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                super::errors::FileProcessorError::DirectoryDecompressorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
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
        let process = DirectoryDecompressor {
            archive_attributes: param.archive_attributes,
        };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct DirectoryDecompressorParam {
    /// # Attribute to extract file path from
    archive_attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
struct DirectoryDecompressor {
    archive_attributes: Vec<Attribute>,
}

impl Processor for DirectoryDecompressor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        for attribute in &self.archive_attributes {
            let Some(AttributeValue::String(source_dataset)) = feature.get(attribute) else {
                continue;
            };
            let Ok(source_dataset) = Uri::from_str(source_dataset.as_str()) else {
                continue;
            };
            if !crate::utils::decompressor::is_extractable_archive(&source_dataset) {
                continue;
            }
            let root_output_path = extract_archive(&source_dataset, ctx.storage_resolver.clone())
                .map_err(|e| {
                super::errors::FileProcessorError::DirectoryDecompressor(format!(
                    "Failed to extract archive: {e}"
                ))
            })?;
            feature.insert(
                attribute.clone(),
                AttributeValue::String(root_output_path.to_string()),
            );
        }
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "DirectoryDecompressor"
    }
}

fn extract_archive(
    source_dataset: &Uri,
    storage_resolver: Arc<StorageResolver>,
) -> super::errors::Result<Uri> {
    let root_output_path =
        project_temp_dir(uuid::Uuid::new_v4().to_string().as_str()).map_err(|e| {
            super::errors::FileProcessorError::DirectoryDecompressor(format!(
                "Failed to create temp directory: {e}"
            ))
        })?;
    let root_output_path = Uri::from_str(root_output_path.to_str().ok_or(
        super::errors::FileProcessorError::DirectoryDecompressor("Invalid path".to_string()),
    )?)
    .map_err(|e| {
        super::errors::FileProcessorError::DirectoryDecompressor(format!(
            "Failed to convert `root_output_path` to URI: {e}"
        ))
    })?;
    let _ = crate::utils::decompressor::extract_archive(
        source_dataset,
        &root_output_path,
        storage_resolver.clone(),
    )
    .map_err(|e| {
        super::errors::FileProcessorError::DirectoryDecompressor(format!(
            "Failed to extract archive: {e}"
        ))
    })?;
    let root_output_path = get_single_subfolder_or_self(&root_output_path)?;
    Ok(root_output_path)
}

fn get_single_subfolder_or_self(parent_dir: &Uri) -> super::errors::Result<Uri> {
    let subfolders: Vec<PathBuf> = fs::read_dir(parent_dir.path())
        .map_err(|e| super::errors::FileProcessorError::DirectoryDecompressor(format!("{e:?}")))?
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
                "Failed to convert `subfolders[0]` to URI: {e}"
            ))
        })?)
    } else {
        Ok(parent_dir.clone())
    }
}
