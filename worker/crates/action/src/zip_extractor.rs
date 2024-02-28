use core::result::Result;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use async_zip::base::read::mem::ZipFileReader;
use directories::ProjectDirs;
use futures::AsyncReadExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use reearth_flow_common::uri::Uri;

use crate::action::{ActionContext, ActionDataframe, ActionValue, DEFAULT_PORT};
use crate::utils::inject_variables_to_scope;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PropertySchema {
    path: String,
    output_path: Option<String>,
}

property_schema!(PropertySchema);

pub(crate) async fn run(
    ctx: ActionContext,
    inputs: Option<ActionDataframe>,
) -> anyhow::Result<ActionDataframe> {
    let props = PropertySchema::try_from(ctx.node_property)?;
    info!(?props, "read");
    let inputs = inputs.unwrap_or_default();
    let expr_engine = Arc::clone(&ctx.expr_engine);

    let scope = expr_engine.new_scope();
    inject_variables_to_scope(&inputs, &scope)?;
    let path = expr_engine
        .eval_scope::<String>(&props.path, &scope)
        .and_then(|s| Uri::from_str(s.as_str()))?;

    let storage = ctx.storage_resolver.resolve(&path)?;
    let file_result = storage.get(path.path().as_path()).await?;
    let bytes = file_result.bytes().await?;
    let reader = ZipFileReader::new(bytes.to_vec()).await?;

    let root_output_path = match &props.output_path {
        Some(output_path) => expr_engine
            .eval_scope::<String>(output_path, &scope)
            .and_then(|s| Uri::from_str(s.as_str()))?,
        None => {
            let p = ProjectDirs::from("reearth", "flow", "worker")
                .ok_or(anyhow!("No output path uri provided"))?;
            let p = p
                .data_dir()
                .to_str()
                .ok_or(anyhow!("Invalid output path uri"))?;
            let p = format!("{}/output/zip-extractor/{}", p, ctx.node_id);
            tokio::fs::create_dir_all(std::path::Path::new(p.as_str())).await?;
            Uri::for_test(format!("file://{}", p).as_str())
        }
    };
    let storage = ctx.storage_resolver.resolve(&root_output_path)?;
    let mut output = ActionDataframe::new();

    for i in 0..reader.file().entries().len() {
        let entry = reader.file().entries().get(i).ok_or(anyhow!("No entry"))?;
        let filename = entry.filename().as_str()?;
        if i == 0 {
            let file_uri = filename
                .split('/')
                .next()
                .ok_or(anyhow!("No file name"))
                .and_then(|s| root_output_path.join(s))?;
            output.insert(
                DEFAULT_PORT.to_string(),
                Some(ActionValue::String(file_uri.to_string())),
            );
        }
        let outpath = root_output_path.join(filename)?;
        let entry_is_dir = filename.ends_with('/');
        if entry_is_dir {
            if storage.exists(outpath.path().as_path()).await? {
                continue;
            }
            storage.create_dir(outpath.path().as_path()).await?;
            continue;
        }
        if let Some(p) = outpath.parent() {
            if !storage.exists(p.path().as_path()).await? {
                storage.create_dir(p.path().as_path()).await?;
            }
        }
        let mut entry_reader = reader.reader_without_entry(i).await?;
        let mut buf = Vec::<u8>::new();
        entry_reader.read_to_end(&mut buf).await?;
        storage
            .put(outpath.path().as_path(), bytes::Bytes::from(buf))
            .await?;
    }
    Ok(output)
}
