use std::{cmp::Ordering, vec};

use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error, Action, ActionContext, ActionDataframe, ActionResult, ActionValue,
};

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
impl Action for AttributeSorter {
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
                    data.sort_by(|a, b| match a {
                        ActionValue::Map(item_a) => {
                            let cmp = match b {
                                ActionValue::Map(item_b) => self
                                    .sort_by
                                    .iter()
                                    .map(|sort_by| {
                                        let attribute = &sort_by.attribute;
                                        let order = &sort_by.order;
                                        let a = item_a.get(attribute);
                                        let b = item_b.get(attribute);
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
                                    .collect::<Vec<_>>(),
                                _ => vec![Some(Ordering::Equal)],
                            };
                            cmp.iter().fold(Ordering::Equal, |acc, item| match acc {
                                Ordering::Equal if item.is_some() => item.unwrap(),
                                _ => acc,
                            })
                        }
                        _ => Ordering::Equal,
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
                    ActionValue::Number(serde_json::Number::from(28)),
                ),
            ])),
        ]
    }

    #[fixture]
    async fn expected() -> Vec<Vec<ActionValue>> {
        vec![
            vec![
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
                        ActionValue::Number(serde_json::Number::from(28)),
                    ),
                ])),
                ActionValue::Map(HashMap::from([
                    ("name".to_string(), ActionValue::String("Alice".to_string())),
                    (
                        "age".to_string(),
                        ActionValue::Number(serde_json::Number::from(20)),
                    ),
                ])),
            ],
            vec![
                ActionValue::Map(HashMap::from([
                    ("name".to_string(), ActionValue::String("Bob".to_string())),
                    (
                        "age".to_string(),
                        ActionValue::Number(serde_json::Number::from(28)),
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
                    ("name".to_string(), ActionValue::String("Alice".to_string())),
                    (
                        "age".to_string(),
                        ActionValue::Number(serde_json::Number::from(20)),
                    ),
                ])),
            ],
        ]
    }

    #[rstest]
    #[case::single_key(vec![SortBy {attribute: "name".to_string(),order: Order::Desc}], 0)]
    #[case::multiple_key(vec![SortBy {attribute: "name".to_string(),order: Order::Desc}, SortBy {attribute: "age".to_string(),order: Order::Desc}], 1)]
    #[tokio::test]
    async fn test_attribute_sorter(
        #[case] arg: Vec<SortBy>,
        #[case] case: usize,
        #[future(awt)] inputs: Vec<ActionValue>,
        #[future(awt)] expected: Vec<Vec<ActionValue>>,
    ) {
        let sorter = AttributeSorter { sort_by: arg };
        let inputs = vec![("default".to_string(), Some(ActionValue::Array(inputs)))]
            .into_iter()
            .collect::<HashMap<_, _>>();
        let expected_output = vec![(
            "default".to_string(),
            Some(ActionValue::Array(expected[case].clone())),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>();
        let result = sorter.run(ActionContext::default(), Some(inputs)).await;
        assert_eq!(result.unwrap(), expected_output);
    }
}
