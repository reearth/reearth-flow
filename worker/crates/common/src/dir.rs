use std::{fs, path::PathBuf, str::FromStr};

use directories::ProjectDirs;

use crate::{uri::Uri, Error};

pub fn project_output_dir(id: &str) -> crate::Result<String> {
    let p = get_project_cache_dir_path("worker")?;
    PathBuf::from(p)
        .join("output")
        .join(id)
        .to_str()
        .map_or_else(
            || Err(Error::dir("Invalid project directory path")),
            |s| Ok(s.to_string()),
        )
}

pub fn get_project_cache_dir_path(key: &str) -> crate::Result<String> {
    ProjectDirs::from("reearth", "flow", key)
        .ok_or(Error::dir("No project directory available"))?
        .cache_dir()
        .to_str()
        .ok_or(Error::dir("Invalid project directory path"))
        .map(String::from)
}

pub fn setup_job_directory(key: &str, sub_dir: &str, job_id: uuid::Uuid) -> crate::Result<Uri> {
    let p = get_project_cache_dir_path(key)?;
    let dir_path = PathBuf::from(p.clone())
        .join(sub_dir)
        .join(job_id.to_string());
    fs::create_dir_all(&dir_path).map_err(Error::dir)?;
    Uri::from_str(
        dir_path
            .as_path()
            .to_str()
            .ok_or(Error::dir("Invalid job directory path"))?,
    )
    .map_err(|e| {
        Error::dir(format!(
            "Failed to create URI from job directory path with error: {}",
            e
        ))
    })
}
