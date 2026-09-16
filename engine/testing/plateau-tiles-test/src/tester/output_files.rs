use serde::Deserialize;
use std::path::Path;
use walkdir::WalkDir;

/// `must_exclude` accepts either an explicit list of relative paths, or the
/// sentinel `"rest"` for closed-world mode: nothing outside `must_include`
/// may exist under the flow output root.
#[derive(Debug, Deserialize, Default)]
#[serde(untagged)]
pub enum MustExclude {
    #[default]
    None,
    Paths(Vec<String>),
    Rest(RestSentinel),
}

/// Deserializes only from the literal string `"rest"`, so a typo (e.g.
/// `"Rest"`, `"all"`) fails config parsing instead of silently matching
/// `Paths` or `None`.
#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct RestSentinel;

impl TryFrom<String> for RestSentinel {
    type Error = String;
    fn try_from(s: String) -> Result<Self, String> {
        if s == "rest" {
            Ok(RestSentinel)
        } else {
            Err(format!(
                "must_exclude: expected a path list or the string \"rest\", got {s:?}"
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OutputFilesConfig {
    /// Exact paths, relative to the flow output root, that must each exist.
    #[serde(default)]
    pub must_include: Vec<String>,
    #[serde(default)]
    pub must_exclude: MustExclude,
}

/// Lists every file under `root`, relative to `root`, using `/` as the
/// separator regardless of platform.
fn list_relative_files(root: &Path) -> Result<Vec<String>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| {
            let rel = entry.path().strip_prefix(root).map_err(|e| e.to_string())?;
            Ok(rel.to_string_lossy().replace('\\', "/"))
        })
        .collect()
}

pub fn test_output_files(output_root: &Path, config: &OutputFilesConfig) -> Result<(), String> {
    let entries = list_relative_files(output_root)?;

    for path in &config.must_include {
        assert!(
            entries.iter().any(|e| e == path),
            "must_include path {path:?} not found under {output_root:?} (found: {entries:?})"
        );
    }

    match &config.must_exclude {
        MustExclude::None => {}
        MustExclude::Paths(paths) => {
            for path in paths {
                assert!(
                    !entries.iter().any(|e| e == path),
                    "must_exclude path {path:?} exists under {output_root:?}"
                );
            }
        }
        MustExclude::Rest(_) => {
            for entry in &entries {
                assert!(
                    config.must_include.iter().any(|p| p == entry),
                    "unexpected file {entry:?} under {output_root:?} (must_exclude = \"rest\", \
                     and it is not listed in must_include: {:?})",
                    config.must_include
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write(dir: &Path, rel: &str) {
        let path = dir.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"").unwrap();
    }

    #[test]
    fn must_include_passes_when_present() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "a.zip");
        let cfg = OutputFilesConfig {
            must_include: vec!["a.zip".to_string()],
            must_exclude: MustExclude::None,
        };
        test_output_files(tmp.path(), &cfg).unwrap();
    }

    #[test]
    fn must_include_fails_when_missing() {
        let tmp = TempDir::new().unwrap();
        let cfg = OutputFilesConfig {
            must_include: vec!["a.zip".to_string()],
            must_exclude: MustExclude::None,
        };
        assert!(std::panic::catch_unwind(|| test_output_files(tmp.path(), &cfg)).is_err());
    }

    #[test]
    fn must_exclude_paths_fail_on_match() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "dictionary.json");
        let cfg = OutputFilesConfig {
            must_include: vec![],
            must_exclude: MustExclude::Paths(vec!["dictionary.json".to_string()]),
        };
        assert!(std::panic::catch_unwind(|| test_output_files(tmp.path(), &cfg)).is_err());
    }

    #[test]
    fn must_exclude_rest_rejects_unlisted_entries() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "expected.zip");
        write(tmp.path(), "surprise.zip");
        let cfg = OutputFilesConfig {
            must_include: vec!["expected.zip".to_string()],
            must_exclude: MustExclude::Rest(RestSentinel),
        };
        assert!(std::panic::catch_unwind(|| test_output_files(tmp.path(), &cfg)).is_err());
    }

    #[test]
    fn must_exclude_rest_passes_when_closed() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "expected.zip");
        let cfg = OutputFilesConfig {
            must_include: vec!["expected.zip".to_string()],
            must_exclude: MustExclude::Rest(RestSentinel),
        };
        test_output_files(tmp.path(), &cfg).unwrap();
    }

    #[test]
    fn rest_sentinel_rejects_typos() {
        let err: Result<RestSentinel, _> = RestSentinel::try_from("Rest".to_string());
        assert!(err.is_err());
    }
}
