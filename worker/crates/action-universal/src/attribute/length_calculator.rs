use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, ActionContext, ActionDataframe, ActionResult, AsyncAction, Attribute,
    AttributeValue, Dataframe, DEFAULT_PORT,
};
use serde_json::Number;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttributeStringLengthCalculator {
    source_attribute: String,
    string_length_attribute: String,
}

#[async_trait::async_trait]
#[typetag::serde(name = "AttributeStringLengthCalculator")]
impl AsyncAction for AttributeStringLengthCalculator {
    async fn run(&self, _ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let dataframe = inputs
            .get(&DEFAULT_PORT)
            .ok_or(Error::input("no default"))?
            .clone();
        let lens = dataframe
            .features
            .into_iter()
            .map(|mut x| {
                if let Some(AttributeValue::String(src)) =
                    x.attributes.get(&Attribute::new(&self.source_attribute))
                {
                    x.attributes.insert(
                        Attribute::new(self.string_length_attribute.clone()),
                        AttributeValue::Number(Number::from(src.len())),
                    );
                    x
                } else {
                    x.attributes.insert(
                        Attribute::new(self.string_length_attribute.clone()),
                        AttributeValue::Number(Number::from(0)),
                    );
                    x
                }
            })
            .collect::<Vec<_>>();
        let output = vec![(DEFAULT_PORT.clone(), Dataframe::new(lens))]
            .into_iter()
            .collect();
        Ok(output)
    }
}
