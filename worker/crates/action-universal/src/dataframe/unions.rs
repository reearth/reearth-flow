use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, Action, ActionContext, ActionDataframe, ActionResult, ActionValue, Port,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DataframeUnion {
    union_input_ports: Vec<Port>,
    output_port: Port,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct Operation {
    transform_expr: String,
    target_port: Port,
}

#[async_trait::async_trait]
#[typetag::serde(name = "DataframeUnion")]
impl Action for DataframeUnion {
    async fn run(&self, _ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(Error::input("No Input"))?;
        let output = self
            .union_input_ports
            .iter()
            .fold(None, |acc: Option<ActionValue>, item| {
                let data = inputs.get(item).cloned().unwrap_or_default();
                match data {
                    Some(data) => match acc {
                        Some(acc) => acc.extend(data).ok(),
                        None => Some(data),
                    },
                    None => acc,
                }
            });
        Ok(ActionDataframe::from([(self.output_port.clone(), output)]))
    }
}
