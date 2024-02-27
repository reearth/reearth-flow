use core::result::Result;
use std::{collections::HashMap, str::FromStr, sync::Arc};

use anyhow::anyhow;
use bytes::Bytes;
use reearth_flow_storage::resolve::StorageResolver;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use reearth_flow_common::uri::Uri;
use reearth_flow_workflow::graph::NodeProperty;

use crate::action::{ActionContext, ActionDataframe, ActionValue, DEFAULT_PORT};
use crate::error::Error;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PropertySchema {
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

impl TryFrom<NodeProperty> for PropertySchema {
    type Error = anyhow::Error;

    fn try_from(node_property: NodeProperty) -> Result<Self, anyhow::Error> {
        serde_json::from_value(Value::Object(node_property)).map_err(|e| {
            anyhow!(
                "Failed to convert NodeProperty to PropertySchema with {}",
                e
            )
        })
    }
}

pub(crate) async fn run(
    ctx: ActionContext,
    inputs: Option<ActionDataframe>,
) -> anyhow::Result<ActionDataframe> {
    let props = PropertySchema::try_from(ctx.node_property)?;
    debug!(?props, "read");
    let storage_resolver = Arc::clone(&ctx.storage_resolver);
    match props.format {
        Format::Csv => write_csv(inputs, b',', &props, storage_resolver).await?,
        Format::Tsv => write_csv(inputs, b'\t', &props, storage_resolver).await?,
        Format::Json => write_json(inputs, &props, storage_resolver).await?,
        Format::Text => write_text(inputs, &props, storage_resolver).await?,
    };
    let mut output: ActionDataframe = HashMap::new();
    let summary = vec![("output".to_owned(), ActionValue::String(props.output))]
        .into_iter()
        .collect::<HashMap<_, _>>();
    output.insert(DEFAULT_PORT.to_string(), Some(ActionValue::Map(summary)));
    Ok(output)
}

async fn write_text(
    inputs: Option<ActionDataframe>,
    props: &PropertySchema,
    storage_resolver: Arc<StorageResolver>,
) -> anyhow::Result<ActionValue> {
    let value = get_input_value(inputs)?;
    let bytes = match value {
        ActionValue::String(s) => Bytes::from(s),
        _ => return Err(anyhow!("Unsupported input")),
    };
    let uri = Uri::from_str(&props.output)?;
    let storage = storage_resolver.resolve(&uri)?;
    storage.put(uri.path().as_path(), bytes).await?;
    Ok(ActionValue::Bool(true))
}

async fn write_json(
    inputs: Option<ActionDataframe>,
    props: &PropertySchema,
    storage_resolver: Arc<StorageResolver>,
) -> anyhow::Result<ActionValue> {
    let value = get_input_value(inputs)?;
    let json_value: serde_json::Value = value.into();

    let uri = Uri::from_str(&props.output)?;
    let storage = storage_resolver.resolve(&uri)?;
    storage
        .put(uri.path().as_path(), Bytes::from(json_value.to_string()))
        .await?;
    Ok(ActionValue::Bool(true))
}

async fn write_csv(
    inputs: Option<ActionDataframe>,
    delimiter: u8,
    props: &PropertySchema,
    storage_resolver: Arc<StorageResolver>,
) -> anyhow::Result<ActionValue> {
    let value = get_input_value(inputs)?;
    match value {
        ActionValue::Array(s) => {
            let mut wtr = csv::WriterBuilder::new()
                .delimiter(delimiter)
                .quote_style(csv::QuoteStyle::NonNumeric)
                .from_writer(vec![]);
            let fields = get_fields(&s);
            if let Some(ref fields) = fields {
                if !fields.is_empty() {
                    wtr.write_record(fields.clone())?;
                }
            }
            for row in s {
                match fields {
                    Some(ref fields) if !fields.is_empty() => {
                        let values = get_row_values(&row, &fields.clone())?;
                        wtr.write_record(values)?;
                    }
                    _ => match row {
                        ActionValue::String(s) => wtr.write_record(vec![s])?,
                        ActionValue::Array(s) => {
                            let values = s
                                .into_iter()
                                .map(|v| match v {
                                    ActionValue::String(s) => s,
                                    _ => "".to_string(),
                                })
                                .collect::<Vec<_>>();
                            wtr.write_record(values)?
                        }
                        _ => return Err(Error::unsupported_feature("Unsupported input").into()),
                    },
                }
            }
            wtr.flush()?;
            let data = String::from_utf8(wtr.into_inner()?)?;
            let uri = Uri::from_str(&props.output)?;
            let storage = storage_resolver.resolve(&uri)?;
            storage.put(uri.path().as_path(), Bytes::from(data)).await?;
        }
        _ => return Err(Error::unsupported_feature("Unsupported input").into()),
    };
    Ok(ActionValue::Bool(true))
}

fn get_input_value(dataframe: Option<ActionDataframe>) -> anyhow::Result<ActionValue> {
    dataframe
        .ok_or(Error::internal_runtime("No input"))?
        .get(DEFAULT_PORT)
        .ok_or(Error::internal_runtime("No input"))?
        .clone()
        .ok_or(Error::internal_runtime("No input").into())
}

fn get_fields(rows: &[ActionValue]) -> Option<Vec<String>> {
    rows.first().map(|row| match row {
        ActionValue::Map(row) => row.keys().cloned().collect::<Vec<_>>(),
        _ => vec![],
    })
}

fn get_row_values(row: &ActionValue, fields: &[String]) -> anyhow::Result<Vec<String>> {
    fields
        .iter()
        .map(|field| match row {
            ActionValue::Map(row) => row
                .get(field)
                .map(|v| v.to_string())
                .ok_or_else(|| anyhow!("Field not found: {}", field)),
            _ => Err(Error::unsupported_feature("Unsupported input").into()),
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
        let props = PropertySchema {
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
        let props = PropertySchema {
            format: Format::Csv,
            output: "ram:///root/output.csv".to_owned(),
        };
        let resolver = Arc::new(StorageResolver::default());
        let result = write_csv(inputs, b',', &props, resolver).await;
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
