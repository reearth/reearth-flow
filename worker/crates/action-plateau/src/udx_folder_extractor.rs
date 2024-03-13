use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error, utils, Action, ActionContext, ActionDataframe, ActionResult, ActionValue, Result,
    DEFAULT_PORT, REJECTED_PORT,
};

const PKG_FOLDERS: &[&str] = &[
    "area", "bldg", "brid", "cons", "dem", "fld", "frn", "gen", "htd", "ifld", "lsld", "luse",
    "rwy", "squr", "tnm", "tran", "trk", "tun", "ubld", "urf", "unf", "veg", "wtr", "wwy",
];

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UdxFolderExtractor {
    city_gml_path: String,
    codelists_path: Option<String>,
    schemas_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Response {
    root: String,
    package: String,
    admin: String,
    area: String,
    udx_dirs: String,
    dir_root: String,
    dir_codelists: String,
    dir_schemas: String,
}

impl TryFrom<Response> for ActionValue {
    type Error = error::Error;
    fn try_from(value: Response) -> Result<Self, error::Error> {
        let value = serde_json::to_value(value).map_err(|e| {
            error::Error::output(format!("Cannot convert to json with error = {:?}", e))
        })?;
        Ok(ActionValue::from(value))
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "PLATEAU.UDXFolderExtractor")]
impl Action for UdxFolderExtractor {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.unwrap_or_default();
        let city_gml_path =
            utils::get_expr_path(&self.city_gml_path, &inputs, Arc::clone(&ctx.expr_engine))
                .await?;
        let folders = city_gml_path
            .path()
            .to_str()
            .ok_or(error::Error::input("Invalid cityGML path"))
            .map(|path_raw| {
                path_raw
                    .to_string()
                    .split('/')
                    .map(String::from)
                    .collect::<Vec<String>>()
            })?;
        let (mut root, mut pkg, mut admin, mut area, mut dirs) = (
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        );
        let mut rtdir = PathBuf::new();
        match folders.as_slice() {
            [.., fourth_last, _third_last, second_last, _last]
                if PKG_FOLDERS.contains(&second_last.as_str()) =>
            {
                root = fourth_last.to_string();
                pkg = second_last.to_string();
                dirs = second_last.to_string();
                rtdir = PathBuf::from(folders[..folders.len() - 3].join("/"));
            }
            [.., fifth_last, _fourth_last, third_last, second_last, _last]
                if PKG_FOLDERS.contains(&third_last.as_str()) =>
            {
                root = fifth_last.to_string();
                pkg = third_last.to_string();
                area = second_last.to_string();
                dirs = format!("{}/{}", pkg, area);
                rtdir = PathBuf::from(folders[..folders.len() - 4].join("/"));
            }
            [.., sixth_last, _fifth_last, fourth_last, third_last, second_last, _last]
                if PKG_FOLDERS.contains(&fourth_last.as_str()) =>
            {
                root = sixth_last.to_string();
                pkg = fourth_last.to_string();
                admin = third_last.to_string();
                area = second_last.to_string();
                dirs = format!("{}/{}/{}", pkg, admin, area);
                rtdir = PathBuf::from(folders[..folders.len() - 5].join("/"));
            }
            _ => (),
        };
        let dir_codelists = rtdir.join("codelists");
        let dir_schemas = rtdir.join("schemas");

        if PKG_FOLDERS.contains(&pkg.as_str()) {
            if !dir_codelists.exists() {
                let dir = PathBuf::from(
                    self.codelists_path
                        .clone()
                        .ok_or(error::Error::input("Invalid codelists path"))?,
                );
                if dir.exists() {
                    tokio::fs::copy(&dir, &dir_codelists).await?;
                }
            }
            if !dir_schemas.exists() {
                let dir = PathBuf::from(
                    self.schemas_path
                        .clone()
                        .ok_or(error::Error::input("Invalid schemas path"))?,
                );
                if dir.exists() {
                    tokio::fs::copy(&dir, &dir_schemas).await?;
                }
            }
        }
        let res = Response {
            root,
            package: pkg.clone(),
            admin,
            area,
            udx_dirs: dirs,
            dir_root: rtdir.to_str().unwrap_or_default().to_string(),
            dir_codelists: dir_codelists.to_str().unwrap_or_default().to_string(),
            dir_schemas: dir_schemas.to_str().unwrap_or_default().to_string(),
        };
        let output_port = if PKG_FOLDERS.contains(&pkg.as_str()) {
            DEFAULT_PORT
        } else {
            REJECTED_PORT
        };
        Ok(ActionDataframe::from([(
            output_port.to_string(),
            Some(res.try_into()?),
        )]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::Builder;

    #[tokio::test]
    async fn test_extract_udx_folder() {
        let temp_dir = Builder::new().prefix("foobar").tempdir_in(".").unwrap();
        let city_gml_path = temp_dir.path().join("city");

        let extractor = UdxFolderExtractor {
            city_gml_path: city_gml_path.to_str().unwrap().to_string(),
            codelists_path: None,
            schemas_path: None,
        };

        let ctx = ActionContext::default(); // Add any required context here
        let result = extractor.run(ctx, Some(ActionDataframe::new())).await;

        assert!(result.is_ok());
    }
}
