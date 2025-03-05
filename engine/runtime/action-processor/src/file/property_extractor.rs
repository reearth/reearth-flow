use std::{collections::HashMap, fmt::Debug, fs, path::Path, str::FromStr};

use reearth_flow_common::{fs::metadata, uri::Uri};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FileProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct FilePropertyExtractorFactory;

impl ProcessorFactory for FilePropertyExtractorFactory {
    fn name(&self) -> &str {
        "FilePropertyExtractor"
    }

    fn description(&self) -> &str {
        "Extracts properties from a file"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FilePropertyExtractor))
    }

    fn categories(&self) -> &[&'static str] {
        &["File"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let process: FilePropertyExtractor = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FileProcessorError::PropertyExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FileProcessorError::PropertyExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(FileProcessorError::PropertyExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
enum FileType {
    File,
    Directory,
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::File => write!(f, "File"),
            FileType::Directory => write!(f, "Directory"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct FileProperty {
    file_type: FileType,
    file_size: i64,
    file_atime: i64,
    file_mtime: i64,
    file_ctime: i64,
}

impl From<FileProperty> for HashMap<Attribute, AttributeValue> {
    fn from(value: FileProperty) -> Self {
        let mut map = HashMap::new();
        map.insert(
            Attribute::new("fileType"),
            AttributeValue::String(value.file_type.to_string()),
        );
        map.insert(
            Attribute::new("fileSize"),
            AttributeValue::Number(serde_json::Number::from(value.file_size)),
        );
        map.insert(Attribute::new("fileAtime"), {
            match value.file_atime.try_into() {
                Ok(v) => AttributeValue::DateTime(v),
                Err(_) => AttributeValue::Null,
            }
        });
        map.insert(Attribute::new("fileMtime"), {
            match value.file_mtime.try_into() {
                Ok(v) => AttributeValue::DateTime(v),
                Err(_) => AttributeValue::Null,
            }
        });
        map.insert(Attribute::new("fileCtime"), {
            match value.file_ctime.try_into() {
                Ok(v) => AttributeValue::DateTime(v),
                Err(_) => AttributeValue::Null,
            }
        });
        map
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct FilePropertyExtractor {
    /// # Attribute to extract file path from
    file_path_attribute: String,
}

impl Processor for FilePropertyExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let Some(AttributeValue::String(file_path)) = feature.get(&self.file_path_attribute) else {
            return Err(FileProcessorError::PropertyExtractor(format!(
                "Attribute {} not found",
                self.file_path_attribute
            ))
            .into());
        };
        let uri = Uri::from_str(file_path.as_str())
            .map_err(|e| FileProcessorError::PropertyExtractor(format!("{:?}", e)))?;
        let path = uri.path();
        if path.exists() && !path.is_symlink() {
            let metadata = metadata(&path)?;
            let file_property = FileProperty {
                file_type: if metadata.is_dir {
                    FileType::Directory
                } else {
                    FileType::File
                },
                file_size: if metadata.is_dir {
                    get_dir_size(&path)? as i64
                } else {
                    metadata.size
                },
                file_atime: metadata.atime,
                file_mtime: metadata.mtime,
                file_ctime: metadata.ctime,
            };
            let mut feature = ctx.feature.clone();
            let attributes: HashMap<Attribute, AttributeValue> = file_property.into();
            feature.attributes.extend(attributes);
            fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        } else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "FilePropertyExtractor"
    }
}

fn get_dir_size(path: &Path) -> super::errors::Result<u64> {
    let total: u64 = fs::read_dir(path)
        .map_err(|e| FileProcessorError::PropertyExtractor(format!("{:?}", e)))?
        .filter_map(Result::ok)
        .map(|entry| {
            let path = entry.path();
            if path.is_file() {
                fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
            } else if path.is_dir() {
                get_dir_size(&path).unwrap_or(0)
            } else {
                0
            }
        })
        .sum();
    Ok(total)
}
