use std::{fs, path::PathBuf};

use directories::ProjectDirs;

use crate::{uri::Uri, Error};

pub fn project_output_dir(id: &str) -> crate::Result<String> {
    let p = ProjectDirs::from("reearth", "flow", "worker")
        .ok_or(Error::dir("No output path uri provided"))?;
    let p = p
        .cache_dir()
        .to_str()
        .ok_or(Error::dir("Invalid output path uri"))?;
    Ok(format!("{}/output/{}/", p, id))
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
    Ok(Uri::for_test(format!("file://{}", p).as_str()))
}
