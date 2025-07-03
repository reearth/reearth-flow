use std::{
    io::{Cursor, Read},
    path::Path,
    sync::Arc,
};

use reearth_flow_common::uri::Uri;
use reearth_flow_sevenz::{decompress_with_extract_fn, default_entry_extract_fn};
use reearth_flow_storage::{resolve::StorageResolver, storage::Storage};
use reearth_flow_types::FilePath;

pub(crate) fn is_extractable_archive(path: &Uri) -> bool {
    if let Some(ext) = path.path().as_path().extension() {
        if let Some(ext) = ext.to_str() {
            return ["zip", "7z", "7zip"].contains(&ext);
        }
    }
    false
}

pub(crate) fn extract_archive(
    source_dataset: &Uri,
    root_output_path: &Uri,
    storage_resolver: Arc<StorageResolver>,
) -> super::errors::Result<Vec<FilePath>> {
    let storage = storage_resolver.resolve(source_dataset).map_err(|e| {
        super::errors::ProcessorUtilError::Decompressor(format!(
            "Failed to resolve `source_dataset` error: {e}"
        ))
    })?;
    let root_output_storage = storage_resolver.resolve(root_output_path).map_err(|e| {
        super::errors::ProcessorUtilError::Decompressor(format!(
            "Failed to resolve `root_output_path` error: {e}"
        ))
    })?;
    let bytes = storage
        .get_sync(source_dataset.path().as_path())
        .map_err(|e| {
            super::errors::ProcessorUtilError::Decompressor(format!(
                "Failed to get `source_dataset` error: {e}"
            ))
        })?;
    if let Some(ext) = source_dataset.path().as_path().extension() {
        if let Some(ext) = ext.to_str() {
            if ["7z", "7zip"].contains(&ext) {
                return extract_sevenz(bytes, root_output_path);
            }
        }
        return extract_zip(bytes, root_output_path, root_output_storage);
    }
    Err(super::errors::ProcessorUtilError::Decompressor(
        "Unsupported archive format".to_string(),
    ))
}

fn extract_zip(
    bytes: bytes::Bytes,
    root_output_path: &Uri,
    storage: Arc<Storage>,
) -> super::errors::Result<Vec<FilePath>> {
    let mut zip_archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| {
        super::errors::ProcessorUtilError::Decompressor(format!(
            "Failed to open `source_dataset` as zip archive: {e}"
        ))
    })?;
    let mut file_paths = Vec::<FilePath>::new();
    for i in 0..zip_archive.len() {
        let mut entry = zip_archive.by_index(i).map_err(|e| {
            super::errors::ProcessorUtilError::Decompressor(format!(
                "Failed to get `source_dataset` entry: {e}"
            ))
        })?;
        let filename = entry.name();
        let outpath = root_output_path.join(filename).map_err(|e| {
            super::errors::ProcessorUtilError::Decompressor(format!(
                "Output path join error with: error = {e:?}"
            ))
        })?;
        let filepath = Path::new(filename);
        if filepath
            .file_name()
            .take_if(|s| s.to_string_lossy().starts_with("."))
            .is_some()
        {
            continue;
        }
        if entry.is_dir() {
            if storage.exists_sync(outpath.path().as_path()).map_err(|e| {
                super::errors::ProcessorUtilError::Decompressor(format!(
                    "Storage exists error with: error = {e:?}"
                ))
            })? {
                continue;
            }
            storage
                .create_dir_sync(outpath.path().as_path())
                .map_err(|e| {
                    super::errors::ProcessorUtilError::Decompressor(format!(
                        "Failed to create directory: error = {e:?}"
                    ))
                })?;
            continue;
        }
        if let Some(p) = outpath.parent() {
            if !storage.exists_sync(p.path().as_path()).map_err(|e| {
                super::errors::ProcessorUtilError::Decompressor(format!(
                    "Storage exists error with: error = {e:?}"
                ))
            })? {
                storage.create_dir_sync(p.path().as_path()).map_err(|e| {
                    super::errors::ProcessorUtilError::Decompressor(format!(
                        "Create dir error with: error = {e:?}"
                    ))
                })?;
            }
        }
        let mut buf = Vec::<u8>::new();
        entry.read_to_end(&mut buf).map_err(|e| {
            super::errors::ProcessorUtilError::Decompressor(format!(
                "Failed to read `source_dataset` entry: {e}"
            ))
        })?;
        let file_path = FilePath::try_from(outpath.clone()).map_err(|e| {
            super::errors::ProcessorUtilError::Decompressor(format!(
                "Filepath convert error with: error = {e:?}"
            ))
        })?;
        storage
            .put_sync(outpath.path().as_path(), bytes::Bytes::from(buf))
            .map_err(|e| {
                super::errors::ProcessorUtilError::Decompressor(format!(
                    "Storage put error with: error = {e:?}"
                ))
            })?;
        file_paths.push(file_path);
    }
    Ok(file_paths)
}

fn extract_sevenz(
    bytes: bytes::Bytes,
    root_output_path: &Uri,
) -> super::errors::Result<Vec<FilePath>> {
    let mut entries = Vec::<Uri>::new();
    let cursor = Cursor::new(bytes);
    decompress_with_extract_fn(cursor, root_output_path.as_path(), |entry, reader, dest| {
        if !entry.is_directory {
            let dest_uri = Uri::try_from(dest.clone()).map_err(|e| {
                reearth_flow_sevenz::Error::Unknown(format!("Failed to convert `dest` to URI: {e}"))
            });
            if let Ok(dest_uri) = dest_uri {
                entries.push(dest_uri);
            }
        }
        default_entry_extract_fn(entry, reader, dest)
    })
    .map_err(|e| {
        super::errors::ProcessorUtilError::Decompressor(format!(
            "Failed to extract `source_dataset` archive: {e}"
        ))
    })?;
    let result = entries
        .iter()
        .flat_map(|entry| {
            let file_path = FilePath::try_from(entry.clone())
                .map_err(|e| {
                    super::errors::ProcessorUtilError::Decompressor(format!(
                        "Filepath convert error with: error = {e:?}"
                    ))
                })
                .ok()?;
            Some(file_path)
        })
        .collect::<Vec<_>>();
    Ok(result)
}
