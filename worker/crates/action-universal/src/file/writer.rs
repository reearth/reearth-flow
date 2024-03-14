use std::{collections::HashMap, str::FromStr, sync::Arc};

use bytes::Bytes;
use reearth_flow_storage::resolve::StorageResolver;
use serde::{Deserialize, Serialize};

use reearth_flow_common::csv::Delimiter;
use reearth_flow_common::uri::Uri;

use reearth_flow_action::error::Error;
use reearth_flow_action::{
    Action, ActionContext, ActionDataframe, ActionResult, ActionValue, Result, DEFAULT_PORT,
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
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "tsv")]
    Tsv,
}

#[async_trait::async_trait]
#[typetag::serde(name = "FileWriter")]
impl Action for FileWriter {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        match self.format {
            Format::Csv => write_csv(inputs, Delimiter::Comma, self, storage_resolver).await?,
            Format::Tsv => write_csv(inputs, Delimiter::Tab, self, storage_resolver).await?,
            Format::Json => write_json(inputs, self, storage_resolver).await?,
            Format::Text => write_text(inputs, self, storage_resolver).await?,
        };
        let mut output: ActionDataframe = HashMap::new();
        let summary = vec![(
            "output".to_owned(),
            ActionValue::String(self.output.clone()),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>();
        output.insert(DEFAULT_PORT.to_string(), Some(ActionValue::Map(summary)));
        Ok(output)
    }
}

async fn write_text(
    inputs: Option<ActionDataframe>,
    props: &FileWriter,
    storage_resolver: Arc<StorageResolver>,
) -> Result<ActionValue> {
    let value = get_input_value(inputs)?;
    let bytes = match value {
        ActionValue::String(s) => Bytes::from(s),
        _ => return Err(Error::unsupported_feature("Unsupported input")),
    };
    let uri = Uri::from_str(&props.output).map_err(Error::input)?;
    let storage = storage_resolver.resolve(&uri).map_err(Error::input)?;
    storage
        .put(uri.path().as_path(), bytes)
        .await
        .map_err(Error::internal_runtime)?;
    Ok(ActionValue::Bool(true))
}

async fn write_json(
    inputs: Option<ActionDataframe>,
    props: &FileWriter,
    storage_resolver: Arc<StorageResolver>,
) -> Result<ActionValue> {
    let value = get_input_value(inputs)?;
    let json_value: serde_json::Value = value.into();

    let uri = Uri::from_str(&props.output).map_err(Error::input)?;
    let storage = storage_resolver.resolve(&uri).map_err(Error::input)?;
    storage
        .put(uri.path().as_path(), Bytes::from(json_value.to_string()))
        .await
        .map_err(Error::internal_runtime)?;
    Ok(ActionValue::Bool(true))
}

async fn write_csv(
    inputs: Option<ActionDataframe>,
    delimiter: Delimiter,
    props: &FileWriter,
    storage_resolver: Arc<StorageResolver>,
) -> Result<ActionValue> {
    let value = get_input_value(inputs)?;
    match value {
        ActionValue::Array(s) => {
            let mut wtr = csv::WriterBuilder::new()
                .delimiter(delimiter.into())
                .quote_style(csv::QuoteStyle::NonNumeric)
                .from_writer(vec![]);
            let fields = get_fields(&s);
            if let Some(ref fields) = fields {
                if !fields.is_empty() {
                    wtr.write_record(fields.clone())
                        .map_err(Error::internal_runtime)?;
                }
            }
            for row in s {
                match fields {
                    Some(ref fields) if !fields.is_empty() => {
                        let values = get_row_values(&row, &fields.clone())?;
                        wtr.write_record(values).map_err(Error::internal_runtime)?;
                    }
                    _ => match row {
                        ActionValue::String(s) => {
                            wtr.write_record(vec![s]).map_err(Error::internal_runtime)?
                        }
                        ActionValue::Array(s) => {
                            let values = s
                                .into_iter()
                                .map(|v| match v {
                                    ActionValue::String(s) => s,
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
        }
        _ => return Err(Error::unsupported_feature("Unsupported input")),
    };
    Ok(ActionValue::Bool(true))
}

fn get_input_value(dataframe: Option<ActionDataframe>) -> Result<ActionValue> {
    dataframe
        .ok_or(Error::internal_runtime("No input"))?
        .get(DEFAULT_PORT)
        .ok_or(Error::internal_runtime("No input"))?
        .clone()
        .ok_or(Error::internal_runtime("No input"))
}

fn get_fields(rows: &[ActionValue]) -> Option<Vec<String>> {
    rows.first().map(|row| match row {
        ActionValue::Map(row) => row.keys().cloned().collect::<Vec<_>>(),
        _ => vec![],
    })
}

fn get_row_values(row: &ActionValue, fields: &[String]) -> Result<Vec<String>> {
    fields
        .iter()
        .map(|field| match row {
            ActionValue::Map(row) => row
                .get(field)
                .map(|v| v.to_string())
                .ok_or_else(|| Error::input(format!("Field not found: {}", field))),
            _ => Err(Error::unsupported_feature("Unsupported input")),
        })
        .collect()
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[tokio::test]
    async fn test_write_text() {
        let inputs = Some(
            vec![(
                DEFAULT_PORT.to_string(),
                Some(ActionValue::String("value".to_owned())),
            )]
            .into_iter()
            .collect::<ActionDataframe>(),
        );
        let props = FileWriter {
            format: Format::Text,
            output: "ram:///root/output.txt".to_owned(),
        };
        let resolver = Arc::new(StorageResolver::default());
        let result = write_text(inputs, &props, resolver).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_write_csv() {
        let inputs = Some(
            vec![(
                DEFAULT_PORT.to_string(),
                Some(ActionValue::Array(vec![
                    ActionValue::Map(
                        vec![(
                            "field1".to_owned(),
                            ActionValue::String("value1".to_owned()),
                        )]
                        .into_iter()
                        .collect(),
                    ),
                    ActionValue::Map(
                        vec![(
                            "field1".to_owned(),
                            ActionValue::String("value2".to_owned()),
                        )]
                        .into_iter()
                        .collect(),
                    ),
                ])),
            )]
            .into_iter()
            .collect::<ActionDataframe>(),
        );
        let props = FileWriter {
            format: Format::Csv,
            output: "ram:///root/output.csv".to_owned(),
        };
        let resolver = Arc::new(StorageResolver::default());
        let result = write_csv(inputs, Delimiter::Comma, &props, resolver).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_fields() {
        let rows = vec![
            ActionValue::Map(
                vec![(
                    "field1".to_owned(),
                    ActionValue::String("value1".to_owned()),
                )]
                .into_iter()
                .collect(),
            ),
            ActionValue::Map(
                vec![(
                    "field1".to_owned(),
                    ActionValue::String("value2".to_owned()),
                )]
                .into_iter()
                .collect(),
            ),
        ];
        let result = get_fields(&rows);
        assert_eq!(result, Some(vec!["field1".to_owned()]));
    }
}
