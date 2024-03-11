use std::str::FromStr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use reearth_flow_action::utils::inject_variables_to_scope;
use reearth_flow_action::{
    error::Error, utils, Action, ActionContext, ActionDataframe, ActionResult, ActionValue,
    DEFAULT_PORT,
};
use reearth_flow_common::uri::Uri;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ZipExtractor {
    path: String,
    output_path: Option<String>,
}

#[async_trait::async_trait]
#[typetag::serde(name = "ZipExtractor")]
impl Action for ZipExtractor {
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
        let file_result = storage
            .get(path.path().as_path())
            .await
            .map_err(Error::internal_runtime)?;
        let bytes = file_result.bytes().await.map_err(Error::internal_runtime)?;

        let root_output_path = match &self.output_path {
            Some(output_path) => {
                let path = expr_engine
                    .eval_scope::<String>(output_path, &scope)
                    .map_err(Error::input)?;
                Uri::from_str(path.as_str()).map_err(Error::input)?
            }
            None => {
                let dir = utils::dir::project_output_dir(ctx.node_id.to_string().as_str())?;
                tokio::fs::create_dir_all(&dir).await?;
                Uri::for_test(format!("file://{}", dir).as_str())
            }
        };
        let storage = ctx
            .storage_resolver
            .resolve(&root_output_path)
            .map_err(Error::input)?;
        let result = utils::zip::extract(bytes, root_output_path, storage).await?;
        let output = ActionDataframe::from([(
            DEFAULT_PORT.to_string(),
            Some(ActionValue::String(result.root.to_string())),
        )]);

        Ok(output)
    }
}
