use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use reearth_flow_action::error::Error;
use reearth_flow_action::{
    ActionContext, ActionDataframe, ActionResult, AsyncAction, Dataframe, Feature, FeatureIndex,
    Port, Result, DEFAULT_PORT,
};

pub static REQUESTOR_PORT: Lazy<Port> = Lazy::new(|| Port::new("requestor"));
pub static SUPPLIER_PORT: Lazy<Port> = Lazy::new(|| Port::new("supplier"));
const ROW_NUMBER: &str = "row_number";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttributeMerger {
    join: Join,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Join {
    requestor: String,
    supplier: String,
}

#[async_trait::async_trait]
#[typetag::serde(name = "AttributeMerger")]
impl AsyncAction for AttributeMerger {
    async fn run(&self, ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let requestor = inputs
            .get(&REQUESTOR_PORT)
            .ok_or(Error::input("No Requestor Port"))?;

        let supplier = inputs
            .get(&SUPPLIER_PORT)
            .ok_or(Error::input("No Supplier Port"))?;
        let requestor_key = &self.join.requestor;
        let supplier_key = &self.join.supplier;
        let is_row_number_join = requestor_key == ROW_NUMBER && supplier_key == ROW_NUMBER;
        let supplier_indexs = if is_row_number_join {
            FeatureIndex::new()
        } else {
            self.create_supplier_index(&supplier.features)?
        };
        let mut result = Vec::<Feature>::new();
        for (idx, row) in requestor.features.iter().enumerate() {
            if is_row_number_join {
                if let Some(supplier_row) = supplier.features.get(idx) {
                    let mut new_row = row.clone();
                    let attributes = supplier_row.attributes.clone();
                    new_row.attributes.extend(attributes);
                    result.push(new_row);
                }
                continue;
            }
            let requestor_value = row.get(requestor_key).ok_or(Error::validate(
                format!("No Requestor Value with requestor = {}", requestor_key).as_str(),
            ))?;
            let supplier_rows = supplier_indexs.get(&requestor_value.to_string());
            let Some(supplier_rows) = supplier_rows else {
                ctx.action_log(format!(
                    "No Supplier Rows with request value = {}",
                    requestor_value
                ));
                continue;
            };
            for supplier_row in supplier_rows {
                let mut new_row = row.clone();
                let attributes = supplier_row.attributes.clone();
                new_row.attributes.extend(attributes);
                result.push(new_row);
            }
        }
        Ok(vec![(DEFAULT_PORT.clone(), Dataframe::new(result))]
            .into_iter()
            .collect::<HashMap<_, _>>())
    }
}

impl AttributeMerger {
    fn create_supplier_index(&self, supplier: &Vec<Feature>) -> Result<FeatureIndex> {
        let mut supplier_indexs = FeatureIndex::new();
        for feature in supplier {
            let supplier = &self.join.supplier;
            let supplier_value = feature.get(supplier).ok_or(Error::validate(
                "No Supplier Value By create supplier index",
            ))?;
            supplier_indexs
                .entry(supplier_value.to_string())
                .or_default()
                .push(feature.clone());
        }
        Ok(supplier_indexs)
    }
}
