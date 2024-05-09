use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use reearth_flow_action::{ActionContext, ActionDataframe, ActionResult, AsyncAction, Port};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DataframeRouter {
    routings: Vec<Routing>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Routing {
    source_port: Port,
    output_port: Port,
}

#[async_trait::async_trait]
#[typetag::serde(name = "DataframeRouter")]
impl AsyncAction for DataframeRouter {
    async fn run(&self, _ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let mappings = self
            .routings
            .iter()
            .map(|r| (r.source_port.clone(), r.output_port.clone()))
            .collect::<HashMap<_, _>>();
        let output = inputs
            .into_iter()
            .map(|(port, df)| {
                let output_port = mappings.get(&port).unwrap_or(&port);
                (output_port.clone(), df)
            })
            .collect::<HashMap<_, _>>();
        Ok(output)
    }
}
