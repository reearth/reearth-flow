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
    executor_operation::{Context, ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory},
};
use reearth_flow_types::{Attribute, CityGmlGeometry, Expr, Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    errors::{self, FeatureProcessorError},
    joiner::ConflictResolution,
};
use crate::ACCUMULATOR_BUFFER_BYTE_THRESHOLD;

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
                conflict_resolution: params
                    .conflict_resolution
                    .unwrap_or(ConflictResolution::SupplierWins),
                merge_geometry: params.merge_geometry.unwrap_or(false),
            },
            requestor_key_map: HashMap::new(),
            supplier_key_map: HashMap::new(),
            requestor_complete: HashMap::new(),
            supplier_complete: HashMap::new(),
            next_requestor_idx: 0,
            next_supplier_idx: 0,
            requestor_before_value: None,
            supplier_before_value: None,
            temp_dir: None,
            requestor_buffer: HashMap::new(),
            supplier_buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: None,
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
    /// Attribute conflict resolution strategy when both requestor and supplier have the same attribute
    conflict_resolution: Option<ConflictResolution>,
    /// Whether to append the supplier's CityGML geometry to the requestor's geometry (default: false)
    merge_geometry: Option<bool>,
}

pub struct FeatureMerger {
    global_params: Option<HashMap<String, serde_json::Value>>,
    params: CompiledParam,
    // Disk-backed state
    requestor_key_map: HashMap<String, usize>,
    supplier_key_map: HashMap<String, usize>,
    requestor_complete: HashMap<String, bool>,
    supplier_complete: HashMap<String, bool>,
    next_requestor_idx: usize,
    next_supplier_idx: usize,
    requestor_before_value: Option<String>,
    supplier_before_value: Option<String>,
    temp_dir: Option<PathBuf>,
    // In-memory buffers: idx -> compressed zstd bytes (concatenated frames)
    requestor_buffer: HashMap<usize, Vec<u8>>,
    supplier_buffer: HashMap<usize, Vec<u8>>,
    buffer_bytes: usize,
    /// Executor ID for cache isolation, set on first process() call
    executor_id: Option<uuid::Uuid>,
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
            next_requestor_idx: 0,
            next_supplier_idx: 0,
            requestor_before_value: None,
            supplier_before_value: None,
            temp_dir: None,
            requestor_buffer: HashMap::new(),
            supplier_buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: self.executor_id,
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
    conflict_resolution: ConflictResolution,
    merge_geometry: bool,
}

fn read_features_from_file(path: &Path) -> Result<Vec<Feature>, BoxedError> {
    let file = File::open(path)?;
    let reader = BufReader::new(zstd::Decoder::new(file)?);
    let mut features = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if !line.is_empty() {
            features.push(serde_json::from_str(&line)?);
        }
    }
    Ok(features)
}

/// Returns `requestor` with the attributes, and optionally the CityGML geometry, of every supplier merged in.
fn merge_features(params: &CompiledParam, requestor: &Feature, suppliers: &[Feature]) -> Feature {
    let mut merged = requestor.clone();
    for supplier in suppliers {
        match params.conflict_resolution {
            ConflictResolution::SupplierWins => merged
                .attributes_mut()
                .extend((*supplier.attributes).clone()),
            ConflictResolution::RequestorWins => {
                let attributes = merged.attributes_mut();
                for (key, value) in supplier.attributes.iter() {
                    attributes
                        .entry(key.clone())
                        .or_insert_with(|| value.clone());
                }
            }
        }
        if params.merge_geometry {
            append_citygml_geometry(&mut merged, supplier);
        }
    }
    merged
}

/// Appends `supplier`'s CityGML geometry to `merged`'s, which must be CityGML or empty.
/// Other geometry kinds on either side are left untouched.
fn append_citygml_geometry(merged: &mut Feature, supplier: &Feature) {
    let GeometryValue::CityGmlGeometry(supplier_geometry) = &supplier.geometry.value else {
        return;
    };
    let geometry = merged.geometry_mut();
    match &mut geometry.value {
        GeometryValue::CityGmlGeometry(merged_geometry) => {
            merged_geometry.append(supplier_geometry.clone());
        }
        GeometryValue::None => {
            let mut merged_geometry = CityGmlGeometry::default();
            merged_geometry.append(supplier_geometry.clone());
            geometry.value = GeometryValue::CityGmlGeometry(merged_geometry);
            geometry.epsg = supplier.geometry.epsg;
        }
        _ => {
            tracing::warn!(
                "FeatureMerger: requestor geometry is not CityGML; supplier geometry not merged"
            );
        }
    }
}

/// Executor-specific engine cache folder for accumulating processors
fn engine_cache_dir(executor_id: uuid::Uuid) -> PathBuf {
    executor_cache_subdir(executor_id, "processors")
}

impl FeatureMerger {
    fn ensure_temp_dir(&mut self) -> Result<&PathBuf, BoxedError> {
        if self.temp_dir.is_none() {
            let executor_id = self.executor_id.unwrap_or_else(uuid::Uuid::nil);
            let dir =
                engine_cache_dir(executor_id).join(format!("merger-{}", uuid::Uuid::new_v4()));
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
            .join(format!("requestor/{idx:06}.jsonl.zst"))
    }

    fn supplier_file_path(&self, idx: usize) -> PathBuf {
        self.temp_dir
            .as_ref()
            .unwrap()
            .join(format!("supplier/{idx:06}.jsonl.zst"))
    }

    fn write_feature_to_requestor(
        &mut self,
        idx: usize,
        feature: &Feature,
    ) -> Result<(), BoxedError> {
        let feature_json = serde_json::to_string(feature)?;
        self.buffer_bytes += feature_json.len();
        let mut src = feature_json.into_bytes();
        src.push(b'\n');
        let frame = zstd::encode_all(src.as_slice(), 1)?;
        self.requestor_buffer.entry(idx).or_default().extend(frame);

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
        let mut src = feature_json.into_bytes();
        src.push(b'\n');
        let frame = zstd::encode_all(src.as_slice(), 1)?;
        self.supplier_buffer.entry(idx).or_default().extend(frame);

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

        for (idx, bytes) in std::mem::take(&mut self.requestor_buffer) {
            let path = self.requestor_file_path(idx);
            let mut file = File::options().create(true).append(true).open(path)?;
            file.write_all(&bytes)?;
        }

        for (idx, bytes) in std::mem::take(&mut self.supplier_buffer) {
            let path = self.supplier_file_path(idx);
            let mut file = File::options().create(true).append(true).open(path)?;
            file.write_all(&bytes)?;
        }

        self.buffer_bytes = 0;
        Ok(())
    }

    fn change_group(&mut self, ctx: Context, fw: &ProcessorChannelForwarder) -> errors::Result<()> {
        if !self.params.complete_grouped {
            return Ok(());
        }
        // Flush buffer before reading files
        self.flush_buffer()
            .map_err(|e| FeatureProcessorError::Merger(format!("Failed to flush buffer: {e}")))?;
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
            let requestor_idx = match self.requestor_key_map.remove(attribute_value) {
                Some(idx) => idx,
                None => return Ok(()),
            };
            self.requestor_complete.remove(attribute_value);
            let requestor_features =
                read_features_from_file(&self.requestor_file_path(requestor_idx)).map_err(|e| {
                    FeatureProcessorError::Merger(format!("Failed to read requestor features: {e}"))
                })?;
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
            let supplier_features = read_features_from_file(&self.supplier_file_path(supplier_idx))
                .map_err(|e| {
                    FeatureProcessorError::Merger(format!("Failed to read supplier features: {e}"))
                })?;
            let _ = std::fs::remove_file(self.supplier_file_path(supplier_idx));

            for request_feature in requestor_features.iter() {
                let merged_feature =
                    merge_features(&self.params, request_feature, &supplier_features);
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
        if self.executor_id.is_none() {
            self.executor_id = Some(fw.executor_id());
        }

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
                    Entry::Occupied(entry) => {
                        self.requestor_before_value = Some(requestor_attribute_value.clone());
                        *entry.get()
                    }
                    Entry::Vacant(entry) => {
                        let idx = self.next_requestor_idx;
                        self.next_requestor_idx += 1;
                        entry.insert(idx);
                        self.requestor_complete
                            .insert(requestor_attribute_value.clone(), false);
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
                let idx = match self
                    .supplier_key_map
                    .entry(supplier_attribute_value.clone())
                {
                    Entry::Occupied(entry) => {
                        self.supplier_before_value = Some(supplier_attribute_value.clone());
                        *entry.get()
                    }
                    Entry::Vacant(entry) => {
                        let idx = self.next_supplier_idx;
                        self.next_supplier_idx += 1;
                        entry.insert(idx);
                        self.supplier_complete
                            .insert(supplier_attribute_value.clone(), false);
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
        // Flush any remaining buffered data to disk
        self.flush_buffer()?;

        let temp_dir = match &self.temp_dir {
            Some(d) => d.clone(),
            None => return Ok(()),
        };

        let merged_path = temp_dir.join("output_merged.jsonl.zst");
        let unmerged_path = temp_dir.join("output_unmerged.jsonl.zst");
        let mut merged_writer = BufWriter::new(zstd::Encoder::new(File::create(&merged_path)?, 1)?);
        let mut unmerged_writer =
            BufWriter::new(zstd::Encoder::new(File::create(&unmerged_path)?, 1)?);
        let mut merged_count: usize = 0;
        let mut unmerged_count: usize = 0;

        for (request_value, &req_idx) in self.requestor_key_map.iter() {
            let requestor_features = read_features_from_file(&self.requestor_file_path(req_idx))?;

            let Some(&sup_idx) = self.supplier_key_map.get(request_value) else {
                for request_feature in requestor_features.iter() {
                    serde_json::to_writer(&mut unmerged_writer, request_feature)?;
                    unmerged_writer.write_all(b"\n")?;
                    unmerged_count += 1;
                }
                continue;
            };

            let supplier_features = read_features_from_file(&self.supplier_file_path(sup_idx))?;

            for request_feature in requestor_features.iter() {
                let merged_feature =
                    merge_features(&self.params, request_feature, &supplier_features);
                serde_json::to_writer(&mut merged_writer, &merged_feature)?;
                merged_writer.write_all(b"\n")?;
                merged_count += 1;
            }
        }

        merged_writer
            .into_inner()
            .map_err(|e| e.into_error())?
            .finish()?;
        unmerged_writer
            .into_inner()
            .map_err(|e| e.into_error())?
            .finish()?;

        let context = ctx.as_context();

        if merged_count > 0 {
            fw.send_file(merged_path, MERGED_PORT.clone(), context.clone());
        }
        if unmerged_count > 0 {
            fw.send_file(unmerged_path, UNMERGED_PORT.clone(), context);
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureMerger"
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_types::{AttributeValue, Attributes, Geometry, GeometryType, GmlGeometry};

    use super::*;

    fn params(conflict_resolution: ConflictResolution, merge_geometry: bool) -> CompiledParam {
        CompiledParam {
            requestor_attribute: None,
            supplier_attribute: None,
            requestor_attribute_value: None,
            supplier_attribute_value: None,
            complete_grouped: false,
            conflict_resolution,
            merge_geometry,
        }
    }

    fn feature(attributes: &[(&str, &str)], geometry: GeometryValue) -> Feature {
        let attributes: Attributes = attributes
            .iter()
            .map(|(k, v)| (Attribute::new(*k), AttributeValue::String(v.to_string())))
            .collect();
        let mut feature = Feature::new_with_attributes(attributes);
        feature.geometry = Arc::new(Geometry::with_value(geometry));
        feature
    }

    fn citygml(feature_id: &str) -> GeometryValue {
        GeometryValue::CityGmlGeometry(CityGmlGeometry::new(
            vec![GmlGeometry {
                feature_id: Some(feature_id.to_string()),
                ..GmlGeometry::new(GeometryType::Surface, Some(2))
            }],
            vec![],
            vec![],
        ))
    }

    fn attribute(feature: &Feature, key: &str) -> Option<AttributeValue> {
        feature.attributes.get(&Attribute::new(key)).cloned()
    }

    fn feature_ids(feature: &Feature) -> Vec<String> {
        feature
            .geometry
            .value
            .as_citygml_geometry()
            .unwrap()
            .gml_geometries
            .iter()
            .filter_map(|g| g.feature_id.clone())
            .collect()
    }

    /// The default keeps the supplier's value on conflict and leaves geometry alone.
    #[test]
    fn test_supplier_wins_without_geometry() {
        let requestor = feature(&[("id", "b"), ("name", "building")], citygml("b"));
        let supplier = feature(&[("id", "i"), ("extra", "x")], citygml("i"));
        let merged = merge_features(
            &params(ConflictResolution::SupplierWins, false),
            &requestor,
            &[supplier],
        );
        assert_eq!(
            attribute(&merged, "id"),
            Some(AttributeValue::String("i".into()))
        );
        assert_eq!(
            attribute(&merged, "extra"),
            Some(AttributeValue::String("x".into()))
        );
        assert_eq!(feature_ids(&merged), vec!["b"]);
    }

    /// `requestorWins` keeps the requestor's value and only adds missing keys.
    #[test]
    fn test_requestor_wins() {
        let requestor = feature(&[("id", "b"), ("name", "building")], GeometryValue::None);
        let supplier = feature(&[("id", "i"), ("extra", "x")], GeometryValue::None);
        let merged = merge_features(
            &params(ConflictResolution::RequestorWins, false),
            &requestor,
            &[supplier],
        );
        assert_eq!(
            attribute(&merged, "id"),
            Some(AttributeValue::String("b".into()))
        );
        assert_eq!(
            attribute(&merged, "name"),
            Some(AttributeValue::String("building".into()))
        );
        assert_eq!(
            attribute(&merged, "extra"),
            Some(AttributeValue::String("x".into()))
        );
    }

    /// Every supplier's CityGML geometry is appended after the requestor's, in supplier order.
    #[test]
    fn test_merge_geometry_appends_suppliers() {
        let requestor = feature(&[("id", "b")], citygml("b"));
        let suppliers = [
            feature(&[("id", "i1")], citygml("i1")),
            feature(&[("id", "i2")], citygml("i2")),
        ];
        let merged = merge_features(
            &params(ConflictResolution::RequestorWins, true),
            &requestor,
            &suppliers,
        );
        assert_eq!(feature_ids(&merged), vec!["b", "i1", "i2"]);
        assert_eq!(
            attribute(&merged, "id"),
            Some(AttributeValue::String("b".into()))
        );
    }

    /// A requestor without geometry takes on the supplier's CityGML geometry.
    #[test]
    fn test_merge_geometry_into_empty_requestor() {
        let requestor = feature(&[("id", "b")], GeometryValue::None);
        let supplier = feature(&[("id", "i")], citygml("i"));
        let merged = merge_features(
            &params(ConflictResolution::RequestorWins, true),
            &requestor,
            &[supplier],
        );
        assert_eq!(feature_ids(&merged), vec!["i"]);
    }
}
