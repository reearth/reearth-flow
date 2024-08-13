use std::{str::FromStr, sync::Arc};

use reearth_flow_common::{csv::Delimiter, uri::Uri};
use reearth_flow_runtime::{
    errors::BoxedError,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, Source},
};
use reearth_flow_types::Expr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

use super::{citygml, csv, json};

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileReaderCommonParam {
    pub(super) dataset: Expr,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", tag = "format")]
pub enum FileReader {
    Csv {
        #[serde(flatten)]
        common_property: FileReaderCommonParam,
        #[serde(flatten)]
        property: csv::CsvReaderParam,
    },
    Tsv {
        #[serde(flatten)]
        common_property: FileReaderCommonParam,
        #[serde(flatten)]
        property: csv::CsvReaderParam,
    },
    Json {
        #[serde(flatten)]
        common_property: FileReaderCommonParam,
    },
    Citygml {
        #[serde(flatten)]
        common_property: FileReaderCommonParam,
    },
}

#[async_trait::async_trait]
impl Source for FileReader {
    async fn initialize(&self, _ctx: NodeContext) {}

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
                let input_path = get_input_path(&ctx, common_property)?;
                let result = json::read_json(input_path, storage_resolver, sender).await;
                match result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(Box::new(e)),
                }
            }
            Self::Csv {
                common_property,
                property,
            } => {
                let input_path = get_input_path(&ctx, common_property)?;
                let result = csv::read_csv(
                    Delimiter::Comma,
                    input_path,
                    property,
                    storage_resolver,
                    sender,
                )
                .await;
                match result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(Box::new(e)),
                }
            }
            Self::Tsv {
                common_property,
                property,
            } => {
                let input_path = get_input_path(&ctx, common_property)?;
                let result = csv::read_csv(
                    Delimiter::Tab,
                    input_path,
                    property,
                    storage_resolver,
                    sender,
                )
                .await;
                match result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(Box::new(e)),
                }
            }
            Self::Citygml { common_property } => {
                let input_path = get_input_path(&ctx, common_property)?;
                let result = citygml::read_citygml(input_path, ctx, sender).await;
                match result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(Box::new(e)),
                }
            }
        }
    }
}

fn get_input_path(
    ctx: &NodeContext,
    common_property: &FileReaderCommonParam,
) -> Result<Uri, BoxedError> {
    let path = &common_property.dataset;
    let scope = ctx.expr_engine.new_scope();
    let path = ctx
        .expr_engine
        .eval_scope::<String>(path.as_ref(), &scope)
        .unwrap_or_else(|_| path.to_string());
    let uri = Uri::from_str(path.as_str());
    let Ok(uri) = uri else {
        return Err(Box::new(crate::errors::SourceError::FileReader(
            "Invalid path".to_string(),
        )));
    };
    Ok(uri)
}
