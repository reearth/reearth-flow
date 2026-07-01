use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Code, CompiledCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::citygml_parser::parser::Parser;
#[cfg(not(feature = "new-geometry"))]
use crate::citygml_parser::pipeline::build_features;
use crate::feature::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub(crate) struct FeatureCityGml3ReaderFactory;

impl ProcessorFactory for FeatureCityGml3ReaderFactory {
    fn name(&self) -> &str {
        "FeatureCityGml3Reader"
    }

    fn description(&self) -> &str {
        "Reads CityGML 3.0 files: resolves gml:id references and xlink:href links across files"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureCityGml3ReaderParam))
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
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureCityGml3ReaderParam = if let Some(ref with) = with {
            let value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FileCityGml3ReaderFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FileCityGml3ReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FileCityGml3ReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let dataset = params
            .dataset
            .compile()
            .map_err(|e| FeatureProcessorError::FileCityGml3ReaderFactory(format!("{e:?}")))?;

        let extract_tags: HashSet<String> = params.extract_tags.into_iter().collect();

        Ok(Box::new(FeatureCityGml3Reader {
            dataset,
            extract_tags,
            parser: Parser::new(),
        }))
    }
}

/// # FeatureCityGml3Reader Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCityGml3ReaderParam {
    /// # Dataset
    /// Path expression resolving to the CityGML 3.0 file to read.
    dataset: Code,
    /// # Extract Tags
    /// Feature type names to flatten as individual features. Accepts qualified (`bldg:Building`),
    /// local (`Building`), or Clark notation (`{http://…}Building`). Empty means emit all
    /// top-level city objects unchanged.
    #[serde(default)]
    extract_tags: Vec<String>,
}

pub struct FeatureCityGml3Reader {
    dataset: CompiledCode,
    extract_tags: HashSet<String>,
    parser: Parser,
}

impl std::fmt::Debug for FeatureCityGml3Reader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeatureCityGml3Reader")
            .field("parser", &self.parser)
            .finish_non_exhaustive()
    }
}

impl Clone for FeatureCityGml3Reader {
    fn clone(&self) -> Self {
        Self {
            dataset: self.dataset.clone(),
            extract_tags: self.extract_tags.clone(),
            parser: Parser::new(),
        }
    }
}

impl Processor for FeatureCityGml3Reader {
    fn num_threads(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let path = self
            .dataset
            .eval_string(&ctx.feature, ctx.env_vars.clone())
            .map_err(|e| {
                FeatureProcessorError::FileCityGml3Reader(format!("Failed to eval dataset: {e:?}"))
            })?;

        let uri = Uri::from_str(&path).map_err(|e| {
            FeatureProcessorError::FileCityGml3Reader(format!("Invalid URI `{path}`: {e}"))
        })?;
        let source_url: Url = uri.clone().into();

        let storage = ctx.storage_resolver.resolve(&uri).map_err(|e| {
            FeatureProcessorError::FileCityGml3Reader(format!("Storage resolve error: {e}"))
        })?;
        let bytes = storage.get_sync(uri.path().as_path()).map_err(|e| {
            FeatureProcessorError::FileCityGml3Reader(format!("File read error: {e}"))
        })?;

        self.parser
            .parse(&bytes, &source_url)
            .map_err(|e| FeatureProcessorError::FileCityGml3Reader(format!("{e}")))?;
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        for feature in build_features(std::mem::take(&mut self.parser), &self.extract_tags) {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                DEFAULT_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureCityGml3Reader"
    }
}
