use std::{collections::HashMap, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};

use reearth_flow_common::uri::Uri;
use reearth_flow_common::xml;

use reearth_flow_action::utils::inject_variables_to_scope;
use reearth_flow_action::{
    error::Error, Action, ActionContext, ActionDataframe, ActionResult, ActionValue, Port,
};

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
#[typetag::serde(name = "XMLXPathExtractor")]
impl Action for XmlXPathExtractor {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.unwrap_or_default();
        let expr_engine = Arc::clone(&ctx.expr_engine);

        let scope = expr_engine.new_scope();
        inject_variables_to_scope(&inputs, &scope)?;
        let path = expr_engine
            .eval_scope::<String>(&self.path, &scope)
            .map_err(Error::input)?;
        let path = Uri::from_str(path.as_str()).map_err(Error::input)?;

        let storage = ctx.storage_resolver.resolve(&path).map_err(Error::input)?;
        let result = storage
            .get(path.path().as_path())
            .await
            .map_err(Error::internal_runtime)?;
        let byte = result.bytes().await.map_err(Error::internal_runtime)?;
        let text = String::from_utf8(byte.to_vec()).map_err(Error::internal_runtime)?;
        let document = xml::parse(text).map_err(Error::internal_runtime)?;

        let mut output = HashMap::<Port, ActionValue>::new();
        for condition in &self.conditions {
            let xpath = &condition.xpath;
            let output_port = &condition.output_port;
            let evaluation_result =
                xml::evaluate(&document, xpath).map_err(Error::internal_runtime)?;
            output.insert(output_port.clone(), evaluation_result.into());
        }
        Ok(output
            .iter()
            .map(|(k, v)| (k.clone(), Some(v.clone())))
            .collect::<ActionDataframe>())
    }
}
