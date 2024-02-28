use core::result::Result;
use std::collections::HashMap;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use reearth_flow_macros::PropertySchema;

use crate::action::{ActionContext, ActionDataframe, ActionValue, ActionValueIndex, DEFAULT_PORT};
use crate::error::Error;

const REQUESTOR_PORT: &str = "requestor";
const SUPPLIER_PORT: &str = "supplier";
const ROW_NUMBER: &str = "row_number";

#[derive(Serialize, Deserialize, Debug, PropertySchema)]
#[serde(rename_all = "camelCase")]
struct PropertySchema {
    join: Join,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Join {
    requestor: String,
    supplier: String,
}

pub(crate) async fn run(
    ctx: ActionContext,
    inputs: Option<ActionDataframe>,
) -> anyhow::Result<ActionDataframe> {
    let props = PropertySchema::try_from(ctx.node_property)?;
    let inputs = inputs.ok_or(Error::input("No Input"))?;
    let requestor = inputs
        .get(REQUESTOR_PORT)
        .ok_or(Error::input("No Requestor Port"))?;
    let requestor = requestor
        .as_ref()
        .ok_or(Error::input("No Requestor Value"))?;

    let supplier = inputs
        .get(SUPPLIER_PORT)
        .ok_or(Error::input("No Supplier Port"))?;
    let supplier = supplier.as_ref().ok_or(Error::input("No Supplier Value"))?;
    let requestor_key = &props.join.requestor;
    let supplier_key = &props.join.supplier;
    let is_row_number_join = requestor_key == ROW_NUMBER && supplier_key == ROW_NUMBER;
    let supplier_indexs = if is_row_number_join {
        ActionValueIndex::new()
    } else {
        create_supplier_index(supplier, &props)?
    };
    let requestor = match requestor {
        ActionValue::Array(rows) => rows,
        _ => return Err(Error::validate("Requestor is not an array").into()),
    };
    let supplier = match supplier {
        ActionValue::Array(rows) => rows,
        _ => return Err(Error::validate("Supplier is not an array").into()),
    };
    let mut result = Vec::<ActionValue>::new();
    for (idx, row) in requestor.iter().enumerate() {
        match row {
            ActionValue::Map(row) => {
                if is_row_number_join {
                    if let Some(ActionValue::Map(supplier_row)) = supplier.get(idx) {
                        let mut new_row = row.clone();
                        new_row.extend(supplier_row.clone());
                        result.push(ActionValue::Map(new_row));
                    }
                    continue;
                }
                let requestor_value = row.get(requestor_key).ok_or(anyhow!(
                    "No Requestor Value with requestor = {}",
                    requestor_key
                ))?;
                let supplier_index = supplier_indexs.get(supplier_key).ok_or(anyhow!(
                    "No Supplier Index with request value = {}",
                    requestor_value
                ))?;
                let supplier_rows = supplier_index.get(&requestor_value.to_string());
                if supplier_rows.is_none() {
                    debug!("No Supplier Rows with request value = {}", requestor_value);
                    continue;
                }
                let supplier_rows = supplier_rows.unwrap();
                for supplier_row in supplier_rows {
                    match supplier_row {
                        ActionValue::Map(supplier_row) => {
                            let mut new_row = row.clone();
                            new_row.extend(supplier_row.clone());
                            result.push(ActionValue::Map(new_row));
                        }
                        _ => return Err(Error::validate("Supplier is not a map").into()),
                    }
                }
            }
            _ => return Err(Error::validate("Requestor is not an array").into()),
        }
    }
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
                        let supplier = &props.join.supplier;
                        let supplier_value = row
                            .get(supplier)
                            .ok_or(anyhow!("No Supplier Value By create supplier index"))?;
                        let supplier_index_entry =
                            supplier_indexs.entry(supplier.to_owned()).or_default();
                        supplier_index_entry
                            .entry(supplier_value.to_string())
                            .or_default()
                            .push(ActionValue::Map(row.clone()));
                    }
                    _ => return Err(Error::validate("Supplier is not an array").into()),
                }
            }
            Ok(supplier_indexs)
        }
        _ => Err(Error::validate("Supplier is not an array").into()),
    }
}
