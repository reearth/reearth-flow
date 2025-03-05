pub(super) mod citygml;
mod csv;
mod json;

use std::{collections::HashMap, sync::Arc};

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::Expr;
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
        "Reads features from various formats"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureReaderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureReaderParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FileReaderFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FileReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FileReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        match params {
            FeatureReaderParam::Csv {
                common_param,
                param,
            } => {
                let common_param = CompiledCommonReaderParam {
                    expr: expr_engine
                        .compile(common_param.dataset.as_ref())
                        .map_err(|e| {
                            FeatureProcessorError::FileReaderFactory(format!("{:?}", e))
                        })?,
                };
                let process = FeatureReader {
                    global_params: with,
                    params: CompiledFeatureReaderParam::Csv {
                        common_param,
                        param,
                    },
                };
                Ok(Box::new(process))
            }
            FeatureReaderParam::Tsv {
                common_param,
                param,
            } => {
                let common_param = CompiledCommonReaderParam {
                    expr: expr_engine
                        .compile(common_param.dataset.as_ref())
                        .map_err(|e| {
                            FeatureProcessorError::FileReaderFactory(format!("{:?}", e))
                        })?,
                };
                let process = FeatureReader {
                    global_params: with,
                    params: CompiledFeatureReaderParam::Tsv {
                        common_param,
                        param,
                    },
                };
                Ok(Box::new(process))
            }
            FeatureReaderParam::Json { common_param } => {
                let common_param = CompiledCommonReaderParam {
                    expr: expr_engine
                        .compile(common_param.dataset.as_ref())
                        .map_err(|e| {
                            FeatureProcessorError::FileReaderFactory(format!("{:?}", e))
                        })?,
                };
                let process = FeatureReader {
                    global_params: with,
                    params: CompiledFeatureReaderParam::Json { common_param },
                };
                Ok(Box::new(process))
            }
        }
    }
}

#[derive(Debug, Clone)]
struct FeatureReader {
    global_params: Option<HashMap<String, serde_json::Value>>,
    params: CompiledFeatureReaderParam,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct CommonReaderParam {
    /// # Dataset
    dataset: Expr,
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
    },
    #[serde(rename = "tsv")]
    Tsv {
        #[serde(flatten)]
        common_param: CommonReaderParam,
        #[serde(flatten)]
        param: csv::CsvReaderParam,
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
    },
    Tsv {
        common_param: CompiledCommonReaderParam,
        param: csv::CsvReaderParam,
    },
    Json {
        common_param: CompiledCommonReaderParam,
    },
}

#[derive(Debug, Clone)]
struct CompiledCommonReaderParam {
    expr: rhai::AST,
}

impl Processor for FeatureReader {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        match self {
            FeatureReader {
                global_params,
                params:
                    CompiledFeatureReaderParam::Csv {
                        common_param,
                        param,
                    },
            } => csv::read_csv(
                reearth_flow_common::csv::Delimiter::Comma,
                global_params,
                common_param,
                param,
                ctx,
                fw,
            )
            .map_err(|e| e.into()),
            FeatureReader {
                global_params,
                params:
                    CompiledFeatureReaderParam::Tsv {
                        common_param,
                        param,
                    },
            } => csv::read_csv(
                reearth_flow_common::csv::Delimiter::Tab,
                global_params,
                common_param,
                param,
                ctx,
                fw,
            )
            .map_err(|e| e.into()),
            FeatureReader {
                global_params,
                params: CompiledFeatureReaderParam::Json { common_param },
            } => json::read_json(ctx, fw, global_params, common_param).map_err(|e| e.into()),
        }
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureReader"
    }
}
