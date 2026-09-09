use std::{
    collections::HashMap,
    io::{BufWriter, Cursor},
    str::FromStr,
    sync::Arc,
    time, vec,
};

use nusamai_citygml::schema::{Schema, TypeDef};
use once_cell::sync::Lazy;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_runtime::{errors::BoxedError, executor_operation::Context};
use reearth_flow_types::geometry as geometry_types;
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::errors::SinkError;

static SCHEMA_PORT: Lazy<Port> = Lazy::new(|| Port::new("schema"));

/// Serde default for flags that are on unless explicitly disabled. Also makes
/// the generated JSON schema advertise `default: true`.
fn default_true() -> bool {
    true
}

/// # Texture Codec
/// Texture image codec for the writer's atlas pages.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq, JsonSchema)]
pub enum TextureCodec {
    /// KTX2 with Basis Universal UASTC supercompression (`KHR_texture_basisu`):
    /// higher quality, larger files.
    #[serde(rename = "KTX2/UASTC")]
    Ktx2Uastc,
    /// KTX2 with Basis Universal ETC1S supercompression (`KHR_texture_basisu`):
    /// smaller files, lower quality.
    #[default]
    #[serde(rename = "KTX2/ETC1S")]
    Ktx2Etc1s,
    /// PNG, lossless with alpha.
    #[serde(rename = "PNG")]
    Png,
    /// JPEG, lossy and opaque (alpha is dropped).
    #[serde(rename = "JPEG")]
    Jpeg,
    /// Attach no textures; textured geometry falls back to its neutral colour.
    Untextured,
}

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
            current_chunk: None,
            chunk_keys: HashMap::new(),
            pending_flush: Vec::new(),
            params: Cesium3DTilesWriterCompiledParam {
                output,
                compress_output,
                draco_compression: params.draco_compression,
                skip_unexposed_attributes: params.skip_unexposed_attributes.unwrap_or(false),
                chunk_by_attribute: params.chunk_by_attribute,
                target_tile_size: params.target_tile_size,
                compute_flat_normal: params.compute_flat_normal,
                texel_size: params.texel_size,
                atlas_size: params.atlas_size,
                atlas_extrusion: params.atlas_extrusion,
                wrap_tolerance: params.wrap_tolerance,
                texture_codec: params.texture_codec,
                schema_key: params.schema_key,
                array_map_separator: params.array_map_separator,
            },
        };
        Ok(Box::new(sink))
    }
}

type BufferKey = (Uri, String, Option<Uri>); // (output, feature_type, compress_output)
type GroupedBuffers = HashMap<(Uri, Option<Uri>), Vec<(String, Vec<Feature>)>>;

#[derive(Debug, Clone)]
pub struct Cesium3DTilesWriter {
    pub(super) global_params: Option<HashMap<String, serde_json::Value>>,
    pub(super) buffer: HashMap<BufferKey, Vec<Feature>>,
    pub(super) schema: Schema,
    // last seen chunk_by_attribute value; a change means the previous chunk is complete and can be flushed
    pub(super) current_chunk: Option<AttributeValue>,
    pub(super) chunk_keys: HashMap<AttributeValue, std::collections::HashSet<BufferKey>>,
    // chunks whose data closed before every feature_type they need has a schema entry
    pub(super) pending_flush: Vec<AttributeValue>,
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
    /// Unused: implicit tiling derives tile depth from `targetTileSize`
    /// instead. Kept only so older workflow files still deserialize.
    pub(super) min_zoom: Option<u8>,
    /// # Maximum Zoom Level
    /// Unused: implicit tiling derives tile depth from `targetTileSize`
    /// instead. Kept only so older workflow files still deserialize.
    pub(super) max_zoom: Option<u8>,
    /// # Attach Textures
    /// Unused: use `textureCodec: Untextured` to attach no textures instead.
    /// Kept only so older workflow files still deserialize.
    pub(super) attach_texture: Option<bool>,
    /// # Compressed Output Path
    /// Optional path for compressed archive output
    pub(super) compress_output: Option<Expr>,
    /// # Draco Compression
    /// Use draco compression. Defaults to true.
    pub(super) draco_compression: Option<bool>,
    /// # Skip unexposed Attributes
    /// Skip attributes with double underscore prefix
    pub(super) skip_unexposed_attributes: Option<bool>,
    /// # Chunk By Attribute
    /// An attribute name to split processing into chunks by. Features sharing a
    /// value must already be adjacent, and every feature must have it set. Each
    /// chunk's value must map to a different output path.
    pub(super) chunk_by_attribute: Option<String>,
    /// # Target Tile Size
    /// Target content size per tile, in bytes. Tiles are split when they'd
    /// exceed it and merged with neighbours when they'd otherwise be smaller;
    /// a single feature that alone exceeds it is kept whole (features are
    /// never clipped). A value of 0 disables merging and splits every feature
    /// into its own content. Defaults to 1,048,576 (1 MiB).
    pub(super) target_tile_size: Option<u64>,
    /// # Compute Flat Normals
    /// Compute per-polygon flat normals for lighting. Defaults to true.
    /// When disabled, no normals are written and the mesh is smaller, but the
    /// tile carries no lighting data (a viewer must derive flat normals itself).
    #[serde(default = "default_true")]
    pub(super) compute_flat_normal: bool,
    /// # Texel Size
    /// Target texel size in metres per pixel. Textures finer than this are
    /// downsampled to it. Defaults to 0, which keeps full texture detail.
    pub(super) texel_size: Option<f64>,
    /// # Atlas Size
    /// Maximum texture atlas dimension in pixels. Textures exceeding this spill
    /// onto additional atlas pages; a single texture larger than it is
    /// downsampled to fit. Defaults to 2048.
    #[schemars(range(min = 1, max = 65536))]
    pub(super) atlas_size: Option<u32>,
    /// # Atlas Extrusion
    /// Ring of pixels blitted around each texture region in the atlas to stop
    /// bilinear bleed between neighbouring regions. Defaults to 0 (disabled).
    #[schemars(range(max = 65536))]
    pub(super) atlas_extrusion: Option<u32>,
    /// # Wrap Tolerance
    /// How far outside 0-1 a texture coordinate may stray and still be clamped as
    /// dataset drift. Past it the texture is taken to tile and is given an atlas
    /// page of its own so the sampler can repeat it. Defaults to 0.
    #[schemars(range(min = 0.0, max = 1.0))]
    pub(super) wrap_tolerance: Option<f64>,
    /// # Texture Codec
    /// Image codec for atlas pages. Defaults to `KTX2/ETC1S`; select
    /// `Untextured` to attach no textures.
    #[serde(default)]
    pub(super) texture_codec: TextureCodec,
    /// # Schema Key
    /// Attribute key whose value identifies the schema type and determines the
    /// output filename: all features sharing the same value are written to the
    /// same file. This attribute is excluded from output.
    pub(super) schema_key: Option<String>,
    /// # Array/Map Separator
    /// Separator joining a nested array or map attribute to its child key or
    /// index when flattening it into metadata columns. Leave unset to drop array
    /// and map attributes from the output entirely.
    pub(super) array_map_separator: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Cesium3DTilesWriterCompiledParam {
    pub(super) output: rhai::AST,
    pub(super) compress_output: Option<rhai::AST>,
    pub(super) draco_compression: Option<bool>,
    pub(super) skip_unexposed_attributes: bool,
    pub(super) chunk_by_attribute: Option<String>,
    pub(super) target_tile_size: Option<u64>,
    pub(super) compute_flat_normal: bool,
    pub(super) texel_size: Option<f64>,
    pub(super) atlas_size: Option<u32>,
    pub(super) atlas_extrusion: Option<u32>,
    pub(super) wrap_tolerance: Option<f64>,
    pub(super) texture_codec: TextureCodec,
    pub(super) schema_key: Option<String>,
    pub(super) array_map_separator: Option<String>,
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

        let chunk = match &self.params.chunk_by_attribute {
            Some(attr) => {
                let Some(v) = ctx.feature.attributes.get(&Attribute::new(attr.clone())) else {
                    return Err(SinkError::Cesium3DTilesWriter(format!(
                        "chunkByAttribute {attr:?} is set but feature is missing that attribute"
                    )));
                };
                Some(v.clone())
            }
            None => None,
        };
        if let Some(chunk) = &chunk {
            if self.current_chunk.as_ref() != Some(chunk) {
                if let Some(prev) = self.current_chunk.take() {
                    self.maybe_flush_chunk(ctx.as_context(), &prev)?;
                }
                self.current_chunk = Some(chunk.clone());
            }
        }

        let output = self.params.output.clone();
        let scope = ctx
            .feature
            .new_scope(ctx.expr_engine.clone(), &self.global_params);
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

        let feature = {
            let mut attrs = crate::schema::filter_and_cast_attributes(&ctx.feature, &self.schema);
            if self.params.skip_unexposed_attributes {
                attrs.retain(|k, _| !k.as_ref().starts_with("__"));
            }
            let mut feature = ctx.feature.clone();
            feature.attributes = Arc::new(attrs);
            feature
        };

        let key = (output, feature_type.clone(), compress_output);
        if let Some(chunk) = &chunk {
            self.chunk_keys
                .entry(chunk.clone())
                .or_default()
                .insert(key.clone());
        }
        self.buffer.entry(key).or_default().push(feature);
        Ok(())
    }

    fn chunk_ready(&self, chunk: &AttributeValue) -> bool {
        if self.schema.types.is_empty() {
            // schema port never wired for this workflow (or hasn't sent anything yet) —
            // don't block flushing on it, otherwise nothing would ever flush incrementally
            return true;
        }
        self.chunk_keys
            .get(chunk)
            .map(|keys| {
                keys.iter()
                    .all(|(_, feature_type, _)| self.schema.types.contains_key(feature_type))
            })
            .unwrap_or(true)
    }

    fn maybe_flush_chunk(
        &mut self,
        ctx: Context,
        chunk: &AttributeValue,
    ) -> crate::errors::Result<()> {
        if self.chunk_ready(chunk) {
            self.flush_chunk(ctx, chunk)
        } else {
            self.pending_flush.push(chunk.clone());
            Ok(())
        }
    }

    fn retry_pending_flush(&mut self, ctx: Context) -> crate::errors::Result<()> {
        let ready: Vec<AttributeValue> = self
            .pending_flush
            .iter()
            .filter(|c| self.chunk_ready(c))
            .cloned()
            .collect();
        for c in ready {
            self.pending_flush.retain(|x| x != &c);
            self.flush_chunk(ctx.clone(), &c)?;
        }
        Ok(())
    }

    fn flush_chunk(&mut self, ctx: Context, chunk: &AttributeValue) -> crate::errors::Result<()> {
        let Some(keys) = self.chunk_keys.remove(chunk) else {
            return Ok(());
        };
        let mut grouped: GroupedBuffers = HashMap::new();
        for key in keys {
            let Some(buffer) = self.buffer.remove(&key) else {
                continue;
            };
            let (output, feature_type, compress_output) = key;
            grouped
                .entry((output, compress_output))
                .or_default()
                .push((feature_type, buffer));
        }
        for ((output, compress_output), upstream) in &grouped {
            self.write(ctx.clone(), upstream, output, compress_output)?;
        }
        Ok(())
    }

    fn process_schema(&mut self, ctx: &ExecutorContext) -> crate::errors::Result<()> {
        let feature = &ctx.feature;
        let Some(feature_type) = &feature.feature_type() else {
            return Err(SinkError::Cesium3DTilesWriter(
                "Failed to get feature type".to_string(),
            ));
        };

        let mut sanitized_feature = feature.clone();
        sanitized_feature.attributes = Arc::new(
            sanitized_feature
                .attributes
                .iter()
                .filter(|(k, _)| {
                    !self.params.skip_unexposed_attributes || !k.as_ref().starts_with("__")
                })
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        );

        let typedef: TypeDef = (&sanitized_feature).into();
        self.schema.types.insert(feature_type.clone(), typedef);
        if !self.pending_flush.is_empty() {
            self.retry_pending_flush(ctx.as_context())?;
        }
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
        let mut features = Vec::new();
        for (_, upstream) in upstream {
            features.extend(upstream.clone());
        }

        let options = super::builder::MetadataOptions {
            schema_key: self.params.schema_key.as_deref(),
            skip_unexposed_attributes: self.params.skip_unexposed_attributes,
            array_map_separator: self.params.array_map_separator.as_deref(),
        };
        let render = super::builder::RenderOptions {
            draco: self.params.draco_compression.unwrap_or(true),
            compute_flat_normal: self.params.compute_flat_normal,
            texel_size: self.params.texel_size.unwrap_or(0.0),
            atlas_size: self.params.atlas_size.unwrap_or(2048),
            atlas_extrusion: self.params.atlas_extrusion.unwrap_or(0),
            wrap_tolerance: self.params.wrap_tolerance.unwrap_or(0.0),
            texture_codec: self.params.texture_codec,
        };
        let target_tile_size = self.params.target_tile_size.unwrap_or(1_048_576);

        let storage = ctx
            .storage_resolver
            .resolve(output)
            .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;
        let write_tile = |relative_path: String, bytes: Vec<u8>| -> crate::errors::Result<()> {
            let path = output.path().join(std::path::Path::new(&relative_path));
            storage
                .put_sync(std::path::Path::new(&path), bytes::Bytes::from(bytes))
                .map_err(crate::errors::SinkError::cesium3dtiles_writer)
        };

        let now = time::Instant::now();
        let built = super::builder::build(&features, options, target_tile_size, render, write_tile)?;
        for (relative_path, bytes) in built.subtrees {
            write_tile(relative_path, bytes)?;
        }
        write_tile("tileset.json".to_string(), built.tileset_json.into_bytes())?;
        ctx.event_hub.info_log(
            None,
            format!(
                "Finish Cesium3DTilesWriter. feature length = {}, tile count = {}, elapsed = {:?}, output = {}",
                features.len(),
                built.tile_count,
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
                let zip_result =
                    reearth_flow_common::zip::write(writer, output.path().as_path()).map_err(
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
                            Ok(_) => {
                                if let Err(e) = std::fs::remove_dir_all(output.path().as_path()) {
                                    ctx.event_hub.error_log(
                                        None,
                                        format!("Failed to remove directory with error = {e:?}"),
                                    );
                                }
                            }
                            Err(e) => {
                                ctx.event_hub.error_log(
                                    None,
                                    format!("Failed to write zip file with error = {e:?}"),
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
        Ok(())
    }
}
