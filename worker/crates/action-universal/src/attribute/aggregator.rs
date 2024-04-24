use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, ActionContext, ActionDataframe, ActionResult, AsyncAction, AttributeValue,
    Dataframe, Feature, DEFAULT_PORT,
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(super) enum Method {
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
impl AsyncAction for AttributeAggregator {
    async fn run(&self, _ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let input = inputs
            .get(&DEFAULT_PORT)
            .ok_or(Error::input("No Default Port"))?;

        let mut feature = Feature::new();
        for aggregation in &self.aggregations {
            match aggregation.method {
                Method::Max => {
                    let result = input
                        .features
                        .iter()
                        .filter_map(|row| row.get(&aggregation.attribute))
                        .max_by(|&a, &b| a.partial_cmp(b).unwrap());
                    feature.insert(
                        format!("max_{}", &aggregation.attribute),
                        result
                            .map(|v| v.to_owned())
                            .unwrap_or(AttributeValue::Number(
                                serde_json::Number::from_f64(0.0).unwrap(),
                            )),
                    );
                }
                Method::Min => {
                    let result = input
                        .features
                        .iter()
                        .filter_map(|row| row.get(&aggregation.attribute))
                        .min_by(|a, b| a.partial_cmp(b).unwrap());
                    feature.insert(
                        format!("min_{}", &aggregation.attribute),
                        result
                            .map(|v| v.to_owned())
                            .unwrap_or(AttributeValue::Number(
                                serde_json::Number::from_f64(0.0).unwrap(),
                            )),
                    );
                }
                Method::Sum => {
                    let result = input
                        .features
                        .iter()
                        .filter_map(|row| {
                            row.get(&aggregation.attribute).and_then(|v| {
                                if let AttributeValue::Number(v) = v {
                                    v.as_f64()
                                } else {
                                    None
                                }
                            })
                        })
                        .collect::<Vec<f64>>();
                    feature.insert(
                        format!("sum_{}", &aggregation.attribute),
                        AttributeValue::Number(
                            serde_json::Number::from_f64(result.iter().sum::<f64>()).unwrap(),
                        ),
                    );
                }
                Method::Avg => {
                    let result = input
                        .features
                        .iter()
                        .filter_map(|row| {
                            row.get(&aggregation.attribute).and_then(|v| {
                                if let AttributeValue::Number(v) = v {
                                    v.as_f64()
                                } else {
                                    None
                                }
                            })
                        })
                        .collect::<Vec<f64>>();
                    if result.is_empty() {
                        feature.insert(
                            format!("avg_{}", &aggregation.attribute),
                            AttributeValue::Number(serde_json::Number::from_f64(0.0).unwrap()),
                        );
                        continue;
                    }
                    let result = result.iter().sum::<f64>() / result.len() as f64;
                    feature.insert(
                        format!("avg_{}", &aggregation.attribute),
                        AttributeValue::Number(serde_json::Number::from_f64(result).unwrap()),
                    );
                }
            }
        }
        Ok(ActionDataframe::from([(
            DEFAULT_PORT.clone(),
            Dataframe::new(vec![feature]),
        )]))
    }
}
