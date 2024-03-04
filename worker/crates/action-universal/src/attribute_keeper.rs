use std::collections::HashMap;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use reearth_flow_action::{Action, ActionContext, ActionDataframe, ActionResult, ActionValue};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttributeKeeper {
    keep_attributes: Vec<String>,
}

#[async_trait::async_trait]
#[typetag::serde(name = "attributeKeeper")]
impl Action for AttributeKeeper {
    async fn run(&self, _ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
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
                                            .filter(|(key, _)| self.keep_attributes.contains(key))
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
                                .filter(|(key, _)| self.keep_attributes.contains(key))
                                .collect();
                            ActionValue::Map(processed_data)
                        }
                        _ => data,
                    };
                    output.insert(port, Some(processed_data));
                }
                output
            }
            None => return Err(anyhow!("No input dataframe")),
        };
        Ok(output)
    }
}
