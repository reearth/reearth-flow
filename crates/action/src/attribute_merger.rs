use core::result::Result;
use std::collections::HashMap;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use reearth_flow_workflow::graph::NodeProperty;

use crate::action::{ActionContext, ActionDataframe, ActionValue, ActionValueIndex, DEFAULT_PORT};

const REQUESTOR_PORT: &str = "requestor";
const SUPPLIER_PORT: &str = "supplier";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PropertySchema {
    joins: Vec<Join>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Join {
    requestor: String,
    supplier: String,
}

impl TryFrom<NodeProperty> for PropertySchema {
    type Error = anyhow::Error;

    fn try_from(node_property: NodeProperty) -> Result<Self, anyhow::Error> {
        serde_json::from_value(Value::Object(node_property)).map_err(|e| {
            anyhow!(
                "Failed to convert NodeProperty to PropertySchema with {}",
                e
            )
        })
    }
}

pub(crate) async fn run(
    ctx: ActionContext,
    inputs: Option<ActionDataframe>,
) -> anyhow::Result<ActionDataframe> {
    let props = PropertySchema::try_from(ctx.node_property)?;
    let inputs = inputs.ok_or(anyhow!("No Input"))?;
    let requestor = inputs
        .get(REQUESTOR_PORT)
        .ok_or(anyhow!("No Requestor Port"))?;
    let requestor = requestor.as_ref().ok_or(anyhow!("No Requestor Value"))?;

    let supplier = inputs
        .get(SUPPLIER_PORT)
        .ok_or(anyhow!("No Supplier Port"))?;
    let supplier = supplier.as_ref().ok_or(anyhow!("No Supplier Value"))?;
    let supplier_indexs = create_supplier_index(supplier, &props)?;
    let result = match requestor {
        ActionValue::Array(rows) => {
            let mut result = Vec::<ActionValue>::new();
            for row in rows {
                match row {
                    ActionValue::Map(row) => {
                        for join in &props.joins {
                            let requestor = &join.requestor;
                            let supplier = &join.supplier;
                            let requestor_value =
                                row.get(requestor).ok_or(anyhow!("No Requestor Value"))?;
                            let supplier_index = supplier_indexs
                                .get(supplier)
                                .ok_or(anyhow!("No Supplier Index"))?;
                            let supplier_rows = supplier_index
                                .get(&requestor_value.to_string())
                                .ok_or(anyhow!("No Supplier Rows"))?;
                            for supplier_row in supplier_rows {
                                match supplier_row {
                                    ActionValue::Map(supplier_row) => {
                                        let mut new_row = row.clone();
                                        new_row.extend(supplier_row.clone());
                                        result.push(ActionValue::Map(new_row));
                                    }
                                    _ => return Err(anyhow!("Supplier is not a map")),
                                }
                            }
                        }
                    }
                    _ => return Err(anyhow!("Requestor is not an array")),
                }
            }
            result
        }
        _ => return Err(anyhow!("Requestor is not an array")),
    };
    Ok(
        vec![(DEFAULT_PORT.to_string(), Some(ActionValue::Array(result)))]
            .into_iter()
            .collect::<HashMap<_, _>>(),
    )
}

fn create_supplier_index(
    supplier: &ActionValue,
    props: &PropertySchema,
) -> anyhow::Result<ActionValueIndex> {
    let mut supplier_indexs = ActionValueIndex::new();
    match supplier {
        ActionValue::Array(rows) => {
            for row in rows {
                match row {
                    ActionValue::Map(row) => {
                        for join in &props.joins {
                            let supplier = &join.supplier;
                            let supplier_value =
                                row.get(supplier).ok_or(anyhow!("No Supplier Value"))?;
                            let supplier_index_entry =
                                supplier_indexs.entry(supplier.to_owned()).or_default();
                            supplier_index_entry
                                .entry(supplier_value.to_string())
                                .or_default()
                                .push(ActionValue::Map(row.clone()));
                        }
                    }
                    _ => return Err(anyhow!("Supplier is not an array")),
                }
            }
            Ok(supplier_indexs)
        }
        _ => Err(anyhow!("Supplier is not an array")),
    }
}
