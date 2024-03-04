use std::{collections::HashMap, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};

use reearth_flow_common::uri::Uri;
use reearth_flow_common::xml;

use crate::action::{Action, ActionContext, ActionDataframe, ActionResult, ActionValue, Port};
use crate::utils::inject_variables_to_scope;

#[derive(Serialize, Deserialize, Debug)]
pub struct XmlXPathExtractor {
    path: String,
    conditions: Vec<Condition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Condition {
    xpath: String,
    output_port: String,
}

#[async_trait::async_trait]
#[typetag::serde(name = "xmlXPathExtractor")]
impl Action for XmlXPathExtractor {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.unwrap_or_default();
        let expr_engine = Arc::clone(&ctx.expr_engine);

        let scope = expr_engine.new_scope();
        inject_variables_to_scope(&inputs, &scope)?;
        let path = expr_engine
            .eval_scope::<String>(&self.path, &scope)
            .and_then(|s| Uri::from_str(s.as_str()))?;

        let storage = ctx.storage_resolver.resolve(&path)?;
        let result = storage.get(path.path().as_path()).await?;
        let byte = result.bytes().await?;
        let text = String::from_utf8(byte.to_vec())?;
        let document = xml::parse(text)?;

        let mut output = HashMap::<Port, ActionValue>::new();
        for condition in &self.conditions {
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
}
