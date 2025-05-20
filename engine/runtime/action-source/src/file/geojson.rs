use std::{collections::HashMap, sync::Arc};

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, Source, SourceFactory, DEFAULT_PORT},
};
use reearth_flow_types::Expr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use super::reader::geojson;
use super::reader::runner::get_content;
use crate::{errors::SourceError, file::reader::runner::FileReaderCommonParam};

#[derive(Debug, Clone, Default)]
pub(crate) struct GeoJsonReaderFactory;

impl SourceFactory for GeoJsonReaderFactory {
    fn name(&self) -> &str {
        "GeoJsonReader"
    }

    fn description(&self) -> &str {
        "Reads features from a geojson file"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FileReaderCommonParam))
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
        let common_property = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SourceError::FileReaderFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SourceError::FileReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(SourceError::FileReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let reader = GeoJsonReader { common_property };
        Ok(Box::new(reader))
    }
}

#[derive(Debug, Clone)]
pub(super) struct GeoJsonReader {
    pub(super) common_property: FileReaderCommonParam,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct GeoJsonReaderParam {
    pub(super) dataset: Option<Expr>,
    pub(super) inline: Option<Expr>,
}

#[async_trait::async_trait]
impl Source for GeoJsonReader {
    async fn initialize(&self, _ctx: NodeContext) {}

    fn name(&self) -> &str {
        "GeoJsonReader"
    }

    async fn serialize_state(&self) -> Result<Vec<u8>, BoxedError> {
        Ok(vec![])
    }

    async fn start(
        &mut self,
        ctx: NodeContext,
        sender: Sender<(Port, IngestionMessage)>,
    ) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);

        let content = get_content(&ctx, &self.common_property, storage_resolver).await?;
        geojson::read_geojson(&content, sender)
            .await
            .map_err(Into::<BoxedError>::into)
    }
}
