use core::result::Result;
use std::collections::HashMap;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use reearth_flow_workflow::graph::NodeProperty;

use crate::action::{ActionContext, ActionDataframe, ActionValue};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PropertySchema {
    keep_attributes: Vec<String>,
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
    let output = match inputs {
        Some(inputs) => {
            let mut output = HashMap::new();
            for (port, data) in inputs {
                let data = match data {
                    Some(data) => data,
                    None => continue,
                };
                let processed_data = match data {
                    ActionValue::Array(data) => {
                        let processed_items = data
                            .into_iter()
                            .map(|item| match item {
                                ActionValue::Map(item) => {
                                    let processed_item = item
                                        .into_iter()
                                        .filter(|(key, _)| props.keep_attributes.contains(key))
                                        .collect::<HashMap<_, _>>();
                                    ActionValue::Map(processed_item)
                                }
                                _ => ActionValue::Map(HashMap::new()),
                            })
                            .collect::<Vec<_>>();
                        ActionValue::Array(processed_items)
                    }
                    ActionValue::Map(data) => {
                        let processed_data = data
                            .into_iter()
                            .filter(|(key, _)| props.keep_attributes.contains(key))
                            .collect();
                        ActionValue::Map(processed_data)
                    }
                    _ => continue,
                };
                output.insert(port, Some(processed_data));
            }
            output
        }
        None => HashMap::new(),
    };
    Ok(output)
}
