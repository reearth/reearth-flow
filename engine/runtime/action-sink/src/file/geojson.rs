use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::vec;

use bytes::Bytes;
use reearth_flow_common::str::to_hash;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
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
        "Writes features to a geojson file"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeoJsonWriterParam))
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

        let sink = GeoJsonWriter {
            params,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(super) struct GeoJsonWriter {
    pub(super) params: GeoJsonWriterParam,
    pub(super) buffer: HashMap<AttributeValue, Vec<Feature>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct GeoJsonWriterParam {
    pub(super) output: Expr,
    pub(super) group_by: Option<Vec<Attribute>>,
}

impl Sink for GeoJsonWriter {
    fn name(&self) -> &str {
        "GeoJsonWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        let key = if let Some(group_by) = &self.params.group_by {
            if group_by.is_empty() {
                AttributeValue::Null
            } else {
                let key = group_by
                    .iter()
                    .map(|k| feature.get(&k).cloned().unwrap_or(AttributeValue::Null))
                    .collect::<Vec<_>>();
                AttributeValue::Array(key)
            }
        } else {
            AttributeValue::Null
        };
        self.buffer.entry(key).or_default().push(feature.clone());
        Ok(())
    }
    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let output = self.params.output.clone();
        let scope = expr_engine.new_scope();
        let path = scope
            .eval::<String>(output.as_ref())
            .unwrap_or_else(|_| output.as_ref().to_string());
        let output = Uri::from_str(path.as_str())?;

        for (key, features) in self.buffer.iter() {
            let file_path = if *key == AttributeValue::Null {
                output.clone()
            } else {
                output.join(format!("{}.geojson", to_hash(key.to_string().as_str())))?
            };
            let storage = storage_resolver
                .resolve(&file_path)
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
            storage.put_sync(file_path.path().as_path(), Bytes::from(buffer))?;
        }
        Ok(())
    }
}
