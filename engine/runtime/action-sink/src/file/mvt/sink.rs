use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use nusamai_citygml::schema::{Schema, TypeDef};
use once_cell::sync::Lazy;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::Context;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::geometry as geometry_types;
use reearth_flow_types::Expr;
use reearth_flow_types::{Attribute, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::errors::SinkError;

static SCHEMA_PORT: Lazy<Port> = Lazy::new(|| Port::new("schema"));

#[derive(Debug, Clone, Default)]
pub struct MVTSinkFactory;

impl SinkFactory for MVTSinkFactory {
    fn name(&self) -> &str {
        "MVTWriter"
    }

    fn description(&self) -> &str {
        "Writes vector features to Mapbox Vector Tiles (MVT) format with TileJSON 3.0.0 metadata."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(MVTWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["File"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), SCHEMA_PORT.clone()]
    }

    fn prepare(&self) -> Result<(), BoxedError> {
        Ok(())
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, JsonValue>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params: MVTWriterParam = if let Some(with) = with.clone() {
            let value: JsonValue = serde_json::to_value(with).map_err(|e| {
                SinkError::MvtWriterFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::MvtWriterFactory(format!("Failed to deserialize `with` parameter: {e}"))
            })?
        } else {
            return Err(SinkError::MvtWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let expr_output = &params.output;
        let output = expr_engine
            .compile(expr_output.as_ref())
            .map_err(|e| SinkError::MvtWriterFactory(format!("{e:?}")))?;
        let expr_layer_name = &params.layer_name;
        let layer_name = expr_engine
            .compile(expr_layer_name.as_ref())
            .map_err(|e| SinkError::MvtWriterFactory(format!("{e:?}")))?;
        let compress_output = if let Some(compress_output) = &params.compress_output {
            let compress_output = expr_engine
                .compile(compress_output.as_ref())
                .map_err(|e| SinkError::MvtWriterFactory(format!("{e:?}")))?;
            Some(compress_output)
        } else {
            None
        };

        let sink = MVTWriter {
            global_params: with,
            buffer: HashMap::new(),
            schema: Default::default(),
            params: MVTWriterCompiledParam {
                output,
                layer_name,
                min_zoom: params.min_zoom,
                max_zoom: params.max_zoom,
                compress_output,
                skip_unexposed_attributes: params.skip_unexposed_attributes.unwrap_or(false),
                colon_to_underscore: params.colon_to_underscore.unwrap_or(false),
                extent: params.extent.unwrap_or(4096) as i32,
                max_tile_bytes: params.max_tile_bytes,
            },
        };
        Ok(Box::new(sink))
    }
}

type BufferValue = Vec<(Feature, String)>;

#[derive(Debug, Clone)]
pub struct MVTWriter {
    pub(super) global_params: Option<HashMap<String, serde_json::Value>>,
    pub(super) params: MVTWriterCompiledParam,
    pub(super) schema: Schema,
    /// (output, compress_output) -> Vec<(Feature, layer_name)>
    pub(super) buffer: HashMap<(Uri, Option<Uri>), BufferValue>,
}

/// # MVTWriter Parameters
///
/// Configuration for writing features to Mapbox Vector Tiles (MVT) format.
/// Generates tiles at /{z}/{x}/{y}.mvt and tilejson.json where the parent directory is treated as HTTP root (tileJSON requires absolute URLs).
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MVTWriterParam {
    /// # Output
    /// Output directory path or expression for the generated MVT tiles
    pub(super) output: Expr,
    /// # Layer Name
    /// Name of the layer within the MVT tiles
    pub(super) layer_name: Expr,
    /// # Minimum Zoom
    /// Minimum zoom level to generate tiles for
    pub(super) min_zoom: u8,
    /// # Maximum Zoom
    /// Maximum zoom level to generate tiles for
    pub(super) max_zoom: u8,
    /// # Compress Output
    /// Optional expression to determine whether to compress the output tiles
    pub(super) compress_output: Option<Expr>,
    /// # Skip Unexposed Attributes
    /// Skip attributes with double underscore prefix
    pub(super) skip_unexposed_attributes: Option<bool>,
    /// # Colon to Underscore
    /// Replace colons in attribute keys (e.g., from XML Namespaces) with underscores
    pub(super) colon_to_underscore: Option<bool>,
    /// # Extent
    /// MVT tile resolution. Default is 4096.
    pub(super) extent: Option<u32>,
    /// # Maximum Tile Size
    /// Maximum size of each tile in bytes, measured before compression, at every zoom. A larger
    /// tile is written with a coarser geometric error until it fits: a coarser grid, simpler
    /// lines and rings, and features smaller than the error left out. Defaults to 10,000,000.
    #[serde(default = "default_max_tile_bytes")]
    pub(super) max_tile_bytes: u64,
}

fn default_max_tile_bytes() -> u64 {
    10_000_000
}

#[derive(Debug, Clone)]
pub struct MVTWriterCompiledParam {
    pub(super) output: rhai::AST,
    pub(super) layer_name: rhai::AST,
    pub(super) min_zoom: u8,
    pub(super) max_zoom: u8,
    pub(super) compress_output: Option<rhai::AST>,
    pub(super) skip_unexposed_attributes: bool,
    pub(super) colon_to_underscore: bool,
    pub(super) extent: i32,
    pub(super) max_tile_bytes: u64,
}

impl Sink for MVTWriter {
    fn name(&self) -> &str {
        "MVTWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        if ctx.port == *SCHEMA_PORT {
            let feature = &ctx.feature;
            if let Some(feature_type) = feature.feature_type() {
                let typedef: TypeDef = feature.into();
                self.schema.types.insert(feature_type, typedef);
            }
            return Ok(());
        }
        let geometry = &ctx.feature.geometry;
        if geometry.is_empty() {
            return Err(Box::new(SinkError::MvtWriter(
                "Unsupported input".to_string(),
            )));
        };

        let feature = &ctx.feature;
        match feature.geometry.value {
            geometry_types::GeometryValue::CityGmlGeometry(_)
            | geometry_types::GeometryValue::FlowGeometry2D(_) => {
                let output = self.params.output.clone();
                let scope = feature.new_scope(ctx.expr_engine.clone(), &self.global_params);
                let path = scope
                    .eval_ast::<String>(&output)
                    .map_err(|e| SinkError::MvtWriter(format!("{e:?}")))?;
                let compress_output = if let Some(compress_output) = &self.params.compress_output {
                    let compress_output = compress_output.clone();
                    let path = scope
                        .eval_ast::<String>(&compress_output)
                        .map_err(|e| SinkError::MvtWriter(format!("{e:?}")))?;
                    Some(Uri::from_str(path.as_str())?)
                } else {
                    None
                };
                let output = Uri::from_str(path.as_str())?;
                let layer_name = scope
                    .eval_ast::<String>(&self.params.layer_name)
                    .map_err(|e| SinkError::MvtWriter(format!("{e:?}")))?;
                // features arrive grouped by output, so a new output means the
                // previous one is complete and can be written out
                if !self
                    .buffer
                    .contains_key(&(output.clone(), compress_output.clone()))
                {
                    self.flush_buffer(ctx.as_context())?;
                    self.buffer.clear();
                }
                let buffer = self.buffer.entry((output, compress_output)).or_default();
                buffer.push((feature.clone(), layer_name));
            }
            _ => {
                return Err(Box::new(SinkError::MvtWriter(
                    "Unsupported input".to_string(),
                )));
            }
        }

        Ok(())
    }

    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        self.flush_buffer(ctx.as_context())?;
        Ok(())
    }
}

impl MVTWriter {
    pub(crate) fn flush_buffer(&self, ctx: Context) -> crate::errors::Result<()> {
        for ((output, compress_output), buffer) in &self.buffer {
            let filtered_buffer: BufferValue = buffer
                .iter()
                .map(|(feature, layer_name)| {
                    let mut new_attrs =
                        crate::schema::filter_and_cast_attributes(feature, &self.schema);
                    if self.params.skip_unexposed_attributes {
                        new_attrs.retain(|k, _| !k.as_ref().starts_with("__"));
                    }
                    if self.params.colon_to_underscore {
                        new_attrs = new_attrs
                            .into_iter()
                            .map(|(k, v)| (Attribute::new(k.inner().replace(":", "_")), v))
                            .collect();
                    }
                    (feature.with_attributes(new_attrs), layer_name.clone())
                })
                .collect();
            super::next::write_tileset(
                &ctx,
                &filtered_buffer,
                output,
                compress_output.as_ref(),
                self.params.min_zoom,
                self.params.max_zoom,
                self.params.extent,
                self.params.max_tile_bytes,
            )?;
        }
        Ok(())
    }
}
