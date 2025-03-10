mod csv;
mod json;

use std::{collections::HashMap, str::FromStr, sync::Arc};

use reearth_flow_common::{csv::Delimiter, uri::Uri};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
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
        let params: FeatureWriterParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FeatureWriterFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FeatureWriterFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FeatureWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        match params {
            FeatureWriterParam::Csv { common_param } => {
                let common_param = CompiledCommonWriterParam {
                    output: expr_engine
                        .compile(common_param.output.as_ref())
                        .map_err(|e| {
                            FeatureProcessorError::FeatureWriterFactory(format!("{:?}", e))
                        })?,
                };
                let process = FeatureWriter {
                    global_params: with,
                    params: CompiledFeatureWriterParam::Csv { common_param },
                    buffer: HashMap::new(),
                };
                Ok(Box::new(process))
            }
            FeatureWriterParam::Tsv { common_param } => {
                let common_param = CompiledCommonWriterParam {
                    output: expr_engine
                        .compile(common_param.output.as_ref())
                        .map_err(|e| {
                            FeatureProcessorError::FeatureWriterFactory(format!("{:?}", e))
                        })?,
                };
                let process = FeatureWriter {
                    global_params: with,
                    params: CompiledFeatureWriterParam::Tsv { common_param },
                    buffer: HashMap::new(),
                };
                Ok(Box::new(process))
            }
            FeatureWriterParam::Json {
                common_param,
                param,
            } => {
                let common_param = CompiledCommonWriterParam {
                    output: expr_engine
                        .compile(common_param.output.as_ref())
                        .map_err(|e| {
                            FeatureProcessorError::FeatureWriterFactory(format!("{:?}", e))
                        })?,
                };
                let converter = if let Some(expr) = param.converter {
                    Some(expr_engine.compile(expr.as_ref()).map_err(|e| {
                        FeatureProcessorError::FeatureWriterFactory(format!("{:?}", e))
                    })?)
                } else {
                    None
                };
                let process = FeatureWriter {
                    global_params: with,
                    params: CompiledFeatureWriterParam::Json {
                        common_param,
                        param: json::CompiledJsonWriterParam { converter },
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
    global_params: Option<HashMap<String, serde_json::Value>>,
    params: CompiledFeatureWriterParam,
    pub(super) buffer: HashMap<Uri, Vec<Feature>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct CommonWriterParam {
    /// # Output path
    pub(super) output: Expr,
}

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
}

#[derive(Debug, Clone)]
enum CompiledFeatureWriterParam {
    Csv {
        common_param: CompiledCommonWriterParam,
    },
    Tsv {
        common_param: CompiledCommonWriterParam,
    },
    Json {
        common_param: CompiledCommonWriterParam,
        param: json::CompiledJsonWriterParam,
    },
}

#[derive(Debug, Clone)]
struct CompiledCommonWriterParam {
    output: rhai::AST,
}

impl CompiledFeatureWriterParam {
    fn output(&self) -> &rhai::AST {
        match self {
            CompiledFeatureWriterParam::Csv { common_param } => &common_param.output,
            CompiledFeatureWriterParam::Tsv { common_param } => &common_param.output,
            CompiledFeatureWriterParam::Json { common_param, .. } => &common_param.output,
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
        let output = self.params.output().clone();
        let scope = feature.new_scope(ctx.expr_engine.clone(), &self.global_params);
        let path = scope
            .eval_ast::<String>(&output)
            .map_err(|e| FeatureProcessorError::FeatureWriterFactory(format!("{:?}", e)))?;
        let output = Uri::from_str(path.as_str())?;
        let buffer = self.buffer.entry(output).or_default();
        buffer.push(ctx.feature);
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        for (output, features) in &self.buffer {
            let feature: Feature = HashMap::<Attribute, AttributeValue>::from([
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
                        &ctx.expr_engine,
                        features,
                    )?;
                }
            }
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                DEFAULT_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureWriter"
    }
}
