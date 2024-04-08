use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use jsonpath_rust::JsonPathQuery;
use reearth_flow_action::{
    error, ActionContext, ActionDataframe, ActionResult, ActionValue, AsyncAction,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JSONExtractor {
    inputs: Vec<AttributeMapping>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct AttributeMapping {
    attribute: String,
    query: String,
}

#[async_trait::async_trait]
#[typetag::serde(name = "JSONExtractor")]
impl AsyncAction for JSONExtractor {
    async fn run(&self, _ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(error::Error::input("No input dataframe"))?;
        let mut output = ActionDataframe::new();
        for (port, data) in inputs {
            let data = match data {
                Some(data) => data,
                None => continue,
            };
            let mut processed_data: HashMap<String, ActionValue> = HashMap::new();
            for input in self.inputs {
                let data: Value = data.clone().into();
                processed_data.insert(
                    input.attribute,
                    data.path(&input.query).unwrap_or_default().into(),
                );
            }
            output.insert(port, Some(ActionValue::Map(processed_data)));
        }
        Ok(output)
    }
}
