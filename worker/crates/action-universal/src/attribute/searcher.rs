use regex::{escape, Regex};
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, ActionContext, ActionDataframe, ActionResult, AsyncAction, AttributeValue,
    Dataframe, Feature,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttributeStringSearcher {
    search_in: String,
    contains_regular_expression: bool,
}

#[async_trait::async_trait]
#[typetag::serde(name = "AttributeStringSearcher")]
impl AsyncAction for AttributeStringSearcher {
    async fn run(&self, _ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let re = if self.contains_regular_expression {
            Regex::new(&self.search_in)
                .map_err(|e| Error::input(format!("Invalid regex pattern with error: {:?}", e)))
        } else {
            Regex::new(&escape(&self.search_in))
                .map_err(|e| Error::input(format!("Invalid regex pattern with error: {:?}", e)))
        }?;
        let output = inputs
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    Dataframe::new(
                        v.features
                            .iter()
                            .flat_map(|v| search(v, &re))
                            .collect::<Vec<_>>(),
                    ),
                )
            })
            .collect();
        Ok(output)
    }
}

fn search(v: &Feature, re: &Regex) -> Option<Feature> {
    if v.attributes.iter().any(|(_, v)| match v {
        AttributeValue::String(s) => re.is_match(s),
        _ => false,
    }) {
        Some(v.clone())
    } else {
        None
    }
}
