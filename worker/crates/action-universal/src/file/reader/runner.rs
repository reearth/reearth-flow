use std::sync::Arc;

use reearth_flow_common::uri::Uri;
use serde::{Deserialize, Serialize};

use reearth_flow_common::csv::Delimiter;

use super::{citygml, csv, json, text};
use reearth_flow_action::{
    ActionContext, ActionDataframe, ActionResult, AsyncAction, AttributeValue, Dataframe, Result,
    DEFAULT_PORT,
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
    #[serde(rename = "citygml")]
    CityGML {
        #[serde(flatten)]
        common_property: CommonPropertySchema,
    },
}

#[async_trait::async_trait]
#[typetag::serde(name = "FileReader")]
impl AsyncAction for FileReader {
    async fn run(&self, ctx: ActionContext, _inputs: ActionDataframe) -> ActionResult {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let results: Dataframe = match self {
            Self::Csv {
                common_property,
                property,
            } => {
                let input_path = get_input_path(&ctx, common_property).await?;
                let result =
                    csv::read_csv(Delimiter::Comma, input_path, property, storage_resolver).await?;
                AttributeValue::Array(result).into()
            }
            Self::Tsv {
                common_property,
                property,
            } => {
                let input_path = get_input_path(&ctx, common_property).await?;
                let result =
                    csv::read_csv(Delimiter::Tab, input_path, property, storage_resolver).await?;
                AttributeValue::Array(result).into()
            }
            Self::Text { common_property } => {
                let input_path = get_input_path(&ctx, common_property).await?;
                text::read_text(input_path, storage_resolver).await?.into()
            }
            Self::Json { common_property } => {
                let input_path = get_input_path(&ctx, common_property).await?;
                json::read_json(input_path, storage_resolver).await?.into()
            }
            Self::CityGML { common_property } => {
                let input_path = get_input_path(&ctx, common_property).await?;
                citygml::read_citygml(input_path, ctx).await?.into()
            }
        };
        let output = ActionDataframe::from([(DEFAULT_PORT.clone(), results)]);
        Ok(output)
    }
}

async fn get_input_path(
    ctx: &ActionContext,
    common_property: &CommonPropertySchema,
) -> Result<Uri> {
    ctx.get_expr_path(&common_property.dataset).await
}
