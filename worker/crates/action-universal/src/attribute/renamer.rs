use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, Action, ActionContext, ActionDataframe, ActionResult, ActionValue
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BulkAttributeRenamer {
    rename: Rename,
    action: RenameAction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) enum Rename {
    AllAttributes,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) enum RenameAction {
    AddStringPreffix(String),
    AddStringSuffix(String),
    RemovePrefixString(String),
    RemoveSuffixString(String),
}

#[async_trait::async_trait]
#[typetag::serde(name = "BulkAttributeRenamer")]
impl Action for BulkAttributeRenamer {
    async fn run(&self, _ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let output = inputs
        Ok(output)
    }
}

fn rename(inputs: ActionDataframe, action: Rename) -> ActionResult {
    inputs.into_iter.map(|(k, v)|
        ( k,
          match v {
            Some(ActionValue::Map(kv) => unimplemented!(),
            x => Some(x),
          }
        )
    )
}

fn rename_port(p: Re)