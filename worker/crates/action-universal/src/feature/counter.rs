use itertools::{self, Itertools};
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, ActionContext, ActionDataframe, ActionResult, AsyncAction, AttributeValue,
    Dataframe, Feature, DEFAULT_PORT,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCounter {
    count_start: i64,
    group_by: Option<Vec<String>>,
    output_attribute: String,
}

#[async_trait::async_trait]
#[typetag::serde(name = "FeatureCounter")]
impl AsyncAction for FeatureCounter {
    async fn run(&self, _ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let input = inputs
            .get(&DEFAULT_PORT)
            .ok_or(Error::input("No Default Port"))?;

        let targets = &input.features;
        let mut output = ActionDataframe::new();
        let mut result = Vec::<Feature>::new();
        if self.group_by.is_none() {
            for (idx, row) in targets.iter().enumerate() {
                let count = self.count_start + idx as i64;
                let mut new_row = row.clone();
                new_row.insert(
                    self.output_attribute.clone(),
                    AttributeValue::Number(serde_json::Number::from(count)),
                );
                result.push(new_row);
            }
        } else {
            let group_by = self.group_by.as_ref().unwrap();
            let grouped = targets
                .iter()
                .group_by(|&row| {
                    let key = group_by
                        .iter()
                        .map(|k| row.get(k).unwrap().to_string())
                        .join(",");
                    key
                })
                .into_iter()
                .map(|(key, group)| (key, group.collect::<Vec<_>>()))
                .collect::<Vec<_>>();
            for (_key, group) in grouped {
                for (idx, &row) in group.iter().enumerate() {
                    let count = self.count_start + idx as i64;
                    let mut new_row = row.clone();
                    new_row.insert(
                        self.output_attribute.clone(),
                        AttributeValue::Number(serde_json::Number::from(count)),
                    );
                    result.push(new_row);
                }
            }
        }
        output.insert(DEFAULT_PORT.clone(), Dataframe::new(result));
        Ok(output)
    }
}
