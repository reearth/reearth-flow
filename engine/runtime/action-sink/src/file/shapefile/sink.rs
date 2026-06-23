use std::collections::HashMap;
use std::vec;

use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{Attribute, AttributeValue, Code, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;

use super::pipeline;

#[derive(Debug, Clone, Default)]
pub(crate) struct ShapefileWriterFactory;

impl SinkFactory for ShapefileWriterFactory {
    fn name(&self) -> &str {
        "ShapefileWriter"
    }

    fn description(&self) -> &str {
        "Writes geographic features to ESRI Shapefile format with optional grouping"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ShapefileWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Output"]
    }

    fn tags(&self) -> &[&'static str] {
        &["shapefile"]
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
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params: ShapefileWriterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::ShapefileWriterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::ShapefileWriterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SinkError::ShapefileWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let output = params
            .output
            .compile()
            .map_err(|e| {
                SinkError::ShapefileWriterFactory(format!("Failed to compile `output`: {e:?}"))
            })?
            .eval_string_env_only(ctx.env_vars.clone())
            .map_err(|e| {
                SinkError::ShapefileWriterFactory(format!("Failed to evaluate `output`: {e:?}"))
            })?;
        let sink = ShapefileWriter {
            output,
            group_by: params.group_by,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ShapefileWriter {
    output: String,
    group_by: Option<Vec<Attribute>>,
    pub(super) buffer: HashMap<AttributeValue, Vec<Feature>>,
}

/// # ShapefileWriter Parameters
///
/// Configuration for writing features to ESRI Shapefile format.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShapefileWriterParam {
    /// Output path or expression for the Shapefile to create
    pub(super) output: Code,
    /// Optional attributes to group features by, creating separate files for each group
    pub(super) group_by: Option<Vec<Attribute>>,
}

impl Sink for ShapefileWriter {
    fn name(&self) -> &str {
        "ShapefileWriter"
    }

    #[cfg(not(feature = "new-geometry"))]
    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        let key = if let Some(group_by) = &self.group_by {
            if group_by.is_empty() {
                AttributeValue::Null
            } else {
                let key = group_by
                    .iter()
                    .map(|k| feature.get(k).cloned().unwrap_or(AttributeValue::Null))
                    .collect::<Vec<_>>();
                AttributeValue::Array(key)
            }
        } else {
            AttributeValue::Null
        };
        self.buffer.entry(key).or_default().push(feature.clone());
        Ok(())
    }
    #[cfg(not(feature = "new-geometry"))]
    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let path = self.output.as_str();
        for (key, features) in self.buffer.iter() {
            pipeline::pipeline(
                &ctx.as_context(),
                &ctx.sandbox_root,
                path,
                key,
                features,
                &ctx.storage_resolver,
            )?;
        }
        Ok(())
    }
}
