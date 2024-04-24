use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::{self, Error},
    types, utils, ActionContext, ActionDataframe, ActionResult, AsyncAction, AttributeValue,
    Dataframe, DEFAULT_PORT,
};
use reearth_flow_common::uri::Uri;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FilePathExtractor {
    source_dataset: String,
    extract_archive: bool,
}

#[async_trait::async_trait]
#[typetag::serde(name = "FilePathExtractor")]
impl AsyncAction for FilePathExtractor {
    async fn run(&self, ctx: ActionContext, _inputs: ActionDataframe) -> ActionResult {
        let source_dataset = ctx.get_expr_path(&self.source_dataset).await?;
        if self.is_extractable_archive(&source_dataset) {
            let root_output_path =
                utils::dir::project_output_dir(ctx.node_id.to_string().as_str())?;
            let root_output_path = Uri::for_test(&root_output_path);
            let source_dataset_storage = ctx
                .storage_resolver
                .resolve(&source_dataset)
                .map_err(Error::input)?;
            let file_result = source_dataset_storage
                .get(source_dataset.path().as_path())
                .await
                .map_err(Error::internal_runtime)?;
            let bytes = file_result.bytes().await.map_err(Error::internal_runtime)?;
            let root_output_storage = ctx
                .storage_resolver
                .resolve(&root_output_path)
                .map_err(Error::input)?;
            root_output_storage
                .create_dir(root_output_path.path().as_path())
                .await
                .map_err(error::Error::input)?;
            let result = utils::zip::extract(bytes, root_output_path, root_output_storage).await?;
            let values = result
                .entries
                .into_iter()
                .map(|entry| {
                    AttributeValue::try_from(
                        types::file::FilePath::try_from(entry).unwrap_or_default(),
                    )
                    .unwrap_or_default()
                })
                .collect::<Vec<AttributeValue>>();

            Ok(ActionDataframe::from([(
                DEFAULT_PORT.clone(),
                Dataframe::from(values),
            )]))
        } else {
            let storage = ctx
                .storage_resolver
                .resolve(&source_dataset)
                .map_err(Error::input)?;
            let entries = storage
                .list_with_result(Some(source_dataset.path().as_path()), true)
                .await
                .map_err(error::Error::input)?;

            let values = entries
                .into_iter()
                .map(|entry| {
                    AttributeValue::try_from(
                        types::file::FilePath::try_from(entry).unwrap_or_default(),
                    )
                    .unwrap_or_default()
                })
                .collect::<Vec<AttributeValue>>();
            Ok(ActionDataframe::from([(
                DEFAULT_PORT.clone(),
                Dataframe::from(values),
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
