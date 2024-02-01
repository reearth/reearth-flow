use core::result::Result;
use std::collections::HashMap;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use reearth_flow_workflow::graph::NodeProperty;

use crate::action::{ActionContext, ActionDataframe, ActionValue, DEFAULT_PORT};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PropertySchema {
    pub format: Format,
    pub dataset: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Format {
    #[serde(rename = "csv")]
    Csv,
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

pub async fn run(
    ctx: ActionContext,
    _inputs: Option<ActionDataframe>,
) -> anyhow::Result<ActionDataframe> {
    let props = PropertySchema::try_from(ctx.node_property)?;
    let mut output = HashMap::new();
    output.insert(
        DEFAULT_PORT.to_string(),
        Some(ActionValue::String("feature".to_string())),
    );
    println!("props: {:?}", props);
    println!("node: {:?}", ctx.node_name);
    Ok(output)
}
