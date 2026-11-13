//! Where one CityGML document and the files staged beside it are written:
//! loose in storage, or as entries of a zip archive.
//!
//! Either way the document names its staged files by paths relative to itself,
//! `{stem}_appearance/…` and `{stem}_codelists/…`, so the same document text
//! resolves whether it is read from storage or from inside the archive.

use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use reearth_flow_common::uri::{Protocol, Uri};
use reearth_flow_common::zip::StreamingZipWriter;
use reearth_flow_storage::resolve::StorageResolver;
use tempfile::{NamedTempFile, TempPath};

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

/// A zip archive built entry by entry in a temporary file.
///
/// The temporary file keeps staged images out of memory while the documents
/// are built. Where it goes depends on where the archive is written:
///
/// - A local file (`file://`) is built beside its final path and renamed onto
///   it once finished, so nothing is read back and a run that fails part way
///   leaves no partial archive behind.
/// - Any other storage takes one buffer, so the archive is built in an
///   anonymous temporary file and its bytes are read back once finished, for
///   the caller to write.
///
/// Entries are added in the order they are written, so one archive can hold
/// any number of documents, each with its staged folders. Paths are checked
/// with [`validate_entry_path`] and must be unique.
pub struct ArchiveWriter {
    /// The local file the archive is renamed onto, or `None` to hand the
    /// finished bytes back.
    in_place: Option<PathBuf>,
    state: Mutex<ArchiveState>,
}

#[derive(Default)]
struct ArchiveState {
    /// Opened at the first entry and closed when the archive is finished, so
    /// an archive holds a file only while its documents are being written.
    zip: Option<StreamingZipWriter<File>>,
    /// The temporary file beside the final path an archive written in place
    /// is built in. Dropping it removes the file, so a run that fails before
    /// the archive is finished leaves nothing behind.
    temp_path: Option<TempPath>,
    entries: HashSet<String>,
    finished: bool,
}

/// What finishing an archive leaves the caller to do.
#[derive(Debug)]
pub enum FinishedArchive {
    /// The archive is at its final path; there is nothing left to write.
    InPlace,
    /// The archive's bytes, for the caller to write to storage.
    Bytes(Bytes),
}

impl Default for ArchiveWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchiveWriter {
    /// An archive whose finished bytes are handed back to be written.
    pub fn new() -> Self {
        Self {
            in_place: None,
            state: Mutex::new(ArchiveState::default()),
        }
    }

    /// An archive written to `output`: in place when that is a local file,
    /// otherwise handed back as bytes.
    pub fn for_output(output: &Uri) -> Self {
        Self {
            in_place: (output.protocol() == Protocol::File).then(|| output.path()),
            ..Self::new()
        }
    }

    /// Add `bytes` as the entry at `path`.
    pub fn add(&self, path: &str, bytes: &[u8]) -> Result<(), String> {
        validate_entry_path(path)?;
        let mut state = self
            .state
            .lock()
            .map_err(|e| format!("archive state is poisoned: {e}"))?;
        if state.finished {
            return Err(format!(
                "cannot add `{path}`: the archive is already finished"
            ));
        }
        if state.entries.contains(path) {
            return Err(format!("`{path}` is already in the archive"));
        }
        if state.zip.is_none() {
            let file = match &self.in_place {
                Some(target) => {
                    let (file, temp_path) = temp_file_beside(target)?.into_parts();
                    state.temp_path = Some(temp_path);
                    file
                }
                None => tempfile::tempfile().map_err(|e| {
                    format!("failed to create a temporary file for the zip archive: {e}")
                })?,
            };
            state.zip = Some(StreamingZipWriter::new(file));
        }
        let zip = state.zip.as_mut().expect("the archive was just opened");
        zip.write_entry(path, bytes).map_err(|e| e.to_string())?;
        state.entries.insert(path.to_string());
        Ok(())
    }

    /// Whether nothing has been added.
    pub fn is_empty(&self) -> bool {
        self.state
            .lock()
            .map_or(true, |state| state.entries.is_empty())
    }

    /// Finish the archive, or return `None` when nothing was added. Its
    /// temporary file is closed, and nothing more can be added.
    pub fn finish(&self) -> Result<Option<FinishedArchive>, SinkError> {
        let failed = |e: &dyn std::fmt::Display| {
            SinkError::CityGmlWriter(format!("failed to finish the zip archive: {e}"))
        };
        let (zip, temp_path) = {
            let mut state = self.state.lock().map_err(|e| failed(&e))?;
            state.finished = true;
            (state.zip.take(), state.temp_path.take())
        };
        let Some(zip) = zip else {
            return Ok(None);
        };
        let mut file = zip.finish().map_err(|e| failed(&e))?;
        match (&self.in_place, temp_path) {
            (Some(target), Some(temp_path)) => {
                file.sync_all().map_err(|e| failed(&e))?;
                drop(file);
                // On failure the error owns the temporary path and removes it.
                temp_path.persist(target).map_err(|e| {
                    failed(&format_args!(
                        "failed to move it into place at {}: {}",
                        target.display(),
                        e.error
                    ))
                })?;
                Ok(Some(FinishedArchive::InPlace))
            }
            _ => {
                file.seek(SeekFrom::Start(0)).map_err(|e| failed(&e))?;
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes).map_err(|e| failed(&e))?;
                Ok(Some(FinishedArchive::Bytes(Bytes::from(bytes))))
            }
        }
    }
}

/// A temporary file in the folder `target` goes in, created with that folder,
/// so the finished archive can be renamed onto `target` without a copy.
fn temp_file_beside(target: &Path) -> Result<NamedTempFile, String> {
    let dir = target
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .ok_or_else(|| format!("the zip archive {} has no folder", target.display()))?;
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("failed to create the folder {}: {e}", dir.display()))?;
    let name = target
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let prefix = format!(".{name}.");
    let mut builder = tempfile::Builder::new();
    builder.prefix(&prefix).suffix(".tmp");
    // A temporary file is private by default; the archive it becomes gets the
    // permissions any other file the writer creates gets, under the umask.
    #[cfg(unix)]
    let permissions = {
        use std::os::unix::fs::PermissionsExt;
        std::fs::Permissions::from_mode(0o666)
    };
    #[cfg(unix)]
    builder.permissions(permissions);
    builder.tempfile_in(dir).map_err(|e| {
        format!(
            "failed to create a temporary file for the zip archive in {}: {e}",
            dir.display()
        )
    })
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
        let archive = ArchiveWriter::new();
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
        let archive = ArchiveWriter::new();
        archive.add("udx/bldg/x.gml", b"<gml/>").unwrap();
        let Some(FinishedArchive::Bytes(bytes)) = archive.finish().unwrap() else {
            panic!("an archive not written in place hands back its bytes");
        };
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
        let archive = ArchiveWriter::new();
        archive.add("a/x.gml", b"one").unwrap();
        assert!(archive.add("a/x.gml", b"two").is_err());
        assert!(archive.add("../x.gml", b"two").is_err());
    }

    #[test]
    fn an_archive_nothing_was_added_to_takes_as_none() {
        let archive = ArchiveWriter::new();
        assert!(archive.is_empty());
        assert!(archive.finish().unwrap().is_none());
    }

    #[test]
    fn a_taken_archive_takes_no_more_entries() {
        let archive = ArchiveWriter::new();
        archive.add("x.gml", b"one").unwrap();
        assert!(archive.finish().unwrap().is_some());
        let error = archive.add("y.gml", b"two").unwrap_err();
        assert!(error.contains("already finished"), "{error}");
        assert!(archive.finish().unwrap().is_none());
    }

    fn local(path: &Path) -> Uri {
        use std::str::FromStr;
        Uri::from_str(&format!("file://{}", path.display())).unwrap()
    }

    /// Every file under `dir`, relative to it.
    fn files_in(dir: &Path) -> Vec<String> {
        let mut files: Vec<String> = walkdir(dir)
            .into_iter()
            .map(|path| {
                path.strip_prefix(dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        files.sort();
        files
    }

    fn walkdir(dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                files.extend(walkdir(&path));
            } else {
                files.push(path);
            }
        }
        files
    }

    #[test]
    fn a_local_archive_is_built_beside_its_path_and_moved_into_place() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("out/sub/x.zip");
        let archive = ArchiveWriter::for_output(&local(&target));

        archive.add("udx/x.gml", b"<gml/>").unwrap();
        // While it is being written, only its temporary file is there, in the
        // folder the archive goes in.
        let pending = files_in(dir.path());
        assert_eq!(pending.len(), 1, "{pending:?}");
        assert!(
            pending[0].starts_with("out/sub/.x.zip.") && pending[0].ends_with(".tmp"),
            "{pending:?}"
        );

        assert!(matches!(
            archive.finish().unwrap(),
            Some(FinishedArchive::InPlace)
        ));
        assert_eq!(files_in(dir.path()), ["out/sub/x.zip"]);
        let mut zip = zip::ZipArchive::new(File::open(&target).unwrap()).unwrap();
        let mut content = String::new();
        zip.by_name("udx/x.gml")
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert_eq!(content, "<gml/>");
    }

    #[test]
    fn a_local_archive_replaces_an_older_file() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("x.zip");
        std::fs::write(&target, b"old").unwrap();
        let archive = ArchiveWriter::for_output(&local(&target));
        archive.add("x.gml", b"<gml/>").unwrap();
        archive.finish().unwrap();
        assert!(zip::ZipArchive::new(File::open(&target).unwrap()).is_ok());
    }

    #[test]
    fn an_unfinished_local_archive_leaves_nothing_behind() {
        let dir = tempfile::tempdir().unwrap();
        let archive = ArchiveWriter::for_output(&local(&dir.path().join("x.zip")));
        archive.add("x.gml", b"<gml/>").unwrap();
        drop(archive);
        assert!(files_in(dir.path()).is_empty());
    }

    #[test]
    fn a_local_archive_nothing_was_added_to_creates_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let archive = ArchiveWriter::for_output(&local(&dir.path().join("out/x.zip")));
        assert!(archive.finish().unwrap().is_none());
        assert!(files_in(dir.path()).is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn a_local_archive_is_not_private_to_its_temporary_file() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("x.zip");
        let archive = ArchiveWriter::for_output(&local(&target));
        archive.add("x.gml", b"<gml/>").unwrap();
        archive.finish().unwrap();
        let loose = dir.path().join("loose");
        std::fs::write(&loose, b"").unwrap();
        let mode = |path: &Path| std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&target), mode(&loose));
    }

    #[test]
    fn an_archive_in_other_storage_hands_back_its_bytes() {
        use std::str::FromStr;
        let archive = ArchiveWriter::for_output(&Uri::from_str("ram:///out/x.zip").unwrap());
        archive.add("x.gml", b"<gml/>").unwrap();
        assert!(matches!(
            archive.finish().unwrap(),
            Some(FinishedArchive::Bytes(_))
        ));
    }
}
