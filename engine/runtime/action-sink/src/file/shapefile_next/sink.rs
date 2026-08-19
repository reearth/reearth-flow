//! The Shapefile Writer action.

use std::collections::HashMap;
use std::vec;

use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, FEATURES_PORT};
use reearth_flow_types::{Attribute, AttributeValue, Code, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;

use super::pipeline;

/// Builds the Shapefile Writer sink.
#[derive(Debug, Clone, Default)]
pub(crate) struct ShapefileWriterFactory;

impl SinkFactory for ShapefileWriterFactory {
    fn name(&self) -> &str {
        "Shapefile Writer"
    }

    fn description(&self) -> &str {
        "Writes features to ESRI Shapefile format, optionally grouping them into separate files."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ShapefileWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Output"]
    }

    fn tags(&self) -> &[&'static str] {
        &["shapefile", "vector"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
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
            .eval_string_variables_only(ctx.variables.clone())
            .map_err(|e| {
                SinkError::ShapefileWriterFactory(format!("Failed to evaluate `output`: {e:?}"))
            })?;
        let compress_output = params
            .compress_output
            .map(|code| {
                code.compile()
                    .map_err(|e| {
                        SinkError::ShapefileWriterFactory(format!(
                            "Failed to compile `compressOutput`: {e:?}"
                        ))
                    })?
                    .eval_string_variables_only(ctx.variables.clone())
                    .map_err(|e| {
                        SinkError::ShapefileWriterFactory(format!(
                            "Failed to evaluate `compressOutput`: {e:?}"
                        ))
                    })
            })
            .transpose()?;
        let sink = ShapefileWriter {
            output,
            compress_output,
            group_by: params.group_by,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

/// The Shapefile Writer sink.
#[derive(Debug, Clone)]
pub(crate) struct ShapefileWriter {
    /// The directory the file sets are written under.
    output: String,
    /// The directory each file set is archived into, if any.
    compress_output: Option<String>,
    /// The attributes features are grouped by, one file set per group.
    group_by: Option<Vec<Attribute>>,
    /// The features gathered so far, by group.
    pub(super) buffer: HashMap<AttributeValue, Vec<Feature>>,
}

/// # ShapefileWriter Parameters
///
/// Configuration for writing features to ESRI Shapefile format.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShapefileWriterParam {
    /// # Output Directory
    /// Output directory path or expression where the generated Shapefile files are written.
    pub(super) output: Code,
    /// # Compressed Output Directory
    /// Optional directory where each Shapefile is written as its own ZIP archive, holding that Shapefile's .shp, .shx, .dbf, .cpg and .prj, instead of as loose files. Leave unset to write loose files.
    pub(super) compress_output: Option<Code>,
    /// # Group By
    /// Attributes to group features by, writing a separate file for each distinct group.
    pub(super) group_by: Option<Vec<Attribute>>,
}

impl Sink for ShapefileWriter {
    fn name(&self) -> &str {
        "Shapefile Writer"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = ctx.feature;

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
        self.buffer.entry(key).or_default().push(feature);
        Ok(())
    }
    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let path = self.output.as_str();
        for (key, features) in self.buffer.iter() {
            pipeline::pipeline(
                &ctx.as_context(),
                &ctx.sandbox_root,
                path,
                self.compress_output.as_deref(),
                key,
                features,
                &ctx.storage_resolver,
            )?;
        }
        Ok(())
    }
}
