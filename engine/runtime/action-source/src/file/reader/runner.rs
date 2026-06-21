use std::{str::FromStr, sync::Arc};

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::executor_operation::NodeContext;

use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{AttributeValue, Code};
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
    pub(crate) fn compile(self, ctx: &NodeContext) -> Result<FileReaderCompiledParam, String> {
        let dataset = self
            .dataset
            .map(|c| {
                let compiled = c.compile().map_err(|e| format!("dataset compile: {e}"))?;
                match compiled.eval_env_only(ctx.expr_engine.vars()) {
                    Ok(AttributeValue::Null) => Ok::<Option<String>, String>(None),
                    _ => {
                        let s = compiled
                            .eval_string_env_only(ctx.expr_engine.vars())
                            .map_err(|e| format!("dataset eval: {e}"))?;
                        Ok(if s.is_empty() { None } else { Some(s) })
                    }
                }
            })
            .transpose()?
            .flatten();
        let inline = self
            .inline
            .map(|c| {
                let compiled = c.compile().map_err(|e| format!("inline compile: {e}"))?;
                let s = compiled
                    .eval_string_env_only(ctx.expr_engine.vars())
                    .map_err(|e| format!("inline eval: {e}"))?;
                Ok::<Bytes, String>(Bytes::from(s))
            })
            .transpose()?;
        Ok(FileReaderCompiledParam { dataset, inline })
    }
}

#[derive(Debug, Clone)]
pub struct FileReaderCompiledParam {
    /// Pre-evaluated dataset path; `None` means absent or expression evaluated to null.
    pub(crate) dataset: Option<String>,
    /// Pre-evaluated inline content; `None` means not provided.
    pub(crate) inline: Option<Bytes>,
}

pub(crate) fn get_input_path(param: &FileReaderCompiledParam) -> Result<Option<Uri>, String> {
    let Some(ref path) = param.dataset else {
        return Ok(None);
    };
    Uri::from_str(path.as_str())
        .map(Some)
        .map_err(|e| format!("Invalid path {path:?}: {e}"))
}

pub(crate) async fn get_content(
    param: &FileReaderCompiledParam,
    storage_resolver: Arc<StorageResolver>,
) -> Result<Bytes, String> {
    if let Some(ref content) = param.inline {
        return Ok(content.clone());
    }
    if let Some(input_path) = get_input_path(param)? {
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
