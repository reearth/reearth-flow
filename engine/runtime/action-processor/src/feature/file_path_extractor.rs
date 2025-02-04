use std::{collections::HashMap, str::FromStr, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_common::{dir::project_temp_dir, uri::Uri};
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{AttributeValue, Expr, Feature, FilePath};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::utils::decompressor::extract_archive;

use super::errors::FeatureProcessorError;

pub static UNFILTERED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unfiltered"));

#[derive(Debug, Clone, Default)]
pub struct FeatureFilePathExtractorFactory;

impl ProcessorFactory for FeatureFilePathExtractorFactory {
    fn name(&self) -> &str {
        "FeatureFilePathExtractor"
    }

    fn description(&self) -> &str {
        "Extracts features by file path"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureFilePathExtractorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), UNFILTERED_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: FeatureFilePathExtractorParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FilePathExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FilePathExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FilePathExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let source_dataset = expr_engine
            .compile(param.source_dataset.as_ref())
            .map_err(|e| {
                FeatureProcessorError::FilePathExtractorFactory(format!(
                    "Failed to compile `source_dataset` expression: {}",
                    e
                ))
            })?;
        let process = FeatureFilePathExtractor {
            params: FeatureFilePathExtractorCompiledParam {
                source_dataset,
                extract_archive: param.extract_archive,
            },
            with,
        };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureFilePathExtractorParam {
    source_dataset: Expr,
    extract_archive: bool,
}

#[derive(Debug, Clone)]
pub struct FeatureFilePathExtractorCompiledParam {
    source_dataset: rhai::AST,
    extract_archive: bool,
}

#[derive(Debug, Clone)]
pub struct FeatureFilePathExtractor {
    params: FeatureFilePathExtractorCompiledParam,
    with: Option<HashMap<String, Value>>,
}

impl Processor for FeatureFilePathExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let base_attributes = feature.attributes.clone();
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = feature.new_scope(expr_engine, &self.with);
        let source_dataset = scope
            .eval_ast::<String>(&self.params.source_dataset)
            .map_err(|e| {
                FeatureProcessorError::FilePathExtractor(format!(
                    "Failed to evaluate `source_dataset` expression: {}",
                    e
                ))
            })?;
        let source_dataset = Uri::from_str(source_dataset.as_str())
            .map_err(|_| FeatureProcessorError::FilePathExtractor("Invalid path".to_string()))?;
        let storage = ctx.storage_resolver.resolve(&source_dataset).map_err(|e| {
            FeatureProcessorError::FilePathExtractor(format!(
                "Failed to resolve `source_dataset` path: {}",
                e
            ))
        })?;

        if self.is_extractable_archive(&source_dataset) {
            let root_output_path = project_temp_dir(uuid::Uuid::new_v4().to_string().as_str())?;
            let root_output_path = Uri::from_str(root_output_path.to_str().ok_or(
                FeatureProcessorError::FilePathExtractor("Invalid path".to_string()),
            )?)
            .map_err(|e| {
                FeatureProcessorError::FilePathExtractor(format!(
                    "Failed to convert `root_output_path` to URI: {}",
                    e
                ))
            })?;
            let features = extract_archive(
                &source_dataset,
                &root_output_path,
                ctx.storage_resolver.clone(),
            )
            .map_err(|e| FeatureProcessorError::FilePathExtractor(format!("{:?}", e)))?
            .into_iter()
            .map(|entry| {
                let attribute_value = AttributeValue::try_from(entry)
                    .map_err(|e| FeatureProcessorError::FilePathExtractor(format!("{:?}", e)))?;
                Ok(Feature::from(attribute_value))
            })
            .collect::<super::errors::Result<Vec<_>>>()?;
            for mut feature in features {
                feature.extend(
                    base_attributes
                        .iter()
                        .filter(|(k, _)| !feature.contains_key(k))
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect::<HashMap<_, _>>(),
                );
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
        } else if source_dataset.is_dir() {
            let entries = storage
                .list_sync(Some(source_dataset.path().as_path()), true)
                .map_err(|e| FeatureProcessorError::FilePathExtractor(format!("{:?}", e)))?;
            for entry in entries {
                let attribute_value =
                    AttributeValue::try_from(FilePath::try_from(entry).unwrap_or_default())?;
                fw.send(ctx.new_with_feature_and_port(
                    Feature::from(attribute_value),
                    DEFAULT_PORT.clone(),
                ));
            }
        } else {
            let attribute_value = AttributeValue::try_from(FilePath::try_from(source_dataset)?)?;
            fw.send(
                ctx.new_with_feature_and_port(Feature::from(attribute_value), DEFAULT_PORT.clone()),
            );
        }
        Ok(())
    }

    fn finish(
        &self,
        _ctx: NodeContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureFilePathExtractor"
    }
}

impl FeatureFilePathExtractor {
    fn is_extractable_archive(&self, path: &Uri) -> bool {
        self.params.extract_archive
            && !path.is_dir()
            && path.extension().is_some()
            && matches!(path.extension().unwrap(), "zip" | "7z" | "7zip")
    }
}
