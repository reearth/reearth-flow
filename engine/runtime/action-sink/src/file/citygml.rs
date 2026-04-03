pub mod converter;
pub mod writer;

use std::collections::HashMap;
use std::io::BufWriter;
use std::str::FromStr;
use std::sync::Arc;

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::geometry::GeometryValue;
use reearth_flow_types::lod::LodMask;
use reearth_flow_types::{CitygmlFeatureExt, Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;
use converter::{
    compute_envelope, convert_citygml_geometry, AppearanceBundle, BoundingEnvelope, CityObjectType,
};
use writer::CityGmlXmlWriter;

/// Write `features` as CityGML 2.0 to `output`, copying texture images alongside it.
///
/// This is the single canonical implementation shared by both the `CityGmlWriter` sink and
/// the `FeatureWriter` processor.
pub fn write_citygml_to_storage(
    output: &Uri,
    features: &[Feature],
    lod_mask: &LodMask,
    epsg_code: Option<u32>,
    pretty_print: bool,
    storage_resolver: &Arc<StorageResolver>,
) -> Result<(), SinkError> {
    if features.is_empty() {
        return Ok(());
    }

    let srs_name = epsg_code
        .or_else(|| {
            features
                .first()
                .and_then(|f| f.geometry.epsg)
                .map(|e| e as u32)
        })
        .map(|code| format!("http://www.opengis.net/def/crs/EPSG/0/{code}"))
        .unwrap_or_else(|| "http://www.opengis.net/def/crs/EPSG/0/4326".to_string());

    // Compute bounding envelope from all features.
    let mut envelope: Option<BoundingEnvelope> = None;
    for feature in features {
        if let GeometryValue::CityGmlGeometry(ref geom) = feature.geometry.value {
            if let Some(env) = compute_envelope(geom) {
                match &mut envelope {
                    Some(existing) => existing.merge(&env),
                    None => envelope = Some(env),
                }
            }
        }
    }

    // Compute appearance directory name from GML output stem (e.g. "foo_appearance")
    let gml_stem = output
        .path()
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let appearance_dir_name = format!("{}_appearance", gml_stem);

    // Parent directory portion of the output URI string
    let output_str = output.to_string();
    let parent_str = output_str
        .rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or(&output_str);

    // Copy texture images to the appearance dir and build a URI → relative-path remap.
    let mut uri_remap: HashMap<String, String> = HashMap::new();
    for feature in features {
        let GeometryValue::CityGmlGeometry(ref geom) = feature.geometry.value else {
            continue;
        };
        for texture in &geom.textures {
            let src_str = texture.uri.to_string();
            if uri_remap.contains_key(&src_str) {
                continue;
            }
            let filename = match texture.uri.path_segments().and_then(|mut s| s.next_back()) {
                Some(name) => name.to_string(),
                None => {
                    tracing::warn!(
                        "texture URI has no path segments, skipping copy: {}",
                        src_str
                    );
                    continue;
                }
            };
            let dst_str = format!("{}/{}/{}", parent_str, appearance_dir_name, filename);
            let src_uri = match Uri::from_str(&src_str) {
                Ok(u) => u,
                Err(e) => {
                    tracing::warn!("failed to parse texture source URI '{}': {}", src_str, e);
                    continue;
                }
            };
            let dst_uri = match Uri::from_str(&dst_str) {
                Ok(u) => u,
                Err(e) => {
                    tracing::warn!(
                        "failed to parse texture destination URI '{}': {}",
                        dst_str,
                        e
                    );
                    continue;
                }
            };
            let src_storage = match storage_resolver.resolve(&src_uri) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(
                        "failed to resolve storage for texture source '{}': {}",
                        src_str,
                        e
                    );
                    continue;
                }
            };
            let dst_storage = match storage_resolver.resolve(&dst_uri) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(
                        "failed to resolve storage for texture destination '{}': {}",
                        dst_str,
                        e
                    );
                    continue;
                }
            };
            let bytes = match src_storage.get_sync(src_uri.path().as_path()) {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!("failed to read texture file '{}': {}", src_str, e);
                    continue;
                }
            };
            if let Err(e) = dst_storage.put_sync(dst_uri.path().as_path(), bytes) {
                tracing::warn!("failed to write texture file '{}': {}", dst_str, e);
                continue;
            }
            uri_remap.insert(src_str, format!("{}/{}", appearance_dir_name, filename));
        }
    }

    // Build and write XML.
    let buffer_size = (features.len() * 4096).clamp(32 * 1024, 512 * 1024);
    let mut xml_buffer = Vec::with_capacity(buffer_size);
    {
        let buf_writer = BufWriter::with_capacity(buffer_size, &mut xml_buffer);
        let mut xml_writer = CityGmlXmlWriter::new(buf_writer, pretty_print, srs_name);
        xml_writer.set_uri_remap(uri_remap);

        xml_writer.write_header(envelope.as_ref())?;

        for feature in features {
            let GeometryValue::CityGmlGeometry(ref geom) = feature.geometry.value else {
                continue;
            };

            let feature_type_str = feature
                .feature_type()
                .unwrap_or_else(|| "gen:GenericCityObject".to_string());
            let feature_type = feature_type_str.as_str();
            let city_type = CityObjectType::from_feature_type(feature_type);

            let (geometries, appearance) = convert_citygml_geometry(geom, lod_mask);
            if geometries.is_empty() {
                continue;
            }

            let gml_id_str = feature
                .feature_id()
                .unwrap_or_else(|| feature.id.to_string());
            let appearance_opt: Option<&AppearanceBundle> = if appearance.has_content() {
                Some(&appearance)
            } else {
                None
            };
            xml_writer.write_city_object(
                city_type,
                &geometries,
                Some(gml_id_str.as_str()),
                appearance_opt,
            )?;
        }

        xml_writer.write_footer()?;
    }

    let storage = storage_resolver
        .resolve(output)
        .map_err(SinkError::citygml_writer)?;
    storage
        .put_sync(output.path().as_path(), Bytes::from(xml_buffer))
        .map_err(SinkError::citygml_writer)?;

    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct CityGmlWriterFactory;

impl SinkFactory for CityGmlWriterFactory {
    fn name(&self) -> &str {
        "CityGmlWriter"
    }

    fn description(&self) -> &str {
        "Writes features to CityGML 2.0 files"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CityGmlWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["File"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn prepare(&self) -> Result<(), BoxedError> {
        Ok(())
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params: CityGmlWriterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::CityGmlWriterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::CityGmlWriterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SinkError::CityGmlWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let lod_mask = build_lod_mask(&params.lod_filter);

        Ok(Box::new(CityGmlWriterSink {
            params,
            lod_mask,
            buffer: Vec::new(),
            envelope: None,
        }))
    }
}

fn build_lod_mask(lod_filter: &Option<Vec<u8>>) -> LodMask {
    match lod_filter {
        Some(lods) if !lods.is_empty() => {
            let mut mask = LodMask::default();
            for lod in lods {
                mask.add_lod(*lod);
            }
            mask
        }
        _ => LodMask::all(),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CityGmlWriterParam {
    /// Output file path expression
    pub output: Expr,
    /// LOD levels to include (e.g., [0, 1, 2]). If empty, includes all LODs.
    #[serde(default)]
    pub lod_filter: Option<Vec<u8>>,
    /// EPSG code for coordinate reference system
    #[serde(default)]
    pub epsg_code: Option<u32>,
    /// Whether to format output with indentation (default: true)
    #[serde(default = "default_pretty_print")]
    pub pretty_print: Option<bool>,
}

fn default_pretty_print() -> Option<bool> {
    Some(true)
}

#[derive(Debug, Clone)]
struct CityGmlWriterSink {
    params: CityGmlWriterParam,
    lod_mask: LodMask,
    buffer: Vec<Feature>,
    envelope: Option<BoundingEnvelope>,
}

impl Sink for CityGmlWriterSink {
    fn name(&self) -> &str {
        "CityGmlWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = ctx.feature;

        if let GeometryValue::CityGmlGeometry(ref geom) = feature.geometry.value {
            if let Some(env) = compute_envelope(geom) {
                match &mut self.envelope {
                    Some(existing) => existing.merge(&env),
                    None => self.envelope = Some(env),
                }
            }
        }

        self.buffer.push(feature);
        Ok(())
    }

    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = expr_engine.new_scope();

        let path = scope
            .eval::<String>(self.params.output.as_ref())
            .unwrap_or_else(|_| self.params.output.as_ref().to_string());
        let output_uri = Uri::from_str(&path)?;

        write_citygml_to_storage(
            &output_uri,
            &self.buffer,
            &self.lod_mask,
            self.params.epsg_code,
            self.params.pretty_print.unwrap_or(true),
            &ctx.storage_resolver,
        )?;

        Ok(())
    }
}
