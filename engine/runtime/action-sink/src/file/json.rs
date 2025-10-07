use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::writer::write_json;
use crate::errors::SinkError;

#[derive(Debug, Clone, Default)]
pub(crate) struct JsonWriterFactory;

impl SinkFactory for JsonWriterFactory {
    fn name(&self) -> &str {
        "JsonWriter"
    }

    fn description(&self) -> &str {
        "Writes features to JSON files."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(JsonWriterParam))
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
        let params = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::JsonWriterFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::JsonWriterFactory(format!("Failed to deserialize `with` parameter: {e}"))
            })?
        } else {
            return Err(SinkError::JsonWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let sink = JsonWriter {
            params,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(super) struct JsonWriter {
    pub(super) params: JsonWriterParam,
    pub(super) buffer: HashMap<Uri, Vec<Feature>>,
}

/// # JsonWriter Parameters
///
/// Configuration for writing features to JSON files.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct JsonWriterParam {
    /// Output path or expression for the JSON file to create
    pub(super) output: Expr,
    /// Optional converter expression to transform features before writing
    pub(super) converter: Option<Expr>,
}

impl Sink for JsonWriter {
    fn name(&self) -> &str {
        "JsonWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = expr_engine.new_scope();
        let output = &self.params.output;
        let path = scope
            .eval::<String>(output.as_ref())
            .unwrap_or_else(|_| output.as_ref().to_string());
        let uri = Uri::from_str(&path)?;
        self.buffer.entry(uri).or_default().push(ctx.feature);
        Ok(())
    }

    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        for (uri, features) in &self.buffer {
            write_json(
                uri,
                &self.params.converter,
                features,
                &ctx.expr_engine,
                &storage_resolver,
            )?;
        }
        Ok(())
    }
}
