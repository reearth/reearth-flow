pub(super) mod citygml;
pub(super) mod citygml2;
pub(super) mod citygml3;
mod csv;
mod json;

use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Code, CompiledCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors;
use super::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureReaderFactory;

impl ProcessorFactory for FeatureReaderFactory {
    fn name(&self) -> &str {
        "FeatureReader"
    }

    fn description(&self) -> &str {
        "Reads features from various file formats (CSV, TSV, JSON) with configurable parsing options"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureReaderParam))
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
        let params: FeatureReaderParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FileReaderFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FileReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FileReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        match params {
            FeatureReaderParam::Csv {
                common_param,
                param,
                encoding,
            } => {
                let compiled_common_param = CompiledCommonReaderParam {
                    dataset: common_param
                        .dataset
                        .compile()
                        .map_err(|e| FeatureProcessorError::FileReaderFactory(format!("{e:?}")))?,
                };
                Ok(Box::new(FeatureReader {
                    params: CompiledFeatureReaderParam::Csv {
                        common_param: compiled_common_param,
                        param,
                        encoding,
                    },
                }))
            }
            FeatureReaderParam::Tsv {
                common_param,
                param,
                encoding,
            } => {
                let compiled_common_param = CompiledCommonReaderParam {
                    dataset: common_param
                        .dataset
                        .compile()
                        .map_err(|e| FeatureProcessorError::FileReaderFactory(format!("{e:?}")))?,
                };
                Ok(Box::new(FeatureReader {
                    params: CompiledFeatureReaderParam::Tsv {
                        common_param: compiled_common_param,
                        param,
                        encoding,
                    },
                }))
            }
            FeatureReaderParam::Json { common_param } => {
                let compiled_common_param = CompiledCommonReaderParam {
                    dataset: common_param
                        .dataset
                        .compile()
                        .map_err(|e| FeatureProcessorError::FileReaderFactory(format!("{e:?}")))?,
                };
                Ok(Box::new(FeatureReader {
                    params: CompiledFeatureReaderParam::Json {
                        common_param: compiled_common_param,
                    },
                }))
            }
        }
    }
}

#[derive(Debug, Clone)]
struct FeatureReader {
    params: CompiledFeatureReaderParam,
}

/// # Common Reader Parameters
///
/// Shared configuration for all feature reader formats.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct CommonReaderParam {
    /// # Dataset
    /// Path or expression to the dataset file to be read
    dataset: Code,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "format")]
enum FeatureReaderParam {
    #[serde(rename = "csv")]
    Csv {
        #[serde(flatten)]
        common_param: CommonReaderParam,
        #[serde(flatten)]
        param: csv::CsvReaderParam,
        /// # Character Encoding
        ///
        /// Character encoding for the CSV file.
        /// If not specified, defaults to UTF-8.
        /// Supported: UTF-8, Shift-JIS, EUC-JP, Windows-1252, ISO-8859-1, etc.
        encoding: Option<String>,
    },
    #[serde(rename = "tsv")]
    Tsv {
        #[serde(flatten)]
        common_param: CommonReaderParam,
        #[serde(flatten)]
        param: csv::CsvReaderParam,
        /// # Character Encoding
        ///
        /// Character encoding for the TSV file.
        /// If not specified, defaults to UTF-8.
        /// Supported: UTF-8, Shift-JIS, EUC-JP, Windows-1252, ISO-8859-1, etc.
        encoding: Option<String>,
    },
    #[serde(rename = "json")]
    Json {
        #[serde(flatten)]
        common_param: CommonReaderParam,
    },
}

#[derive(Debug, Clone)]
enum CompiledFeatureReaderParam {
    Csv {
        common_param: CompiledCommonReaderParam,
        param: csv::CsvReaderParam,
        encoding: Option<String>,
    },
    Tsv {
        common_param: CompiledCommonReaderParam,
        param: csv::CsvReaderParam,
        encoding: Option<String>,
    },
    Json {
        common_param: CompiledCommonReaderParam,
    },
}

#[derive(Debug, Clone)]
struct CompiledCommonReaderParam {
    dataset: CompiledCode,
}

impl Processor for FeatureReader {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        match self {
            FeatureReader {
                params:
                    CompiledFeatureReaderParam::Csv {
                        common_param,
                        param,
                        encoding,
                    },
            } => csv::read_csv(
                reearth_flow_common::csv::Delimiter::Comma,
                common_param,
                param,
                encoding.as_deref(),
                ctx,
                fw,
            )
            .map_err(|e| e.into()),
            FeatureReader {
                params:
                    CompiledFeatureReaderParam::Tsv {
                        common_param,
                        param,
                        encoding,
                    },
            } => csv::read_csv(
                reearth_flow_common::csv::Delimiter::Tab,
                common_param,
                param,
                encoding.as_deref(),
                ctx,
                fw,
            )
            .map_err(|e| e.into()),
            FeatureReader {
                params: CompiledFeatureReaderParam::Json { common_param },
            } => json::read_json(ctx, fw, common_param).map_err(|e| e.into()),
        }
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureReader"
    }
}
