use std::{collections::HashMap, sync::Arc};

use reearth_flow_common::csv::Delimiter;
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

use super::reader::csv;
use super::reader::runner::get_content;
use crate::{errors::SourceError, file::reader::runner::FileReaderCommonParam};

#[derive(Debug, Clone, Default)]
pub(crate) struct CsvReaderFactory;

impl SourceFactory for CsvReaderFactory {
    fn name(&self) -> &str {
        "CsvReader"
    }

    fn description(&self) -> &str {
        "Reads features from a csv/tsv file"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CsvReaderParam))
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
        let reader = CsvReader { params };
        Ok(Box::new(reader))
    }
}

#[derive(Debug, Clone)]
pub(super) struct CsvReader {
    params: CsvReaderParam,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct CsvReaderParam {
    #[serde(flatten)]
    pub(super) common_property: FileReaderCommonParam,
    #[serde(flatten)]
    property: csv::CsvReaderParam,
    format: CsvFormat,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum CsvFormat {
    Csv,
    Tsv,
}

impl CsvFormat {
    fn delimiter(&self) -> Delimiter {
        match self {
            CsvFormat::Csv => Delimiter::Comma,
            CsvFormat::Tsv => Delimiter::Tab,
        }
    }
}

#[async_trait::async_trait]
impl Source for CsvReader {
    async fn initialize(&self, _ctx: NodeContext) {}

    fn name(&self) -> &str {
        "CsvReader"
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

        let content = get_content(&ctx, &self.params.common_property, storage_resolver).await?;
        csv::read_csv(
            self.params.format.delimiter(),
            &content,
            &self.params.property,
            sender,
        )
        .await
        .map_err(Into::<BoxedError>::into)
    }
}
