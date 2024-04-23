use std::{collections::HashMap, str::FromStr, sync::Arc};

use bytes::Bytes;
use reearth_flow_storage::resolve::StorageResolver;
use serde::{Deserialize, Serialize};

use reearth_flow_common::csv::Delimiter;
use reearth_flow_common::uri::Uri;

use reearth_flow_action::error::Error;
use reearth_flow_action::{
    ActionContext, ActionDataframe, ActionResult, AsyncAction, AttributeValue, Dataframe, Feature,
    Result, DEFAULT_PORT,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileWriter {
    format: Format,
    output: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Format {
    #[serde(rename = "csv")]
    Csv,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "tsv")]
    Tsv,
}

#[async_trait::async_trait]
#[typetag::serde(name = "FileWriter")]
impl AsyncAction for FileWriter {
    async fn run(&self, ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        match self.format {
            Format::Csv => write_csv(inputs, Delimiter::Comma, self, storage_resolver).await?,
            Format::Tsv => write_csv(inputs, Delimiter::Tab, self, storage_resolver).await?,
            Format::Json => write_json(inputs, self, storage_resolver).await?,
        };
        let mut output: ActionDataframe = HashMap::new();
        let summary = vec![(
            "output".to_owned(),
            AttributeValue::String(self.output.clone()),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>();
        output.insert(
            DEFAULT_PORT.clone(),
            Dataframe::new(vec![summary.into()] as Vec<Feature>),
        );
        Ok(output)
    }
}

async fn write_json(
    inputs: ActionDataframe,
    props: &FileWriter,
    storage_resolver: Arc<StorageResolver>,
) -> Result<AttributeValue> {
    let value = inputs
        .get(&DEFAULT_PORT)
        .ok_or(Error::input("No Default Port"))?;
    let json_value: serde_json::Value = value.clone().into();

    let uri = Uri::from_str(&props.output).map_err(Error::input)?;
    let storage = storage_resolver.resolve(&uri).map_err(Error::input)?;
    storage
        .put(uri.path().as_path(), Bytes::from(json_value.to_string()))
        .await
        .map_err(Error::internal_runtime)?;
    Ok(AttributeValue::Bool(true))
}

async fn write_csv(
    inputs: ActionDataframe,
    delimiter: Delimiter,
    props: &FileWriter,
    storage_resolver: Arc<StorageResolver>,
) -> Result<AttributeValue> {
    let value = inputs
        .get(&DEFAULT_PORT)
        .ok_or(Error::input("No Default Port"))?;
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter.into())
        .quote_style(csv::QuoteStyle::NonNumeric)
        .from_writer(vec![]);
    let rows: Vec<AttributeValue> = value.clone().into();
    let fields = get_fields(&rows);
    if let Some(ref fields) = fields {
        if !fields.is_empty() {
            wtr.write_record(fields.clone())
                .map_err(Error::internal_runtime)?;
        }
    }
    for row in rows {
        match fields {
            Some(ref fields) if !fields.is_empty() => {
                let values = get_row_values(&row, &fields.clone())?;
                wtr.write_record(values).map_err(Error::internal_runtime)?;
            }
            _ => match row {
                AttributeValue::String(s) => {
                    wtr.write_record(vec![s]).map_err(Error::internal_runtime)?
                }
                AttributeValue::Array(s) => {
                    let values = s
                        .into_iter()
                        .map(|v| match v {
                            AttributeValue::String(s) => s,
                            _ => "".to_string(),
                        })
                        .collect::<Vec<_>>();
                    wtr.write_record(values).map_err(Error::internal_runtime)?
                }
                _ => return Err(Error::unsupported_feature("Unsupported input")),
            },
        }
    }
    wtr.flush()?;
    let data = String::from_utf8(wtr.into_inner().map_err(Error::internal_runtime)?)
        .map_err(Error::internal_runtime)?;
    let uri = Uri::from_str(&props.output).map_err(Error::input)?;
    let storage = storage_resolver.resolve(&uri).map_err(Error::input)?;
    storage
        .put(uri.path().as_path(), Bytes::from(data))
        .await
        .map_err(Error::internal_runtime)?;
    Ok(AttributeValue::Bool(true))
}

fn get_fields(rows: &[AttributeValue]) -> Option<Vec<String>> {
    rows.first().map(|row| match row {
        AttributeValue::Map(row) => row.keys().cloned().collect::<Vec<_>>(),
        _ => vec![],
    })
}

fn get_row_values(row: &AttributeValue, fields: &[String]) -> Result<Vec<String>> {
    fields
        .iter()
        .map(|field| match row {
            AttributeValue::Map(row) => row
                .get(field)
                .map(|v| v.to_string())
                .ok_or_else(|| Error::input(format!("Field not found: {}", field))),
            _ => Err(Error::unsupported_feature("Unsupported input")),
        })
        .collect()
}
