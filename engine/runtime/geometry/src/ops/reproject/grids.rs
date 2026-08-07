//! The geodetic grids carried inside the binary, and the PROJ search path built
//! from them.
//!
//! The grids in `grids/` are compiled in. Further grids come from the
//! directories named by [`GRID_DIR_VAR`], which are searched first.

use std::ffi::{CStr, CString};
use std::fs;
use std::io;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use proj_sys::{
    proj_context_create, proj_context_destroy, proj_context_get_user_writable_directory,
    proj_context_set_search_paths, PJ_CONTEXT,
};

use crate::error::{Error, Result};

/// Extra directories of grid files to search before the embedded ones, in the
/// platform's path-list syntax (`:`-separated on Unix, `;` on Windows).
pub const GRID_DIR_VAR: &str = "FLOW_PROJ_GRID_DIR";

/// Where the embedded grids are unpacked. Defaults to the user cache directory,
/// falling back to the temporary directory.
pub const GRID_CACHE_DIR_VAR: &str = "FLOW_PROJ_GRID_CACHE_DIR";

/// A grid file compiled into the binary.
struct EmbeddedGrid {
    /// The PROJ-data file name, which is also what `proj.db` refers to it by.
    name: &'static str,
    /// The file's contents.
    bytes: &'static [u8],
}

// Defines EMBEDDED_GRIDS from the rows of `grids/MANIFEST.tsv`.
include!(concat!(env!("OUT_DIR"), "/embedded_grids.rs"));

/// Create a PROJ context that can see the embedded and external grids.
///
/// Every context in this crate must come from here, or it will not see them.
pub(crate) fn create_context() -> Result<*mut PJ_CONTEXT> {
    let paths: Vec<*const c_char> = resolved().search.iter().map(|p| p.as_ptr()).collect();
    // SAFETY: the context is checked for null before it is handed out; the
    // pointers are into strings that outlive the call, which is all PROJ needs
    // as it copies the paths it is given.
    unsafe {
        let ctx = proj_context_create();
        if ctx.is_null() {
            return Err(Error::projection("proj_context_create returned null"));
        }
        if !paths.is_empty() {
            proj_context_set_search_paths(ctx, paths.len() as i32, paths.as_ptr());
        }
        Ok(ctx)
    }
}

/// How a missing grid can be supplied, for the errors PROJ raises when it cannot
/// build a transformation. Reports the unpack failure instead when the embedded
/// grids could not be written out.
pub(crate) fn supply_hint() -> String {
    match &resolved().unpack_failure {
        None => format!(
            "supply it in a directory named by {GRID_DIR_VAR}; grids are published at \
             https://cdn.proj.org"
        ),
        Some(e) => format!(
            "the embedded grids could not be unpacked ({e}), so every vertical datum change \
             will fail: set {GRID_CACHE_DIR_VAR} to a writable directory, or {GRID_DIR_VAR} \
             to one holding the grids"
        ),
    }
}

/// Where this process looks for grids, worked out once.
struct Resolved {
    /// The search paths, in the order PROJ should try them.
    search: Vec<CString>,
    /// Why the embedded grids could not be written out, when they could not be.
    unpack_failure: Option<String>,
}

/// Resolve the grid directories, unpacking the embedded grids that are not
/// already supplied from elsewhere. Runs once per process.
fn resolved() -> &'static Resolved {
    static RESOLVED: OnceLock<Resolved> = OnceLock::new();
    RESOLVED.get_or_init(|| {
        let external = external_dirs();
        let (embedded, unpack_failure) = match unpack_embedded(&external) {
            Embedded::Dir(dir) => (Some(dir), None),
            Embedded::AlreadySupplied => (None, None),
            Embedded::Failed(why) => (None, Some(why)),
        };
        let dirs = ordered_dirs(external, embedded, proj_default_grid_dirs());
        Resolved {
            search: dirs
                .iter()
                .filter_map(|d| CString::new(d.as_os_str().as_encoded_bytes()).ok())
                .collect(),
            unpack_failure,
        }
    })
}

/// The directories named by [`GRID_DIR_VAR`], without the empty entries a path
/// list can carry.
fn external_dirs() -> Vec<PathBuf> {
    let Some(list) = std::env::var_os(GRID_DIR_VAR) else {
        return Vec::new();
    };
    std::env::split_paths(&list)
        .filter(|p| !p.as_os_str().is_empty())
        .collect()
}

/// Order the three sources of grid directories, most specific first: the
/// directories named by [`GRID_DIR_VAR`], the unpacked embedded grids, then
/// whatever PROJ would have searched on its own.
fn ordered_dirs(
    external: Vec<PathBuf>,
    embedded: Option<PathBuf>,
    defaults: Vec<PathBuf>,
) -> Vec<PathBuf> {
    let mut dirs = external;
    dirs.extend(embedded);
    dirs.extend(defaults);
    dirs
}

/// The directories PROJ would look in for a grid on its own, in its own order:
/// its user-writable directory, then those named by `PROJ_DATA`.
fn proj_default_grid_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(dir) = user_writable_dir() {
        dirs.push(dir);
    }
    if let Some(list) = std::env::var_os("PROJ_DATA").or_else(|| std::env::var_os("PROJ_LIB")) {
        dirs.extend(std::env::split_paths(&list).filter(|p| !p.as_os_str().is_empty()));
    }
    dirs
}

/// The directory PROJ treats as its own writable store. Does not create it.
fn user_writable_dir() -> Option<PathBuf> {
    // SAFETY: the returned string belongs to the context, so it is copied before
    // the context is destroyed; a null context is never passed on.
    unsafe {
        let ctx = proj_context_create();
        if ctx.is_null() {
            return None;
        }
        let dir = proj_context_get_user_writable_directory(ctx, 0);
        let dir = (!dir.is_null()).then(|| CStr::from_ptr(dir).to_string_lossy().into_owned());
        proj_context_destroy(ctx);
        dir.filter(|d| !d.is_empty()).map(PathBuf::from)
    }
}

/// What became of the embedded grids.
enum Embedded {
    /// They are readable in this directory, which belongs on the search path.
    Dir(PathBuf),
    /// Every one was already available elsewhere, so nothing was written.
    AlreadySupplied,
    /// They could not be written anywhere, with every attempt reported.
    Failed(String),
}

/// Write out the embedded grids that `external` does not already supply.
fn unpack_embedded(external: &[PathBuf]) -> Embedded {
    let missing: Vec<&EmbeddedGrid> = EMBEDDED_GRIDS
        .iter()
        .filter(|grid| !external.iter().any(|dir| dir.join(grid.name).is_file()))
        .collect();
    if missing.is_empty() {
        return Embedded::AlreadySupplied;
    }
    let mut failures = Vec::new();
    for dir in cache_dirs() {
        match unpack(&dir, &missing) {
            Ok(()) => return Embedded::Dir(dir),
            Err(e) => failures.push(format!("{}: {e}", dir.display())),
        }
    }
    Embedded::Failed(failures.join("; "))
}

/// Where to unpack the embedded grids, in order of preference: the directory
/// named by [`GRID_CACHE_DIR_VAR`] alone when it is set, otherwise the user
/// cache directory then the temporary directory.
fn cache_dirs() -> Vec<PathBuf> {
    if let Some(dir) = std::env::var_os(GRID_CACHE_DIR_VAR) {
        return vec![PathBuf::from(dir)];
    }
    let mut dirs = Vec::with_capacity(2);
    if let Some(base) = directories::BaseDirs::new() {
        dirs.push(base.cache_dir().join("reearth-flow").join("proj-grids"));
    }
    dirs.push(std::env::temp_dir().join("reearth-flow-proj-grids"));
    dirs
}

/// Write each of `grids` into `dir` unless it is already there at the right
/// size. Writes are renamed into place, so a concurrent reader sees one whole
/// file or the other.
fn unpack(dir: &Path, grids: &[&EmbeddedGrid]) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    for grid in grids {
        let dest = dir.join(grid.name);
        if is_current(&dest, grid.bytes.len()) {
            continue;
        }
        let tmp = dir.join(format!(".{}.{}.tmp", grid.name, std::process::id()));
        fs::write(&tmp, grid.bytes)?;
        if let Err(e) = fs::rename(&tmp, &dest) {
            let _ = fs::remove_file(&tmp);
            // Another process may have completed the same write in between,
            // which is a success for our purposes.
            if !is_current(&dest, grid.bytes.len()) {
                return Err(e);
            }
        }
    }
    Ok(())
}

/// Whether `path` already holds a file of exactly `len` bytes.
fn is_current(path: &Path, len: usize) -> bool {
    fs::metadata(path).is_ok_and(|m| m.is_file() && m.len() == len as u64)
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use super::*;
    use crate::coordinate::EpsgCode;
    use crate::ops::reproject::ReprojectionCache;

    #[test]
    fn every_embedded_grid_is_known_and_available_to_proj() {
        let ctx = create_context().unwrap();
        for grid in EMBEDDED_GRIDS {
            let name = CString::new(grid.name).unwrap();
            let mut available = 0;
            // SAFETY: `ctx` is a live context and `name` outlives the call; every
            // out-parameter we do not want is passed as null, which PROJ allows.
            let known = unsafe {
                proj_sys::proj_grid_get_info_from_database(
                    ctx,
                    name.as_ptr(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    &mut available,
                )
            };
            assert_eq!(known, 1, "{} is not referenced by proj.db", grid.name);
            assert_eq!(available, 1, "{} is not on the search path", grid.name);
        }
        // SAFETY: `ctx` came from `create_context` and is not used again.
        unsafe { proj_sys::proj_context_destroy(ctx) };
    }

    /// An empty list means a grid installed the way PROJ installs them would
    /// stop being found.
    #[test]
    fn proj_own_grid_directories_are_kept_on_the_search_path() {
        let defaults = proj_default_grid_dirs();
        assert!(
            !defaults.is_empty(),
            "PROJ's own grid directories were dropped from the search path"
        );
        assert!(
            resolved().search.iter().any(|path| defaults
                .iter()
                .any(|dir| path.as_bytes() == dir.as_os_str().as_encoded_bytes())),
            "the resolved search path keeps none of {defaults:?}"
        );
    }

    fn all_grids() -> Vec<&'static EmbeddedGrid> {
        EMBEDDED_GRIDS.iter().collect()
    }

    #[test]
    fn grids_supplied_externally_are_not_unpacked_again() {
        // An external directory holding every embedded grid leaves nothing to
        // write and no directory to add.
        let external = tempfile::tempdir().unwrap();
        for grid in EMBEDDED_GRIDS {
            fs::write(external.path().join(grid.name), grid.bytes).unwrap();
        }
        assert!(matches!(
            unpack_embedded(&[external.path().to_path_buf()]),
            Embedded::AlreadySupplied
        ));
    }

    #[test]
    fn unpacking_is_idempotent_and_repairs_a_damaged_grid() {
        let dir = tempfile::tempdir().unwrap();
        unpack(dir.path(), &all_grids()).unwrap();
        let victim = dir.path().join(EMBEDDED_GRIDS[0].name);
        let written = fs::metadata(&victim).unwrap().len();
        assert_eq!(written, EMBEDDED_GRIDS[0].bytes.len() as u64);

        fs::write(&victim, b"truncated").unwrap();
        unpack(dir.path(), &all_grids()).unwrap();
        assert_eq!(fs::metadata(&victim).unwrap().len(), written);
        assert!(
            fs::read_dir(dir.path()).unwrap().all(|e| !e
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp")),
            "unpacking left a temporary file behind"
        );
    }

    /// The vertical transformations the embedded set exists to make work. Each
    /// pair fails outright without its geoid grid, so reaching the expected
    /// height at all is the assertion.
    #[test]
    fn embedded_geoids_drive_vertical_transformations() {
        // (label, source CRS, target CRS, point, expected height)
        let cases = [
            // JGD2011 + height -> WGS84 3D, central Tokyo. Both Japanese models
            // are embedded and PROJ picks GSIGEO2024, the only one of the two
            // whose operation states an accuracy; GSIGEO2011 puts the same point
            // at 46.59.
            ("japan", 6697u16, 4979u16, [35.6586, 139.7454, 10.0], 46.69),
            // RD New + NAP -> WGS84 3D: nlgeo2018.
            (
                "netherlands",
                7415,
                4979,
                [204000.0, 325300.0, 95.0],
                140.73,
            ),
            // RGF93 / Lambert-93 + NGF-IGN69 -> WGS84 3D: RAF20.
            ("france", 5698, 4979, [650000.0, 6860000.0, 100.0], 143.81),
            // WGS84 3D -> EGM96 height, the global fallback.
            ("egm96", 4979, 5773, [35.6586, 139.7454, 10.0], -26.41),
        ];
        let mut cache = ReprojectionCache::new();
        for (label, from, to, point, expected) in cases {
            let got = cache
                .transform(EpsgCode::new(from), EpsgCode::new(to), point)
                .unwrap_or_else(|e| panic!("{label} EPSG:{from}->EPSG:{to}: {e}"));
            assert!(
                (got[2] - expected).abs() < 0.01,
                "{label}: expected a height near {expected}, got {}",
                got[2]
            );
        }
    }
}
