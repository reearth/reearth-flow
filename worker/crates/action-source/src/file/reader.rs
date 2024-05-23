use std::collections::HashMap;

use reearth_flow_eval_expr::Value;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{Port, Source, SourceFactory, DEFAULT_PORT},
};

use crate::errors::SourceError;

use self::runner::FileReader;

pub mod citygml;
pub mod csv;
pub mod json;
pub mod runner;

#[derive(Debug, Clone, Default)]
pub struct FileReaderFactory;

impl SourceFactory for FileReaderFactory {
    fn name(&self) -> &str {
        "FileReader"
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
        _state: Option<Vec<u8>>,
    ) -> Result<Box<dyn Source>, BoxedError> {
        let processor: FileReader = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SourceError::FileReaderFactory(format!("Failed to serialize with: {}", e))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SourceError::FileReaderFactory(format!("Failed to deserialize with: {}", e))
            })?
        } else {
            return Err(SourceError::FileReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(processor))
    }
}
