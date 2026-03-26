use std::{
    collections::{hash_map::Entry, HashMap},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    cache::executor_cache_subdir,
    errors::{BoxedError, ExecutionError},
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory},
};
use reearth_flow_types::{Attribute, Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;
use crate::ACCUMULATOR_BUFFER_BYTE_THRESHOLD;

static REQUESTOR_PORT: Lazy<Port> = Lazy::new(|| Port::new("requestor"));
static SUPPLIER_PORT: Lazy<Port> = Lazy::new(|| Port::new("supplier"));
static JOINED_PORT: Lazy<Port> = Lazy::new(|| Port::new("joined"));
static UNJOINED_REQUESTOR_PORT: Lazy<Port> = Lazy::new(|| Port::new("unjoinedRequestor"));
static UNJOINED_SUPPLIER_PORT: Lazy<Port> = Lazy::new(|| Port::new("unjoinedSupplier"));

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureJoinerFactory;

impl ProcessorFactory for FeatureJoinerFactory {
    fn name(&self) -> &str {
        "FeatureJoiner"
    }

    fn description(&self) -> &str {
        "Joins requestor and supplier features based on matching attribute values with configurable join types"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureJoinerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn get_input_ports(&self, _with: &HashMap<String, Value>) -> Vec<Port> {
        vec![REQUESTOR_PORT.clone(), SUPPLIER_PORT.clone()]
    }

    fn get_output_ports(&self, _with: &HashMap<String, Value>) -> Vec<Port> {
        vec![
            JOINED_PORT.clone(),
            UNJOINED_REQUESTOR_PORT.clone(),
            UNJOINED_SUPPLIER_PORT.clone(),
        ]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureJoinerParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::JoinerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::JoinerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::JoinerFactory(
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
                        FeatureProcessorError::JoinerFactory(format!(
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
                        FeatureProcessorError::JoinerFactory(format!(
                            "Failed to compile supplier attribute value: {e}"
                        ))
                    })?;
                Some(result)
            } else {
                None
            };

        if requestor_attribute_value.is_none() && params.requestor_attribute.is_none() {
            return Err(FeatureProcessorError::JoinerFactory(
                "At least one of requestorAttribute or requestorAttributeValue must be provided"
                    .to_string(),
            )
            .into());
        }

        if supplier_attribute_value.is_none() && params.supplier_attribute.is_none() {
            return Err(FeatureProcessorError::JoinerFactory(
                "At least one of supplierAttribute or supplierAttributeValue must be provided"
                    .to_string(),
            )
            .into());
        }

        let conflict_resolution = params
            .conflict_resolution
            .unwrap_or(ConflictResolution::SupplierWins);

        let process = FeatureJoiner {
            global_params: with,
            params: CompiledParam {
                join_type: params.join_type,
                requestor_attribute_value,
                supplier_attribute_value,
                requestor_attribute: params.requestor_attribute,
                supplier_attribute: params.supplier_attribute,
                conflict_resolution,
            },
            requestor_key_map: HashMap::new(),
            supplier_key_map: HashMap::new(),
            next_requestor_idx: 0,
            next_supplier_idx: 0,
            temp_dir: None,
            requestor_buffer: HashMap::new(),
            supplier_buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: None,
        };

        Ok(Box::new(process))
    }
}

/// # FeatureJoiner Parameters
///
/// Configuration for joining requestor and supplier features based on matching attributes or expressions.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[schemars(default)]
#[serde(rename_all = "camelCase")]
pub struct FeatureJoinerParam {
    /// Join type: inner, left, or full
    join_type: JoinType,
    /// Attributes from requestor features to use for matching (alternative to requestorAttributeValue)
    requestor_attribute: Option<Vec<Attribute>>,
    /// Attributes from supplier features to use for matching (alternative to supplierAttributeValue)
    supplier_attribute: Option<Vec<Attribute>>,
    /// Expression to evaluate for requestor feature matching values (alternative to requestorAttribute)
    requestor_attribute_value: Option<Expr>,
    /// Expression to evaluate for supplier feature matching values (alternative to supplierAttribute)
    supplier_attribute_value: Option<Expr>,
    /// Attribute conflict resolution strategy when both requestor and supplier have the same attribute
    conflict_resolution: Option<ConflictResolution>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
enum JoinType {
    /// Only emit features where a match exists
    #[default]
    Inner,
    /// Emit all requestor features (default)
    Left,
    /// Emit all features from both sides
    Full,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum ConflictResolution {
    /// Requestor attributes win on conflict
    RequestorWins,
    /// Supplier attributes win on conflict (default)
    SupplierWins,
}

pub struct FeatureJoiner {
    global_params: Option<HashMap<String, serde_json::Value>>,
    params: CompiledParam,
    // Disk-backed state
    requestor_key_map: HashMap<String, usize>,
    supplier_key_map: HashMap<String, usize>,
    next_requestor_idx: usize,
    next_supplier_idx: usize,
    temp_dir: Option<PathBuf>,
    // In-memory buffers: idx -> Vec<feature_json>
    requestor_buffer: HashMap<usize, Vec<String>>,
    supplier_buffer: HashMap<usize, Vec<String>>,
    buffer_bytes: usize,
    /// Executor ID for cache isolation, set on first process() call
    executor_id: Option<uuid::Uuid>,
}

impl std::fmt::Debug for FeatureJoiner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeatureJoiner")
            .field("requestor_keys", &self.requestor_key_map.len())
            .field("supplier_keys", &self.supplier_key_map.len())
            .field("join_type", &self.params.join_type)
            .finish_non_exhaustive()
    }
}

impl Clone for FeatureJoiner {
    fn clone(&self) -> Self {
        Self {
            global_params: self.global_params.clone(),
            params: self.params.clone(),
            requestor_key_map: HashMap::new(),
            supplier_key_map: HashMap::new(),
            next_requestor_idx: 0,
            next_supplier_idx: 0,
            temp_dir: None,
            requestor_buffer: HashMap::new(),
            supplier_buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: self.executor_id,
        }
    }
}

impl Drop for FeatureJoiner {
    fn drop(&mut self) {
        if let Some(ref dir) = self.temp_dir {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

#[derive(Debug, Clone)]
struct CompiledParam {
    join_type: JoinType,
    requestor_attribute: Option<Vec<Attribute>>,
    supplier_attribute: Option<Vec<Attribute>>,
    requestor_attribute_value: Option<rhai::AST>,
    supplier_attribute_value: Option<rhai::AST>,
    conflict_resolution: ConflictResolution,
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

/// Executor-specific engine cache folder for accumulating processors
fn engine_cache_dir(executor_id: uuid::Uuid) -> PathBuf {
    executor_cache_subdir(executor_id, "processors")
}

impl FeatureJoiner {
    fn ensure_temp_dir(&mut self) -> Result<&PathBuf, BoxedError> {
        if self.temp_dir.is_none() {
            let executor_id = self.executor_id.unwrap_or_else(uuid::Uuid::nil);
            let dir =
                engine_cache_dir(executor_id).join(format!("joiner-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&dir)?;
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

    fn write_feature_to_requestor(
        &mut self,
        idx: usize,
        feature: &Feature,
    ) -> Result<(), BoxedError> {
        let feature_json = serde_json::to_string(feature)?;
        self.buffer_bytes += feature_json.len();
        self.requestor_buffer
            .entry(idx)
            .or_default()
            .push(feature_json);

        if self.buffer_bytes >= ACCUMULATOR_BUFFER_BYTE_THRESHOLD {
            self.flush_buffer()?;
        }
        Ok(())
    }

    fn write_feature_to_supplier(
        &mut self,
        idx: usize,
        feature: &Feature,
    ) -> Result<(), BoxedError> {
        let feature_json = serde_json::to_string(feature)?;
        self.buffer_bytes += feature_json.len();
        self.supplier_buffer
            .entry(idx)
            .or_default()
            .push(feature_json);

        if self.buffer_bytes >= ACCUMULATOR_BUFFER_BYTE_THRESHOLD {
            self.flush_buffer()?;
        }
        Ok(())
    }

    fn flush_buffer(&mut self) -> Result<(), BoxedError> {
        if self.requestor_buffer.is_empty() && self.supplier_buffer.is_empty() {
            return Ok(());
        }

        self.ensure_temp_dir()?;

        // Flush requestor buffer
        for (idx, entries) in std::mem::take(&mut self.requestor_buffer) {
            let path = self.requestor_file_path(idx);
            let file = File::options().create(true).append(true).open(path)?;
            let mut writer = BufWriter::new(file);
            for feature_json in entries {
                writer.write_all(feature_json.as_bytes())?;
                writer.write_all(b"\n")?;
            }
            writer.flush()?;
        }

        // Flush supplier buffer
        for (idx, entries) in std::mem::take(&mut self.supplier_buffer) {
            let path = self.supplier_file_path(idx);
            let file = File::options().create(true).append(true).open(path)?;
            let mut writer = BufWriter::new(file);
            for feature_json in entries {
                writer.write_all(feature_json.as_bytes())?;
                writer.write_all(b"\n")?;
            }
            writer.flush()?;
        }

        self.buffer_bytes = 0;
        Ok(())
    }

    fn create_joined_feature(&self, requestor: &Feature, supplier: &Feature) -> Feature {
        let mut result = requestor.clone();

        match self.params.conflict_resolution {
            ConflictResolution::SupplierWins => {
                // Supplier attributes override requestor attributes on conflict
                result
                    .attributes_mut()
                    .extend((*supplier.attributes).clone());
            }
            ConflictResolution::RequestorWins => {
                // Only add supplier attributes that don't exist in requestor
                for (key, value) in supplier.attributes.iter() {
                    result
                        .attributes_mut()
                        .entry(key.clone())
                        .or_insert(value.clone());
                }
            }
        }

        result
    }
}

impl Processor for FeatureJoiner {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // Capture executor_id on first process call for cache isolation
        if self.executor_id.is_none() {
            self.executor_id = Some(fw.executor_id());
        }

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

                let idx = match self
                    .requestor_key_map
                    .entry(requestor_attribute_value.clone())
                {
                    Entry::Occupied(entry) => *entry.get(),
                    Entry::Vacant(entry) => {
                        let idx = self.next_requestor_idx;
                        self.next_requestor_idx += 1;
                        entry.insert(idx);
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

                let idx = match self
                    .supplier_key_map
                    .entry(supplier_attribute_value.clone())
                {
                    Entry::Occupied(entry) => *entry.get(),
                    Entry::Vacant(entry) => {
                        let idx = self.next_supplier_idx;
                        self.next_supplier_idx += 1;
                        entry.insert(idx);
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
        // Flush any remaining buffered data to disk
        self.flush_buffer()?;

        let temp_dir = match &self.temp_dir {
            Some(d) => d.clone(),
            None => return Ok(()),
        };

        let joined_path = temp_dir.join("output_joined.jsonl");
        let unjoined_requestor_path = temp_dir.join("output_unjoined_requestor.jsonl");
        let unjoined_supplier_path = temp_dir.join("output_unjoined_supplier.jsonl");

        let mut joined_writer = BufWriter::new(File::create(&joined_path)?);
        let mut unjoined_requestor_writer = BufWriter::new(File::create(&unjoined_requestor_path)?);
        let mut unjoined_supplier_writer = BufWriter::new(File::create(&unjoined_supplier_path)?);

        let mut joined_count: usize = 0;
        let mut unjoined_requestor_count: usize = 0;
        let mut unjoined_supplier_count: usize = 0;

        // Process all requestor keys
        for (request_value, &req_idx) in self.requestor_key_map.iter() {
            let requestor_features = read_features_from_file(&self.requestor_file_path(req_idx))?;

            match self.supplier_key_map.get(request_value) {
                Some(&sup_idx) => {
                    // MATCH FOUND - many-to-many join
                    let supplier_features =
                        read_features_from_file(&self.supplier_file_path(sup_idx))?;

                    // Generate one output feature per combination (many-to-many)
                    for request_feature in requestor_features.iter() {
                        for supplier_feature in supplier_features.iter() {
                            let joined =
                                self.create_joined_feature(request_feature, supplier_feature);
                            serde_json::to_writer(&mut joined_writer, &joined)?;
                            joined_writer.write_all(b"\n")?;
                            joined_count += 1;
                        }
                    }
                }
                None => {
                    // NO MATCH
                    match self.params.join_type {
                        JoinType::Inner => {
                            // Inner join: drop features with no match
                            // (no output)
                        }
                        JoinType::Left | JoinType::Full => {
                            // Left/Full join: emit unmatched requestors to unjoined_requestor
                            for request_feature in requestor_features.iter() {
                                serde_json::to_writer(
                                    &mut unjoined_requestor_writer,
                                    request_feature,
                                )?;
                                unjoined_requestor_writer.write_all(b"\n")?;
                                unjoined_requestor_count += 1;
                            }
                        }
                    }
                }
            }
        }

        // For Full Join: Emit unmatched suppliers
        if matches!(self.params.join_type, JoinType::Full) {
            for (supplier_value, &sup_idx) in self.supplier_key_map.iter() {
                // Check if this supplier key has any matching requestor
                let has_match = self.requestor_key_map.contains_key(supplier_value);

                if !has_match {
                    let supplier_features =
                        read_features_from_file(&self.supplier_file_path(sup_idx))?;
                    for supplier_feature in supplier_features.iter() {
                        serde_json::to_writer(&mut unjoined_supplier_writer, supplier_feature)?;
                        unjoined_supplier_writer.write_all(b"\n")?;
                        unjoined_supplier_count += 1;
                    }
                }
            }
        }

        // Flush all writers
        joined_writer.flush()?;
        unjoined_requestor_writer.flush()?;
        unjoined_supplier_writer.flush()?;

        drop(joined_writer);
        drop(unjoined_requestor_writer);
        drop(unjoined_supplier_writer);

        let context = ctx.as_context();

        // Send output files via forwarder
        if joined_count > 0 {
            fw.send_file(joined_path, JOINED_PORT.clone(), context.clone());
        }
        if unjoined_requestor_count > 0 {
            fw.send_file(
                unjoined_requestor_path,
                UNJOINED_REQUESTOR_PORT.clone(),
                context.clone(),
            );
        }
        if unjoined_supplier_count > 0 {
            fw.send_file(
                unjoined_supplier_path,
                UNJOINED_SUPPLIER_PORT.clone(),
                context,
            );
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureJoiner"
    }
}
