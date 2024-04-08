use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use std::collections::HashMap;

use jsonpath_rust::JsonPathQuery;
use reearth_flow_action::{
    error, ActionContext, ActionDataframe, ActionResult, ActionValue, AsyncAction,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JSONExtractor {
    inputs: Vec<AttributeMapping>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct AttributeMapping {
    attribute: String,
    query: String,
}

#[async_trait::async_trait]
#[typetag::serde(name = "JSONExtractor")]
impl AsyncAction for JSONExtractor {
    async fn run(&self, _ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(error::Error::input("No input dataframe"))?;
        let mut output = ActionDataframe::new();
        for (port, data) in inputs {
            let data = match data {
                Some(data) => data,
                None => continue,
            };
            let mut processed_data: HashMap<String, ActionValue> = HashMap::new();

            for input in &self.inputs {
                let data: Value = data.clone().into();
                let value: ActionValue = match data.path(&input.query).unwrap_or_default() {
                    serde_json::Value::Array(arr) if !arr.is_empty() => arr[0].clone().into(),
                    _ => ActionValue::Null,
                };
                processed_data.insert(String::from(&input.attribute), value);
            }
            output.insert(port, Some(ActionValue::Map(processed_data)));
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
    async fn inputs() -> ActionValue {
        from_str::<Value>(
            r#"
                    {
                        "store": {
                            "book": [
                                {
                                    "category": "reference",
                                    "author": "Nigel Rees",
                                    "title": "Sayings of the Century",
                                    "price": 8.95
                                },
                                {
                                    "category": "fiction",
                                    "author": "Evelyn Waugh",
                                    "title": "Sword of Honour",
                                    "price": 12.99
                                },
                                {
                                    "category": "fiction",
                                    "author": "Herman Melville",
                                    "title": "Moby Dick",
                                    "isbn": "0-553-21311-3",
                                    "price": 8.99
                                },
                                {
                                    "category": "fiction",
                                    "author": "J. R. R. Tolkien",
                                    "title": "The Lord of the Rings",
                                    "isbn": "0-395-19395-8",
                                    "price": 22.99
                                }
                            ],
                            "bicycle": {
                                "color": "red",
                                "price": 19.95
                            }
                        },
                        "expensive": 10
                    }
                "#,
        )
        .unwrap()
        .into()
    }

    #[fixture]
    async fn expected() -> Vec<ActionValue> {
        let mut authors: HashMap<String, ActionValue> = HashMap::new();
        authors.insert(
            "author 1".to_string(),
            ActionValue::String("Nigel Rees".to_string()),
        );

        authors.insert(
            "author 2".to_string(),
            ActionValue::String("Evelyn Waugh".to_string()),
        );
        authors.insert(
            "author 3".to_string(),
            ActionValue::String("Herman Melville".to_string()),
        );

        vec![
            from_str::<Value>(
                r#"
                    {
                        "object": [
                            {
                                "category": "reference",
                                "author": "Nigel Rees",
                                "title": "Sayings of the Century",
                                "price": 8.95
                            },
                            {
                                "category": "fiction",
                                "author": "Evelyn Waugh",
                                "title": "Sword of Honour",
                                "price": 12.99
                            },
                            {
                                "category": "fiction",
                                "author": "Herman Melville",
                                "title": "Moby Dick",
                                "isbn": "0-553-21311-3",
                                "price": 8.99
                            },
                            {
                                "category": "fiction",
                                "author": "J. R. R. Tolkien",
                                "title": "The Lord of the Rings",
                                "isbn": "0-395-19395-8",
                                "price": 22.99
                            }
                        ]
                    }
                    "#,
            )
            .unwrap()
            .into(),
            ActionValue::Map(authors),
        ]
    }

    #[rstest]
    #[case::single_mapping(vec![AttributeMapping {attribute: "object".to_string(), query: "$.store.book".to_string()}], 0)]
    #[case::multiple_mapping(vec![AttributeMapping {attribute: "author 1".to_string(), query: "$.store.book[0].author".to_string()},
    AttributeMapping{attribute: "author 2".to_string(), query: "$.store.book[1].author".to_string()},
    AttributeMapping{attribute: "author 3".to_string(), query: "$.store.book[2].author".to_string()}], 1)]
    #[tokio::test]
    async fn test_json_extractor(
        #[case] arg: Vec<AttributeMapping>,
        #[case] case: usize,
        #[future(awt)] inputs: ActionValue,
        #[future(awt)] expected: Vec<ActionValue>,
    ) {
        let sorter = JSONExtractor { inputs: arg };
        let inputs = vec![(DEFAULT_PORT.clone(), Some(inputs))]
            .into_iter()
            .collect::<HashMap<_, _>>();

        let expected_output = vec![(DEFAULT_PORT.clone(), Some(expected[case].clone()))]
            .into_iter()
            .collect::<HashMap<_, _>>();
        let result = sorter.run(ActionContext::default(), Some(inputs)).await;
        assert_eq!(result.unwrap(), expected_output);
    }
}
