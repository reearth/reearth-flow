use core::result::Result;
use std::io::Cursor;
use std::{collections::HashMap, str::FromStr};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve;
use reearth_flow_workflow::graph::NodeProperty;

use crate::action::{ActionContext, ActionDataframe, ActionValue, DEFAULT_PORT};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PropertySchema {
    pub format: Format,
    pub dataset: String,
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
    _inputs: Option<ActionDataframe>,
) -> anyhow::Result<ActionDataframe> {
    let props = PropertySchema::try_from(ctx.node_property)?;
    let data = match props.format {
        Format::Csv => {
            let result = read_csv(&props).await?;
            ActionValue::ArrayMap(result)
        }
        Format::Text => read_text(&props).await?,
        _ => panic!("Unsupported format"),
    };
    let mut output = HashMap::new();
    output.insert(DEFAULT_PORT.to_string(), Some(data));
    Ok(output)
}

async fn read_text(props: &PropertySchema) -> anyhow::Result<ActionValue> {
    let uri = Uri::from_str(&props.dataset)?;
    let storage = resolve(&uri)?;
    let result = storage.get(uri.path().as_path()).await?;
    let byte = result.bytes().await?;
    let text = String::from_utf8(byte.to_vec())?;
    Ok(ActionValue::String(text))
}

async fn read_csv(props: &PropertySchema) -> anyhow::Result<Vec<HashMap<String, ActionValue>>> {
    let uri = Uri::from_str(&props.dataset)?;
    let storage = resolve(&uri)?;
    let result = storage.get(uri.path().as_path()).await?;
    let byte = result.bytes().await?;
    let cursor = Cursor::new(byte);
    let mut rdr = csv::Reader::from_reader(cursor);
    let mut result: Vec<HashMap<String, ActionValue>> = Vec::new();
    for rd in rdr.deserialize() {
        let record: HashMap<String, String> = rd?;
        let mut row: HashMap<String, ActionValue> = HashMap::new();
        record.iter().for_each(|(k, v)| {
            row.insert(k.to_string(), ActionValue::String(v.to_string()));
        });
        result.push(row);
    }
    Ok(result)
}
