use regex::Regex;
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, ActionContext, ActionDataframe, ActionResult, AsyncAction, Attribute, Dataframe,
    Feature, DEFAULT_PORT,
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
    async fn run(&self, _ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let dataframe = inputs
            .get(&DEFAULT_PORT)
            .ok_or(Error::input("no default port"))?;
        let features = dataframe
            .features
            .iter()
            .map(|data| rename(data, &self.action))
            .collect::<Vec<_>>();
        Ok(ActionDataframe::from([(
            DEFAULT_PORT.clone(),
            Dataframe::new(features),
        )]))
    }
}

fn rename(feature: &Feature, action: &RenameAction) -> Feature {
    let attributes = feature
        .attributes
        .iter()
        .map(|(k, v)| (rename_key(k.clone(), action), v.clone()))
        .collect();
    feature.with_attributes(attributes)
}

fn rename_key(k: Attribute, action: &RenameAction) -> Attribute {
    let inner = k.into_inner();
    let key = match action {
        RenameAction::AddStringPrefix(p) => format!("{}{}", p, inner),
        RenameAction::AddStringSuffix(s) => format!("{}{}", inner, s),
        RenameAction::RemovePrefixString(p) => inner.strip_prefix(p).unwrap_or(&inner).to_string(),
        RenameAction::RemoveSuffixString(s) => inner.strip_suffix(s).unwrap_or(&inner).to_string(),
        RenameAction::StringReplace(ReplaceString { from, to }) => inner.replace(from, to),
        RenameAction::RegularExpressionReplace(ReplaceString { from, to }) => Regex::new(from)
            .unwrap()
            .replace_all(&inner, to)
            .to_string(),
    };
    Attribute::new(key)
}
