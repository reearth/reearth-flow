use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::Arc};

use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_storage::resolve::StorageResolver;

use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::PlateauProcessorError;

const PKG_FOLDERS: &[&str] = &[
    "area", "bldg", "brid", "cons", "dem", "fld", "frn", "gen", "htd", "ifld", "lsld", "luse",
    "rwy", "squr", "tnm", "tran", "trk", "tun", "ubld", "urf", "unf", "veg", "wtr", "wwy",
];

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

#[derive(Debug, Clone, Default)]
pub struct UdxFolderExtractorFactory;

impl ProcessorFactory for UdxFolderExtractorFactory {
    fn name(&self) -> &str {
        "PLATEAU.UDXFolderExtractor"
    }

    fn description(&self) -> &str {
        "Extracts UDX folders from cityGML path"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: UdxFolderExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::UdxFolderExtractorFactory(format!(
                    "Failed to serialize with: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::UdxFolderExtractorFactory(format!(
                    "Failed to deserialize with: {}",
                    e
                ))
            })?
        } else {
            return Err(PlateauProcessorError::UdxFolderExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let city_gml_path = expr_engine
            .compile(params.city_gml_path.as_ref())
            .map_err(|e| {
                PlateauProcessorError::UdxFolderExtractorFactory(format!(
                    "Failed to compile city_gml_path: {}",
                    e
                ))
            })?;
        let process = UdxFolderExtractor {
            city_gml_path,
            codelists_path: params.codelists_path,
            schemas_path: params.schemas_path,
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct UdxFolderExtractor {
    city_gml_path: rhai::AST,
    codelists_path: Option<String>,
    schemas_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UdxFolderExtractorParam {
    city_gml_path: Expr,
    codelists_path: Option<String>,
    schemas_path: Option<String>,
}

impl Processor for UdxFolderExtractor {
    fn initialize(&mut self, _ctx: NodeContext) {}

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let res = mapper(
            feature,
            &self.city_gml_path,
            Arc::clone(&ctx.expr_engine),
            Arc::clone(&ctx.storage_resolver),
            &self.codelists_path,
            &self.schemas_path,
        )?;
        let port = if PKG_FOLDERS.contains(&res.package.as_str()) {
            DEFAULT_PORT.clone()
        } else {
            REJECTED_PORT.clone()
        };
        let attributes: HashMap<Attribute, AttributeValue> = res.into();
        let feature = Feature {
            attributes,
            ..feature.clone()
        };
        fw.send(ctx.new_with_feature_and_port(feature, port));
        Ok(())
    }

    fn finish(
        &self,
        _ctx: NodeContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "UdxFolderExtractor"
    }
}

fn mapper(
    row: &Feature,
    expr: &rhai::AST,
    expr_engine: Arc<Engine>,
    storage_resolver: Arc<StorageResolver>,
    codelists_path: &Option<String>,
    schemas_path: &Option<String>,
) -> super::errors::Result<Response> {
    let city_gml_path = {
        let scope = expr_engine.new_scope();
        for (k, v) in &row.attributes {
            scope.set(k.clone().into_inner().as_str(), v.clone().into());
        }
        scope
            .eval_ast::<String>(expr)
            .map_err(|e| PlateauProcessorError::UdxFolderExtractor(format!("{:?}", e)))?
    };
    let folders = city_gml_path
        .split('/')
        .map(String::from)
        .collect::<Vec<String>>();
    let city_gml_path = Uri::from_str(city_gml_path.to_string().as_str())
        .map_err(|e| PlateauProcessorError::UdxFolderExtractor(format!("{:?}", e)))?;
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
        )?;
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

fn gen_codelists_and_schemas_path(
    codelists_path: &Option<String>,
    schemas_path: &Option<String>,
    rtdir: PathBuf,
    pkg: String,
    storage_resolver: Arc<StorageResolver>,
) -> super::errors::Result<(Uri, Uri, Uri)> {
    let rtdir: Uri = rtdir
        .try_into()
        .map_err(|e| PlateauProcessorError::UdxFolderExtractor(format!("{:?}", e)))?;
    let storage = storage_resolver
        .resolve(&rtdir)
        .map_err(|e| PlateauProcessorError::UdxFolderExtractor(format!("{:?}", e)))?;

    let dir_codelists = rtdir
        .join("codelists")
        .map_err(|e| PlateauProcessorError::UdxFolderExtractor(format!("{:?}", e)))?;
    let dir_schemas = rtdir
        .join("schemas")
        .map_err(|e| PlateauProcessorError::UdxFolderExtractor(format!("{:?}", e)))?;

    if PKG_FOLDERS.contains(&pkg.as_str()) {
        if !storage
            .exists_sync(dir_codelists.path().as_path())
            .map_err(|e| PlateauProcessorError::UdxFolderExtractor(format!("{:?}", e)))?
        {
            let dir = Uri::for_test(&codelists_path.clone().ok_or(
                PlateauProcessorError::UdxFolderExtractor("Invalid codelists path".to_string()),
            )?);
            if !storage
                .exists_sync(dir.path().as_path())
                .map_err(|e| PlateauProcessorError::UdxFolderExtractor(format!("{:?}", e)))?
            {
                storage
                    .copy_sync(dir.path().as_path(), dir_codelists.path().as_path())
                    .map_err(|e| PlateauProcessorError::UdxFolderExtractor(format!("{:?}", e)))?;
            }
        }
        if !storage
            .exists_sync(dir_schemas.path().as_path())
            .map_err(|e| PlateauProcessorError::UdxFolderExtractor(format!("{:?}", e)))?
        {
            let dir = Uri::for_test(&schemas_path.clone().ok_or(
                PlateauProcessorError::UdxFolderExtractor("Invalid codelists path".to_string()),
            )?);
            if !storage
                .exists_sync(dir.path().as_path())
                .map_err(|e| PlateauProcessorError::UdxFolderExtractor(format!("{:?}", e)))?
            {
                storage
                    .copy_sync(dir.path().as_path(), dir_codelists.path().as_path())
                    .map_err(|e| PlateauProcessorError::UdxFolderExtractor(format!("{:?}", e)))?;
            }
        }
    }
    Ok((rtdir, dir_codelists, dir_schemas))
}
