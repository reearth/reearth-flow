use std::collections::HashMap;
use std::io::BufWriter;
use std::io::Cursor;
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::vec;

use nusamai_citygml::schema::{Schema, TypeDef};
use once_cell::sync::Lazy;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::Event;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::Context;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::geometry as geometry_types;
use reearth_flow_types::Expr;
use reearth_flow_types::{Attribute, Attributes, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::errors::SinkError;

use super::tileid::TileIdMethod;

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
            },
            join_handles: Vec::new(),
        };
        Ok(Box::new(sink))
    }
}

type JoinHandle = Arc<parking_lot::Mutex<Receiver<Result<(), SinkError>>>>;
type BufferValue = Vec<(Feature, String)>;

#[derive(Debug, Clone)]
pub struct MVTWriter {
    pub(super) global_params: Option<HashMap<String, serde_json::Value>>,
    pub(super) params: MVTWriterCompiledParam,
    pub(super) schema: Schema,
    /// (output, compress_output) -> Vec<(Feature, layer_name)>
    pub(super) buffer: HashMap<(Uri, Option<Uri>), BufferValue>,
    #[allow(clippy::type_complexity)]
    pub(super) join_handles: Vec<JoinHandle>,
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
        let context = ctx.as_context();
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
                // the flushing logic requires sorted features, or the output file will be corrupted
                if !self
                    .buffer
                    .contains_key(&(output.clone(), compress_output.clone()))
                {
                    let result = self.flush_buffer(context)?;
                    self.buffer.clear();
                    self.join_handles.extend(result);
                }
                // Apply attribute processing before buffering:
                // schema filtering/casting, skip_unexposed_attributes, colon_to_underscore
                let schema_key = feature
                    .get("__schema_definition")
                    .and_then(|v| v.as_string())
                    .or_else(|| feature.feature_type());
                let schema_attrs = schema_key
                    .as_ref()
                    .and_then(|ft| crate::schema::schema_attributes(ft, &self.schema));

                let mut new_attrs = Attributes::new();
                for (key, value) in feature.attributes.iter() {
                    if self.params.skip_unexposed_attributes && key.as_ref().starts_with("__") {
                        continue;
                    }
                    let value = if let Some(attrs) = schema_attrs {
                        let Some(attr_def) = attrs.get(key.as_ref()) else {
                            continue;
                        };
                        crate::schema::cast_attribute_value(value, &attr_def.type_ref)
                    } else {
                        value.clone()
                    };
                    let key = if self.params.colon_to_underscore {
                        Attribute::new(key.inner().replace(":", "_"))
                    } else {
                        key.clone()
                    };
                    new_attrs.insert(key, value);
                }
                let cleaned_feature = feature.with_attributes(new_attrs);

                let buffer = self.buffer.entry((output, compress_output)).or_default();
                buffer.push((cleaned_feature, layer_name));
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
        let result = self.flush_buffer(ctx.as_context())?;
        let mut join_handles = self.join_handles.clone();
        join_handles.extend(result);

        let timeout = std::time::Duration::from_secs(60 * 60);
        let mut errors = Vec::new();

        for (i, join) in join_handles.iter().enumerate() {
            match join.lock().recv_timeout(timeout) {
                Ok(_) => continue,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    errors.push(format!("Worker thread {i} timed out after {timeout:?}"));
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    ctx.event_hub
                        .warn_log(None, format!("Worker thread {i} disconnected unexpectedly"));
                }
            }
        }
        if !errors.is_empty() {
            return Err(SinkError::MvtWriter(format!(
                "Failed to complete all worker threads: {}",
                errors.join("; ")
            ))
            .into());
        }
        Ok(())
    }
}

impl MVTWriter {
    #[allow(clippy::type_complexity)]
    pub(crate) fn flush_buffer(&self, ctx: Context) -> crate::errors::Result<Vec<JoinHandle>> {
        let mut result = Vec::new();
        let mut features = HashMap::<(Uri, Option<Uri>), BufferValue>::new();
        for ((output, compress_output), buffer) in &self.buffer {
            features
                .entry((output.clone(), compress_output.clone()))
                .or_default()
                .extend(buffer.clone());
        }
        for ((output, compress_output), buffer) in &features {
            let res = self.write(ctx.clone(), buffer.clone(), output, compress_output)?;
            result.extend(res);
        }
        Ok(result)
    }

    pub fn write(
        &self,
        ctx: Context,
        upstream: BufferValue,
        output: &Uri,
        compress_output: &Option<Uri>,
    ) -> crate::errors::Result<Vec<JoinHandle>> {
        let tile_id_conv = TileIdMethod::Hilbert;
        let name = self.name().to_string();
        let (sender_sliced, receiver_sliced) = std::sync::mpsc::sync_channel(2000);
        let (sender_sorted, receiver_sorted) = std::sync::mpsc::sync_channel(2000);
        let min_zoom = self.params.min_zoom;
        let max_zoom = self.params.max_zoom;
        let gctx = ctx.clone();
        let out = output.clone();

        let mut result = Vec::new();

        let (tx, rx) = std::sync::mpsc::channel();
        result.push(Arc::new(parking_lot::Mutex::new(rx)));
        std::thread::spawn(move || {
            let result = super::pipeline::geometry_slicing_stage(
                gctx.clone(),
                &upstream,
                tile_id_conv,
                sender_sliced,
                &out,
                min_zoom,
                max_zoom,
            );
            if let Err(err) = &result {
                gctx.event_hub.error_log(
                    None,
                    format!("Failed to geometry_slicing_stage with error =  {err:?}"),
                );
                gctx.event_hub
                    .send(Event::SinkFinishFailed { name: name.clone() });
            }
            tx.send(result).unwrap();
        });
        let name = self.name().to_string();
        let gctx = ctx.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        result.push(Arc::new(parking_lot::Mutex::new(rx)));
        std::thread::spawn(move || {
            let result = super::pipeline::feature_sorting_stage(receiver_sliced, sender_sorted);
            if let Err(err) = &result {
                ctx.event_hub.error_log(
                    None,
                    format!("Failed to feature_sorting_stage with error =  {err:?}"),
                );
                ctx.event_hub
                    .send(Event::SinkFinishFailed { name: name.clone() });
            }
            tx.send(result).unwrap();
        });
        let out = output.clone();
        let gctx = gctx.clone();
        let name = self.name().to_string();
        let compress_output = compress_output.clone();
        let extent = self.params.extent;
        let (tx, rx) = std::sync::mpsc::channel();
        result.push(Arc::new(parking_lot::Mutex::new(rx)));
        std::thread::spawn(move || {
            let pool = rayon::ThreadPoolBuilder::new()
                .use_current_thread()
                .build()
                .unwrap();
            pool.install(|| {
                let result = super::pipeline::tile_writing_stage(
                    gctx.clone(),
                    &out,
                    receiver_sorted,
                    tile_id_conv,
                    extent,
                );
                if let Err(err) = &result {
                    gctx.event_hub.error_log(
                        None,
                        format!("Failed to tile_writing_stage with error =  {err:?}"),
                    );
                    gctx.event_hub
                        .send(Event::SinkFinishFailed { name: name.clone() });
                }

                if let Some(compress_output) = compress_output {
                    if let Ok(storage) = gctx.storage_resolver.resolve(&compress_output) {
                        let buffer = Vec::new();
                        let mut cursor = Cursor::new(buffer);
                        let writer = BufWriter::new(&mut cursor);
                        let zip_result =
                            reearth_flow_common::zip::write(writer, out.path().as_path()).map_err(
                                |e| crate::errors::SinkError::cesium3dtiles_writer(e.to_string()),
                            );
                        match zip_result {
                            Ok(_) => {
                                match storage
                                    .put_sync(
                                        compress_output.path().as_path(),
                                        bytes::Bytes::from(cursor.into_inner()),
                                    )
                                    .map_err(crate::errors::SinkError::cesium3dtiles_writer)
                                {
                                    Ok(_) => match std::fs::remove_dir_all(out.path().as_path()) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            gctx.event_hub.error_log(
                                                None,
                                                format!(
                                                    "Failed to remove directory with error = {e:?}"
                                                ),
                                            );
                                        }
                                    },
                                    Err(e) => {
                                        gctx.event_hub.error_log(
                                            None,
                                            format!("Failed to write zip file with error = {e:?}"),
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                gctx.event_hub.error_log(
                                    None,
                                    format!("Failed to write zip file with error = {e:?}"),
                                );
                            }
                        }
                    }
                }
                tx.send(result).unwrap();
            })
        });
        Ok(result)
    }
}
