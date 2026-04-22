use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Arc,
};

use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::Expr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_types::{
    AttributeValue, CityGmlGeometry, Feature, Geometry, GeometryType, GeometryValue, GmlGeometry,
    CITYGML_PARENT_GML_ID_KEY, CITYGML_ROOT_GML_ID_KEY,
};

use crate::feature::errors::FeatureProcessorError;

use super::{
    codespace, flatten, geometry,
    parser::{self, Parser},
    utils::{gml_id_attr, XmlNode},
    xlink,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct FeatureCityGml3ReaderFactory;

impl ProcessorFactory for FeatureCityGml3ReaderFactory {
    fn name(&self) -> &str {
        "FeatureCityGml3Reader"
    }

    fn description(&self) -> &str {
        "Reads CityGML 3.0 files: resolves gml:id references and xlink:href links across files"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureCityGml3ReaderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureCityGml3ReaderParam = if let Some(ref with) = with {
            let value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FileCityGml3ReaderFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FileCityGml3ReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FileCityGml3ReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let dataset_ast = Arc::clone(&ctx.expr_engine)
            .compile(params.dataset.as_ref())
            .map_err(|e| FeatureProcessorError::FileCityGml3ReaderFactory(format!("{e:?}")))?;

        let extract_tags: HashSet<String> = params.extract_tags.into_iter().collect();

        Ok(Box::new(FeatureCityGml3Reader {
            global_params: with,
            dataset_ast,
            original_dataset: params.dataset,
            extract_tags,
            parser: Parser::new(),
        }))
    }
}

/// # FeatureCityGml3Reader Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCityGml3ReaderParam {
    /// # Dataset
    /// Path expression resolving to the CityGML 3.0 file to read.
    dataset: Expr,
    /// # Extract Tags
    /// Feature type names to flatten as individual features. Accepts qualified (`bldg:Building`),
    /// local (`Building`), or Clark notation (`{http://…}Building`). Empty means emit all
    /// top-level city objects unchanged.
    #[serde(default)]
    extract_tags: Vec<String>,
}

pub struct FeatureCityGml3Reader {
    global_params: Option<HashMap<String, serde_json::Value>>,
    dataset_ast: rhai::AST,
    original_dataset: Expr,
    extract_tags: HashSet<String>,
    parser: Parser,
}

impl std::fmt::Debug for FeatureCityGml3Reader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeatureCityGml3Reader")
            .field("parser", &self.parser)
            .finish_non_exhaustive()
    }
}

impl Clone for FeatureCityGml3Reader {
    fn clone(&self) -> Self {
        Self {
            global_params: self.global_params.clone(),
            dataset_ast: self.dataset_ast.clone(),
            original_dataset: self.original_dataset.clone(),
            extract_tags: self.extract_tags.clone(),
            parser: Parser::new(),
        }
    }
}

impl Processor for FeatureCityGml3Reader {
    fn num_threads(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let scope = ctx
            .feature
            .new_scope(Arc::clone(&ctx.expr_engine), &self.global_params);

        let path = scope
            .eval_ast::<String>(&self.dataset_ast)
            .unwrap_or_else(|_| self.original_dataset.to_string());

        let uri = Uri::from_str(&path).map_err(|e| {
            FeatureProcessorError::FileCityGml3Reader(format!("Invalid URI `{path}`: {e}"))
        })?;
        let source_url: Url = uri.clone().into();

        let storage = ctx.storage_resolver.resolve(&uri).map_err(|e| {
            FeatureProcessorError::FileCityGml3Reader(format!("Storage resolve error: {e}"))
        })?;
        let bytes = storage.get_sync(uri.path().as_path()).map_err(|e| {
            FeatureProcessorError::FileCityGml3Reader(format!("File read error: {e}"))
        })?;

        self.parser
            .parse(&bytes, &source_url)
            .map_err(|e| FeatureProcessorError::FileCityGml3Reader(format!("{e}")))?;
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let (pending, raw_registry, ns_registry) = std::mem::take(&mut self.parser).finish();
        let mut codelist_resolver = codespace::CodelistResolver::new();
        for feature_root in codespace::resolve(
            xlink::resolve(pending, &raw_registry),
            &mut codelist_resolver,
        ) {
            if self.extract_tags.is_empty() {
                let feature = build_feature(&feature_root);
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    feature,
                    DEFAULT_PORT.clone(),
                ));
            } else {
                let root_gml_id = gml_id_attr(&feature_root);

                for (node, parent_id) in
                    flatten::extract(&feature_root, &self.extract_tags, &ns_registry)
                {
                    let mut feature = build_feature(&node);
                    if let Some(id) = parent_id {
                        feature.insert(CITYGML_PARENT_GML_ID_KEY, AttributeValue::String(id));
                    }
                    if let Some(ref id) = root_gml_id {
                        feature.insert(CITYGML_ROOT_GML_ID_KEY, AttributeValue::String(id.clone()));
                    }
                    fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                        &ctx,
                        feature,
                        DEFAULT_PORT.clone(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureCityGml3Reader"
    }
}

fn build_feature(node: &Arc<XmlNode>) -> Feature {
    let (stripped, raw_geoms) = geometry::extract_geometries(node);
    let mut feature = parser::to_feature(&stripped);
    if !raw_geoms.is_empty() {
        *feature.geometry_mut() = Geometry::with_value(GeometryValue::CityGmlGeometry(
            build_citygml_geometry(raw_geoms),
        ));
    }
    feature
}

// pos is assigned here; neutral appearance arrays prevent out-of-bounds access in downstream consumers.
fn build_citygml_geometry(raw: Vec<GmlGeometry>) -> CityGmlGeometry {
    let mut polygon_materials: Vec<Option<u32>> = Vec::new();
    let mut polygon_textures: Vec<Option<u32>> = Vec::new();
    let mut polygon_uvs: Vec<Polygon2D<f64>> = Vec::new();
    let mut current_pos: u32 = 0;
    let mut gml_geometries: Vec<GmlGeometry> = Vec::with_capacity(raw.len());

    for mut g in raw {
        if matches!(
            g.ty,
            GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle
        ) {
            g.pos = current_pos;
            current_pos += g.len;
            for poly in &g.polygons {
                polygon_materials.push(None);
                polygon_textures.push(None);
                polygon_uvs.push(neutral_uv_polygon(poly));
            }
        }
        gml_geometries.push(g);
    }

    CityGmlGeometry {
        gml_geometries,
        materials: Vec::new(),
        textures: Vec::new(),
        polygon_materials,
        polygon_textures,
        polygon_uvs: MultiPolygon2D::new(polygon_uvs),
    }
}

fn neutral_uv_polygon(poly: &Polygon3D<f64>) -> Polygon2D<f64> {
    let ext = LineString2D::new(vec![[0.0f64, 0.0f64].into(); poly.exterior().0.len()]);
    let ints = poly
        .interiors()
        .iter()
        .map(|ring| LineString2D::new(vec![[0.0f64, 0.0f64].into(); ring.0.len()]))
        .collect();
    Polygon2D::new(ext, ints)
}
