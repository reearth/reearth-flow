use std::{
    collections::HashMap,
    io::{BufWriter, Cursor},
    sync::Arc,
    time, vec,
};

use nusamai_citygml::schema::{Schema, TypeDef};
use once_cell::sync::Lazy;
use reearth_flow_runtime::event::{Event, EventHub};
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, FEATURES_PORT};
use reearth_flow_runtime::{errors::BoxedError, executor_operation::Context};
use reearth_flow_types::geometry as geometry_types;
use reearth_flow_types::{Code, CompiledCode, Feature};
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
        &["Output"]
    }

    fn tags(&self) -> &[&'static str] {
        &["3d-tiles", "3d"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), SCHEMA_PORT.clone()]
    }

    fn prepare(&self) -> Result<(), BoxedError> {
        Ok(())
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, JsonValue>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params: Cesium3DTilesWriterParam = if let Some(with) = with {
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

        let output = params
            .output
            .compile()
            .map_err(|e| SinkError::Cesium3DTilesWriterFactory(format!("{e:?}")))?;
        let compress_output = params
            .compress_output
            .as_ref()
            .map(|c| {
                c.compile()
                    .map_err(|e| SinkError::Cesium3DTilesWriterFactory(format!("{e:?}")))
            })
            .transpose()?;

        let sink = Cesium3DTilesWriter {
            buffer: HashMap::new(),
            schema: Default::default(),
            params: Cesium3DTilesWriterCompiledParam {
                output,
                min_zoom: params.min_zoom,
                max_zoom: params.max_zoom,
                attach_texture: params.attach_texture,
                compress_output,
                draco_compression: params.draco_compression,
                skip_unexposed_attributes: params.skip_unexposed_attributes.unwrap_or(false),
                schema_key: params.schema_key,
            },
        };
        Ok(Box::new(sink))
    }
}

type BufferKey = (String, Option<String>, Option<String>); // (output_rel_path, filename, compress_output_rel_path)

#[derive(Debug, Clone)]
pub struct Cesium3DTilesWriter {
    pub(super) buffer: HashMap<BufferKey, Vec<Feature>>,
    pub(super) schema: Schema,
    pub(super) params: Cesium3DTilesWriterCompiledParam,
}

/// # Cesium3DTilesWriter Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Cesium3DTilesWriterParam {
    /// # Output Path
    /// Directory path where the 3D tiles will be written
    pub(super) output: Code,
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
    pub(super) compress_output: Option<Code>,
    /// # Draco Compression
    /// Use draco compression. Defaults to true.
    pub(super) draco_compression: Option<bool>,
    /// # Skip unexposed Attributes
    /// Skip attributes with double underscore prefix
    pub(super) skip_unexposed_attributes: Option<bool>,
    /// # Schema Key
    /// Attribute key whose value identifies the schema type and determines the output
    /// filename: all features sharing the same value are written to the same file.
    /// This attribute is excluded from output.
    pub(super) schema_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Cesium3DTilesWriterCompiledParam {
    pub(super) output: CompiledCode,
    pub(super) min_zoom: u8,
    pub(super) max_zoom: u8,
    pub(super) attach_texture: Option<bool>,
    pub(super) compress_output: Option<CompiledCode>,
    pub(super) draco_compression: Option<bool>,
    pub(super) skip_unexposed_attributes: bool,
    pub(super) schema_key: Option<String>,
}

impl Sink for Cesium3DTilesWriter {
    fn name(&self) -> &str {
        "Cesium3DTilesWriter"
    }

    #[cfg(not(feature = "new-geometry"))]
    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        match &ctx.port {
            port if *port == *FEATURES_PORT => self.process_default(&ctx)?,
            port if *port == SCHEMA_PORT.clone() => self.process_schema(&ctx),
            port => {
                return Err(
                    SinkError::Cesium3DTilesWriter(format!("Unknown port with: {port:?}")).into(),
                )
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        self.flush_buffer(ctx.as_context())?;
        Ok(())
    }

    #[cfg(feature = "new-geometry")]
    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        self.process_new_geometry(&ctx)?;
        Ok(())
    }

    #[cfg(feature = "new-geometry")]
    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        self.finish_new_geometry(ctx)?;
        Ok(())
    }
}

impl Cesium3DTilesWriter {
    #[cfg(not(feature = "new-geometry"))]
    fn process_default(&mut self, ctx: &ExecutorContext) -> crate::errors::Result<()> {
        let geometry = &ctx.feature.geometry;
        if geometry.is_empty() {
            tracing::warn!("Cesium3DTilesWriter: skipping feature with no geometry");
            return Ok(());
        };
        let geometry_value = &geometry.value;
        if !matches!(
            geometry_value,
            geometry_types::GeometryValue::CityGmlGeometry(_)
        ) {
            tracing::warn!("Cesium3DTilesWriter: skipping feature with non-CityGML geometry");
            return Ok(());
        }

        let filename = self
            .params
            .schema_key
            .as_ref()
            .and_then(|key| ctx.feature.get(key).and_then(|v| v.as_string()));

        let env_vars = ctx.env_vars.clone();
        let output = self
            .params
            .output
            .eval_string(&ctx.feature, Arc::clone(&env_vars))
            .map_err(|e| SinkError::Cesium3DTilesWriter(format!("{e:?}")))?;
        let compress_output = self
            .params
            .compress_output
            .as_ref()
            .map(|c| -> crate::errors::Result<String> {
                let compress_path = c
                    .eval_string(&ctx.feature, Arc::clone(&env_vars))
                    .map_err(|e| SinkError::Cesium3DTilesWriter(format!("{e:?}")))?;
                Ok(compress_path)
            })
            .transpose()?;

        let feature = {
            let mut attrs = crate::schema::filter_and_cast_attributes(
                &ctx.feature,
                &self.schema,
                self.params.schema_key.as_deref(),
            );
            let skip_unexp = self.params.skip_unexposed_attributes;
            attrs.retain(|k, _| {
                let key = k.as_ref();
                !(skip_unexp && key.starts_with("__"))
                    && self.params.schema_key.as_deref() != Some(key)
            });
            let mut feature = ctx.feature.clone();
            feature.attributes = Arc::new(attrs);
            feature
        };

        let buffer = self
            .buffer
            .entry((output, filename, compress_output.clone()))
            .or_default();
        buffer.push(feature);
        Ok(())
    }

    fn process_schema(&mut self, ctx: &ExecutorContext) {
        let Some(ref schema_key) = self.params.schema_key else {
            return;
        };

        let feature = &ctx.feature;
        let Some(schema_type) = feature.get(schema_key).and_then(|v| v.as_string()) else {
            tracing::warn!("Feature missing '{}' attribute for schema_key", schema_key);
            return;
        };

        let skip_unexp = self.params.skip_unexposed_attributes;
        let mut sanitized_feature = feature.clone();
        sanitized_feature.attributes = Arc::new(
            sanitized_feature
                .attributes
                .iter()
                .filter(|(k, _)| {
                    let key = k.as_ref();
                    !(skip_unexp && key.starts_with("__"))
                        && self.params.schema_key.as_deref() != Some(key)
                })
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        );

        let typedef: TypeDef = (&sanitized_feature).into();
        self.schema.types.insert(schema_type, typedef);
    }

    #[cfg(not(feature = "new-geometry"))]
    pub(crate) fn flush_buffer(&self, ctx: Context) -> crate::errors::Result<()> {
        let mut features =
            HashMap::<(String, Option<String>), Vec<(Option<String>, Vec<Feature>)>>::new();
        for ((output, filename, compress_output), buffer) in &self.buffer {
            features
                .entry((output.clone(), compress_output.clone()))
                .or_default()
                .push((filename.clone(), buffer.clone()));
        }
        for ((output, compress_output), buffer) in &features {
            self.write(ctx.clone(), buffer, output, compress_output)?;
        }
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    #[cfg(not(feature = "new-geometry"))]
    pub(crate) fn write(
        &self,
        ctx: Context,
        upstream: &[(Option<String>, Vec<Feature>)],
        output: &str,
        compress_output: &Option<String>,
    ) -> crate::errors::Result<()> {
        let tile_id_conv = TileIdMethod::Hilbert;
        let attach_texture = self.params.attach_texture.unwrap_or(false);
        let mut schema = self.schema.clone();
        for (typename, features) in upstream {
            let key = typename.as_deref().unwrap_or("Feature");
            if !schema.types.contains_key(key) {
                if let Some(feature) = features.first() {
                    let typedef: TypeDef = feature.into();
                    schema.types.insert(key.to_string(), typedef);
                }
            }
        }
        // Resolve the output URI once here; the pipeline uses it as the base directory.
        let node_ctx = NodeContext::from(ctx.clone());
        let output_sink =
            crate::SinkOutput::new(&node_ctx.sandbox_root, output, &node_ctx.storage_resolver)
                .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;
        let output_uri = output_sink.uri().clone();

        let grouped_features: Vec<(Option<String>, Vec<Feature>)> = upstream.to_owned();

        let (sender_sliced, receiver_sliced) = std::sync::mpsc::sync_channel(2000);
        let (sender_sorted, receiver_sorted) = std::sync::mpsc::sync_channel(2000);
        let min_zoom = self.params.min_zoom;
        let max_zoom = self.params.max_zoom;

        let output_uri_for_log = output_uri.clone();
        let compress_output = compress_output.clone();

        std::thread::scope(|s| {
            {
                let ctx = ctx.clone();
                let out_log = output_uri_for_log.clone();
                s.spawn(move || {
                    let feature_count: usize =
                        grouped_features.iter().map(|(_, fs)| fs.len()).sum();
                    let now = time::Instant::now();
                    let result = super::pipeline::geometry_slicing_stage(
                        &grouped_features,
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
                            feature_count,
                            now.elapsed(),
                            out_log
                        ),
                    );
                });
            }
            {
                let ctx = ctx.clone();
                let out_log = output_uri_for_log.clone();
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
                            out_log
                        ),
                    );
                });
            }
            {
                let ctx = ctx.clone();
                let schema = schema.clone();
                let output_uri_inner = output_uri.clone();
                s.spawn(move || {
                    let pool = rayon::ThreadPoolBuilder::new()
                        .use_current_thread()
                        .build()
                        .unwrap();
                    pool.install(|| {
                        let now = time::Instant::now();
                        let result = super::pipeline::tile_writing_stage(
                            ctx.clone(),
                            output_uri_inner.clone(),
                            receiver_sorted,
                            tile_id_conv,
                            &schema,
                            self.params.draco_compression.unwrap_or(true),
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
                                output_uri_inner
                            ),
                        );

                        if let Some(ref compress_rel) = compress_output {
                            let compress_node_ctx = NodeContext::from(ctx.clone());
                            match crate::SinkOutput::new(
                                &compress_node_ctx.sandbox_root,
                                compress_rel,
                                &compress_node_ctx.storage_resolver,
                            ) {
                                Ok(compress_sink_out) => {
                                    let now = time::Instant::now();
                                    let buffer = Vec::new();
                                    let mut cursor = Cursor::new(buffer);
                                    let writer = BufWriter::new(&mut cursor);
                                    let zip_result = reearth_flow_common::zip::write(
                                        writer,
                                        output_uri_inner.path().as_path(),
                                    )
                                    .map_err(|e| {
                                        crate::errors::SinkError::cesium3dtiles_writer(
                                            e.to_string(),
                                        )
                                    });
                                    match zip_result {
                                        Ok(_) => {
                                            match compress_sink_out
                                                .write(bytes::Bytes::from(cursor.into_inner()))
                                                .map_err(|e| {
                                                    crate::errors::SinkError::cesium3dtiles_writer(
                                                        e.to_string(),
                                                    )
                                                })
                                            {
                                                Ok(_) => {
                                                    match std::fs::remove_dir_all(
                                                        output_uri_inner.path().as_path(),
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
                                                format!(
                                                    "Failed to write zip file with error = {e:?}"
                                                ),
                                            );
                                        }
                                    }
                                    ctx.event_hub.info_log(
                                        None,
                                        format!(
                                            "Finish write zip file. elapsed = {:?}, output = {}",
                                            now.elapsed(),
                                            output_uri_inner
                                        ),
                                    );
                                }
                                Err(e) => {
                                    ctx.event_hub.error_log(
                                        None,
                                        format!(
                                            "Failed to resolve compress output with error = {e:?}"
                                        ),
                                    );
                                }
                            }
                        }
                    });
                });
            }
        });
        Ok(())
    }
}
