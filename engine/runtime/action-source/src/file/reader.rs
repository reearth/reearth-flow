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

pub(crate) mod citygml;
pub(crate) mod csv;
pub(crate) mod geojson;
pub(crate) mod json;
pub(crate) mod runner;
pub(crate) mod shapefile;

#[derive(Debug, Clone, Default)]
pub struct FileReaderFactory;

impl SourceFactory for FileReaderFactory {
    fn name(&self) -> &str {
        "FileReader"
    }

    fn description(&self) -> &str {
        "Reads features from a file"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FileReader))
    }

    fn categories(&self) -> &[&'static str] {
        &["File"]
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
                SourceError::FileReaderFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SourceError::FileReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
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
