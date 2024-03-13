use directories::ProjectDirs;

use crate::error::Error;

pub fn project_output_dir(node_id: &str) -> crate::Result<String> {
    let p = ProjectDirs::from("reearth", "flow", "worker")
        .ok_or(Error::input("No output path uri provided"))?;
    let p = p
        .data_dir()
        .to_str()
        .ok_or(Error::input("Invalid output path uri"))?;
    Ok(format!("{}/output/{}", p, node_id))
}
