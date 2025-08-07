use std::collections::HashMap;
use std::io::BufWriter;
use std::io::Cursor;
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::vec;

use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::Event;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::Context;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::geometry as geometry_types;
use reearth_flow_types::Expr;
use reearth_flow_types::Feature;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::errors::SinkError;

use super::tileid::TileIdMethod;

#[derive(Debug, Clone, Default)]
pub struct MVTSinkFactory;

impl SinkFactory for MVTSinkFactory {
    fn name(&self) -> &str {
        "MVTWriter"
    }

    fn description(&self) -> &str {
        "Writes vector features to Mapbox Vector Tiles (MVT) format for web mapping"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(MVTWriterParam))
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
            params: MVTWriterCompiledParam {
                output,
                layer_name,
                min_zoom: params.min_zoom,
                max_zoom: params.max_zoom,
                compress_output,
            },
            join_handles: Vec::new(),
        };
        Ok(Box::new(sink))
    }
}

type BufferKey = (Uri, String, Option<Uri>); // (output, feature_type, compress_output)
type JoinHandle = Arc<parking_lot::Mutex<Receiver<Result<(), SinkError>>>>;

#[derive(Debug, Clone)]
pub struct MVTWriter {
    pub(super) global_params: Option<HashMap<String, serde_json::Value>>,
    pub(super) params: MVTWriterCompiledParam,
    pub(super) buffer: HashMap<BufferKey, Vec<Feature>>,
    #[allow(clippy::type_complexity)]
    pub(super) join_handles: Vec<JoinHandle>,
}

/// # MVTWriter Parameters
/// 
/// Configuration for writing features to Mapbox Vector Tiles (MVT) format.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MVTWriterParam {
    /// Output directory path or expression for the generated MVT tiles
    pub(super) output: Expr,
    /// Name of the layer within the MVT tiles
    pub(super) layer_name: Expr,
    /// Minimum zoom level to generate tiles for
    pub(super) min_zoom: u8,
    /// Maximum zoom level to generate tiles for
    pub(super) max_zoom: u8,
    /// Optional expression to determine whether to compress the output tiles
    pub(super) compress_output: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct MVTWriterCompiledParam {
    pub(super) output: rhai::AST,
    pub(super) layer_name: rhai::AST,
    pub(super) min_zoom: u8,
    pub(super) max_zoom: u8,
    pub(super) compress_output: Option<rhai::AST>,
}

impl Sink for MVTWriter {
    fn name(&self) -> &str {
        "MVTWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let geometry = &ctx.feature.geometry;
        if geometry.is_empty() {
            return Err(Box::new(SinkError::MvtWriter(
                "Unsupported input".to_string(),
            )));
        };

        let feature = &ctx.feature;
        let context = ctx.as_context();
        match feature.geometry.value {
            geometry_types::GeometryValue::CityGmlGeometry(_) => {
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
                if !self.buffer.contains_key(&(
                    output.clone(),
                    layer_name.clone(),
                    compress_output.clone(),
                )) {
                    let result = self.flush_buffer(context)?;
                    self.buffer.clear();
                    self.join_handles.extend(result);
                }
                let buffer = self
                    .buffer
                    .entry((output, layer_name, compress_output))
                    .or_default();
                buffer.push(feature.clone());
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
        let mut features = HashMap::<(Uri, Option<Uri>, String), Vec<Feature>>::new();
        for ((output, layer_name, compress_output), buffer) in &self.buffer {
            features
                .entry((output.clone(), compress_output.clone(), layer_name.clone()))
                .or_default()
                .extend(buffer.clone());
        }
        for ((output, compress_output, layer_name), buffer) in &features {
            let res = self.write(
                ctx.clone(),
                buffer.clone(),
                output,
                layer_name,
                compress_output,
            )?;
            result.extend(res);
        }
        Ok(result)
    }

    pub fn write(
        &self,
        ctx: Context,
        upstream: Vec<Feature>,
        output: &Uri,
        layer_name: &str,
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
        let layer_name = layer_name.to_string();

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
                &layer_name,
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
