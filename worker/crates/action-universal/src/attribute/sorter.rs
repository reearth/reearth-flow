use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use reearth_flow_action::{ActionContext, ActionDataframe, ActionResult, AsyncAction, Dataframe};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttributeSorter {
    sort_by: Vec<SortBy>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct SortBy {
    attribute: String,
    order: Order,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Order {
    #[serde(rename = "ascending")]
    Asc,
    #[serde(rename = "descending")]
    Desc,
}

#[async_trait::async_trait]
#[typetag::serde(name = "AttributeSorter")]
impl AsyncAction for AttributeSorter {
    async fn run(&self, _ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let mut result = ActionDataframe::new();
        for (port, data) in inputs {
            let mut features = data.features;
            features.sort_by(|a, b| {
                let cmp = self
                    .sort_by
                    .iter()
                    .map(|sort_by| {
                        let attribute = &sort_by.attribute;
                        let order = &sort_by.order;
                        let a = a.get(attribute);
                        let b = b.get(attribute);
                        match (a, b) {
                            (Some(a), Some(b)) => {
                                if *order == Order::Asc {
                                    a.partial_cmp(b)
                                } else {
                                    b.partial_cmp(a)
                                }
                            }
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>();
                cmp.iter().fold(Ordering::Equal, |acc, item| match acc {
                    Ordering::Equal if item.is_some() => item.unwrap(),
                    _ => acc,
                })
            });
            result.insert(port, Dataframe::new(features));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_action::{Attribute, AttributeValue, Dataframe, Feature, DEFAULT_PORT};
    use rstest::*;
    use std::collections::HashMap;
    use uuid::uuid;

    #[fixture]
    async fn inputs() -> Vec<Feature> {
        vec![
            Feature::new_with_id_and_attributes(
                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4a"),
                HashMap::from([
                    (
                        Attribute::new("name"),
                        AttributeValue::String("Alice".to_string()),
                    ),
                    (
                        Attribute::new("age"),
                        AttributeValue::Number(serde_json::Number::from(20)),
                    ),
                ]),
            ),
            Feature::new_with_id_and_attributes(
                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4b"),
                HashMap::from([
                    (
                        Attribute::new("name"),
                        AttributeValue::String("Bob".to_string()),
                    ),
                    (
                        Attribute::new("age"),
                        AttributeValue::Number(serde_json::Number::from(25)),
                    ),
                ]),
            ),
            Feature::new_with_id_and_attributes(
                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4c"),
                HashMap::from([
                    (
                        Attribute::new("name"),
                        AttributeValue::String("Bob".to_string()),
                    ),
                    (
                        Attribute::new("age"),
                        AttributeValue::Number(serde_json::Number::from(28)),
                    ),
                ]),
            ),
        ]
    }

    #[fixture]
    async fn expected() -> Vec<Vec<Feature>> {
        vec![
            vec![
                Feature::new_with_id_and_attributes(
                    uuid!("2830de29-b6bd-4783-9a89-042a587c2b4b"),
                    HashMap::from([
                        (
                            Attribute::new("name"),
                            AttributeValue::String("Bob".to_string()),
                        ),
                        (
                            Attribute::new("age"),
                            AttributeValue::Number(serde_json::Number::from(25)),
                        ),
                    ]),
                ),
                Feature::new_with_id_and_attributes(
                    uuid!("2830de29-b6bd-4783-9a89-042a587c2b4c"),
                    HashMap::from([
                        (
                            Attribute::new("name"),
                            AttributeValue::String("Bob".to_string()),
                        ),
                        (
                            Attribute::new("age"),
                            AttributeValue::Number(serde_json::Number::from(28)),
                        ),
                    ]),
                ),
                Feature::new_with_id_and_attributes(
                    uuid!("2830de29-b6bd-4783-9a89-042a587c2b4a"),
                    HashMap::from([
                        (
                            Attribute::new("name"),
                            AttributeValue::String("Alice".to_string()),
                        ),
                        (
                            Attribute::new("age"),
                            AttributeValue::Number(serde_json::Number::from(20)),
                        ),
                    ]),
                ),
            ],
            vec![
                Feature::new_with_id_and_attributes(
                    uuid!("2830de29-b6bd-4783-9a89-042a587c2b4c"),
                    HashMap::from([
                        (
                            Attribute::new("name"),
                            AttributeValue::String("Bob".to_string()),
                        ),
                        (
                            Attribute::new("age"),
                            AttributeValue::Number(serde_json::Number::from(28)),
                        ),
                    ]),
                ),
                Feature::new_with_id_and_attributes(
                    uuid!("2830de29-b6bd-4783-9a89-042a587c2b4b"),
                    HashMap::from([
                        (
                            Attribute::new("name"),
                            AttributeValue::String("Bob".to_string()),
                        ),
                        (
                            Attribute::new("age"),
                            AttributeValue::Number(serde_json::Number::from(25)),
                        ),
                    ]),
                ),
                Feature::new_with_id_and_attributes(
                    uuid!("2830de29-b6bd-4783-9a89-042a587c2b4a"),
                    HashMap::from([
                        (
                            Attribute::new("name"),
                            AttributeValue::String("Alice".to_string()),
                        ),
                        (
                            Attribute::new("age"),
                            AttributeValue::Number(serde_json::Number::from(20)),
                        ),
                    ]),
                ),
            ],
        ]
    }

    #[rstest]
    #[case::single_key(vec![SortBy {attribute: "name".to_string() ,order: Order::Desc}], 0)]
    #[case::multiple_key(vec![SortBy {attribute: "name".to_string(),order: Order::Desc}, SortBy {attribute: "age".to_string(),order: Order::Desc}], 1)]
    #[tokio::test]
    async fn test_attribute_sorter(
        #[case] arg: Vec<SortBy>,
        #[case] case: usize,
        #[future(awt)] inputs: Vec<Feature>,
        #[future(awt)] expected: Vec<Vec<Feature>>,
    ) {
        let sorter = AttributeSorter { sort_by: arg };
        let inputs = vec![(DEFAULT_PORT.clone(), Dataframe::new(inputs))]
            .into_iter()
            .collect::<HashMap<_, _>>();
        let expected_output =
            ActionDataframe::from([(DEFAULT_PORT.clone(), Dataframe::new(expected[case].clone()))])
                .into_iter()
                .collect::<HashMap<_, _>>();
        let result = sorter.run(ActionContext::default(), inputs).await;
        assert_eq!(result.unwrap(), expected_output);
    }
}
