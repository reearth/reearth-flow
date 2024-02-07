use core::result::Result;
use std::{collections::HashMap, str::FromStr};

use anyhow::anyhow;
use bytes::Bytes;
use csv::Writer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve;
use reearth_flow_workflow::error::Error;
use reearth_flow_workflow::graph::NodeProperty;

use crate::action::{ActionContext, ActionDataframe, ActionValue, DEFAULT_PORT};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PropertySchema {
    pub format: Format,
    pub output: String,
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
        let value = Value::Object(node_property);
        serde_json::from_value(value).map_err(|e| {
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
    info!(?props, "read");
    match props.format {
        Format::Csv => write_csv(inputs, &props).await?,
        Format::Text => write_text(inputs, &props).await?,
        _ => panic!("Unsupported format"),
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
) -> anyhow::Result<ActionValue> {
    let value = get_input_value(inputs)?;
    let bytes = match value {
        ActionValue::String(s) => Bytes::from(s),
        _ => return Err(anyhow!("Unsupported input")),
    };
    let uri = Uri::from_str(&props.output)?;
    let storage = resolve(&uri)?;
    storage.put(uri.path().as_path(), bytes).await?;
    Ok(ActionValue::Bool(true))
}

async fn write_csv(
    inputs: Option<ActionDataframe>,
    props: &PropertySchema,
) -> anyhow::Result<ActionValue> {
    let value = get_input_value(inputs)?;
    match value {
        ActionValue::ArrayMap(s) => {
            let mut wtr = Writer::from_writer(vec![]);
            let fields = get_fields(&s);
            wtr.write_record(fields.clone())?;
            for row in s {
                let values = get_row_values(&row, &fields)?;
                wtr.write_record(values)?;
            }
            wtr.flush()?;
            let data = String::from_utf8(wtr.into_inner()?)?;
            let uri = Uri::from_str(&props.output)?;
            let storage = resolve(&uri)?;
            storage.put(uri.path().as_path(), Bytes::from(data)).await?;
        }
        _ => return Err(anyhow!("Unsupported input")),
    };
    Ok(ActionValue::Bool(true))
}

fn get_input_value(dataframe: Option<ActionDataframe>) -> anyhow::Result<ActionValue> {
    dataframe
        .ok_or(Error::internal_runtime_error("No input"))?
        .get(DEFAULT_PORT)
        .ok_or(Error::internal_runtime_error("No input"))?
        .clone()
        .ok_or(Error::internal_runtime_error("No input").into())
}

fn get_fields(rows: &[HashMap<String, ActionValue>]) -> Vec<String> {
    rows.first()
        .map(|row| row.keys().cloned().collect())
        .unwrap_or_default()
}

fn get_row_values(
    row: &HashMap<String, ActionValue>,
    fields: &[String],
) -> anyhow::Result<Vec<String>> {
    fields
        .iter()
        .map(|field| {
            row.get(field)
                .map(|value| match value {
                    ActionValue::String(s) => Ok(s.to_owned()),
                    _ => Err(anyhow!("Unsupported input")),
                })
                .unwrap_or_else(|| Err(anyhow!("Field not found")))
        })
        .collect()
}
