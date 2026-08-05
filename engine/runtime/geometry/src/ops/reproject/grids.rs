//! The geodetic grids carried inside the binary, and the PROJ search path built
//! from them.
//!
//! PROJ resolves a vertical datum change by reading a geoid grid; with no grid
//! it has no operation to offer and the reprojection fails outright. The grids
//! in `grids/` are compiled in so that the transformations they cover work with
//! nothing installed on the machine. Everything else is external: point
//! [`GRID_DIR_VAR`] at a directory of `.tif` grids and they are searched first.

use std::ffi::{CStr, CString};
use std::fs;
use std::io;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use proj_sys::{proj_context_create, proj_context_set_search_paths, proj_info, PJ_CONTEXT};

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

macro_rules! embedded {
    ($($name:literal),* $(,)?) => {
        &[$(EmbeddedGrid {
            name: $name,
            bytes: include_bytes!(concat!("../../../grids/", $name)),
        }),*]
    };
}

/// The embedded set: current-generation geoid models, one per national vertical
/// datum, plus the global EGM96 fallback. Kept in step with `grids/MANIFEST.tsv`
/// by [`tests::embedded_set_matches_manifest`].
static EMBEDDED_GRIDS: &[EmbeddedGrid] = embedded![
    "at_bev_GV_Hoehengrid_plus_Geoid_V2.tif",
    "be_ign_hBG18.tif",
    "ch_swisstopo_chgeo2004_ETRS89_LHN95.tif",
    "ch_swisstopo_chgeo2004_ETRS89_LN02.tif",
    "cz_cuzk_CR-2005.tif",
    "de_bkg_gcg2016.tif",
    "dk_sdfi_dvr90_2023.tif",
    "es_ign_egm08-rednap-canarias.tif",
    "es_ign_egm08-rednap.tif",
    "fi_nls_fin2023n2000.tif",
    "fr_ign_RAF20.tif",
    "hu_bme_geoid2014.tif",
    "is_lmi_Icegeoid_ISN2016.tif",
    "jp_gsi_gsigeo2011.tif",
    "jp_gsi_jpgeo2024.tif",
    "lv_lgia_lv14.tif",
    "nl_nsgi_nlgeo2018.tif",
    "no_kv_HREF2018B_NN2000_EUREF89.tif",
    "pl_gugik_geoid2021-PL-EVRF2007-NH.tif",
    "pt_dgt_GeodPT08.tif",
    "se_lantmateriet_SWEN17_RH2000.tif",
    "si_gurs_SLO-VRP2016-Koper.tif",
    "sk_gku_Slovakia_ETRS89h_to_EVRF2007.tif",
    "uk_os_OSGM15_Belfast.tif",
    "uk_os_OSGM15_GB.tif",
    "uk_os_OSGM15_Malin.tif",
    "us_nga_egm96_15.tif",
];

/// Create a PROJ context that can see the embedded and external grids.
///
/// Every context in this crate must come from here: PROJ resolves grids against
/// the search paths of the context that builds the transformation, so a context
/// created directly would only see whatever the machine happens to have.
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
/// grids could not be written out, since that turns transformations that normally
/// work into failures and is the more useful thing to say.
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
        let dirs = ordered_dirs(external, embedded, proj_default_paths());
        Resolved {
            search: dirs
                .iter()
                .filter_map(|d| CString::new(d.as_os_str().as_encoded_bytes()).ok())
                .collect(),
            unpack_failure,
        }
    })
}

/// The directories named by [`GRID_DIR_VAR`], dropping the empty entries a path
/// list picks up from a trailing or doubled separator.
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
///
/// The last part matters because setting search paths *replaces* PROJ's
/// defaults rather than adding to them, and a build linked against a system
/// PROJ reads `proj.db` from one of those default directories.
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

/// The directories PROJ searches by default, read before any context of ours
/// overrides them.
fn proj_default_paths() -> Vec<PathBuf> {
    // SAFETY: `proj_info` returns a struct of pointers into PROJ's own static
    // storage, valid for the life of the process; `path_count` bounds the array.
    unsafe {
        let info = proj_info();
        if info.paths.is_null() {
            return Vec::new();
        }
        (0..info.path_count)
            .filter_map(|i| {
                let p = *info.paths.add(i);
                (!p.is_null()).then(|| PathBuf::from(CStr::from_ptr(p).to_string_lossy().as_ref()))
            })
            .collect()
    }
}

/// What became of the embedded grids.
enum Embedded {
    /// They are readable in this directory, which belongs on the search path.
    Dir(PathBuf),
    /// Every one of them was already available from an earlier directory, so
    /// nothing was written and there is no directory to add.
    AlreadySupplied,
    /// They could not be written anywhere, with every attempt reported.
    Failed(String),
}

/// Write out the embedded grids that `external` does not already supply.
///
/// An external directory holding the same grids under the same names makes the
/// write redundant, and skipping it costs a container whose filesystem is in RAM
/// nothing at start-up.
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

/// Where to unpack the embedded grids, in order of preference. A configured
/// directory is the only candidate, since silently writing somewhere else would
/// defeat the point of configuring one; otherwise the user cache directory is
/// tried first and the temporary directory second, which is what a container
/// with no writable home falls back to.
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

/// Write each of `grids` into `dir` unless it is already there.
///
/// A grid file is immutable under its name, so one whose size already matches is
/// left alone. Writes go to a temporary name and are renamed into place, so a
/// reader in another process sees either the old file or the whole new one, and
/// concurrent writers cannot interleave.
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
    use std::collections::BTreeSet;
    use std::ptr;

    use super::*;
    use crate::coordinate::EpsgCode;
    use crate::ops::reproject::ReprojectionCache;

    /// The names and sizes recorded in the manifest the vendoring script writes.
    fn manifest() -> Vec<(&'static str, usize)> {
        include_str!("../../../grids/MANIFEST.tsv")
            .lines()
            .skip(2)
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let mut fields = line.split('\t');
                let name = fields.next().expect("manifest row has a name");
                let size = fields.next().expect("manifest row has a size");
                (name, size.parse().expect("manifest size is a number"))
            })
            .collect()
    }

    #[test]
    fn embedded_set_matches_manifest() {
        let expected: BTreeSet<&str> = manifest().into_iter().map(|(name, _)| name).collect();
        let embedded: BTreeSet<&str> = EMBEDDED_GRIDS.iter().map(|g| g.name).collect();
        assert_eq!(embedded, expected);
    }

    #[test]
    fn embedded_grids_are_whole() {
        for (name, size) in manifest() {
            let grid = EMBEDDED_GRIDS
                .iter()
                .find(|g| g.name == name)
                .expect("checked by embedded_set_matches_manifest");
            assert_eq!(grid.bytes.len(), size, "{name} is not the vendored size");
        }
    }

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

    /// Unpack `grids` into the first of `candidates` that accepts the write, the
    /// way [`unpack_embedded`] does, without consulting the environment.
    fn unpack_into_first_writable(
        candidates: Vec<PathBuf>,
        grids: &[&EmbeddedGrid],
    ) -> std::result::Result<PathBuf, String> {
        let mut failures = Vec::new();
        for dir in candidates {
            match unpack(&dir, grids) {
                Ok(()) => return Ok(dir),
                Err(e) => failures.push(format!("{}: {e}", dir.display())),
            }
        }
        Err(failures.join("; "))
    }

    fn all_grids() -> Vec<&'static EmbeddedGrid> {
        EMBEDDED_GRIDS.iter().collect()
    }

    #[test]
    fn external_directories_are_searched_before_the_embedded_ones() {
        let dirs = ordered_dirs(
            vec![PathBuf::from("/srv/grids"), PathBuf::from("/mnt/grids")],
            Some(PathBuf::from("/cache/embedded")),
            vec![PathBuf::from("/usr/share/proj")],
        );
        assert_eq!(
            dirs,
            [
                PathBuf::from("/srv/grids"),
                PathBuf::from("/mnt/grids"),
                PathBuf::from("/cache/embedded"),
                PathBuf::from("/usr/share/proj"),
            ]
        );
    }

    #[test]
    fn proj_own_directories_are_kept_when_there_is_nothing_of_ours() {
        // Setting search paths replaces PROJ's defaults, so dropping them would
        // cost a system-PROJ build its `proj.db`.
        let dirs = ordered_dirs(Vec::new(), None, vec![PathBuf::from("/usr/share/proj")]);
        assert_eq!(dirs, [PathBuf::from("/usr/share/proj")]);
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
    fn only_the_grids_an_external_directory_lacks_are_unpacked() {
        let external = tempfile::tempdir().unwrap();
        let supplied = EMBEDDED_GRIDS[0].name;
        fs::write(external.path().join(supplied), EMBEDDED_GRIDS[0].bytes).unwrap();
        let cache = tempfile::tempdir().unwrap();
        let missing: Vec<&EmbeddedGrid> = EMBEDDED_GRIDS
            .iter()
            .filter(|g| !external.path().join(g.name).is_file())
            .collect();
        assert_eq!(missing.len(), EMBEDDED_GRIDS.len() - 1);
        unpack(cache.path(), &missing).unwrap();
        assert!(!cache.path().join(supplied).exists());
        assert!(cache.path().join(EMBEDDED_GRIDS[1].name).is_file());
    }

    #[test]
    fn an_unwritable_cache_directory_falls_through_to_the_next_one() {
        let usable = tempfile::tempdir().unwrap();
        fs::write(usable.path().join("blocker"), b"").unwrap();
        let dir = unpack_into_first_writable(
            vec![
                // A directory cannot be created underneath a regular file.
                usable.path().join("blocker").join("grids"),
                usable.path().join("grids"),
            ],
            &all_grids(),
        )
        .unwrap();
        assert_eq!(dir, usable.path().join("grids"));
        assert!(dir.join(EMBEDDED_GRIDS[0].name).is_file());
    }

    #[test]
    fn no_writable_cache_directory_reports_every_attempt() {
        let blocked = tempfile::tempdir().unwrap();
        fs::write(blocked.path().join("file"), b"").unwrap();
        let err = unpack_into_first_writable(
            vec![
                blocked.path().join("file").join("a"),
                blocked.path().join("file").join("b"),
            ],
            &all_grids(),
        )
        .unwrap_err();
        assert!(err.contains("/a:") && err.contains("/b:"), "{err}");
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
