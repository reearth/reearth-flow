use std::{str::FromStr, sync::Arc};

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::executor_operation::NodeContext;

use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Expr;
use schemars::JsonSchema;

use serde::{Deserialize, Serialize};
use crate::errors::SourceError;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileReaderCommonParam {
    /// # File Path
    /// Expression that returns the path to the input file (e.g., "data.csv" or variable reference)
    pub(crate) dataset: Option<Expr>,
    /// # Inline Content
    /// Expression that returns the file content as text instead of reading from a file path
    pub(crate) inline: Option<Expr>,
}

pub(crate) fn get_input_path(
    ctx: &NodeContext,
    common_property: &FileReaderCommonParam,
) -> Result<Option<Uri>, SourceError> {
    let Some(path) = &common_property.dataset else {
        return Ok(None);
    };
    let scope = ctx.expr_engine.new_scope();
    let path = ctx
        .expr_engine
        .eval_scope::<String>(path.as_ref(), &scope)
        .unwrap_or_else(|_| path.to_string());
    let uri = Uri::from_str(path.as_str());
    let Ok(uri) = uri else {
        return Err(crate::errors::SourceError::FileReader(
            "Invalid path".to_string(),
        ));
    };
    Ok(Some(uri))
}

fn get_inline_content(
    ctx: &NodeContext,
    common_property: &FileReaderCommonParam,
) -> Result<Option<Bytes>, SourceError> {
    let Some(inline) = &common_property.inline else {
        return Ok(None);
    };
    let scope = ctx.expr_engine.new_scope();
    let content = ctx
        .expr_engine
        .eval_scope::<String>(inline.as_ref(), &scope)
        .unwrap_or_else(|_| inline.to_string());
    Ok(Some(Bytes::from(content)))
}

pub(crate) async fn get_content(
    ctx: &NodeContext,
    common_property: &FileReaderCommonParam,
    storage_resolver: Arc<StorageResolver>,
) -> Result<Bytes, SourceError> {
    if let Some(content) = get_inline_content(ctx, common_property)? {
        return Ok(content);
    }
    if let Some(input_path) = get_input_path(ctx, common_property)? {
        let storage = storage_resolver
            .resolve(&input_path)
            .map_err(|e| crate::errors::SourceError::FileReader(format!("{e:?}")))?;
        let result = storage
            .get(input_path.path().as_path())
            .await
            .map_err(|e| crate::errors::SourceError::FileReader(format!("{e:?}")))?;
        let byte = result
            .bytes()
            .await
            .map_err(|e| crate::errors::SourceError::FileReader(format!("{e:?}")))?;
        return Ok(byte);
    }
    Err(crate::errors::SourceError::FileReader(
        "Missing required parameter `dataset` or `inline`".to_string(),
    ))
}
