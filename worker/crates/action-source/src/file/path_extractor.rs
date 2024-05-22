use std::{str::FromStr, sync::Arc};

use async_zip::base::read::mem::ZipFileReader;
use futures::AsyncReadExt;
use reearth_flow_common::{dir, uri::Uri};
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_runtime::{
    errors::BoxedError,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, DEFAULT_PORT},
};
use reearth_flow_storage::storage::Storage;
use reearth_flow_types::{AttributeValue, Expr, Feature, FilePath};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

use crate::universal::UniversalSource;

pub async fn extract(
    bytes: bytes::Bytes,
    root_output_path: Uri,
    storage: Arc<Storage>,
    sender: Sender<(Port, IngestionMessage)>,
) -> crate::errors::Result<()> {
    let reader = ZipFileReader::new(bytes.to_vec())
        .await
        .map_err(|e| crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e)))?;
    if reader.file().entries().is_empty() {
        return Err(crate::errors::UniversalSourceError::FilePathExtractor(
            "No entries".to_string(),
        ));
    }
    for i in 0..reader.file().entries().len() {
        let entry = reader.file().entries().get(i).ok_or(
            crate::errors::UniversalSourceError::FilePathExtractor("No entry".to_string()),
        )?;
        let filename = entry.filename().as_str().map_err(|e| {
            crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
        })?;
        let outpath = root_output_path.join(filename).map_err(|e| {
            crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
        })?;
        let entry_is_dir = filename.ends_with('/');
        if entry_is_dir {
            if storage
                .exists(outpath.path().as_path())
                .await
                .map_err(|e| {
                    crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
                })?
            {
                continue;
            }
            storage
                .create_dir(outpath.path().as_path())
                .await
                .map_err(|e| {
                    crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
                })?;
            continue;
        }
        if let Some(p) = outpath.parent() {
            if !storage.exists(p.path().as_path()).await.map_err(|e| {
                crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
            })? {
                storage.create_dir(p.path().as_path()).await.map_err(|e| {
                    crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
                })?;
            }
        }
        let mut entry_reader = reader.reader_without_entry(i).await.map_err(|e| {
            crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
        })?;
        let mut buf = Vec::<u8>::new();
        entry_reader.read_to_end(&mut buf).await.map_err(|e| {
            crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
        })?;
        storage
            .put(outpath.path().as_path(), bytes::Bytes::from(buf))
            .await
            .map_err(|e| {
                crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
            })?;
        let file_path = FilePath::try_from(outpath.clone()).map_err(|e| {
            crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
        })?;
        let attribute_value = AttributeValue::try_from(file_path).map_err(|e| {
            crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
        })?;
        let feature = Feature::from(attribute_value);
        sender
            .send((
                DEFAULT_PORT.clone(),
                IngestionMessage::OperationEvent { feature },
            ))
            .await
            .map_err(|e| {
                crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
            })?;
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilePathExtractor {
    source_dataset: Expr,
    extract_archive: bool,
}

#[async_trait::async_trait]
#[typetag::serde(name = "FilePathExtractor")]
impl UniversalSource for FilePathExtractor {
    async fn initialize(&self, _ctx: NodeContext) {}

    async fn serialize_state(&self) -> Result<Vec<u8>, BoxedError> {
        Ok(vec![])
    }

    async fn start(
        &mut self,
        ctx: NodeContext,
        sender: Sender<(Port, IngestionMessage)>,
    ) -> Result<(), BoxedError> {
        let source_dataset = get_expr_path(&self.source_dataset, ctx.expr_engine.clone())?;
        if self.is_extractable_archive(&source_dataset) {
            let root_output_path =
                dir::project_output_dir(uuid::Uuid::new_v4().to_string().as_str())?;
            let root_output_path = Uri::for_test(&root_output_path);
            let source_dataset_storage =
                ctx.storage_resolver.resolve(&source_dataset).map_err(|e| {
                    crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
                })?;
            let file_result = source_dataset_storage
                .get(source_dataset.path().as_path())
                .await
                .map_err(|e| {
                    crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
                })?;
            let bytes = file_result.bytes().await.map_err(|e| {
                crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
            })?;
            let root_output_storage =
                ctx.storage_resolver
                    .resolve(&root_output_path)
                    .map_err(|e| {
                        crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
                    })?;
            root_output_storage
                .create_dir(root_output_path.path().as_path())
                .await
                .map_err(|e| {
                    crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
                })?;
            extract(bytes, root_output_path, root_output_storage, sender).await?;
        } else {
            let storage = ctx.storage_resolver.resolve(&source_dataset).map_err(|e| {
                crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
            })?;
            let entries = storage
                .list_with_result(Some(source_dataset.path().as_path()), true)
                .await
                .map_err(|e| {
                    crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
                })?;
            for entry in entries {
                let attribute_value =
                    AttributeValue::try_from(FilePath::try_from(entry).unwrap_or_default())?;
                let feature = Feature::from(attribute_value);
                sender
                    .send((
                        DEFAULT_PORT.clone(),
                        IngestionMessage::OperationEvent { feature },
                    ))
                    .await
                    .map_err(|e| {
                        crate::errors::UniversalSourceError::FilePathExtractor(format!("{:?}", e))
                    })?;
            }
        }
        Ok(())
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

fn get_expr_path<T: AsRef<str> + std::fmt::Display>(
    path: &T,
    expr_engine: Arc<Engine>,
) -> crate::errors::Result<Uri> {
    let scope = expr_engine.new_scope();
    let path = expr_engine
        .eval_scope::<String>(path.as_ref(), &scope)
        .unwrap_or_else(|_| path.to_string());
    Uri::from_str(path.as_str()).map_err(|_| {
        crate::errors::UniversalSourceError::FilePathExtractor("Invalid path".to_string())
    })
}
