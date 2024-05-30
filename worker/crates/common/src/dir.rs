use directories::ProjectDirs;

use crate::Error;

pub fn project_output_dir(id: &str) -> crate::Result<String> {
    let p = ProjectDirs::from("reearth", "flow", "worker")
        .ok_or(Error::dir("No output path uri provided"))?;
    let p = p
        .cache_dir()
        .to_str()
        .ok_or(Error::dir("Invalid output path uri"))?;
    Ok(format!("{}/output/{}/", p, id))
}
