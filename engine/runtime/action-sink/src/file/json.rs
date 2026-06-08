use std::collections::HashMap;
use std::sync::Arc;

use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{Attribute, AttributeValue, Code, CompiledCode, Feature};
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
        &["Output"]
    }

    fn tags(&self) -> &[&'static str] {
        &["json"]
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
        let params: JsonWriterParam = if let Some(with) = with {
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
        let output = params.output.compile().map_err(|e| {
            SinkError::JsonWriterFactory(format!("Failed to compile `output`: {e:?}"))
        })?;
        let compiled_converter = params
            .converter
            .as_ref()
            .map(|code| code.compile())
            .transpose()
            .map_err(|e| SinkError::JsonWriterFactory(format!("{e:?}")))?;
        let sink = JsonWriter {
            output,
            compiled_converter,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(super) struct JsonWriter {
    output: CompiledCode,
    pub(super) compiled_converter: Option<CompiledCode>,
    pub(super) buffer: HashMap<String, (SinkOutput, Vec<Feature>)>,
}

/// # JsonWriter Parameters
///
/// Configuration for writing features to JSON files.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct JsonWriterParam {
    /// Output path or expression for the JSON file to create
    pub(super) output: Code,
    /// Optional converter expression to transform features before writing
    pub(super) converter: Option<Code>,
}

impl Sink for JsonWriter {
    fn name(&self) -> &str {
        "JsonWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let path = self
            .output
            .eval_string(&ctx.feature, ctx.expr_engine.vars())
            .map_err(|e| SinkError::JsonWriter(format!("{e:?}")))?;
        let feature = ctx.feature.clone();
        let node_ctx: NodeContext = ctx.into();
        use std::collections::hash_map::Entry;
        match self.buffer.entry(path.clone()) {
            Entry::Occupied(mut e) => {
                e.get_mut().1.push(feature);
            }
            Entry::Vacant(e) => {
                let out =
                    SinkOutput::new(&node_ctx.sandbox_root, &path, &node_ctx.storage_resolver)
                        .map_err(|e| SinkError::JsonWriter(e.to_string()))?;
                e.insert((out, vec![feature]));
            }
        }
        Ok(())
    }

    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let env_vars = ctx.expr_engine.vars();
        for (out, features) in self.buffer.values() {
            write_json(
                out,
                &self.compiled_converter,
                features,
                Arc::clone(&env_vars),
            )?;
        }
        Ok(())
    }
}

fn write_json(
    out: &SinkOutput,
    converter: &Option<CompiledCode>,
    features: &[Feature],
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
) -> Result<(), crate::errors::SinkError> {
    let json_value: serde_json::Value = if let Some(converter) = converter {
        let synthetic = synthetic_feature(features);
        converter
            .eval(&synthetic, env_vars)
            .map_err(|e| {
                crate::errors::SinkError::JsonWriter(format!("Failed to evaluate converter: {e:?}"))
            })?
            .into()
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

fn synthetic_feature(features: &[Feature]) -> Feature {
    let packed = AttributeValue::Array(
        features
            .iter()
            .map(|f| {
                AttributeValue::Map(
                    f.attributes
                        .iter()
                        .map(|(k, v)| (k.clone().into_inner().to_string(), v.clone()))
                        .collect(),
                )
            })
            .collect(),
    );
    Feature::from(IndexMap::from([(
        Attribute::new("__features".to_string()),
        packed,
    )]))
}
