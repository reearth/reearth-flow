use std::{
    env, fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use directories::ProjectDirs;
use once_cell::sync::Lazy;

use crate::{uri::Uri, Error};

static WORKING_DIRECTORY: Lazy<Option<String>> =
    Lazy::new(|| env::var("FLOW_RUNTIME_WORKING_DIRECTORY").ok());

static JOB_ARTIFACT_DIR: Lazy<Option<String>> =
    Lazy::new(|| env::var("FLOW_RUNTIME_JOB_ARTIFACT_DIR").ok());

static JOB_TEMP_DIR: Lazy<Option<String>> =
    Lazy::new(|| env::var("FLOW_RUNTIME_JOB_TEMP_DIR").ok());

static PREVIOUS_JOB_ARTIFACT_DIR: Lazy<Option<String>> =
    Lazy::new(|| env::var("FLOW_RUNTIME_PREVIOUS_JOB_ARTIFACT_DIR").ok());

static PREVIOUS_JOB_TEMP_DIR: Lazy<Option<String>> =
    Lazy::new(|| env::var("FLOW_RUNTIME_PREVIOUS_JOB_TEMP_DIR").ok());

pub fn job_artifact_dir(id: &str) -> crate::Result<PathBuf> {
    let root = current_job_artifact_root()?;
    fs::create_dir_all(&root).map_err(Error::dir)?;
    Ok(root)
}

// Compatibility: old API name (the word "temp" is used, but the actual location is under artifacts)
#[deprecated(note = "use job_artifact_dir(); temp directory is deprecated")]
pub fn project_temp_dir(id: &str) -> crate::Result<PathBuf> {
    job_artifact_dir(id)
}

pub fn current_job_artifact_root() -> crate::Result<PathBuf> {
    // Check the recommended env first, then the compatibility env
    let dir = JOB_ARTIFACT_DIR
        .as_ref()
        .or(JOB_TEMP_DIR.as_ref())
        .ok_or(Error::dir(
            "FLOW_RUNTIME_JOB_ARTIFACT_DIR / FLOW_RUNTIME_JOB_TEMP_DIR is not set",
        ))?;
    Ok(PathBuf::from(dir))
}

#[deprecated(note = "use current_job_artifact_root(); temp root is deprecated")]
pub fn current_job_temp_root() -> crate::Result<PathBuf> {
    current_job_artifact_root()
}

pub fn previous_job_artifact_root_opt() -> Option<PathBuf> {
    PREVIOUS_JOB_ARTIFACT_DIR
        .as_ref()
        .or(PREVIOUS_JOB_TEMP_DIR.as_ref())
        .map(PathBuf::from)
}

#[deprecated(note = "use previous_job_artifact_root_opt(); temp root is deprecated")]
pub fn previous_job_temp_root_opt() -> Option<PathBuf> {
    previous_job_artifact_root_opt()
}

pub fn get_project_cache_dir_path(key: &str) -> crate::Result<String> {
    if let Some(dir) = WORKING_DIRECTORY.as_ref() {
        PathBuf::from(dir)
            .join("projects")
            .join(key)
            .to_str()
            .ok_or(Error::dir("Invalid project directory path"))
            .map(String::from)
    } else {
        ProjectDirs::from("reearth", "flow", key)
            .ok_or(Error::dir("No project directory available"))?
            .cache_dir()
            .to_str()
            .ok_or(Error::dir("Invalid project directory path"))
            .map(String::from)
    }
}

pub fn get_job_root_dir_path(key: &str, job_id: uuid::Uuid) -> crate::Result<PathBuf> {
    let p = get_project_cache_dir_path(key)?;
    let dir_path = PathBuf::from(p.clone())
        .join("jobs")
        .join(job_id.to_string());
    Ok(dir_path)
}

pub fn setup_job_directory(key: &str, sub_dir: &str, job_id: uuid::Uuid) -> crate::Result<Uri> {
    let dir_path = get_job_root_dir_path(key, job_id)?.join(sub_dir);
    fs::create_dir_all(&dir_path).map_err(Error::dir)?;
    Uri::from_str(
        dir_path
            .as_path()
            .to_str()
            .ok_or(Error::dir("Invalid job directory path"))?,
    )
    .map_err(|e| {
        Error::dir(format!(
            "Failed to create URI from job directory path with error: {e}"
        ))
    })
}

pub fn copy_files(dest: &Path, files: &[Uri]) -> crate::Result<()> {
    for file in files {
        let file_path = dest.join(file.file_name().ok_or(Error::dir("Invalid file path"))?);
        fs::copy(file.path(), file_path).map_err(Error::dir)?;
    }
    Ok(())
}

pub fn move_files(dest: &Path, files: &[Uri]) -> crate::Result<()> {
    for file in files {
        let file_path = dest.join(file.file_name().ok_or(Error::dir("Invalid file path"))?);
        fs::rename(file.path(), file_path.clone()).map_err(Error::dir)?;
    }
    Ok(())
}
