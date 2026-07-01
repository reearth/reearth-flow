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
pub(crate) struct FeatureCityGml2ReaderFactory;

impl ProcessorFactory for FeatureCityGml2ReaderFactory {
    fn name(&self) -> &str {
        "FeatureCityGml2Reader"
    }

    fn description(&self) -> &str {
        "Reads CityGML 2.0 files: resolves gml:id references and xlink:href links across files"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureCityGml2ReaderParam))
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
        let params: FeatureCityGml2ReaderParam = if let Some(ref with) = with {
            let value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FileCityGml2ReaderFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FileCityGml2ReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FileCityGml2ReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let dataset = params
            .dataset
            .compile()
            .map_err(|e| FeatureProcessorError::FileCityGml2ReaderFactory(format!("{e:?}")))?;

        let extract_tags: HashSet<String> = params.extract_tags.into_iter().collect();

        Ok(Box::new(FeatureCityGml2Reader {
            dataset,
            extract_tags,
            parser: Parser::new(),
        }))
    }
}

/// # FeatureCityGml2Reader Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCityGml2ReaderParam {
    /// # Dataset
    /// Path expression resolving to the CityGML 2.0 file to read.
    dataset: Code,
    /// # Extract Tags
    /// Feature type names to flatten as individual features. Accepts qualified (`bldg:Building`),
    /// local (`Building`), or Clark notation (`{http://…}Building`). Empty means emit all
    /// top-level city objects unchanged.
    #[serde(default)]
    extract_tags: Vec<String>,
}

pub struct FeatureCityGml2Reader {
    dataset: CompiledCode,
    extract_tags: HashSet<String>,
    parser: Parser,
}

impl std::fmt::Debug for FeatureCityGml2Reader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeatureCityGml2Reader")
            .field("parser", &self.parser)
            .finish_non_exhaustive()
    }
}

impl Clone for FeatureCityGml2Reader {
    fn clone(&self) -> Self {
        Self {
            dataset: self.dataset.clone(),
            extract_tags: self.extract_tags.clone(),
            parser: Parser::new(),
        }
    }
}

impl Processor for FeatureCityGml2Reader {
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
                FeatureProcessorError::FileCityGml2Reader(format!("Failed to eval dataset: {e:?}"))
            })?;

        let uri = Uri::from_str(&path).map_err(|e| {
            FeatureProcessorError::FileCityGml2Reader(format!("Invalid URI `{path}`: {e}"))
        })?;
        let source_url: Url = uri.clone().into();

        let storage = ctx.storage_resolver.resolve(&uri).map_err(|e| {
            FeatureProcessorError::FileCityGml2Reader(format!("Storage resolve error: {e}"))
        })?;
        let bytes = storage.get_sync(uri.path().as_path()).map_err(|e| {
            FeatureProcessorError::FileCityGml2Reader(format!("File read error: {e}"))
        })?;

        self.parser
            .parse(&bytes, &source_url)
            .map_err(|e| FeatureProcessorError::FileCityGml2Reader(format!("{e}")))?;
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
        "FeatureCityGml2Reader"
    }
}
