use std::collections::HashMap;
use std::vec;

use bytes::Bytes;
use reearth_flow_common::str::to_hash;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, FEATURES_PORT};
use reearth_flow_types::{Attribute, AttributeValue, Code, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;

#[derive(Debug, Clone, Default)]
pub(crate) struct GeoJsonWriterFactory;

impl SinkFactory for GeoJsonWriterFactory {
    fn name(&self) -> &str {
        "GeoJsonWriter"
    }

    fn description(&self) -> &str {
        "Writes geographic features to GeoJSON files with optional grouping"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeoJsonWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Output"]
    }

    fn tags(&self) -> &[&'static str] {
        &["geojson"]
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
        let params: GeoJsonWriterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::GeoJsonWriterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::GeoJsonWriterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SinkError::GeoJsonWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let output = params
            .output
            .compile()
            .map_err(|e| {
                SinkError::GeoJsonWriterFactory(format!("Failed to compile `output`: {e:?}"))
            })?
            .eval_string_env_only(ctx.env_vars.clone())
            .map_err(|e| {
                SinkError::GeoJsonWriterFactory(format!("Failed to evaluate `output`: {e:?}"))
            })?;
        let sink = GeoJsonWriter {
            output,
            group_by: params.group_by,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(super) struct GeoJsonWriter {
    output: String,
    group_by: Option<Vec<Attribute>>,
    pub(super) buffer: HashMap<AttributeValue, Vec<Feature>>,
}

/// # GeoJsonWriter Parameters
///
/// Configuration for writing features to GeoJSON files.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct GeoJsonWriterParam {
    /// Output path or expression for the GeoJSON file to create
    pub(super) output: Code,
    /// Optional attributes to group features by, creating separate files for each group
    pub(super) group_by: Option<Vec<Attribute>>,
}

impl Sink for GeoJsonWriter {
    fn name(&self) -> &str {
        "GeoJsonWriter"
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
            let out_path = if *key == AttributeValue::Null {
                path.to_string()
            } else {
                format!("{}/{}.geojson", path, to_hash(key.to_string().as_str()))
            };
            let out = crate::SinkOutput::new(&ctx.sandbox_root, &out_path, &ctx.storage_resolver)
                .map_err(crate::errors::SinkError::geojson_writer)?;

            let mut buffer = Vec::from(b"{\"type\":\"FeatureCollection\",\"features\":[");

            let geojsons: Vec<geojson::Feature> = features
                .iter()
                .flat_map(|f| {
                    let geojsons: Option<Vec<geojson::Feature>> = f.clone().try_into().ok();
                    geojsons
                })
                .flatten()
                .collect();
            for (index, geojson) in geojsons.iter().enumerate() {
                if index > 0 {
                    buffer.push(b',');
                }
                let bytes = serde_json::to_vec(&geojson)
                    .map_err(|e| crate::errors::SinkError::GeoJsonWriter(format!("{e}")))?;
                buffer.extend(bytes);
            }
            buffer.extend(Vec::from(b"]}\n"));
            out.write(Bytes::from(buffer))
                .map_err(crate::errors::SinkError::geojson_writer)?;
        }
        Ok(())
    }
}
