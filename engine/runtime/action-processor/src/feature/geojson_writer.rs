use std::collections::HashMap;

use indexmap::IndexMap;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_action_sink::file::geojson::write_geojson_to_storage;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use reearth_flow_types::{Code, CompiledCode, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureGeoJsonWriterFactory;

impl ProcessorFactory for FeatureGeoJsonWriterFactory {
    fn name(&self) -> &str {
        "Feature GeoJSON Writer"
    }

    fn description(&self) -> &str {
        "Writes features to a GeoJSON file for each resolved output path."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureGeoJsonWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn tags(&self) -> &[&'static str] {
        &["geojson"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureGeoJsonWriterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FeatureGeoJsonWriterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FeatureGeoJsonWriterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FeatureGeoJsonWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let output = params
            .output
            .compile()
            .map_err(|e| FeatureProcessorError::FeatureGeoJsonWriterFactory(format!("{e:?}")))?;
        Ok(Box::new(FeatureGeoJsonWriter {
            output,
            buffer: HashMap::new(),
        }))
    }
}

/// # FeatureGeoJsonWriter Parameters
///
/// Configuration for writing features to GeoJSON files.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(description = "Configuration for writing features to GeoJSON files.")]
struct FeatureGeoJsonWriterParam {
    /// # Output
    ///
    /// Path (or expression evaluated per feature) of the GeoJSON file to write.
    /// Features sharing a resolved path are written to the same file, so the
    /// expression can split features across files by attribute value.
    #[schemars(
        title = "Output",
        description = "Path (or expression evaluated per feature) of the GeoJSON file to write. Features sharing a resolved path are written to the same file, so the expression can split features across files by attribute value."
    )]
    output: Code,
}

#[derive(Debug, Clone)]
struct FeatureGeoJsonWriter {
    output: CompiledCode,
    buffer: HashMap<String, Vec<Feature>>,
}

impl Processor for FeatureGeoJsonWriter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let path = self
            .output
            .eval_string(feature, ctx.env_vars.clone())
            .map_err(|e| FeatureProcessorError::FeatureGeoJsonWriter(format!("{e:?}")))?;
        // Validation happens at flush time via SinkOutput::new; the buffer is
        // keyed by the raw relative-path string.
        self.buffer.entry(path).or_default().push(ctx.feature);
        Ok(())
    }

    // Gated on `not(new-geometry)`, like `FeatureWriter::finish`: the GeoJSON
    // write path depends on `TryFrom<Feature> for Vec<geojson::Feature>`, which
    // is only available in the current geometry world.
    #[cfg(not(feature = "new-geometry"))]
    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        for (rel_path, features) in &self.buffer {
            // SinkOutput::new validates the path and acquires the storage
            // backend, providing the sandbox gate at flush time.
            let sink_output = reearth_flow_action_sink::SinkOutput::new(
                &ctx.sandbox_root,
                rel_path,
                &ctx.storage_resolver,
            )
            .map_err(|e| {
                FeatureProcessorError::FeatureGeoJsonWriter(format!(
                    "sink output {rel_path:?} rejected by sandbox: {e}"
                ))
            })?;
            write_geojson_to_storage(&sink_output, features)
                .map_err(|e| FeatureProcessorError::FeatureGeoJsonWriter(format!("{e:?}")))?;

            let feature: Feature = IndexMap::<Attribute, AttributeValue>::from([(
                Attribute::new("filePath".to_string()),
                AttributeValue::String(
                    sink_output
                        .uri()
                        .path()
                        .to_str()
                        .unwrap_or_default()
                        .to_string(),
                ),
            )])
            .into();
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                FEATURES_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Feature GeoJSON Writer"
    }
}
