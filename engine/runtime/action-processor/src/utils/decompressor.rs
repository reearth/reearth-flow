use std::{
    io::{Read, Seek},
    path::Path,
    sync::Arc,
};

use reearth_flow_common::uri::{Protocol, Uri};
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
    let root_output_storage = storage_resolver.resolve(root_output_path).map_err(|e| {
        super::errors::ProcessorUtilError::Decompressor(format!(
            "Failed to resolve `root_output_path` error: {e}"
        ))
    })?;
    if let Some(ext) = source_dataset.path().as_path().extension() {
        if let Some(ext) = ext.to_str() {
            if ["7z", "7zip"].contains(&ext) {
                let file = get_archive_file_handle(source_dataset, Arc::clone(&storage_resolver))?;
                return extract_sevenz(file, root_output_path);
            }
            if ext == "zip" {
                let file = get_archive_file_handle(source_dataset, Arc::clone(&storage_resolver))?;
                return extract_zip(file, root_output_path, root_output_storage);
            }
        }
    }
    Err(super::errors::ProcessorUtilError::Decompressor(
        "Unsupported archive format".to_string(),
    ))
}

/// Returns a seekable `File` handle for the archive at `source`.
///
/// - **Local (`file://`)**: opens the file directly — zero extra memory.
/// - **Remote (GCS, HTTP, …)**: streams via storage into an anonymous
///   temporary file and returns it rewound to offset 0.  The temp file is
///   deleted automatically when dropped.
fn get_archive_file_handle(
    source: &Uri,
    storage_resolver: Arc<StorageResolver>,
) -> super::errors::Result<std::fs::File> {
    if matches!(source.protocol(), Protocol::File) {
        let local_path = source.path();
        return std::fs::File::open(&local_path).map_err(|e| {
            super::errors::ProcessorUtilError::Decompressor(format!(
                "Failed to open local archive '{local_path:?}': {e}"
            ))
        });
    }
    // Remote: stream directly into an anonymous temp file — the full archive
    // never lives in memory.
    let storage = storage_resolver.resolve(source).map_err(|e| {
        super::errors::ProcessorUtilError::Decompressor(format!(
            "Failed to resolve `source_dataset` error: {e}"
        ))
    })?;
    let mut tmp = tempfile::tempfile().map_err(|e| {
        super::errors::ProcessorUtilError::Decompressor(format!(
            "Failed to create temporary file: {e}"
        ))
    })?;
    storage
        .stream_to_file_sync(source.path().as_path(), &mut tmp)
        .map_err(|e| {
            super::errors::ProcessorUtilError::Decompressor(format!(
                "Failed to stream archive to temporary file: {e}"
            ))
        })?;
    std::io::Seek::seek(&mut tmp, std::io::SeekFrom::Start(0)).map_err(|e| {
        super::errors::ProcessorUtilError::Decompressor(format!(
            "Failed to rewind temporary file: {e}"
        ))
    })?;
    Ok(tmp)
}

fn extract_zip<R: Read + Seek>(
    reader: R,
    root_output_path: &Uri,
    storage: Arc<Storage>,
) -> super::errors::Result<Vec<FilePath>> {
    let mut zip_archive = zip::ZipArchive::new(reader).map_err(|e| {
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
        // Non-standard workaround: Normalize backslashes to forward slashes.
        // While ZIP format specifies forward slashes, some tools (especially on Windows)
        // may create archives with backslashes. This ensures proper path extraction.
        // Note: Does not prevent directory traversal attacks (e.g., "../../../etc/passwd").
        // Callers should validate archive contents from trusted sources only.
        let normalized_filename = filename.replace('\\', "/");
        let outpath = root_output_path.join(&normalized_filename).map_err(|e| {
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

fn extract_sevenz<R: Read + Seek>(
    reader: R,
    root_output_path: &Uri,
) -> super::errors::Result<Vec<FilePath>> {
    let mut entries = Vec::<Uri>::new();
    decompress_with_extract_fn(reader, root_output_path.as_path(), |entry, reader, dest| {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;
    use zip::write::FileOptions;

    #[test]
    fn test_backslash_separator_in_zip() {
        // Create a zip with backslash separators
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            zip.start_file("folder\\file.txt", FileOptions::<()>::default())
                .unwrap();
            zip.write_all(b"test").unwrap();
            zip.finish().unwrap();
        }

        // Extract and verify proper directory structure
        let temp_dir = TempDir::new().unwrap();
        let root_uri = Uri::for_test(temp_dir.path().to_str().unwrap());
        let storage_resolver = Arc::new(StorageResolver::new());
        let storage = storage_resolver.resolve(&root_uri).unwrap();

        let mut tmp = tempfile::tempfile().unwrap();
        std::io::Write::write_all(&mut tmp, &buf).unwrap();
        std::io::Seek::seek(&mut tmp, std::io::SeekFrom::Start(0)).unwrap();
        let result = extract_zip(tmp, &root_uri, storage);
        assert!(result.is_ok());

        // Verify file extracted to folder/file.txt, not "folder\file.txt"
        let extracted_path = temp_dir.path().join("folder").join("file.txt");
        assert!(
            extracted_path.exists(),
            "File should be in folder/file.txt structure"
        );
    }
}
