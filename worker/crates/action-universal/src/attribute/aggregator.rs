use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, Action, ActionContext, ActionDataframe, ActionResult, ActionValue, Port,
    DEFAULT_PORT,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttributeAggregator {
    aggregations: Vec<Aggregation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Aggregation {
    attribute: String,
    method: Method,
    output_port: Port,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) enum Method {
    #[serde(rename = "max")]
    Max,
    #[serde(rename = "min")]
    Min,
    #[serde(rename = "sum")]
    Sum,
    #[serde(rename = "avg")]
    Avg,
}

#[async_trait::async_trait]
#[typetag::serde(name = "AttributeAggregator")]
impl Action for AttributeAggregator {
    async fn run(&self, _ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(Error::input("No Input"))?;
        let input = inputs
            .get(DEFAULT_PORT)
            .ok_or(Error::input("No Default Port"))?;
        let input = input.as_ref().ok_or(Error::input("No Value"))?;

        let targets = match input {
            ActionValue::Array(rows) => rows
                .iter()
                .filter_map(|v| match v {
                    ActionValue::Map(row) => Some(row),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            _ => return Err(Error::input("Invalid Input. supported only Array")),
        };
        let mut output = ActionDataframe::new();
        for aggregation in &self.aggregations {
            match aggregation.method {
                Method::Max => {
                    let result = targets
                        .iter()
                        .filter_map(|row| row.get(&aggregation.attribute))
                        .max_by(|&a, &b| a.partial_cmp(b).unwrap());
                    output.insert(
                        aggregation.output_port.clone(),
                        Some(result.map(|v| v.to_owned()).unwrap_or(ActionValue::Number(
                            serde_json::Number::from_f64(0.0).unwrap(),
                        ))),
                    );
                }
                Method::Min => {
                    let result = targets
                        .iter()
                        .filter_map(|row| row.get(&aggregation.attribute))
                        .min_by(|a, b| a.partial_cmp(b).unwrap());
                    output.insert(
                        aggregation.output_port.clone(),
                        Some(result.map(|v| v.to_owned()).unwrap_or(ActionValue::Number(
                            serde_json::Number::from_f64(0.0).unwrap(),
                        ))),
                    );
                }
                Method::Sum => {
                    let result = targets
                        .iter()
                        .filter_map(|row| {
                            row.get(&aggregation.attribute).and_then(|v| {
                                if let ActionValue::Number(v) = v {
                                    v.as_f64()
                                } else {
                                    None
                                }
                            })
                        })
                        .collect::<Vec<f64>>();
                    output.insert(
                        aggregation.output_port.clone(),
                        Some(ActionValue::Number(
                            serde_json::Number::from_f64(result.iter().sum::<f64>()).unwrap(),
                        )),
                    );
                }
                Method::Avg => {
                    let result = targets
                        .iter()
                        .filter_map(|row| {
                            row.get(&aggregation.attribute).and_then(|v| {
                                if let ActionValue::Number(v) = v {
                                    v.as_f64()
                                } else {
                                    None
                                }
                            })
                        })
                        .collect::<Vec<f64>>();
                    if result.is_empty() {
                        output.insert(
                            aggregation.output_port.clone(),
                            Some(ActionValue::Number(
                                serde_json::Number::from_f64(0.0).unwrap(),
                            )),
                        );
                        continue;
                    }
                    let result = result.iter().sum::<f64>() / result.len() as f64;
                    output.insert(
                        aggregation.output_port.clone(),
                        Some(ActionValue::Number(
                            serde_json::Number::from_f64(result).unwrap(),
                        )),
                    );
                }
            }
        }
        Ok(output)
    }
}
