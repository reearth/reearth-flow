use core::result::Result;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use reearth_flow_macros::PropertySchema;

use crate::action::{ActionContext, ActionDataframe, ActionValue, DEFAULT_PORT};

#[derive(Serialize, Deserialize, Debug, PropertySchema)]
#[serde(rename_all = "camelCase")]
struct PropertySchema {
    aggregations: Vec<Aggregation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Aggregation {
    attribute: String,
    method: Method,
    output_port: String,
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

pub(crate) async fn run(
    ctx: ActionContext,
    inputs: Option<ActionDataframe>,
) -> anyhow::Result<ActionDataframe> {
    let props = PropertySchema::try_from(ctx.node_property)?;
    debug!(?props, "read");
    let inputs = inputs.ok_or(anyhow!("No Input"))?;
    let input = inputs.get(DEFAULT_PORT).ok_or(anyhow!("No Default Port"))?;
    let input = input.as_ref().ok_or(anyhow!("No Value"))?;

    let targets = match input {
        ActionValue::Array(rows) => rows
            .iter()
            .filter(|&v| matches!(v, ActionValue::Map(_)))
            .filter_map(|v| match v {
                ActionValue::Map(row) => Some(row),
                _ => None,
            })
            .collect::<Vec<_>>(),
        _ => return Err(anyhow!("Invalid Input. supported only Array")),
    };
    let mut output = ActionDataframe::new();
    for aggregation in &props.aggregations {
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
