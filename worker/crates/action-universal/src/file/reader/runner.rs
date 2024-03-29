use std::collections::HashMap;
use std::sync::Arc;

use reearth_flow_common::uri::Uri;
use reearth_flow_eval_expr::engine::Engine;
use serde::{Deserialize, Serialize};

use reearth_flow_common::csv::Delimiter;

use super::{csv, text};
use reearth_flow_action::error::Error;
use reearth_flow_action::{
    utils, Action, ActionContext, ActionDataframe, ActionResult, ActionValue, Result, DEFAULT_PORT,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommonPropertySchema {
    pub(super) dataset: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "format")]
pub enum FileReader {
    #[serde(rename = "csv")]
    Csv {
        #[serde(flatten)]
        common_property: CommonPropertySchema,
        #[serde(flatten)]
        property: csv::CsvPropertySchema,
    },
    #[serde(rename = "tsv")]
    Tsv {
        #[serde(flatten)]
        common_property: CommonPropertySchema,
        #[serde(flatten)]
        property: csv::CsvPropertySchema,
    },
    #[serde(rename = "text")]
    Text {
        #[serde(flatten)]
        common_property: CommonPropertySchema,
    },
    #[serde(rename = "json")]
    Json {
        #[serde(flatten)]
        common_property: CommonPropertySchema,
    },
}

#[async_trait::async_trait]
#[typetag::serde(name = "FileReader")]
impl Action for FileReader {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let data = match self {
            Self::Csv {
                common_property,
                property,
            } => {
                let input_path = get_input_path(
                    &inputs.unwrap_or_default(),
                    common_property,
                    Arc::clone(&ctx.expr_engine),
                )
                .await?;
                let result =
                    csv::read_csv(Delimiter::Comma, input_path, property, storage_resolver).await?;
                ActionValue::Array(result)
            }
            Self::Tsv {
                common_property,
                property,
            } => {
                let input_path = get_input_path(
                    &inputs.unwrap_or_default(),
                    common_property,
                    Arc::clone(&ctx.expr_engine),
                )
                .await?;
                let result =
                    csv::read_csv(Delimiter::Tab, input_path, property, storage_resolver).await?;
                ActionValue::Array(result)
            }
            Self::Text { common_property } => {
                let input_path = get_input_path(
                    &inputs.unwrap_or_default(),
                    common_property,
                    Arc::clone(&ctx.expr_engine),
                )
                .await?;
                text::read_text(input_path, storage_resolver).await?
            }
            _ => return Err(Error::unsupported_feature("Unsupported format")),
        };
        Ok(HashMap::from([(DEFAULT_PORT.clone(), Some(data))]))
    }
}

async fn get_input_path(
    inputs: &ActionDataframe,
    common_property: &CommonPropertySchema,
    expr_engine: Arc<Engine>,
) -> Result<Uri> {
    utils::get_expr_path(common_property.dataset.as_str(), inputs, expr_engine).await
}
