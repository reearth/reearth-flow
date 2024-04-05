use reearth_flow_action::{
    error, Action, ActionContext, ActionDataframe, ActionResult, ActionValue,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttributeExposer {
    exposed_attributes: Vec<String>,
}

#[async_trait::async_trait]
#[typetag::serde(name = "AttributeExposer")]
impl Action for AttributeExposer {
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
                    let filtered = data
                        .iter()
                        .map(|item| {
                            let mut filtered_map = HashMap::new();
                            for attribute in &self.exposed_attributes {
                                if let ActionValue::Map(map) = item {
                                    if let Some(value) = map.get(attribute) {
                                        filtered_map.insert(attribute.clone(), value.clone());
                                    }
                                }
                            }
                            ActionValue::Map(filtered_map)
                        })
                        .collect();
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
                    ActionValue::Number(serde_json::Number::from(30)),
                ),
            ])),
        ]
    }

    #[fixture]
    async fn expected() -> Vec<Vec<ActionValue>> {
        vec![
            vec![
                ActionValue::Map(HashMap::from([(
                    "name".to_string(),
                    ActionValue::String("Alice".to_string()),
                )])),
                ActionValue::Map(HashMap::from([(
                    "name".to_string(),
                    ActionValue::String("Bob".to_string()),
                )])),
                ActionValue::Map(HashMap::from([(
                    "name".to_string(),
                    ActionValue::String("Bob".to_string()),
                )])),
            ],
            vec![
                ActionValue::Map(HashMap::from([(
                    "age".to_string(),
                    ActionValue::Number(serde_json::Number::from(20)),
                )])),
                ActionValue::Map(HashMap::from([(
                    "age".to_string(),
                    ActionValue::Number(serde_json::Number::from(25)),
                )])),
                ActionValue::Map(HashMap::from([(
                    "age".to_string(),
                    ActionValue::Number(serde_json::Number::from(30)),
                )])),
            ],
        ]
    }

    #[rstest]
    #[case::expose_name_only(vec!["name".to_string()], 0)]
    #[case::expose_age_only(vec!["age".to_string()], 1)]
    #[tokio::test]
    async fn test_attribute_exposer(
        #[case] exposed_attributes: Vec<String>,
        #[case] case: usize,
        #[future(awt)] inputs: Vec<ActionValue>,
        #[future(awt)] expected: Vec<Vec<ActionValue>>,
    ) {
        let exposer = AttributeExposer { exposed_attributes };
        let inputs = vec![(DEFAULT_PORT.clone(), Some(ActionValue::Array(inputs)))]
            .into_iter()
            .collect::<HashMap<_, _>>();
        let expected_output = vec![(
            DEFAULT_PORT.clone(),
            Some(ActionValue::Array(expected[case].clone())),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>();
        let result = exposer.run(ActionContext::default(), Some(inputs)).await;
        assert_eq!(result.unwrap(), expected_output);
    }
}
