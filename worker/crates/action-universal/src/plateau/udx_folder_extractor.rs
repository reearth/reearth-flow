use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::Arc};

use reearth_flow_common::uri::Uri;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_storage::resolve::StorageResolver;
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error, ActionContext, ActionDataframe, ActionResult, AsyncAction, Attribute, AttributeValue,
    Dataframe, Feature, Result, DEFAULT_PORT, REJECTED_PORT,
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
    city_gml_path: String,
    root: String,
    package: String,
    admin: String,
    area: String,
    udx_dirs: String,
    dir_root: String,
    dir_codelists: String,
    dir_schemas: String,
}

impl From<Response> for HashMap<Attribute, AttributeValue> {
    fn from(value: Response) -> Self {
        serde_json::to_value(value)
            .unwrap()
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (Attribute::new(k), AttributeValue::from(v.clone())))
            .collect()
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "PLATEAU.UDXFolderExtractor")]
impl AsyncAction for UdxFolderExtractor {
    async fn run(&self, ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let input = inputs
            .get(&DEFAULT_PORT)
            .ok_or(error::Error::input("No Default Port"))?;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let ast = ctx
            .expr_engine
            .compile(self.city_gml_path.as_str())
            .map_err(error::Error::internal_runtime)?;

        let mut success = Vec::<HashMap<Attribute, AttributeValue>>::new();
        let mut rejected = Vec::<HashMap<Attribute, AttributeValue>>::new();
        for row in &input.features {
            let res = mapper(
                row,
                &ast,
                Arc::clone(&expr_engine),
                Arc::clone(&storage_resolver),
                &self.codelists_path,
                &self.schemas_path,
            )
            .await?;
            if PKG_FOLDERS.contains(&res.package.as_str()) {
                success.push(res.into());
            } else {
                rejected.push(res.into());
            };
        }
        Ok(ActionDataframe::from([
            (DEFAULT_PORT.clone(), Dataframe::from(success)),
            (REJECTED_PORT.clone(), Dataframe::from(rejected)),
        ]))
    }
}

async fn mapper(
    row: &Feature,
    expr: &rhai::AST,
    expr_engine: Arc<Engine>,
    storage_resolver: Arc<StorageResolver>,
    codelists_path: &Option<String>,
    schemas_path: &Option<String>,
) -> Result<Response> {
    let city_gml_path = {
        let scope = expr_engine.new_scope();
        for (k, v) in &row.attributes {
            scope.set(k.clone().into_inner().as_str(), v.clone().into());
        }
        scope
            .eval_ast::<String>(expr)
            .map_err(error::Error::input)?
    };
    let folders = city_gml_path
        .split('/')
        .map(String::from)
        .collect::<Vec<String>>();
    let city_gml_path =
        Uri::from_str(city_gml_path.to_string().as_str()).map_err(error::Error::input)?;
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
    let (dir_root, dir_codelists, dir_schemas) = if !rtdir.as_os_str().is_empty() {
        let (dir_root, dir_codelists, dir_schemas) = gen_codelists_and_schemas_path(
            codelists_path,
            schemas_path,
            rtdir,
            pkg.clone(),
            Arc::clone(&storage_resolver),
        )
        .await?;
        (
            dir_root.to_string(),
            dir_codelists.to_string(),
            dir_schemas.to_string(),
        )
    } else {
        ("".to_string(), "".to_string(), "".to_string())
    };
    Ok(Response {
        city_gml_path: city_gml_path.to_string(),
        root,
        package: pkg.clone(),
        admin,
        area,
        udx_dirs: dirs,
        dir_root,
        dir_codelists,
        dir_schemas,
    })
}

async fn gen_codelists_and_schemas_path(
    codelists_path: &Option<String>,
    schemas_path: &Option<String>,
    rtdir: PathBuf,
    pkg: String,
    storage_resolver: Arc<StorageResolver>,
) -> Result<(Uri, Uri, Uri)> {
    let rtdir: Uri = rtdir.try_into().map_err(error::Error::internal_runtime)?;
    let storage = storage_resolver
        .resolve(&rtdir)
        .map_err(error::Error::internal_runtime)?;

    let dir_codelists = rtdir
        .join("codelists")
        .map_err(error::Error::internal_runtime)?;
    let dir_schemas = rtdir
        .join("schemas")
        .map_err(error::Error::internal_runtime)?;

    if PKG_FOLDERS.contains(&pkg.as_str()) {
        if !storage
            .exists(dir_codelists.path().as_path())
            .await
            .map_err(error::Error::internal_runtime)?
        {
            let dir = Uri::for_test(
                &codelists_path
                    .clone()
                    .ok_or(error::Error::input("Invalid codelists path"))?,
            );
            if !storage
                .exists(dir.path().as_path())
                .await
                .map_err(error::Error::internal_runtime)?
            {
                storage
                    .copy(dir.path().as_path(), dir_codelists.path().as_path())
                    .await
                    .map_err(error::Error::internal_runtime)?;
            }
        }
        if !storage
            .exists(dir_schemas.path().as_path())
            .await
            .map_err(error::Error::internal_runtime)?
        {
            let dir = Uri::for_test(
                &schemas_path
                    .clone()
                    .ok_or(error::Error::input("Invalid codelists path"))?,
            );
            if !storage
                .exists(dir.path().as_path())
                .await
                .map_err(error::Error::internal_runtime)?
            {
                storage
                    .copy(dir.path().as_path(), dir_codelists.path().as_path())
                    .await
                    .map_err(error::Error::internal_runtime)?;
            }
        }
    }
    Ok((rtdir, dir_codelists, dir_schemas))
}
