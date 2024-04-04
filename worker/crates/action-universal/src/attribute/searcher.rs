use regex::{escape, Regex};
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, Action, ActionContext, ActionDataframe, ActionResult, ActionValue,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttributeStringSearcher {
    search_in: String,
    contains_regular_expression: bool,
}

#[async_trait::async_trait]
#[typetag::serde(name = "AttributeStringSearcher")]
impl Action for AttributeStringSearcher {
    async fn run(&self, _ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let re = if self.contains_regular_expression {
            Regex::new(&self.search_in).map_err(|_| Error::input("Invalid regex"))
        } else {
            Regex::new(&escape(&self.search_in)).map_err(|_| Error::input("Invalid regex"))
        }?;
        let output = inputs
            .ok_or(Error::input("No input dataframe"))?
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.as_ref().map(|v| search(v.clone(), &re)),
                )
            })
            .collect();
        Ok(output)
    }
}

fn search(v: ActionValue, re: &Regex) -> ActionValue {
    match v {
        ActionValue::String(s) => ActionValue::Array(
            re.find_iter(&s)
                .map(|m| ActionValue::String(m.as_str().to_string()))
                .collect(),
        ),
        ActionValue::Map(kv) => ActionValue::Map(
            kv.into_iter()
                .map(|(k, v)| (k.clone(), search(v, re)))
                .collect(),
        ),
        x => x,
    }
}
