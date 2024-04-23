use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use reearth_flow_action::{ActionContext, ActionDataframe, ActionResult, AsyncAction, Dataframe};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttributeKeeper {
    keep_attributes: Vec<String>,
}

#[async_trait::async_trait]
#[typetag::serde(name = "AttributeKeeper")]
impl AsyncAction for AttributeKeeper {
    async fn run(&self, _ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let mut output = ActionDataframe::new();
        for (port, data) in inputs {
            let processed_data = data
                .features
                .into_iter()
                .filter_map(|item| {
                    let feature = item
                        .iter()
                        .filter(|(key, _)| self.keep_attributes.contains(&key.inner()))
                        .collect::<HashMap<_, _>>();
                    if feature.is_empty() {
                        None
                    } else {
                        let attributes = feature
                            .iter()
                            .map(|(&key, &value)| (key.clone(), value.clone()))
                            .collect::<HashMap<_, _>>();
                        Some(item.with_attributes(attributes))
                    }
                })
                .collect::<Vec<_>>();
            output.insert(port, Dataframe::new(processed_data));
        }
        Ok(output)
    }
}
