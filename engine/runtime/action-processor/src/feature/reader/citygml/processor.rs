use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::Arc};

use nusamai_citygml::GeometryStore;
use nusamai_plateau::appearance::AppearanceStore;
use reearth_flow_runtime::{
    cache::executor_cache_subdir,
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
use std::sync::RwLock;
use url::Url;

use crate::feature::errors::FeatureProcessorError;

use super::reader::{emit_buffered, parse_and_register};

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
            geom_registry: HashMap::new(),
            app_registry: HashMap::new(),
            store_pool: Vec::new(),
            cache_paths: Vec::new(),
            cache_dir: None,
        };
        Ok(Box::new(process))
    }
}

pub struct FeatureCityGmlReader {
    global_params: Option<HashMap<String, serde_json::Value>>,
    params: CompiledFeatureCityGmlReaderParam,
    /// Pass 1 registry: polygon URL → owning GeometryStore (needed for cross-file ref resolution)
    geom_registry: HashMap<Url, Arc<RwLock<GeometryStore>>>,
    /// Pass 1 registry: polygon URL → owning AppearanceStore
    app_registry: HashMap<Url, Arc<RwLock<AppearanceStore>>>,
    /// One entry per top-level city object parsed; indexed by store_id in the JSONL cache.
    store_pool: Vec<(Arc<RwLock<GeometryStore>>, Arc<RwLock<AppearanceStore>>)>,
    /// Per-file JSONL cache paths written during pass 1.
    cache_paths: Vec<PathBuf>,
    /// Root of the executor-specific cache directory, set on first process() call.
    cache_dir: Option<PathBuf>,
}

impl std::fmt::Debug for FeatureCityGmlReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeatureCityGmlReader")
            .field("cache_paths", &self.cache_paths.len())
            .field("store_pool", &self.store_pool.len())
            .finish_non_exhaustive()
    }
}

impl Clone for FeatureCityGmlReader {
    fn clone(&self) -> Self {
        Self {
            global_params: self.global_params.clone(),
            params: self.params.clone(),
            geom_registry: HashMap::new(),
            app_registry: HashMap::new(),
            store_pool: Vec::new(),
            cache_paths: Vec::new(),
            cache_dir: None,
        }
    }
}

impl Drop for FeatureCityGmlReader {
    fn drop(&mut self) {
        if let Some(ref dir) = self.cache_dir {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
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
        // Initialize cache directory on first call
        if self.cache_dir.is_none() {
            let executor_id = fw.executor_id();
            let dir = executor_cache_subdir(executor_id, "citygml-reader");
            std::fs::create_dir_all(&dir).map_err(|e| {
                FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
            })?;
            self.cache_dir = Some(dir);
        }
        // Pass 1: parse file, populate registries, write entities to per-file JSONL cache
        let cache_path = parse_and_register(
            ctx,
            feature,
            dataset,
            original_dataset,
            flatten,
            global_params,
            codelists_url,
            &mut self.geom_registry,
            &mut self.app_registry,
            &mut self.store_pool,
            self.cache_dir.as_deref().unwrap(),
        )
        .map_err(|e| -> BoxedError { e.into() })?;
        self.cache_paths.push(cache_path);
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let Some(cache_dir) = self.cache_dir.as_deref() else {
            return Ok(());
        };
        // Pass 2: stream per-file, resolve cross-file refs, emit
        emit_buffered(
            ctx.as_context(),
            fw,
            cache_dir,
            &self.cache_paths,
            &self.store_pool,
            &self.geom_registry,
            &self.app_registry,
        )
    }

    fn name(&self) -> &str {
        "FeatureCityGmlReader"
    }
}
