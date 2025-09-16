use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::runtime_config::WORKING_DIRECTORY;
use crate::{uri::Uri, Error};
use directories::ProjectDirs;

pub fn project_temp_dir(id: &str) -> crate::Result<PathBuf> {
    let p = get_project_cache_dir_path("temp")?;
    let dir_path = PathBuf::from(p).join("temp").join(id);
    fs::create_dir_all(&dir_path).map_err(Error::dir)?;
    Ok(dir_path)
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
