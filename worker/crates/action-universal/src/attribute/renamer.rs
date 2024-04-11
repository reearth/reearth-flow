use regex::Regex;
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, ActionContext, ActionDataframe, ActionResult, ActionValue, AsyncAction,
    DEFAULT_PORT,
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
    StringReplace(ReplaceString),
    RegularExpressionReplace(ReplaceString),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReplaceString {
    from: String,
    to: String,
}

#[async_trait::async_trait]
#[typetag::serde(name = "BulkAttributeRenamer")]
impl AsyncAction for BulkAttributeRenamer {
    async fn run(&self, _ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(Error::input("no input"))?;
        let default = inputs.get(&DEFAULT_PORT).ok_or(Error::input("no default port"))?;
        let mut output = ActionDataframe::new();
        match default {
            Some(av@ActionValue::Array(_)) => {
                output.insert(DEFAULT_PORT.clone(), Some(rename(av.clone(), &self.action)));
                Ok(output)
            },
            _ => Err(Error::input("input must be Array")),
        }
    }
}

fn rename(
    value: ActionValue,
    action: &RenameAction,
) -> ActionValue {
    match value {
        ActionValue::Map(kv) => 
            ActionValue::Map(
                kv.into_iter().map(|(k,v)| (rename_key(k, action), rename(v, action))).collect()
            ),
        ActionValue::Array(xs) =>
            ActionValue::Array(
                xs.into_iter().map(|v| rename(v, action)).collect()
            ),
        x => x
    }
}

fn rename_key(k: String, action: &RenameAction) -> String {
    match action {
        RenameAction::AddStringPrefix(p) => format!("{}{}", p, k),
        RenameAction::AddStringSuffix(s) => format!("{}{}", k, s),
        RenameAction::RemovePrefixString(p) => k.strip_prefix(p).unwrap_or(&k).to_string(),
        RenameAction::RemoveSuffixString(s) => k.strip_suffix(s).unwrap_or(&k).to_string(),
        RenameAction::StringReplace(ReplaceString { from, to }) => k.replace(from, to),
        RenameAction::RegularExpressionReplace(ReplaceString { from, to }) => {
            Regex::new(from).unwrap().replace_all(&k, to).to_string()
        }
    }
}
