use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, utils, Action, ActionContext, ActionDataframe, ActionResult, ActionValue,
    DEFAULT_PORT,
};
use reearth_flow_common::uri::Uri;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FilePathExtractor {
    source_dataset: String,
    extract_archive: bool,
}

#[async_trait::async_trait]
#[typetag::serde(name = "PLATEAU.FilePathExtractor")]
impl Action for FilePathExtractor {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.unwrap_or_default();
        let source_dataset =
            utils::get_expr_path(&self.source_dataset, &inputs, Arc::clone(&ctx.expr_engine))
                .await?;
        if self.is_extractable_archive(&source_dataset) {
            let root_output_path =
                utils::dir::project_output_dir(ctx.node_id.to_string().as_str())?;
            tokio::fs::create_dir_all(&root_output_path).await?;
            let root_output_path = Uri::for_test(format!("file://{}", root_output_path).as_str());
            let storage = ctx
                .storage_resolver
                .resolve(&source_dataset)
                .map_err(Error::input)?;
            let file_result = storage
                .get(source_dataset.path().as_path())
                .await
                .map_err(Error::internal_runtime)?;
            let bytes = file_result.bytes().await.map_err(Error::internal_runtime)?;
            let storage = ctx
                .storage_resolver
                .resolve(&root_output_path)
                .map_err(Error::input)?;
            let result = utils::zip::extract(bytes, root_output_path, storage).await?;
            let values = result
                .entries
                .into_iter()
                .map(|entry| {
                    ActionValue::Map(HashMap::from([(
                        "path".to_string(),
                        ActionValue::try_from(entry).unwrap_or_default(),
                    )]))
                })
                .collect::<Vec<ActionValue>>();

            Ok(ActionDataframe::from([(
                DEFAULT_PORT.to_string(),
                Some(ActionValue::Array(values)),
            )]))
        } else {
            Ok(ActionDataframe::from([(
                DEFAULT_PORT.to_string(),
                Some(ActionValue::Array(vec![ActionValue::Map(HashMap::from([
                    ("path".to_string(), ActionValue::try_from(source_dataset)?),
                ]))])),
            )]))
        }
    }
}

impl FilePathExtractor {
    fn is_extractable_archive(&self, path: &Uri) -> bool {
        self.extract_archive
            && !path.is_dir()
            && path.extension().is_some()
            && matches!(path.extension().unwrap(), "zip" | "7z" | "7zip")
    }
}
