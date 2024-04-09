use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, ActionContext, ActionDataframe, ActionResult, ActionValue, AsyncAction, Port, DEFAULT_PORT
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
    AddStringPrefix(String),
    AddStringSuffix(String),
    RemovePrefixString(String),
    RemoveSuffixString(String),
}

#[async_trait::async_trait]
#[typetag::serde(name = "BulkAttributeRenamer")]
impl AsyncAction for BulkAttributeRenamer {
    async fn run(&self, _ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let output = 
            inputs
                .ok_or(Error::input("no input"))?
                .into_iter()
                .map(|(k, v)| (
                    if k == *DEFAULT_PORT { k } else { Port::new(rename_key(k.into_inner(), &self.action)) },
                    match v {
                        Some(ActionValue::Map(kv)) => Some(ActionValue::Map(rename(kv, &self.action))),
                        x => x
                    },
                ))
                .collect();
        Ok(output)
    }
}

fn rename(inputs: HashMap<String, ActionValue>, action: &RenameAction) -> HashMap<String, ActionValue> {
    inputs.into_iter().map(|(k, v)|
        ( rename_key(k, action),
          match v {
            ActionValue::Map(kv) => ActionValue::Map(rename(kv, action)),
            x => x,
          }
        )
    ).collect()
}

fn rename_key(k: String, action: &RenameAction) -> String {
    match action {
        RenameAction::AddStringPrefix(p) => format!("{}{}", p, k),
        RenameAction::AddStringSuffix(s) => format!("{}{}", k, s),
        RenameAction::RemovePrefixString(p) => k.strip_prefix(p).unwrap_or(&k).to_string(),
        RenameAction::RemoveSuffixString(s) => k.strip_suffix(s).unwrap_or(&k).to_string(),
        _ => unimplemented!(),
    }
}