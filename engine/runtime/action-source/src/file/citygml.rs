use std::{collections::HashMap, sync::Arc};

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, Source, SourceFactory, DEFAULT_PORT},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use super::reader::citygml;
use super::reader::runner::{get_content, get_input_path};
use crate::{errors::SourceError, file::reader::runner::FileReaderCommonParam};

#[derive(Debug, Clone, Default)]
pub(crate) struct CityGmlReaderFactory;

impl SourceFactory for CityGmlReaderFactory {
    fn name(&self) -> &str {
        "CityGmlReader"
    }

    fn description(&self) -> &str {
        "Reads 3D city models from CityGML files."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CityGmlReaderParam))
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
        let params = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SourceError::CityGmlReaderFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SourceError::CityGmlReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SourceError::CityGmlReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let reader = CityGmlReader { params };
        Ok(Box::new(reader))
    }
}

#[derive(Debug, Clone)]
pub(super) struct CityGmlReader {
    pub(super) params: CityGmlReaderParam,
}

/// # CityGmlReader Parameters
///
/// Configuration for reading CityGML files as 3D city models.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct CityGmlReaderParam {
    #[serde(flatten)]
    pub(super) common_property: FileReaderCommonParam,
    #[serde(flatten)]
    pub(super) property: citygml::CityGmlReaderParam,
}

#[async_trait::async_trait]
impl Source for CityGmlReader {
    async fn initialize(&self, _ctx: NodeContext) {}

    fn name(&self) -> &str {
        "CityGmlReader"
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
        let input_path = get_input_path(&ctx, &self.params.common_property)?;
        let content = get_content(&ctx, &self.params.common_property, storage_resolver).await?;
        citygml::read_citygml(&content, input_path, &self.params.property, sender)
            .await
            .map_err(Into::<BoxedError>::into)
    }
}
