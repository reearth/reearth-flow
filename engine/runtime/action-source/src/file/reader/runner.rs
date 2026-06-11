use std::{str::FromStr, sync::Arc};

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::executor_operation::NodeContext;

use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{Code, CompiledCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileReaderCommonParam {
    /// # File Path
    /// Expression that returns the path to the input file (e.g., "data.csv" or variable reference)
    pub(crate) dataset: Option<Code>,
    /// # Inline Content
    /// Expression that returns the file content as text instead of reading from a file path
    pub(crate) inline: Option<Code>,
}

impl FileReaderCommonParam {
    pub(crate) fn compile(self) -> Result<FileReaderCompiledParam, String> {
        Ok(FileReaderCompiledParam {
            dataset: self
                .dataset
                .map(|c| c.compile().map_err(|e| format!("dataset: {e}")))
                .transpose()?,
            inline: self
                .inline
                .map(|c| c.compile().map_err(|e| format!("inline: {e}")))
                .transpose()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FileReaderCompiledParam {
    pub(crate) dataset: Option<CompiledCode>,
    pub(crate) inline: Option<CompiledCode>,
}

pub(crate) fn get_input_path(
    ctx: &NodeContext,
    param: &FileReaderCompiledParam,
) -> Result<Option<Uri>, String> {
    let Some(ref dataset) = param.dataset else {
        return Ok(None);
    };
    let path = dataset
        .eval_string_env_only(ctx.expr_engine.vars())
        .map_err(|e| format!("{e:?}"))?;
    if path.is_empty() {
        return Ok(None);
    }
    Uri::from_str(path.as_str())
        .map(Some)
        .map_err(|e| format!("Invalid path {path:?}: {e}"))
}

fn get_inline_content(
    ctx: &NodeContext,
    param: &FileReaderCompiledParam,
) -> Result<Option<Bytes>, String> {
    let Some(ref inline) = param.inline else {
        return Ok(None);
    };
    let content = inline
        .eval_string_env_only(ctx.expr_engine.vars())
        .map_err(|e| format!("{e:?}"))?;
    Ok(Some(Bytes::from(content)))
}

pub(crate) async fn get_content(
    ctx: &NodeContext,
    param: &FileReaderCompiledParam,
    storage_resolver: Arc<StorageResolver>,
) -> Result<Bytes, String> {
    if let Some(content) = get_inline_content(ctx, param)? {
        return Ok(content);
    }
    if let Some(input_path) = get_input_path(ctx, param)? {
        let storage = storage_resolver
            .resolve(&input_path)
            .map_err(|e| format!("{e:?}"))?;
        let result = storage
            .get(input_path.path().as_path())
            .await
            .map_err(|e| format!("{e:?}"))?;
        let byte = result.bytes().await.map_err(|e| format!("{e:?}"))?;
        return Ok(byte);
    }
    Err("Missing required parameter `dataset` or `inline`".to_string())
}
