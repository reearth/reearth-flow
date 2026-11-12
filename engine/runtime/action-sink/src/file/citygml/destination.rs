//! Where one CityGML document and the files staged beside it are written:
//! loose in storage, or as entries of a zip archive.
//!
//! Either way the document names its staged files by paths relative to itself,
//! `{stem}_appearance/…` and `{stem}_codelists/…`, so the same document text
//! resolves whether it is read from storage or from inside the archive.

use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_common::zip::StreamingZipWriter;
use reearth_flow_storage::resolve::StorageResolver;

use crate::errors::SinkError;

/// A directory files are staged into beside the GML file.
pub(super) struct StagingDir {
    /// `{gml_stem}_{suffix}`, the GML-relative prefix the document names
    /// staged files by.
    pub(super) name: String,
    /// The same directory relative to where it is written: the sandbox root
    /// for loose files, the archive root for an archive.
    pub(super) path: String,
}

/// Where one document goes.
pub enum Destination<'a> {
    /// The document at `output`, with its staged folders beside it in storage.
    Storage {
        output: &'a Uri,
        sandbox_root: &'a Uri,
    },
    /// The document at `path` inside `archive`, with its staged folders beside
    /// it there. `path` must have passed [`validate_entry_path`].
    Archive {
        archive: &'a ArchiveWriter,
        path: &'a str,
    },
}

impl Destination<'_> {
    /// The folder `{gml_stem}_{suffix}` beside the document.
    pub(super) fn staging_dir(&self, suffix: &str) -> Result<StagingDir, SinkError> {
        match self {
            Self::Storage {
                output,
                sandbox_root,
            } => storage_staging_dir(output, sandbox_root, suffix),
            Self::Archive { path, .. } => {
                let (parent, file_name) = match path.rsplit_once('/') {
                    Some((parent, file_name)) => (Some(parent), file_name),
                    None => (None, *path),
                };
                let name = format!("{}_{suffix}", file_stem(file_name));
                let path = match parent {
                    Some(parent) => format!("{parent}/{name}"),
                    None => name.clone(),
                };
                Ok(StagingDir { name, path })
            }
        }
    }

    /// Write one staged file, `name` inside `dir`. `kind` names the file in
    /// the error, such as `texture`.
    pub(super) fn write_staged(
        &self,
        dir: &StagingDir,
        name: &str,
        bytes: Bytes,
        kind: &str,
        storage_resolver: &Arc<StorageResolver>,
    ) -> Result<(), String> {
        let rel_path = format!("{}/{}", dir.path, name);
        match self {
            Self::Storage { sandbox_root, .. } => {
                crate::SinkOutput::new(sandbox_root, &rel_path, storage_resolver)
                    .map_err(|e| {
                        format!(
                            "failed to acquire sandboxed SinkOutput for {kind} destination \
                             '{rel_path}': {e}"
                        )
                    })?
                    .write(bytes)
                    .map_err(|e| format!("failed to write {kind} file '{rel_path}': {e}"))
            }
            Self::Archive { archive, .. } => archive
                .add(&rel_path, &bytes)
                .map_err(|e| format!("failed to add {kind} file '{rel_path}' to the archive: {e}")),
        }
    }

    /// Write the document itself.
    pub(super) fn write_document(
        &self,
        bytes: Bytes,
        storage_resolver: &Arc<StorageResolver>,
    ) -> Result<(), SinkError> {
        match self {
            Self::Storage { output, .. } => {
                let storage = storage_resolver
                    .resolve(output)
                    .map_err(SinkError::citygml_writer)?;
                storage
                    .put_sync(output.path().as_path(), bytes)
                    .map_err(SinkError::citygml_writer)
            }
            Self::Archive { archive, path } => archive.add(path, &bytes).map_err(|e| {
                SinkError::CityGmlWriter(format!("failed to add '{path}' to the archive: {e}"))
            }),
        }
    }
}

/// A file name without its last extension, as `Path::file_stem` takes it.
pub(super) fn file_stem(file_name: &str) -> &str {
    match file_name.rsplit_once('.') {
        // A leading dot is part of the name (".gml"), not an extension.
        Some((stem, _)) if !stem.is_empty() => stem,
        _ => file_name,
    }
}

fn storage_staging_dir(
    output: &Uri,
    sandbox_root: &Uri,
    suffix: &str,
) -> Result<StagingDir, SinkError> {
    let gml_stem = output
        .path()
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let name = format!("{gml_stem}_{suffix}");

    let sandbox_root_str = sandbox_root.as_str().trim_end_matches('/');
    let output_str = output.as_str();
    // `SinkOutput::new` builds `output` from `sandbox_root`, so a failed strip
    // means something upstream is broken. Fail rather than flatten the layout.
    let gml_rel_path: String = output_str
        .strip_prefix(sandbox_root_str)
        .map(|s| s.trim_start_matches('/').to_string())
        .ok_or_else(|| {
            SinkError::CityGmlWriter(format!(
                "output URI {output} is not under sandbox_root {sandbox_root_str}; \
                 refusing to fall back to a flat {suffix} directory"
            ))
        })?;
    let path = match gml_rel_path.rsplit_once('/') {
        Some((parent, _)) if !parent.is_empty() => format!("{parent}/{name}"),
        _ => name.clone(),
    };
    Ok(StagingDir { name, path })
}

/// Whether `path` ends in `.zip`, ignoring case: an Output File such as this
/// is written as one archive.
pub fn is_archive_path(path: &str) -> bool {
    path.len() >= 4
        && path
            .get(path.len() - 4..)
            .is_some_and(|ext| ext.eq_ignore_ascii_case(".zip"))
}

/// Why `path` cannot name an entry of an archive, if it cannot.
///
/// An entry path is relative and normalized: no leading `/`, no backslash
/// (which some extractors read as a separator), and no empty, `.` or `..`
/// segment, so no extractor can resolve it outside the folder it extracts to.
pub fn validate_entry_path(path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("is blank".to_string());
    }
    if path.starts_with('/') {
        return Err(format!(
            "`{path}` is absolute; give a path relative to the top of the archive, such as `udx/bldg/city.gml`"
        ));
    }
    if path.contains('\\') {
        return Err(format!(
            "`{path}` contains a backslash; separate folders with `/`"
        ));
    }
    if path.contains(':') {
        return Err(format!(
            "`{path}` contains `:`; give a path relative to the top of the archive, such as `udx/bldg/city.gml`"
        ));
    }
    for segment in path.split('/') {
        match segment {
            ".." => {
                return Err(format!(
                    "`{path}` contains `..`; a path must stay inside the archive"
                ))
            }
            "" | "." => {
                return Err(format!(
                    "`{path}` has an empty or `.` folder; give a path such as `udx/bldg/city.gml`"
                ))
            }
            _ => {}
        }
    }
    Ok(())
}

/// The default entry path of the document in `archive_output`: the archive's
/// own name with `.gml`, at the top of the archive, so `out/foo.zip` holds
/// `foo.gml`.
pub fn default_entry_path(archive_output: &str) -> String {
    let file_name = archive_output
        .rsplit_once('/')
        .map_or(archive_output, |(_, file_name)| file_name);
    // `is_archive_path` held, so the last four bytes are `.zip`.
    let stem = &file_name[..file_name.len().saturating_sub(4)];
    format!("{stem}.gml")
}

/// A zip archive built in a temporary file and written in one piece once
/// finished.
///
/// The temporary file keeps staged images out of memory while the document is
/// built: only the finished archive is read back, as storage takes one buffer.
///
/// Entries are added in the order they are written, so one archive can hold
/// any number of documents, each with its staged folders. Paths are checked
/// with [`validate_entry_path`] and must be unique.
pub struct ArchiveWriter {
    zip: StreamingZipWriter<File>,
    entries: Mutex<HashSet<String>>,
}

impl ArchiveWriter {
    pub fn new() -> Result<Self, SinkError> {
        let file = tempfile::tempfile().map_err(|e| {
            SinkError::CityGmlWriter(format!(
                "failed to create a temporary file for the zip archive: {e}"
            ))
        })?;
        Ok(Self {
            zip: StreamingZipWriter::new(file),
            entries: Mutex::new(HashSet::new()),
        })
    }

    /// Add `bytes` as the entry at `path`.
    pub fn add(&self, path: &str, bytes: &[u8]) -> Result<(), String> {
        validate_entry_path(path)?;
        {
            let mut entries = self
                .entries
                .lock()
                .map_err(|e| format!("archive entry list is poisoned: {e}"))?;
            if !entries.insert(path.to_string()) {
                return Err(format!("`{path}` is already in the archive"));
            }
        }
        self.zip.write_entry(path, bytes).map_err(|e| e.to_string())
    }

    /// Whether nothing has been added.
    pub fn is_empty(&self) -> bool {
        self.entries
            .lock()
            .map_or(true, |entries| entries.is_empty())
    }

    /// The finished archive's bytes.
    pub fn finish(self) -> Result<Bytes, SinkError> {
        let failed = |e: &dyn std::fmt::Display| {
            SinkError::CityGmlWriter(format!("failed to finish the zip archive: {e}"))
        };
        let mut file = self.zip.finish().map_err(|e| failed(&e))?;
        file.seek(SeekFrom::Start(0)).map_err(|e| failed(&e))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(|e| failed(&e))?;
        Ok(Bytes::from(bytes))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn a_zip_extension_is_matched_in_any_case() {
        assert!(is_archive_path("out/city.zip"));
        assert!(is_archive_path("CITY.ZIP"));
        assert!(!is_archive_path("city.gml"));
        assert!(!is_archive_path("city.zip.gml"));
        assert!(!is_archive_path("zip"));
    }

    #[test]
    fn the_default_entry_is_the_archive_name_with_gml() {
        assert_eq!(
            default_entry_path("13105_bunkyo-ku_bldg.zip"),
            "13105_bunkyo-ku_bldg.gml"
        );
        assert_eq!(default_entry_path("out/foo.ZIP"), "foo.gml");
    }

    #[test]
    fn entry_paths_that_could_leave_the_archive_are_refused() {
        assert_eq!(validate_entry_path("udx/bldg/x.gml"), Ok(()));
        assert_eq!(validate_entry_path("x.gml"), Ok(()));
        for path in [
            "",
            "  ",
            "/udx/x.gml",
            "udx\\x.gml",
            "../x.gml",
            "udx/../../x.gml",
            "udx//x.gml",
            "./x.gml",
            "udx/",
            "C:/x.gml",
        ] {
            assert!(validate_entry_path(path).is_err(), "{path:?} was accepted");
        }
    }

    #[test]
    fn staging_folders_sit_beside_the_document_in_the_archive() {
        let archive = ArchiveWriter::new().unwrap();
        let nested = Destination::Archive {
            archive: &archive,
            path: "udx/bldg/x.gml",
        };
        let dir = nested.staging_dir("appearance").unwrap();
        assert_eq!(dir.name, "x_appearance");
        assert_eq!(dir.path, "udx/bldg/x_appearance");

        let top = Destination::Archive {
            archive: &archive,
            path: "x.gml",
        };
        let dir = top.staging_dir("codelists").unwrap();
        assert_eq!(dir.name, "x_codelists");
        assert_eq!(dir.path, "x_codelists");
    }

    #[test]
    fn a_finished_archive_holds_its_entries() {
        let archive = ArchiveWriter::new().unwrap();
        archive.add("udx/bldg/x.gml", b"<gml/>").unwrap();
        let bytes = archive.finish().unwrap();
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        let mut content = String::new();
        zip.by_name("udx/bldg/x.gml")
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert_eq!(content, "<gml/>");
    }

    #[test]
    fn an_entry_cannot_be_added_twice() {
        let archive = ArchiveWriter::new().unwrap();
        archive.add("a/x.gml", b"one").unwrap();
        assert!(archive.add("a/x.gml", b"two").is_err());
        assert!(archive.add("../x.gml", b"two").is_err());
    }
}
