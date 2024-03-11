use std::sync::Arc;

use async_zip::base::read::mem::ZipFileReader;
use futures::AsyncReadExt;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::storage::Storage;

pub struct ZipExtract {
    pub root: Uri,
    pub entries: Vec<Uri>,
}

impl ZipExtract {
    pub fn new(root: Uri, entries: Vec<Uri>) -> Self {
        Self { root, entries }
    }
}

pub async fn extract(
    bytes: bytes::Bytes,
    root_output_path: Uri,
    storage: Arc<Storage>,
) -> crate::Result<ZipExtract> {
    let reader = ZipFileReader::new(bytes.to_vec())
        .await
        .map_err(crate::error::Error::input)?;

    let mut root: Option<Uri> = None;
    let mut entries = Vec::new();

    for i in 0..reader.file().entries().len() {
        let entry = reader
            .file()
            .entries()
            .get(i)
            .ok_or(crate::error::Error::internal_runtime("No entry"))?;
        let filename = entry
            .filename()
            .as_str()
            .map_err(crate::error::Error::internal_runtime)?;
        if i == 0 {
            let file_uri = filename
                .split('/')
                .next()
                .ok_or(crate::error::Error::internal_runtime("No file name"))?;
            let file_uri = root_output_path
                .join(file_uri)
                .map_err(crate::error::Error::internal_runtime)?;
            root = Some(file_uri);
        }
        let outpath = root_output_path
            .join(filename)
            .map_err(crate::error::Error::internal_runtime)?;
        let entry_is_dir = filename.ends_with('/');
        if entry_is_dir {
            if storage
                .exists(outpath.path().as_path())
                .await
                .map_err(crate::error::Error::internal_runtime)?
            {
                continue;
            }
            storage
                .create_dir(outpath.path().as_path())
                .await
                .map_err(crate::error::Error::internal_runtime)?;
            continue;
        }
        if let Some(p) = outpath.parent() {
            if !storage
                .exists(p.path().as_path())
                .await
                .map_err(crate::error::Error::internal_runtime)?
            {
                storage
                    .create_dir(p.path().as_path())
                    .await
                    .map_err(crate::error::Error::internal_runtime)?;
            }
        }
        entries.push(outpath.clone());
        let mut entry_reader = reader
            .reader_without_entry(i)
            .await
            .map_err(crate::error::Error::internal_runtime)?;
        let mut buf = Vec::<u8>::new();
        entry_reader.read_to_end(&mut buf).await?;
        storage
            .put(outpath.path().as_path(), bytes::Bytes::from(buf))
            .await
            .map_err(crate::error::Error::internal_runtime)?;
    }
    Ok(ZipExtract::new(
        root.ok_or(crate::error::Error::internal_runtime("No root"))?,
        entries,
    ))
}
