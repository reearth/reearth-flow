mod citygml;
mod csv;
mod json;

use std::collections::HashMap;

use indexmap::IndexMap;
use reearth_flow_common::csv::Delimiter;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{lod::LodMask, Attribute, AttributeValue, Code, CompiledCode, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureWriterFactory;

impl ProcessorFactory for FeatureWriterFactory {
    fn name(&self) -> &str {
        "FeatureWriter"
    }

    fn description(&self) -> &str {
        "Writes features from various formats"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
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
        let params: FeatureWriterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FeatureWriterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FeatureWriterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FeatureWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        match params {
            FeatureWriterParam::Csv { common_param } => {
                let common_param = CommonWriterCompiledParam {
                    output: common_param.output.compile().map_err(|e| {
                        FeatureProcessorError::FeatureWriterFactory(format!("{e:?}"))
                    })?,
                };
                let process = FeatureWriter {
                    params: CompiledFeatureWriterParam::Csv { common_param },
                    buffer: HashMap::new(),
                };
                Ok(Box::new(process))
            }
            FeatureWriterParam::Tsv { common_param } => {
                let common_param = CommonWriterCompiledParam {
                    output: common_param.output.compile().map_err(|e| {
                        FeatureProcessorError::FeatureWriterFactory(format!("{e:?}"))
                    })?,
                };
                let process = FeatureWriter {
                    params: CompiledFeatureWriterParam::Tsv { common_param },
                    buffer: HashMap::new(),
                };
                Ok(Box::new(process))
            }
            FeatureWriterParam::Json {
                common_param,
                param,
            } => {
                let common_param = CommonWriterCompiledParam {
                    output: common_param.output.compile().map_err(|e| {
                        FeatureProcessorError::FeatureWriterFactory(format!("{e:?}"))
                    })?,
                };
                let converter = param
                    .converter
                    .map(|code| code.compile())
                    .transpose()
                    .map_err(|e| FeatureProcessorError::FeatureWriterFactory(format!("{e:?}")))?;
                let process = FeatureWriter {
                    params: CompiledFeatureWriterParam::Json {
                        common_param,
                        param: json::CompiledJsonWriterParam { converter },
                    },
                    buffer: HashMap::new(),
                };
                Ok(Box::new(process))
            }
            FeatureWriterParam::CityGml {
                common_param,
                param,
            } => {
                let common_param = CommonWriterCompiledParam {
                    output: common_param.output.compile().map_err(|e| {
                        FeatureProcessorError::FeatureWriterFactory(format!("{e:?}"))
                    })?,
                };
                let lod_mask = citygml::build_lod_mask(&param.lod_filter);
                let process = FeatureWriter {
                    params: CompiledFeatureWriterParam::CityGml {
                        common_param,
                        lod_mask,
                        epsg_code: param.epsg_code,
                        pretty_print: param.pretty_print.unwrap_or(true),
                    },
                    buffer: HashMap::new(),
                };
                Ok(Box::new(process))
            }
        }
    }
}

#[derive(Debug, Clone)]
struct FeatureWriter {
    params: CompiledFeatureWriterParam,
    pub(super) buffer: HashMap<String, Vec<Feature>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct CommonWriterParam {
    /// # Output path
    pub(super) output: Code,
}

/// # FeatureWriter Parameters
///
/// Configuration for writing features to different file formats.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "format")]
enum FeatureWriterParam {
    #[serde(rename = "csv")]
    Csv {
        #[serde(flatten)]
        common_param: CommonWriterParam,
    },
    #[serde(rename = "tsv")]
    Tsv {
        #[serde(flatten)]
        common_param: CommonWriterParam,
    },
    #[serde(rename = "json")]
    Json {
        #[serde(flatten)]
        common_param: CommonWriterParam,
        #[serde(flatten)]
        param: json::JsonWriterParam,
    },
    #[serde(rename = "citygml")]
    CityGml {
        #[serde(flatten)]
        common_param: CommonWriterParam,
        #[serde(flatten)]
        param: citygml::CityGmlWriterParam,
    },
}

#[derive(Debug, Clone)]
enum CompiledFeatureWriterParam {
    Csv {
        common_param: CommonWriterCompiledParam,
    },
    Tsv {
        common_param: CommonWriterCompiledParam,
    },
    Json {
        common_param: CommonWriterCompiledParam,
        param: json::CompiledJsonWriterParam,
    },
    CityGml {
        common_param: CommonWriterCompiledParam,
        lod_mask: LodMask,
        epsg_code: Option<u32>,
        pretty_print: bool,
    },
}

#[derive(Debug, Clone)]
struct CommonWriterCompiledParam {
    output: CompiledCode,
}

impl CompiledFeatureWriterParam {
    fn output(&self) -> &CompiledCode {
        match self {
            CompiledFeatureWriterParam::Csv { common_param } => &common_param.output,
            CompiledFeatureWriterParam::Tsv { common_param } => &common_param.output,
            CompiledFeatureWriterParam::Json { common_param, .. } => &common_param.output,
            CompiledFeatureWriterParam::CityGml { common_param, .. } => &common_param.output,
        }
    }
}

impl Processor for FeatureWriter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let path = self
            .params
            .output()
            .eval_string(feature, ctx.env_vars.clone())
            .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))?;
        // Validation happens at flush time via SinkOutput::new; nothing to
        // pre-check here. The buffer is keyed by the raw relative-path string.
        let buffer = self.buffer.entry(path).or_default();
        buffer.push(ctx.feature);
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        for (rel_path, features) in &self.buffer {
            // SinkOutput::new validates the path and acquires the storage backend,
            // providing the sandbox gate at flush time.
            let sink_output = reearth_flow_action_sink::SinkOutput::new(
                &ctx.sandbox_root,
                rel_path,
                &ctx.storage_resolver,
            )
            .map_err(|e| {
                FeatureProcessorError::FeatureWriter(format!(
                    "sink output {rel_path:?} rejected by sandbox: {e}"
                ))
            })?;
            let output = sink_output.uri();
            let feature: Feature = IndexMap::<Attribute, AttributeValue>::from([
                (
                    Attribute::new("filePath".to_string()),
                    AttributeValue::String(output.path().to_str().unwrap_or_default().to_string()),
                ),
                (
                    Attribute::new("rowCount".to_string()),
                    AttributeValue::Number(serde_json::Number::from(features.len())),
                ),
            ])
            .into();
            match self.params {
                CompiledFeatureWriterParam::Csv { .. } => {
                    csv::write_csv(output, Delimiter::Comma, &ctx.storage_resolver, features)?;
                }
                CompiledFeatureWriterParam::Tsv { .. } => {
                    csv::write_csv(output, Delimiter::Tab, &ctx.storage_resolver, features)?;
                }
                CompiledFeatureWriterParam::Json {
                    common_param: _,
                    ref param,
                } => {
                    json::write_json(
                        output,
                        &param.converter,
                        &ctx.storage_resolver,
                        ctx.env_vars.clone(),
                        features,
                    )?;
                }
                CompiledFeatureWriterParam::CityGml {
                    lod_mask,
                    epsg_code,
                    pretty_print,
                    ..
                } => {
                    citygml::write_citygml(
                        output,
                        &ctx.sandbox_root,
                        features,
                        &lod_mask,
                        &epsg_code,
                        &pretty_print,
                        &ctx.storage_resolver,
                    )?;
                }
            }
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                FEATURES_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureWriter"
    }
}
