use reearth_flow_action::{
    error, Action, ActionContext, ActionDataframe, ActionResult, ActionValue,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
                ActionValue::Array(data) => {
                    #[allow(clippy::mutable_key_type)]
                    let mut seen_values = HashSet::new();
                    let mut filtered = vec![];
                    for item in data.iter() {
                        let key_values: Vec<Option<&ActionValue>> = self
                            .filter_by
                            .iter()
                            .map(|attribute| match item {
                                ActionValue::Map(map) => map.get(attribute),
                                _ => None,
                            })
                            .collect();
                        if seen_values.contains(&key_values) {
                            continue;
                        } else {
                            seen_values.insert(key_values.clone());
                            filtered.push(item.clone())
                        }
                    }
                    ActionValue::Array(filtered)
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
                    ActionValue::Number(serde_json::Number::from(25)),
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
                    ActionValue::Number(serde_json::Number::from(25)),
                ),
            ])),
        ]
    }

    #[fixture]
    async fn expected() -> Vec<Vec<ActionValue>> {
        vec![
            vec![ActionValue::Map(HashMap::from([
                ("name".to_string(), ActionValue::String("Alice".to_string())),
                (
                    "age".to_string(),
                    ActionValue::Number(serde_json::Number::from(25)),
                ),
            ]))],
            vec![
                ActionValue::Map(HashMap::from([
                    ("name".to_string(), ActionValue::String("Alice".to_string())),
                    (
                        "age".to_string(),
                        ActionValue::Number(serde_json::Number::from(25)),
                    ),
                ])),
                ActionValue::Map(HashMap::from([
                    ("name".to_string(), ActionValue::String("Bob".to_string())),
                    (
                        "age".to_string(),
                        ActionValue::Number(serde_json::Number::from(25)),
                    ),
                ])),
            ],
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
        #[future(awt)] expected: Vec<Vec<ActionValue>>,
    ) {
        let filter = AttributeDuplicateFilter { filter_by };
        let inputs = vec![(DEFAULT_PORT.clone(), Some(ActionValue::Array(inputs)))]
            .into_iter()
            .collect::<HashMap<_, _>>();
        let expected_output = vec![(
            DEFAULT_PORT.clone(),
            Some(ActionValue::Array(expected[case].clone())),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>();
        let result = filter.run(ActionContext::default(), Some(inputs)).await;
        assert_eq!(result.unwrap(), expected_output);
    }
}
