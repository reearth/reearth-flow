use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    ActionContext, ActionDataframe, ActionResult, AsyncAction, Dataframe, Port, DEFAULT_PORT,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DataframeUnion {
    source_ports: Vec<Port>,
}

#[async_trait::async_trait]
#[typetag::serde(name = "DataframeUnion")]
impl AsyncAction for DataframeUnion {
    async fn run(&self, _ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let mut output = ActionDataframe::new();
        let values = inputs
            .into_iter()
            .filter(|(port, _)| self.source_ports.contains(port))
            .map(|(_, df)| df.features)
            .collect::<Vec<_>>();
        output.insert(
            DEFAULT_PORT.clone(),
            Dataframe::new(values.into_iter().flatten().collect::<Vec<_>>()),
        );
        Ok(output)
    }
}
