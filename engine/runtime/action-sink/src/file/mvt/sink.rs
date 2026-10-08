use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::vec;

use nusamai_citygml::schema::{Schema, TypeDef};
use once_cell::sync::Lazy;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::cache::executor_cache_subdir;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::geometry as geometry_types;
use reearth_flow_types::Expr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::errors::SinkError;

use super::pipeline::{write_tileset, TilesetBuffer};

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
            executor_id: uuid::Uuid::nil(),
        };
        Ok(Box::new(sink))
    }
}

type BufferKey = (Uri, Option<Uri>); // (output, compress_output)

#[derive(Debug)]
pub struct MVTWriter {
    pub(super) global_params: Option<HashMap<String, serde_json::Value>>,
    pub(super) params: MVTWriterCompiledParam,
    pub(super) schema: Schema,
    pub(super) buffer: HashMap<BufferKey, TilesetBuffer>,
    /// Execution this writer runs in; its cache directory holds the buffered
    /// features. Nil until the runtime sets it.
    pub(super) executor_id: uuid::Uuid,
}

impl Clone for MVTWriter {
    /// Buffered features are per instance; a clone starts empty.
    fn clone(&self) -> Self {
        Self {
            global_params: self.global_params.clone(),
            params: self.params.clone(),
            schema: self.schema.clone(),
            buffer: HashMap::new(),
            executor_id: self.executor_id,
        }
    }
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

    fn set_executor_id(&mut self, executor_id: uuid::Uuid) {
        self.executor_id = executor_id;
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
        let feature = &ctx.feature;
        if feature.geometry.is_empty() {
            return Err(Box::new(SinkError::MvtWriter(
                "Unsupported input".to_string(),
            )));
        };
        if !matches!(
            feature.geometry.value,
            geometry_types::GeometryValue::CityGmlGeometry(_)
                | geometry_types::GeometryValue::FlowGeometry2D(_)
        ) {
            return Err(Box::new(SinkError::MvtWriter(
                "Unsupported input".to_string(),
            )));
        }

        let scope = feature.new_scope(ctx.expr_engine.clone(), &self.global_params);
        let eval = |ast: &rhai::AST| {
            scope
                .eval_ast::<String>(ast)
                .map_err(|e| SinkError::MvtWriter(format!("{e:?}")))
        };
        let output = Uri::from_str(&eval(&self.params.output)?)?;
        let compress_output = match &self.params.compress_output {
            Some(ast) => Some(Uri::from_str(&eval(ast)?)?),
            None => None,
        };
        let layer_name = eval(&self.params.layer_name)?;

        // A new output means the previous ones are complete.
        let key = (output, compress_output);
        if !self.buffer.contains_key(&key) {
            let context = ctx.as_context();
            for ((output, compress_output), buffer) in self.buffer.drain() {
                write_tileset(
                    &context,
                    &buffer,
                    &output,
                    compress_output.as_ref(),
                    &self.schema,
                    &self.params,
                )?;
            }
        }
        let executor_id = self.executor_id;
        self.buffer
            .entry(key)
            .or_insert_with(|| {
                TilesetBuffer::new(&executor_cache_subdir(executor_id, "mvt-writer"))
            })
            .push(feature, layer_name)?;
        Ok(())
    }

    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let context = ctx.as_context();
        for ((output, compress_output), buffer) in &self.buffer {
            write_tileset(
                &context,
                buffer,
                output,
                compress_output.as_ref(),
                &self.schema,
                &self.params,
            )?;
        }
        Ok(())
    }
}
