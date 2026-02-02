use std::{
    collections::{hash_map::Entry, HashMap},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use once_cell::sync::Lazy;
use reearth_flow_common::dir::project_temp_dir;
use reearth_flow_runtime::{
    errors::{BoxedError, ExecutionError},
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory},
};
use reearth_flow_types::{Attribute, Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::{self, FeatureProcessorError};

static REQUESTOR_PORT: Lazy<Port> = Lazy::new(|| Port::new("requestor"));
static SUPPLIER_PORT: Lazy<Port> = Lazy::new(|| Port::new("supplier"));
static MERGED_PORT: Lazy<Port> = Lazy::new(|| Port::new("merged"));
static UNMERGED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unmerged"));

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureMergerFactory;

impl ProcessorFactory for FeatureMergerFactory {
    fn name(&self) -> &str {
        "FeatureMerger"
    }

    fn description(&self) -> &str {
        "Merges requestor and supplier features based on matching attribute values"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureMergerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![REQUESTOR_PORT.clone(), SUPPLIER_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![MERGED_PORT.clone(), UNMERGED_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureMergerParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::MergerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::MergerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::MergerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let requestor_attribute_value =
            if let Some(requestor_attribute_value) = params.requestor_attribute_value {
                let result = expr_engine
                    .compile(requestor_attribute_value.as_ref())
                    .map_err(|e| {
                        FeatureProcessorError::MergerFactory(format!(
                            "Failed to compile requestor attribute value: {e}"
                        ))
                    })?;
                Some(result)
            } else {
                None
            };
        let supplier_attribute_value =
            if let Some(supplier_attribute_value) = params.supplier_attribute_value {
                let result = expr_engine
                    .compile(supplier_attribute_value.as_ref())
                    .map_err(|e| {
                        FeatureProcessorError::MergerFactory(format!(
                            "Failed to compile supplier attribute value: {e}"
                        ))
                    })?;
                Some(result)
            } else {
                None
            };
        if requestor_attribute_value.is_none() && params.requestor_attribute.is_none() {
            return Err(FeatureProcessorError::MergerFactory(
                "At least one of requestor_attribute_value or requestor_attribute must be provided"
                    .to_string(),
            )
            .into());
        }
        if supplier_attribute_value.is_none() && params.supplier_attribute.is_none() {
            return Err(FeatureProcessorError::MergerFactory(
                "At least one of supplier_attribute_value or supplier_attribute must be provided"
                    .to_string(),
            )
            .into());
        }
        let process = FeatureMerger {
            global_params: with,
            params: CompiledParam {
                requestor_attribute_value,
                supplier_attribute_value,
                requestor_attribute: params.requestor_attribute,
                supplier_attribute: params.supplier_attribute,
                complete_grouped: params.complete_grouped.unwrap_or(false),
            },
            requestor_key_map: HashMap::new(),
            supplier_key_map: HashMap::new(),
            requestor_complete: HashMap::new(),
            supplier_complete: HashMap::new(),
            requestor_writers: HashMap::new(),
            supplier_writers: HashMap::new(),
            next_requestor_idx: 0,
            next_supplier_idx: 0,
            requestor_before_value: None,
            supplier_before_value: None,
            temp_dir: None,
        };
        Ok(Box::new(process))
    }
}

/// # FeatureMerger Parameters
///
/// Configuration for merging requestor and supplier features based on matching attributes or expressions.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureMergerParam {
    /// Attributes from requestor features to use for matching (alternative to requestor_attribute_value)
    requestor_attribute: Option<Vec<Attribute>>,
    /// Attributes from supplier features to use for matching (alternative to supplier_attribute_value)
    supplier_attribute: Option<Vec<Attribute>>,
    /// Expression to evaluate for requestor feature matching values (alternative to requestor_attribute)
    requestor_attribute_value: Option<Expr>,
    /// Expression to evaluate for supplier feature matching values (alternative to supplier_attribute)
    supplier_attribute_value: Option<Expr>,
    /// Whether to complete grouped features before processing the next group
    complete_grouped: Option<bool>,
}

pub struct FeatureMerger {
    global_params: Option<HashMap<String, serde_json::Value>>,
    params: CompiledParam,
    // Disk-backed state
    requestor_key_map: HashMap<String, usize>,
    supplier_key_map: HashMap<String, usize>,
    requestor_complete: HashMap<String, bool>,
    supplier_complete: HashMap<String, bool>,
    requestor_writers: HashMap<usize, BufWriter<File>>,
    supplier_writers: HashMap<usize, BufWriter<File>>,
    next_requestor_idx: usize,
    next_supplier_idx: usize,
    requestor_before_value: Option<String>,
    supplier_before_value: Option<String>,
    temp_dir: Option<PathBuf>,
}

impl std::fmt::Debug for FeatureMerger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeatureMerger")
            .field("requestor_keys", &self.requestor_key_map.len())
            .field("supplier_keys", &self.supplier_key_map.len())
            .finish_non_exhaustive()
    }
}

impl Clone for FeatureMerger {
    fn clone(&self) -> Self {
        Self {
            global_params: self.global_params.clone(),
            params: self.params.clone(),
            requestor_key_map: HashMap::new(),
            supplier_key_map: HashMap::new(),
            requestor_complete: HashMap::new(),
            supplier_complete: HashMap::new(),
            requestor_writers: HashMap::new(),
            supplier_writers: HashMap::new(),
            next_requestor_idx: 0,
            next_supplier_idx: 0,
            requestor_before_value: None,
            supplier_before_value: None,
            temp_dir: None,
        }
    }
}

impl Drop for FeatureMerger {
    fn drop(&mut self) {
        if let Some(ref dir) = self.temp_dir {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

#[derive(Debug, Clone)]
struct CompiledParam {
    requestor_attribute: Option<Vec<Attribute>>,
    supplier_attribute: Option<Vec<Attribute>>,
    requestor_attribute_value: Option<rhai::AST>,
    supplier_attribute_value: Option<rhai::AST>,
    complete_grouped: bool,
}

fn read_features_from_file(path: &Path) -> Result<Vec<Feature>, BoxedError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut features = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if !line.is_empty() {
            features.push(serde_json::from_str(&line)?);
        }
    }
    Ok(features)
}

impl FeatureMerger {
    fn ensure_temp_dir(&mut self) -> Result<&PathBuf, BoxedError> {
        if self.temp_dir.is_none() {
            let dir = project_temp_dir(uuid::Uuid::new_v4().to_string().as_str())?;
            std::fs::create_dir_all(dir.join("requestor"))?;
            std::fs::create_dir_all(dir.join("supplier"))?;
            self.temp_dir = Some(dir);
        }
        Ok(self.temp_dir.as_ref().unwrap())
    }

    fn requestor_file_path(&self, idx: usize) -> PathBuf {
        self.temp_dir
            .as_ref()
            .unwrap()
            .join(format!("requestor/{idx:06}.jsonl"))
    }

    fn supplier_file_path(&self, idx: usize) -> PathBuf {
        self.temp_dir
            .as_ref()
            .unwrap()
            .join(format!("supplier/{idx:06}.jsonl"))
    }

    fn ensure_requestor_writer(&mut self, idx: usize) -> Result<&mut BufWriter<File>, BoxedError> {
        if !self.requestor_writers.contains_key(&idx) {
            let path = self.requestor_file_path(idx);
            let file = File::options().create(true).append(true).open(path)?;
            self.requestor_writers.insert(idx, BufWriter::new(file));
        }
        Ok(self.requestor_writers.get_mut(&idx).unwrap())
    }

    fn ensure_supplier_writer(&mut self, idx: usize) -> Result<&mut BufWriter<File>, BoxedError> {
        if !self.supplier_writers.contains_key(&idx) {
            let path = self.supplier_file_path(idx);
            let file = File::options().create(true).append(true).open(path)?;
            self.supplier_writers.insert(idx, BufWriter::new(file));
        }
        Ok(self.supplier_writers.get_mut(&idx).unwrap())
    }

    fn write_feature_to_requestor(
        &mut self,
        idx: usize,
        feature: &Feature,
    ) -> Result<(), BoxedError> {
        let writer = self.ensure_requestor_writer(idx)?;
        serde_json::to_writer(&mut *writer, feature)?;
        writer.write_all(b"\n")?;
        Ok(())
    }

    fn write_feature_to_supplier(
        &mut self,
        idx: usize,
        feature: &Feature,
    ) -> Result<(), BoxedError> {
        let writer = self.ensure_supplier_writer(idx)?;
        serde_json::to_writer(&mut *writer, feature)?;
        writer.write_all(b"\n")?;
        Ok(())
    }

    fn flush_and_drop_writers_for_key(&mut self, key: &str) {
        if let Some(idx) = self.requestor_key_map.get(key) {
            if let Some(mut w) = self.requestor_writers.remove(idx) {
                let _ = w.flush();
            }
        }
        if let Some(idx) = self.supplier_key_map.get(key) {
            if let Some(mut w) = self.supplier_writers.remove(idx) {
                let _ = w.flush();
            }
        }
    }

    fn change_group(&mut self, ctx: Context, fw: &ProcessorChannelForwarder) -> errors::Result<()> {
        if !self.params.complete_grouped {
            return Ok(());
        }
        let mut complete_keys = Vec::new();
        for (attribute, complete) in self.requestor_complete.iter() {
            if !complete {
                continue;
            }
            let Some(supplier_complete) = self.supplier_complete.get(attribute) else {
                continue;
            };
            if !*supplier_complete {
                continue;
            }
            complete_keys.push(attribute.clone());
        }
        for attribute_value in complete_keys.iter() {
            self.flush_and_drop_writers_for_key(attribute_value);

            let requestor_idx = match self.requestor_key_map.remove(attribute_value) {
                Some(idx) => idx,
                None => return Ok(()),
            };
            self.requestor_complete.remove(attribute_value);
            let requestor_features =
                read_features_from_file(&self.requestor_file_path(requestor_idx))
                    .map_err(|e| FeatureProcessorError::Merger(format!("Failed to read requestor features: {e}")))?;
            let _ = std::fs::remove_file(self.requestor_file_path(requestor_idx));

            let supplier_idx = match self.supplier_key_map.remove(attribute_value) {
                Some(idx) => {
                    self.supplier_complete.remove(attribute_value);
                    idx
                }
                None => {
                    for request_feature in requestor_features.iter() {
                        fw.send(
                            ctx.as_executor_context(request_feature.clone(), UNMERGED_PORT.clone()),
                        );
                    }
                    return Ok(());
                }
            };
            let supplier_features =
                read_features_from_file(&self.supplier_file_path(supplier_idx))
                    .map_err(|e| FeatureProcessorError::Merger(format!("Failed to read supplier features: {e}")))?;
            let _ = std::fs::remove_file(self.supplier_file_path(supplier_idx));

            for request_feature in requestor_features.iter() {
                let mut merged_feature = request_feature.clone();
                for supplier_feature in supplier_features.iter() {
                    merged_feature
                        .attributes_mut()
                        .extend((*supplier_feature.attributes).clone());
                }
                fw.send(ctx.as_executor_context(merged_feature, MERGED_PORT.clone()));
            }
        }
        Ok(())
    }
}

impl Processor for FeatureMerger {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        self.ensure_temp_dir()?;
        match ctx.port {
            port if port == REQUESTOR_PORT.clone() => {
                let feature = &ctx.feature;
                let expr_engine = Arc::clone(&ctx.expr_engine);
                let requestor_attribute_value = feature.fetch_attribute_value(
                    expr_engine,
                    &self.global_params,
                    &self.params.requestor_attribute,
                    &self.params.requestor_attribute_value,
                );
                let idx = match self.requestor_key_map.entry(requestor_attribute_value.clone()) {
                    Entry::Occupied(entry) => {
                        self.requestor_before_value = Some(requestor_attribute_value.clone());
                        *entry.get()
                    }
                    Entry::Vacant(entry) => {
                        let idx = self.next_requestor_idx;
                        self.next_requestor_idx += 1;
                        entry.insert(idx);
                        self.requestor_complete.insert(requestor_attribute_value.clone(), false);
                        if self.requestor_before_value.is_some() {
                            let prev = self.requestor_before_value.clone().unwrap();
                            self.requestor_complete.insert(prev, true);
                            self.change_group(
                                Context {
                                    expr_engine: ctx.expr_engine.clone(),
                                    storage_resolver: ctx.storage_resolver.clone(),
                                    kv_store: ctx.kv_store.clone(),
                                    event_hub: ctx.event_hub.clone(),
                                },
                                fw,
                            )?;
                        }
                        self.requestor_before_value = Some(requestor_attribute_value.clone());
                        idx
                    }
                };
                self.write_feature_to_requestor(idx, feature)?;
            }
            port if port == SUPPLIER_PORT.clone() => {
                let feature = &ctx.feature;
                let expr_engine = Arc::clone(&ctx.expr_engine);
                let supplier_attribute_value = feature.fetch_attribute_value(
                    expr_engine,
                    &self.global_params,
                    &self.params.supplier_attribute,
                    &self.params.supplier_attribute_value,
                );
                let idx = match self.supplier_key_map.entry(supplier_attribute_value.clone()) {
                    Entry::Occupied(entry) => {
                        self.supplier_before_value = Some(supplier_attribute_value.clone());
                        *entry.get()
                    }
                    Entry::Vacant(entry) => {
                        let idx = self.next_supplier_idx;
                        self.next_supplier_idx += 1;
                        entry.insert(idx);
                        self.supplier_complete.insert(supplier_attribute_value.clone(), false);
                        if self.supplier_before_value.is_some() {
                            let prev = self.supplier_before_value.clone().unwrap();
                            self.supplier_complete.insert(prev, true);
                            self.change_group(
                                Context {
                                    expr_engine: ctx.expr_engine.clone(),
                                    storage_resolver: ctx.storage_resolver.clone(),
                                    kv_store: ctx.kv_store.clone(),
                                    event_hub: ctx.event_hub.clone(),
                                },
                                fw,
                            )?;
                        }
                        self.supplier_before_value = Some(supplier_attribute_value.clone());
                        idx
                    }
                };
                self.write_feature_to_supplier(idx, feature)?;
            }
            port => return Err(ExecutionError::InvalidPortHandle(port).into()),
        }
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // Flush all writers
        for (_, w) in self.requestor_writers.drain() {
            let mut w = w;
            w.flush()?;
        }
        for (_, w) in self.supplier_writers.drain() {
            let mut w = w;
            w.flush()?;
        }

        for (request_value, &req_idx) in self.requestor_key_map.iter() {
            let requestor_features = read_features_from_file(&self.requestor_file_path(req_idx))?;

            let Some(&sup_idx) = self.supplier_key_map.get(request_value) else {
                for request_feature in requestor_features.iter() {
                    fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                        &ctx,
                        request_feature.clone(),
                        UNMERGED_PORT.clone(),
                    ));
                }
                continue;
            };

            let supplier_features = read_features_from_file(&self.supplier_file_path(sup_idx))?;

            for request_feature in requestor_features.iter() {
                let mut merged_feature = request_feature.clone();
                for supplier_feature in supplier_features.iter() {
                    merged_feature
                        .attributes_mut()
                        .extend((*supplier_feature.attributes).clone());
                }
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    merged_feature,
                    MERGED_PORT.clone(),
                ));
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureMerger"
    }
}
