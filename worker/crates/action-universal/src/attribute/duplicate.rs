use reearth_flow_action::{ActionContext, ActionDataframe, ActionResult, AsyncAction, Dataframe};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttributeDuplicateFilter {
    filter_by: Vec<String>,
}

#[async_trait::async_trait]
#[typetag::serde(name = "AttributeDuplicateFilter")]
impl AsyncAction for AttributeDuplicateFilter {
    async fn run(&self, _ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let mut output = ActionDataframe::new();
        for (port, data) in inputs {
            let processed_data = {
                #[allow(clippy::mutable_key_type)]
                let mut seen_values = HashSet::new();
                let mut filtered = vec![];
                for item in data.features.iter() {
                    let key_values = self
                        .filter_by
                        .iter()
                        .flat_map(|attribute| item.get(attribute))
                        .collect::<Vec<_>>();
                    let key_values = key_values.iter().map(|&v| v.clone()).collect::<Vec<_>>();
                    if seen_values.contains(&key_values) {
                        continue;
                    } else {
                        seen_values.insert(key_values.clone());
                        filtered.push(item.clone())
                    }
                }
                filtered
            };
            output.insert(port, Dataframe::new(processed_data));
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_action::{Attribute, AttributeValue, Feature, DEFAULT_PORT};
    use rstest::*;
    use std::collections::HashMap;

    #[fixture]
    async fn inputs() -> Vec<Feature> {
        vec![
            Feature::new_with_attributes(HashMap::from([
                (
                    Attribute::new("name"),
                    AttributeValue::String("Alice".to_string()),
                ),
                (
                    Attribute::new("age"),
                    AttributeValue::Number(serde_json::Number::from(25)),
                ),
            ])),
            Feature::new_with_attributes(HashMap::from([
                (
                    Attribute::new("name"),
                    AttributeValue::String("Bob".to_string()),
                ),
                (
                    Attribute::new("age"),
                    AttributeValue::Number(serde_json::Number::from(25)),
                ),
            ])),
            Feature::new_with_attributes(HashMap::from([
                (
                    Attribute::new("name"),
                    AttributeValue::String("Bob".to_string()),
                ),
                (
                    Attribute::new("age"),
                    AttributeValue::Number(serde_json::Number::from(25)),
                ),
            ])),
        ]
    }

    #[fixture]
    async fn expected() -> Vec<Vec<Feature>> {
        vec![
            vec![Feature::new_with_attributes(HashMap::from([
                (
                    Attribute::new("name"),
                    AttributeValue::String("Alice".to_string()),
                ),
                (
                    Attribute::new("age"),
                    AttributeValue::Number(serde_json::Number::from(25)),
                ),
            ]))],
            vec![
                Feature::new_with_attributes(HashMap::from([
                    (
                        Attribute::new("name"),
                        AttributeValue::String("Alice".to_string()),
                    ),
                    (
                        Attribute::new("age"),
                        AttributeValue::Number(serde_json::Number::from(25)),
                    ),
                ])),
                Feature::new_with_attributes(HashMap::from([
                    (
                        Attribute::new("name"),
                        AttributeValue::String("Bob".to_string()),
                    ),
                    (
                        Attribute::new("age"),
                        AttributeValue::Number(serde_json::Number::from(25)),
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
        #[future(awt)] inputs: Vec<Feature>,
        #[future(awt)] expected: Vec<Vec<Feature>>,
    ) {
        let filter = AttributeDuplicateFilter { filter_by };
        let inputs = vec![(DEFAULT_PORT.clone(), Dataframe::new(inputs))]
            .into_iter()
            .collect::<HashMap<_, _>>();
        let expected_output =
            ActionDataframe::from([(DEFAULT_PORT.clone(), Dataframe::new(expected[case].clone()))])
                .into_iter()
                .collect::<HashMap<_, _>>();
        let result = filter.run(ActionContext::default(), inputs).await;
        assert_eq!(result.unwrap(), expected_output);
    }
}
