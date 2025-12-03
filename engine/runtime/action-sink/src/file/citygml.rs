mod converter;
mod writer;

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
use reearth_flow_types::geometry::GeometryValue;
use reearth_flow_types::lod::LodMask;
use reearth_flow_types::{Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;
use converter::{compute_envelope, convert_citygml_geometry, BoundingEnvelope, CityObjectType};
use writer::CityGmlXmlWriter;

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
        if self.buffer.is_empty() {
            return Ok(());
        }

        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = expr_engine.new_scope();

        let path = scope
            .eval::<String>(self.params.output.as_ref())
            .unwrap_or_else(|_| self.params.output.as_ref().to_string());
        let output_uri = Uri::from_str(&path)?;

        let srs_name = self
            .params
            .epsg_code
            .or_else(|| {
                self.buffer
                    .first()
                    .and_then(|f| f.geometry.epsg)
                    .map(|e| e as u32)
            })
            .map(|code| format!("http://www.opengis.net/def/crs/EPSG/0/{}", code))
            .unwrap_or_else(|| "http://www.opengis.net/def/crs/EPSG/0/4326".to_string());

        let pretty = self.params.pretty_print.unwrap_or(true);

        // Dynamic buffer sizing based on feature count (32KB min, 512KB max)
        let buffer_size = (self.buffer.len() * 4096).clamp(32 * 1024, 512 * 1024);
        let mut xml_buffer = Vec::with_capacity(buffer_size);
        {
            let buf_writer = BufWriter::with_capacity(buffer_size, &mut xml_buffer);
            let mut xml_writer = CityGmlXmlWriter::new(buf_writer, pretty, srs_name);

            xml_writer.write_header(self.envelope.as_ref())?;

            for feature in &self.buffer {
                let GeometryValue::CityGmlGeometry(ref geom) = feature.geometry.value else {
                    continue;
                };

                let feature_type = feature
                    .metadata
                    .feature_type
                    .as_deref()
                    .unwrap_or("gen:GenericCityObject");
                let city_type = CityObjectType::from_feature_type(feature_type);

                let geometries = convert_citygml_geometry(geom, &self.lod_mask);
                if geometries.is_empty() {
                    continue;
                }

                xml_writer.write_city_object(city_type, &geometries)?;
            }

            xml_writer.write_footer()?;
        }

        let storage = storage_resolver
            .resolve(&output_uri)
            .map_err(SinkError::citygml_writer)?;
        storage
            .put_sync(output_uri.path().as_path(), Bytes::from(xml_buffer))
            .map_err(SinkError::citygml_writer)?;

        Ok(())
    }
}
