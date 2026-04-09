use std::{collections::HashMap, str::FromStr, sync::Arc};

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
use reearth_flow_types::{CityGmlGeometry, Geometry, GeometryType, GeometryValue, GmlGeometry};

use crate::feature::errors::FeatureProcessorError;

use super::{
    geometry,
    parser::{self, TopLevelFeature},
    utils::IdRegistry,
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

        Ok(Box::new(FeatureCityGml3Reader {
            global_params: with,
            dataset_ast,
            original_dataset: params.dataset,
            id_registry: IdRegistry::new(),
            pending: Vec::new(),
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

    /// # Flatten
    /// Reserved for future subfeature extraction; currently has no effect.
    flatten: Option<bool>,
}

pub struct FeatureCityGml3Reader {
    global_params: Option<HashMap<String, serde_json::Value>>,
    dataset_ast: rhai::AST,
    original_dataset: Expr,
    /// Accumulates gml:id mappings across all processed files.
    /// Populated in `process`, consumed in `finish`.
    id_registry: IdRegistry,
    /// Top-level features buffered during `process`, emitted in `finish`.
    pending: Vec<TopLevelFeature>,
}

impl std::fmt::Debug for FeatureCityGml3Reader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeatureCityGml3Reader")
            .field("pending", &self.pending.len())
            .field("id_registry", &self.id_registry.len())
            .finish_non_exhaustive()
    }
}

impl Clone for FeatureCityGml3Reader {
    fn clone(&self) -> Self {
        Self {
            global_params: self.global_params.clone(),
            dataset_ast: self.dataset_ast.clone(),
            original_dataset: self.original_dataset.clone(),
            id_registry: IdRegistry::new(),
            pending: Vec::new(),
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

        let mut features = parser::parse(&bytes, &source_url, &mut self.id_registry)
            .map_err(|e| FeatureProcessorError::FileCityGml3Reader(format!("{e}")))?;

        self.pending.append(&mut features);
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        for tlf in &self.pending {
            // Resolve xlinks once; share the result across all passes.
            let resolved = xlink::resolve_xlinks(&tlf.node, &tlf.source_url, &self.id_registry);

            // Pass 1: build attribute Feature from the resolved node.
            let mut feature = parser::to_feature(tlf, &resolved);

            // Pass 2: extract geometries from the same resolved node.
            let raw_geoms = geometry::extract_geometries(&resolved);
            if !raw_geoms.is_empty() {
                *feature.geometry_mut() = Geometry::with_value(GeometryValue::CityGmlGeometry(
                    build_citygml_geometry(raw_geoms),
                ));
            }

            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                DEFAULT_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureCityGml3Reader"
    }
}

/// Assigns `pos` to each polygon-type geometry and builds neutral appearance arrays
/// so downstream consumers can index into them without panicking.
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

/// Zero-UV polygon matching the ring structure of a 3D polygon.
/// Each vertex maps to `(0.0, 0.0)` so texturing is a no-op.
fn neutral_uv_polygon(poly: &Polygon3D<f64>) -> Polygon2D<f64> {
    let ext = LineString2D::new(vec![[0.0f64, 0.0f64].into(); poly.exterior().0.len()]);
    let ints = poly
        .interiors()
        .iter()
        .map(|ring| LineString2D::new(vec![[0.0f64, 0.0f64].into(); ring.0.len()]))
        .collect();
    Polygon2D::new(ext, ints)
}
