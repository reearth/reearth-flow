use std::{
    collections::HashMap,
    io::{BufWriter, Cursor},
    str::FromStr,
    sync::Arc,
    time, vec,
};

use once_cell::sync::Lazy;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::event::{Event, EventHub};
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_runtime::{errors::BoxedError, executor_operation::Context};
use reearth_flow_types::geometry as geometry_types;
use reearth_flow_types::Expr;
use reearth_flow_types::Feature;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::errors::SinkError;
use crate::file::mvt::tileid::TileIdMethod;

static SCHEMA_PORT: Lazy<Port> = Lazy::new(|| Port::new("schema"));

#[derive(Debug, Clone, Default)]
pub struct Cesium3DTilesSinkFactory;

impl SinkFactory for Cesium3DTilesSinkFactory {
    fn name(&self) -> &str {
        "Cesium3DTilesWriter"
    }

    fn description(&self) -> &str {
        "Export Features as Cesium 3D Tiles for Web Visualization"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(Cesium3DTilesWriterParam))
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
        let params: Cesium3DTilesWriterParam = if let Some(with) = with.clone() {
            let value: serde_json::Value = serde_json::to_value(with).map_err(|e| {
                SinkError::Cesium3DTilesWriterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::Cesium3DTilesWriterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SinkError::Cesium3DTilesWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let expr_output = &params.output;
        let output = expr_engine
            .compile(expr_output.as_ref())
            .map_err(|e| SinkError::Cesium3DTilesWriterFactory(format!("{e:?}")))?;
        let compress_output = if let Some(compress_output) = &params.compress_output {
            let compress_output = expr_engine
                .compile(compress_output.as_ref())
                .map_err(|e| SinkError::Cesium3DTilesWriterFactory(format!("{e:?}")))?;
            Some(compress_output)
        } else {
            None
        };

        let sink = Cesium3DTilesWriter {
            global_params: with,
            buffer: HashMap::new(),
            schema: Default::default(),
            params: Cesium3DTilesWriterCompiledParam {
                output,
                min_zoom: params.min_zoom,
                max_zoom: params.max_zoom,
                attach_texture: params.attach_texture,
                compress_output,
                draco_compression_enabled: params.draco_compression_enabled,
            },
        };
        Ok(Box::new(sink))
    }
}

type BufferKey = (Uri, String, Option<Uri>); // (output, feature_type, compress_output)

#[derive(Debug, Clone)]
pub struct Cesium3DTilesWriter {
    pub(super) global_params: Option<HashMap<String, serde_json::Value>>,
    pub(super) buffer: HashMap<BufferKey, Vec<Feature>>,
    pub(super) schema: nusamai_citygml::schema::Schema,
    pub(super) params: Cesium3DTilesWriterCompiledParam,
}

/// # Cesium3DTilesWriter Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Cesium3DTilesWriterParam {
    /// # Output Path
    /// Directory path where the 3D tiles will be written
    pub(super) output: Expr,
    /// # Minimum Zoom Level
    /// Minimum zoom level for tile generation (0-24)
    pub(super) min_zoom: u8,
    /// # Maximum Zoom Level
    /// Maximum zoom level for tile generation (0-24)
    pub(super) max_zoom: u8,
    /// # Attach Textures
    /// Whether to include texture information in the generated tiles
    pub(super) attach_texture: Option<bool>,
    /// # Compressed Output Path
    /// Optional path for compressed archive output
    pub(super) compress_output: Option<Expr>,
    /// # Draco Compression
    /// Enable Draco compression for mesh geometry (defaults to true)
    pub(super) draco_compression_enabled: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct Cesium3DTilesWriterCompiledParam {
    pub(super) output: rhai::AST,
    pub(super) min_zoom: u8,
    pub(super) max_zoom: u8,
    pub(super) attach_texture: Option<bool>,
    pub(super) compress_output: Option<rhai::AST>,
    pub(super) draco_compression_enabled: Option<bool>, // Draco compression. Defaults to true.
}

impl Sink for Cesium3DTilesWriter {
    fn name(&self) -> &str {
        "Cesium3DTilesWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        match &ctx.port {
            port if *port == *DEFAULT_PORT => self.process_default(&ctx)?,
            port if *port == SCHEMA_PORT.clone() => self.process_schema(&ctx)?,
            port => {
                return Err(
                    SinkError::Cesium3DTilesWriter(format!("Unknown port with: {port:?}")).into(),
                )
            }
        }
        Ok(())
    }

    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        self.flush_buffer(ctx.as_context())?;
        Ok(())
    }
}

impl Cesium3DTilesWriter {
    fn process_default(&mut self, ctx: &ExecutorContext) -> crate::errors::Result<()> {
        let Some(feature_type) = &ctx.feature.feature_type() else {
            return Err(SinkError::Cesium3DTilesWriter(
                "Failed to get feature type".to_string(),
            ));
        };
        let geometry = &ctx.feature.geometry;
        if geometry.is_empty() {
            return Err(SinkError::Cesium3DTilesWriter(
                "Unsupported input".to_string(),
            ));
        };
        let geometry_value = &geometry.value;
        if !matches!(
            geometry_value,
            geometry_types::GeometryValue::CityGmlGeometry(_)
        ) {
            return Err(SinkError::Cesium3DTilesWriter(
                "Unsupported input".to_string(),
            ));
        }
        let feature = &ctx.feature;
        let output = self.params.output.clone();
        let scope = feature.new_scope(ctx.expr_engine.clone(), &self.global_params);
        let path = scope
            .eval_ast::<String>(&output)
            .map_err(|e| SinkError::Cesium3DTilesWriter(format!("{e:?}")))?;
        let output = Uri::from_str(path.as_str()).map_err(SinkError::cesium3dtiles_writer)?;
        let compress_output = if let Some(compress_output) = &self.params.compress_output {
            let compress_output = compress_output.clone();
            let path = scope
                .eval_ast::<String>(&compress_output)
                .map_err(|e| SinkError::Cesium3DTilesWriter(format!("{e:?}")))?;
            Some(Uri::from_str(path.as_str()).map_err(SinkError::cesium3dtiles_writer)?)
        } else {
            None
        };
        let buffer = self
            .buffer
            .entry((output, feature_type.clone(), compress_output.clone()))
            .or_default();
        buffer.push(feature.clone());
        Ok(())
    }

    fn process_schema(&mut self, ctx: &ExecutorContext) -> crate::errors::Result<()> {
        let feature = &ctx.feature;
        let typedef: nusamai_citygml::schema::TypeDef = feature.into();
        let Some(feature_type) = &feature.feature_type() else {
            return Err(SinkError::Cesium3DTilesWriter(
                "Failed to get feature type".to_string(),
            ));
        };
        self.schema.types.insert(feature_type.clone(), typedef);
        Ok(())
    }

    pub(crate) fn flush_buffer(&self, ctx: Context) -> crate::errors::Result<()> {
        let mut features = HashMap::<(Uri, Option<Uri>), Vec<(String, Vec<Feature>)>>::new();
        for ((output, feature_type, compress_output), buffer) in &self.buffer {
            features
                .entry((output.clone(), compress_output.clone()))
                .or_default()
                .push((feature_type.clone(), buffer.clone()));
        }
        for ((output, compress_output), buffer) in &features {
            self.write(ctx.clone(), buffer, output, compress_output)?;
        }
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    pub(crate) fn write(
        &self,
        ctx: Context,
        upstream: &Vec<(String, Vec<Feature>)>,
        output: &Uri,
        compress_output: &Option<Uri>,
    ) -> crate::errors::Result<()> {
        let tile_id_conv = TileIdMethod::Hilbert;
        let attach_texture = self.params.attach_texture.unwrap_or(false);
        let mut features = Vec::new();
        let mut schema: nusamai_citygml::schema::Schema = self.schema.clone();
        for (feature_type, upstream) in upstream {
            let Some(feature) = upstream.first() else {
                continue;
            };
            if !schema.types.contains_key(feature_type) {
                let typedef: nusamai_citygml::schema::TypeDef = feature.into();
                schema.types.insert(feature_type.clone(), typedef);
            }
            features.extend(upstream.clone().into_iter());
        }

        let (sender_sliced, receiver_sliced) = std::sync::mpsc::sync_channel(2000);
        let (sender_sorted, receiver_sorted) = std::sync::mpsc::sync_channel(2000);
        let min_zoom = self.params.min_zoom;
        let max_zoom = self.params.max_zoom;

        std::thread::scope(|s| {
            {
                let ctx = ctx.clone();
                let schema = schema.clone();
                s.spawn(move || {
                    let now = time::Instant::now();
                    let result = super::pipeline::geometry_slicing_stage(
                        &features,
                        &schema,
                        tile_id_conv,
                        sender_sliced,
                        min_zoom,
                        max_zoom,
                        attach_texture,
                    );
                    if let Err(e) = &result {
                        ctx.event_hub.error_log(
                            None,
                            format!("Failed to geometry_slicing_stage with error = {e:?}"),
                        );
                        ctx.event_hub.send(Event::SinkFinishFailed {
                            name: "geometry_slicing_stage".to_string(),
                        });
                    }
                    ctx.event_hub.info_log(
                        None,
                        format!(
                            "Finish geometry_slicing_stage. feature length = {}, elapsed = {:?}, output = {}",
                            features.len(),
                            now.elapsed(),
                            output
                        ),
                    );
                });
            }
            {
                let ctx = ctx.clone();
                s.spawn(move || {
                    let now = time::Instant::now();
                    let result =
                        super::pipeline::feature_sorting_stage(receiver_sliced, sender_sorted);
                    if let Err(e) = &result {
                        ctx.event_hub.error_log(
                            None,
                            format!("Failed to feature_sorting_stage with error = {e:?}"),
                        );
                        ctx.event_hub.send(Event::SinkFinishFailed {
                            name: "feature_sorting_stage".to_string(),
                        });
                    }
                    ctx.event_hub.info_log(
                        None,
                        format!(
                            "Finish feature_sorting_stage. elapsed = {:?}, output = {}",
                            now.elapsed(),
                            output
                        ),
                    );
                });
            }
            {
                let ctx = ctx.clone();
                let schema = schema.clone();
                s.spawn(move || {
                    let pool = rayon::ThreadPoolBuilder::new()
                        .use_current_thread()
                        .build()
                        .unwrap();
                    pool.install(|| {
                        let now = time::Instant::now();
                        let result = super::pipeline::tile_writing_stage(
                            ctx.clone(),
                            output.clone(),
                            receiver_sorted,
                            tile_id_conv,
                            &schema,
                            None,
                            self.params.draco_compression_enabled.unwrap_or(true), // On by default
                        );
                        if let Err(e) = &result {
                            let ctx = ctx.clone();
                            ctx.event_hub.error_log(
                                None,
                                format!("Failed to tile_writing_stage with error = {e:?}"),
                            );
                            ctx.event_hub.send(Event::SinkFinishFailed {
                                name: "tile_writing_stage".to_string(),
                            });
                        }
                        ctx.event_hub.info_log(
                            None,
                            format!(
                                "Finish tile_writing_stage. elapsed = {:?}, output = {}",
                                now.elapsed(),
                                output
                            ),
                        );

                        if let Some(compress_output) = compress_output {
                            if let Ok(storage) = ctx.storage_resolver.resolve(compress_output) {
                                let now = time::Instant::now();
                                let buffer = Vec::new();
                                let mut cursor = Cursor::new(buffer);
                                let writer = BufWriter::new(&mut cursor);
                                let zip_result = reearth_flow_common::zip::write(
                                    writer,
                                    output.path().as_path(),
                                )
                                .map_err(|e| {
                                    crate::errors::SinkError::cesium3dtiles_writer(e.to_string())
                                });
                                match zip_result {
                                    Ok(_) => {
                                        match storage
                                            .put_sync(
                                                compress_output.path().as_path(),
                                                bytes::Bytes::from(cursor.into_inner()),
                                            )
                                            .map_err(crate::errors::SinkError::cesium3dtiles_writer)
                                        {
                                            Ok(_) => {
                                                match std::fs::remove_dir_all(
                                                    output.path().as_path(),
                                                ) {
                                                    Ok(_) => {}
                                                    Err(e) => {
                                                        ctx.event_hub.error_log(
                                                            None,
                                                            format!(
                                                    "Failed to remove directory with error = {e:?}"
                                                ),
                                                        );
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                ctx.event_hub.error_log(
                                                    None,
                                                    format!(
                                                    "Failed to write zip file with error = {e:?}"
                                                ),
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        ctx.event_hub.error_log(
                                            None,
                                            format!("Failed to write zip file with error = {e:?}"),
                                        );
                                    }
                                }
                                ctx.event_hub.info_log(
                                    None,
                                    format!(
                                        "Finish write zip file. elapsed = {:?}, output = {}",
                                        now.elapsed(),
                                        output
                                    ),
                                );
                            }
                        }
                    });
                });
            }
        });
        Ok(())
    }
}
