use std::{str::FromStr, sync::Arc};

use bytes::Bytes;
use reearth_flow_common::{csv::Delimiter, uri::Uri};
use reearth_flow_runtime::{
    errors::BoxedError,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, Source},
};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Expr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

use crate::errors::SourceError;

use super::{citygml, csv, geojson, json};

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileReaderCommonParam {
    pub(crate) dataset: Option<Expr>,
    pub(crate) inline: Option<Expr>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", tag = "format")]
pub enum FileReader {
    /// # CSV
    Csv {
        #[serde(flatten)]
        common_property: FileReaderCommonParam,
        #[serde(flatten)]
        property: csv::CsvReaderParam,
    },
    /// # TSV
    Tsv {
        #[serde(flatten)]
        common_property: FileReaderCommonParam,
        #[serde(flatten)]
        property: csv::CsvReaderParam,
    },
    /// # JSON
    Json {
        #[serde(flatten)]
        common_property: FileReaderCommonParam,
    },
    /// # CityGML
    Citygml {
        #[serde(flatten)]
        common_property: FileReaderCommonParam,
        #[serde(flatten)]
        property: citygml::CityGmlReaderParam,
    },
    /// # GeoJSON
    #[serde(rename = "geojson")]
    GeoJson {
        #[serde(flatten)]
        common_property: FileReaderCommonParam,
    },
}

#[async_trait::async_trait]
impl Source for FileReader {
    async fn initialize(&self, _ctx: NodeContext) {}

    fn name(&self) -> &str {
        "FileReader"
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
        match self {
            Self::Json { common_property } => {
                let content = get_content(&ctx, common_property, storage_resolver).await?;
                json::read_json(&content, sender)
                    .await
                    .map_err(Into::<BoxedError>::into)
            }
            Self::GeoJson { common_property } => {
                let content = get_content(&ctx, common_property, storage_resolver).await?;
                geojson::read_geojson(&content, sender)
                    .await
                    .map_err(Into::<BoxedError>::into)
            }
            Self::Csv {
                common_property,
                property,
            } => {
                let content = get_content(&ctx, common_property, storage_resolver).await?;
                csv::read_csv(Delimiter::Comma, &content, property, sender)
                    .await
                    .map_err(Into::<BoxedError>::into)
            }
            Self::Tsv {
                common_property,
                property,
            } => {
                let content = get_content(&ctx, common_property, storage_resolver).await?;
                csv::read_csv(Delimiter::Tab, &content, property, sender)
                    .await
                    .map_err(Into::<BoxedError>::into)
            }
            Self::Citygml {
                common_property,
                property,
            } => {
                let input_path = get_input_path(&ctx, common_property)?;
                let content = get_content(&ctx, common_property, storage_resolver).await?;
                citygml::read_citygml(&content, input_path, property, sender)
                    .await
                    .map_err(Into::<BoxedError>::into)
            }
        }
    }
}

fn get_input_path(
    ctx: &NodeContext,
    common_property: &FileReaderCommonParam,
) -> Result<Option<Uri>, SourceError> {
    let Some(path) = &common_property.dataset else {
        return Ok(None);
    };
    let scope = ctx.expr_engine.new_scope();
    let path = ctx
        .expr_engine
        .eval_scope::<String>(path.as_ref(), &scope)
        .unwrap_or_else(|_| path.to_string());
    let uri = Uri::from_str(path.as_str());
    let Ok(uri) = uri else {
        return Err(crate::errors::SourceError::FileReader(
            "Invalid path".to_string(),
        ));
    };
    Ok(Some(uri))
}

fn get_inline_content(
    ctx: &NodeContext,
    common_property: &FileReaderCommonParam,
) -> Result<Option<Bytes>, SourceError> {
    let Some(inline) = &common_property.inline else {
        return Ok(None);
    };
    let scope = ctx.expr_engine.new_scope();
    let content = ctx
        .expr_engine
        .eval_scope::<String>(inline.as_ref(), &scope)
        .unwrap_or_else(|_| inline.to_string());
    Ok(Some(Bytes::from(content)))
}

pub(crate) async fn get_content(
    ctx: &NodeContext,
    common_property: &FileReaderCommonParam,
    storage_resolver: Arc<StorageResolver>,
) -> Result<Bytes, SourceError> {
    if let Some(content) = get_inline_content(ctx, common_property)? {
        return Ok(content);
    }
    if let Some(input_path) = get_input_path(ctx, common_property)? {
        let storage = storage_resolver
            .resolve(&input_path)
            .map_err(|e| crate::errors::SourceError::FileReader(format!("{e:?}")))?;
        let result = storage
            .get(input_path.path().as_path())
            .await
            .map_err(|e| crate::errors::SourceError::FileReader(format!("{e:?}")))?;
        let byte = result
            .bytes()
            .await
            .map_err(|e| crate::errors::SourceError::FileReader(format!("{e:?}")))?;
        return Ok(byte);
    }
    Err(crate::errors::SourceError::FileReader(
        "Missing required parameter `dataset` or `inline`".to_string(),
    ))
}
