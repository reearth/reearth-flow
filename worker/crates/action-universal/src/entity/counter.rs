use itertools::{self, Itertools};
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, Action, ActionContext, ActionDataframe, ActionResult, ActionValue, DEFAULT_PORT,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntityCounter {
    count_start: i64,
    group_by: Option<Vec<String>>,
    output_attribute: String,
}

#[async_trait::async_trait]
#[typetag::serde(name = "EntityCounter")]
impl Action for EntityCounter {
    async fn run(&self, _ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(Error::input("No Input"))?;
        let input = inputs
            .get(&DEFAULT_PORT)
            .ok_or(Error::input("No Default Port"))?;
        let input = input.as_ref().ok_or(Error::input("No Value"))?;

        let targets = match input {
            ActionValue::Array(rows) => rows
                .iter()
                .filter_map(|v| match v {
                    ActionValue::Map(row) => Some(row.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            _ => return Err(Error::input("Invalid Input. supported only Array")),
        };
        let mut output = ActionDataframe::new();
        let mut result = Vec::<ActionValue>::new();
        if self.group_by.is_none() {
            for (idx, row) in targets.iter().enumerate() {
                let count = self.count_start + idx as i64;
                let mut new_row = row.clone();
                new_row.insert(
                    self.output_attribute.clone(),
                    ActionValue::Number(serde_json::Number::from(count)),
                );
                result.push(ActionValue::Map(new_row));
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
                        ActionValue::Number(serde_json::Number::from(count)),
                    );
                    result.push(ActionValue::Map(new_row));
                }
            }
        }
        output.insert(DEFAULT_PORT.clone(), Some(ActionValue::Array(result)));
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_entity_counter_no_group_by() {
        let action = EntityCounter {
            count_start: 1,
            group_by: None,
            output_attribute: "count".to_string(),
        };
        let ctx = ActionContext::default();
        let mut inputs = ActionDataframe::new();
        let row1 = vec![
            ("name".to_string(), ActionValue::String("John".to_string())),
            (
                "age".to_string(),
                ActionValue::Number(serde_json::Number::from(25)),
            ),
        ]
        .into_iter()
        .collect();
        let row2 = vec![
            ("name".to_string(), ActionValue::String("Jane".to_string())),
            (
                "age".to_string(),
                ActionValue::Number(serde_json::Number::from(30)),
            ),
        ]
        .into_iter()
        .collect();
        inputs.insert(
            DEFAULT_PORT.clone(),
            Some(ActionValue::Array(vec![
                ActionValue::Map(row1),
                ActionValue::Map(row2),
            ])),
        );
        let result = action.run(ctx, Some(inputs)).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        let output_array = output.get(&DEFAULT_PORT).unwrap().clone().unwrap();
        if let ActionValue::Array(output_array) = output_array {
            assert_eq!(output_array.len(), 2);
            match (output_array[0].clone(), output_array[1].clone()) {
                (ActionValue::Map(output_row1), ActionValue::Map(output_row2)) => {
                    assert_eq!(
                        output_row1.get("count").unwrap(),
                        &ActionValue::Number(serde_json::Number::from(1))
                    );
                    assert_eq!(
                        output_row2.get("count").unwrap(),
                        &ActionValue::Number(serde_json::Number::from(2))
                    );
                }
                _ => panic!("output is not a map"),
            }
        } else {
            panic!("output is not an array");
        }
    }

    #[tokio::test]
    async fn test_entity_counter_with_group_by() {
        let action = EntityCounter {
            count_start: 1,
            group_by: Some(vec!["name".to_string()]),
            output_attribute: "count".to_string(),
        };
        let ctx = ActionContext::default();
        let mut inputs = ActionDataframe::new();
        let row1 = vec![
            ("name".to_string(), ActionValue::String("John".to_string())),
            (
                "age".to_string(),
                ActionValue::Number(serde_json::Number::from(25)),
            ),
        ]
        .into_iter()
        .collect();
        let row2 = vec![
            ("name".to_string(), ActionValue::String("John".to_string())),
            (
                "age".to_string(),
                ActionValue::Number(serde_json::Number::from(30)),
            ),
        ]
        .into_iter()
        .collect();
        let row3 = vec![
            ("name".to_string(), ActionValue::String("Jane".to_string())),
            (
                "age".to_string(),
                ActionValue::Number(serde_json::Number::from(35)),
            ),
        ]
        .into_iter()
        .collect();
        inputs.insert(
            DEFAULT_PORT.clone(),
            Some(ActionValue::Array(vec![
                ActionValue::Map(row1),
                ActionValue::Map(row2),
                ActionValue::Map(row3),
            ])),
        );
        let result = action.run(ctx, Some(inputs)).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        let output_array = output.get(&DEFAULT_PORT).unwrap().clone().unwrap();
        if let ActionValue::Array(output_array) = output_array {
            assert_eq!(output_array.len(), 3);
            match (
                output_array[0].clone(),
                output_array[1].clone(),
                output_array[2].clone(),
            ) {
                (
                    ActionValue::Map(output_row1),
                    ActionValue::Map(output_row2),
                    ActionValue::Map(output_row3),
                ) => {
                    assert_eq!(
                        output_row1.get("count").unwrap(),
                        &ActionValue::Number(serde_json::Number::from(1))
                    );
                    assert_eq!(
                        output_row2.get("count").unwrap(),
                        &ActionValue::Number(serde_json::Number::from(2))
                    );
                    assert_eq!(
                        output_row3.get("count").unwrap(),
                        &ActionValue::Number(serde_json::Number::from(1))
                    );
                }
                _ => panic!("output is not a map"),
            }
        } else {
            panic!("output is not an array");
        }
    }
}
