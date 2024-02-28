use core::result::Result;
use std::{collections::HashMap, str::FromStr, sync::Arc};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use reearth_flow_common::uri::Uri;
use reearth_flow_common::xml;

use crate::action::{ActionContext, ActionDataframe, ActionValue, Port};
use crate::utils::inject_variables_to_scope;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PropertySchema {
    path: String,
    conditions: Vec<Condition>,
}

property_schema!(PropertySchema);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Condition {
    xpath: String,
    output_port: String,
}

pub(crate) async fn run(
    ctx: ActionContext,
    inputs: Option<ActionDataframe>,
) -> anyhow::Result<ActionDataframe> {
    let props = PropertySchema::try_from(ctx.node_property)?;
    debug!(?props, "read");
    let inputs = inputs.unwrap_or_default();
    let expr_engine = Arc::clone(&ctx.expr_engine);

    let scope = expr_engine.new_scope();
    inject_variables_to_scope(&inputs, &scope)?;
    let path = expr_engine
        .eval_scope::<String>(&props.path, &scope)
        .and_then(|s| Uri::from_str(s.as_str()))?;

    let storage = ctx.storage_resolver.resolve(&path)?;
    let result = storage.get(path.path().as_path()).await?;
    let byte = result.bytes().await?;
    let text = String::from_utf8(byte.to_vec())?;
    let document = xml::parse(text)?;

    let mut output = HashMap::<Port, ActionValue>::new();
    for condition in &props.conditions {
        let xpath = &condition.xpath;
        let output_port = &condition.output_port;
        let evaluation_result = xml::evaluate(&document, xpath)?;
        output.insert(output_port.clone(), evaluation_result.into());
    }
    Ok(output
        .iter()
        .map(|(k, v)| (k.clone(), Some(v.clone())))
        .collect::<ActionDataframe>())
}
