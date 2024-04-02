use reearth_flow_action::{
    error, Action, ActionContext, ActionDataframe, ActionResult, ActionValue,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttributeDuplicateFilter {
    filter_by: Vec<String>,
}

#[async_trait::async_trait]
#[typetag::serde(name = "AttributeDuplicateFilter")]
impl Action for AttributeDuplicateFilter {
    async fn run(&self, _ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(error::Error::input("No input dataframe"))?;
        let mut output = ActionDataframe::new();
        for (port, data) in inputs {
            let data = match data {
                Some(data) => data,
                None => continue,
            };
            let processed_data = match data {
                ActionValue::Array(mut data) => {
                    let mut seen_values = HashSet::new();
                    data.retain(|item| {
                        let key_attributes: Vec<String> = self.filter_by.iter().cloned().collect();
                        let key_values: Vec<Option<&ActionValue>> = key_attributes
                            .iter()
                            .map(|attribute| match item {
                                ActionValue::Map(map) => map.get(attribute),
                                _ => None,
                            })
                            .collect();
                        if seen_values.contains(&key_values) {
                            false
                        } else {
                            seen_values.insert(key_values.clone());
                            true
                        }
                    });
                    ActionValue::Array(data)
                }
                _ => data,
            };
            output.insert(port, Some(processed_data));
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_action::DEFAULT_PORT;
    use rstest::*;
    use std::collections::HashMap;

    #[fixture]
    async fn inputs() -> Vec<ActionValue> {
        vec![
            ActionValue::Map(HashMap::from([
                ("name".to_string(), ActionValue::String("Alice".to_string())),
                (
                    "age".to_string(),
                    ActionValue::Number(serde_json::Number::from(20)),
                ),
            ])),
            ActionValue::Map(HashMap::from([
                ("name".to_string(), ActionValue::String("Bob".to_string())),
                (
                    "age".to_string(),
                    ActionValue::Number(serde_json::Number::from(25)),
                ),
            ])),
            ActionValue::Map(HashMap::from([
                ("name".to_string(), ActionValue::String("Bob".to_string())),
                (
                    "age".to_string(),
                    ActionValue::Number(serde_json::Number::from(25)), // Same age as previous entry
                ),
            ])),
        ]
    }

    #[fixture]
    async fn expected() -> Vec<ActionValue> {
        vec![
            ActionValue::Map(HashMap::from([
                ("name".to_string(), ActionValue::String("Alice".to_string())),
                (
                    "age".to_string(),
                    ActionValue::Number(serde_json::Number::from(20)),
                ),
            ])),
            ActionValue::Map(HashMap::from([
                ("name".to_string(), ActionValue::String("Bob".to_string())),
                (
                    "age".to_string(),
                    ActionValue::Number(serde_json::Number::from(25)),
                ),
            ])),
        ]
    }

    #[rstest]
    #[case::single_key(vec!["age".to_string()], 0)]
    #[case::multiple_key(vec!["name".to_string(), "age".to_string()], 1)]
    #[tokio::test]
    async fn test_attribute_duplicate_filter(
        #[case] filter_by: Vec<String>,
        #[case] case: usize,
        #[future(awt)] inputs: Vec<ActionValue>,
        #[future(awt)] expected: Vec<ActionValue>,
    ) {
        let filter = AttributeDuplicateFilter { filter_by };
        let inputs = vec![(DEFAULT_PORT.clone(), Some(ActionValue::Array(inputs)))]
            .into_iter()
            .collect::<HashMap<_, _>>();
        let expected_output = vec![(DEFAULT_PORT.clone(), Some(ActionValue::Array(expected)))]
            .into_iter()
            .collect::<HashMap<_, _>>();
        let result = filter.run(ActionContext::default(), Some(inputs)).await;
        assert_eq!(result.unwrap(), expected_output);
    }
}
