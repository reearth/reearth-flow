use std::{collections::HashMap, ops::Not};

use regex::RegexBuilder;
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, ActionContext, ActionDataframe, ActionResult, AsyncAction, Attribute,
    AttributeValue, Dataframe, Feature, Port, DEFAULT_PORT,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttributeStringSearcher {
    search_in: String,
    contains_regular_expression: String,
    matched_result: String,
    case_sensitive: bool,
}

#[async_trait::async_trait]
#[typetag::serde(name = "AttributeStringSearcher")]
impl AsyncAction for AttributeStringSearcher {
    async fn run(&self, _ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let re = RegexBuilder::new(&self.contains_regular_expression)
            .case_insensitive(self.case_sensitive.not())
            .build()
            .map_err(|e| Error::input(format!("Invalid regex pattern with error: {:?}", e)))?;
        let dataframe = inputs
            .get(&DEFAULT_PORT)
            .ok_or(Error::input("no default"))?;
        let mut matched: Vec<Feature> = vec![];
        let mut not_matched: Vec<Feature> = vec![];
        for x in dataframe.features.clone() {
            if let Some(v) = x.get(&self.search_in) {
                match v {
                    AttributeValue::String(s) => {
                        if let Some(m) = re.find(s) {
                            let mut new_line = x.attributes.clone();
                            new_line.insert(
                                Attribute::new(self.matched_result.to_string()),
                                AttributeValue::String(m.as_str().to_string()),
                            );
                            matched.push(Feature::new_with_attributes(new_line));
                        } else {
                            not_matched.push(x);
                        }
                    }
                    _ => not_matched.push(x),
                }
            } else {
                not_matched.push(x)
            }
        }
        let output = HashMap::from([
            (Port::new("matched"), Dataframe::new(matched)),
            (Port::new("notMatched"), Dataframe::new(not_matched)),
        ]);
        Ok(output)
    }
}
