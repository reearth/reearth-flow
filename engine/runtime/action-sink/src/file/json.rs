use std::collections::HashMap;
use std::sync::Arc;

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_eval_expr::utils::dynamic_to_value;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{Expr, Feature};
use rhai::Dynamic;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;
use crate::SinkOutput;

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
    pub(super) buffer: HashMap<Uri, (SinkOutput, Vec<Feature>)>,
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
        let node_ctx: NodeContext = ctx.clone().into();
        let out = SinkOutput::from_expr(&node_ctx, &self.params.output)
            .map_err(|e| SinkError::JsonWriter(e.to_string()))?;
        self.buffer
            .entry(out.uri().clone())
            .or_insert_with(|| (out, Vec::new()))
            .1
            .push(ctx.feature);
        Ok(())
    }

    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        for (out, features) in self.buffer.values() {
            write_json(out, &self.params, features, &ctx.expr_engine)?;
        }
        Ok(())
    }
}

fn write_json(
    out: &SinkOutput,
    params: &JsonWriterParam,
    features: &[Feature],
    expr_engine: &Arc<Engine>,
) -> Result<(), crate::errors::SinkError> {
    let json_value: serde_json::Value = if let Some(converter) = &params.converter {
        let scope = expr_engine.new_scope();
        let value: serde_json::Value = serde_json::Value::Array(
            features
                .iter()
                .map(|feature| {
                    serde_json::Value::Object(
                        feature
                            .attributes
                            .iter()
                            .map(|(k, v)| (k.clone().into_inner().to_string(), v.clone().into()))
                            .collect::<serde_json::Map<_, _>>(),
                    )
                })
                .collect::<Vec<_>>(),
        );
        scope.set("__features", value);
        let convert = scope.eval::<Dynamic>(converter.as_ref()).map_err(|e| {
            crate::errors::SinkError::JsonWriter(format!("Failed to evaluate converter: {e:?}"))
        })?;
        dynamic_to_value(&convert)
    } else {
        let attributes = features
            .iter()
            .map(|f| {
                serde_json::Value::Object(
                    f.attributes
                        .iter()
                        .map(|(k, v)| (k.clone().into_inner().to_string(), v.clone().into()))
                        .collect::<serde_json::Map<_, _>>(),
                )
            })
            .collect::<Vec<serde_json::Value>>();
        serde_json::Value::Array(attributes)
    };
    out.write(Bytes::from(json_value.to_string()))
        .map_err(|e| crate::errors::SinkError::JsonWriter(format!("{e:?}")))?;
    Ok(())
}
