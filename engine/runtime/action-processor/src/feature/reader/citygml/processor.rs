use std::{collections::HashMap, str::FromStr, sync::Arc};

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
use url::Url;

use crate::feature::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub(crate) struct FeatureCityGmlReaderFactory;

impl ProcessorFactory for FeatureCityGmlReaderFactory {
    fn name(&self) -> &str {
        "FeatureCityGmlReader"
    }

    fn description(&self) -> &str {
        "Reads and processes features from CityGML files with optional flattening"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureCityGmlReaderParam))
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
        let params: FeatureCityGmlReaderParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FileCityGmlReaderFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FileCityGmlReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FileCityGmlReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let codelists_path = params
            .codelists_path
            .as_ref()
            .map(|p| expr_engine.compile(p.as_ref()))
            .transpose()
            .map_err(|e| FeatureProcessorError::FileCityGmlReaderFactory(format!("{e:?}")))?;
        let compiled_params = CompiledFeatureCityGmlReaderParam {
            dataset: expr_engine
                .compile(params.dataset.as_ref())
                .map_err(|e| FeatureProcessorError::FileCityGmlReaderFactory(format!("{e:?}")))?,
            original_dataset: params.dataset.clone(),
            flatten: params.flatten,
            codelists_path,
        };
        let process = FeatureCityGmlReader {
            global_params: with,
            params: compiled_params,
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct FeatureCityGmlReader {
    global_params: Option<HashMap<String, serde_json::Value>>,
    params: CompiledFeatureCityGmlReaderParam,
}

/// # FeatureCityGmlReader Parameters
///
/// Configuration for reading and processing CityGML files as features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCityGmlReaderParam {
    /// # Dataset
    /// Path or expression to the CityGML dataset file to be read
    dataset: Expr,
    /// # Flatten
    /// Whether to flatten the hierarchical structure of the CityGML data
    flatten: Option<bool>,
    /// # Codelists Path
    /// Optional path to the codelists directory for resolving codelist values
    codelists_path: Option<Expr>,
}

#[derive(Debug, Clone)]
struct CompiledFeatureCityGmlReaderParam {
    dataset: rhai::AST,
    original_dataset: Expr,
    flatten: Option<bool>,
    codelists_path: Option<rhai::AST>,
}

impl Processor for FeatureCityGmlReader {
    fn num_threads(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let fw = fw.clone();
        let feature = ctx.feature.clone();
        let ctx = ctx.as_context();
        let global_params = self.global_params.clone();
        let dataset = self.params.dataset.clone();
        let original_dataset = self.params.original_dataset.clone();
        let flatten = self.params.flatten;
        let codelists_url = self.params.codelists_path.clone().and_then(|ast| {
            let expr_engine = Arc::clone(&ctx.expr_engine);
            let scope = feature.new_scope(expr_engine.clone(), &global_params);
            scope
                .eval_ast::<String>(&ast)
                .ok()
                .and_then(|s| Url::from_str(&s).ok())
        });
        super::reader::read_citygml(
            ctx,
            fw,
            feature,
            dataset,
            original_dataset,
            flatten,
            global_params,
            codelists_url,
        )
        .map_err(|e| e.into())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureCityGmlReader"
    }
}
