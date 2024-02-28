use core::result::Result;
use std::collections::HashMap;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use reearth_flow_macros::PropertySchema;

use crate::action::{ActionContext, ActionDataframe, ActionValue};

#[derive(Serialize, Deserialize, Debug, PropertySchema)]
#[serde(rename_all = "camelCase")]
struct PropertySchema {
    keep_attributes: Vec<String>,
}

pub(crate) async fn run(
    ctx: ActionContext,
    inputs: Option<ActionDataframe>,
) -> anyhow::Result<ActionDataframe> {
    let props = PropertySchema::try_from(ctx.node_property)?;
    debug!(?props, "read");
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
                            .filter_map(|item| match item {
                                ActionValue::Map(item) => {
                                    let processed_item = item
                                        .into_iter()
                                        .filter(|(key, _)| props.keep_attributes.contains(key))
                                        .collect::<HashMap<_, _>>();
                                    Some(ActionValue::Map(processed_item))
                                }
                                _ => None,
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
        None => return Err(anyhow!("No input dataframe")),
    };
    Ok(output)
}
