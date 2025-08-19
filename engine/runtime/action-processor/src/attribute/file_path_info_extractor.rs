use std::{collections::HashMap, path::Path};

use reearth_flow_common::fs::{get_dir_size, metadata};
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
use serde_json::{Number, Value};

use super::errors::AttributeProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct AttributeFilePathInfoExtractorFactory;

impl ProcessorFactory for AttributeFilePathInfoExtractorFactory {
    fn name(&self) -> &str {
        "AttributeFilePathInfoExtractor"
    }

    fn description(&self) -> &str {
        "Extract File System Information from Path Attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AttributeFilePathInfoExtractor))
    }

    fn categories(&self) -> &[&'static str] {
        &["Attribute"]
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
        let processor: AttributeFilePathInfoExtractor = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::FilePathInfoExtractor(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::FilePathInfoExtractor(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(AttributeProcessorError::FilePathInfoExtractor(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(processor))
    }
}

/// # AttributeFilePathInfoExtractor Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AttributeFilePathInfoExtractor {
    /// # Source Path Attribute
    /// Attribute containing the file path to analyze for extracting file system information
    attribute: Attribute,
}

impl Processor for AttributeFilePathInfoExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let Some(path) = feature.get(&self.attribute) else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        let AttributeValue::String(path) = path else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        let path = Path::new(path);
        let mut attributes = feature.attributes.clone();
        if path.exists() && !path.is_symlink() {
            let metadata = metadata(&path)?;
            if metadata.is_dir {
                attributes.insert(
                    Attribute::new("fileType"),
                    AttributeValue::String("Directory".to_string()),
                );
                let size = get_dir_size(path)?;
                attributes.insert(
                    Attribute::new("fileSize"),
                    AttributeValue::Number(Number::from(size)),
                );
            } else {
                attributes.insert(
                    Attribute::new("fileType"),
                    AttributeValue::String("File".to_string()),
                );
                attributes.insert(
                    Attribute::new("fileSize"),
                    AttributeValue::Number(Number::from(metadata.size)),
                );
            }

            if let Some(atime) = chrono::DateTime::<chrono::Utc>::from_timestamp(metadata.atime, 0)
            {
                attributes.insert(
                    Attribute::new("fileAtime"),
                    AttributeValue::DateTime(atime.into()),
                );
            }
            if let Some(mtime) = chrono::DateTime::<chrono::Utc>::from_timestamp(metadata.mtime, 0)
            {
                attributes.insert(
                    Attribute::new("fileMtime"),
                    AttributeValue::DateTime(mtime.into()),
                );
            }
            if let Some(ctime) = chrono::DateTime::<chrono::Utc>::from_timestamp(metadata.ctime, 0)
            {
                attributes.insert(
                    Attribute::new("fileCtime"),
                    AttributeValue::DateTime(ctime.into()),
                );
            }
        }
        fw.send(
            ctx.new_with_feature_and_port(
                feature.with_attributes(attributes),
                DEFAULT_PORT.clone(),
            ),
        );
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeFilePathInfoExtractor"
    }
}
