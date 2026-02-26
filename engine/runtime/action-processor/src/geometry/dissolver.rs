use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use reearth_flow_geometry::{
    algorithm::{bool_ops::BooleanOps, tolerance::glue_vertices_closer_than},
    types::multi_polygon::MultiPolygon2D,
};
use reearth_flow_runtime::{
    cache::executor_cache_subdir,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;
use crate::ACCUMULATOR_BUFFER_BYTE_THRESHOLD;

/// Executor-specific engine cache folder for accumulating processors
fn engine_cache_dir(executor_id: uuid::Uuid) -> PathBuf {
    executor_cache_subdir(executor_id, "processors")
}

pub static AREA_PORT: Lazy<Port> = Lazy::new(|| Port::new("area"));

/// # Attribute Accumulation Strategy
/// Defines how attributes should be handled when dissolving multiple features into one
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum AttributeAccumulationStrategy {
    /// # Drop Incoming Attributes
    /// No attributes from any incoming features will be preserved in the output (except group_by attributes if specified)
    DropAttributes,
    /// # Merge Incoming Attributes
    /// The output feature will merge all input attributes. When multiple features have the same attribute with different values, all values are collected into an array
    MergeAttributes,
    /// # Use Attributes From One Feature
    /// The output inherits the attributes of one representative feature (the last feature in the group)
    #[default]
    UseOneFeature,
}

#[derive(Debug, Clone, Default)]
pub struct DissolverFactory;

impl ProcessorFactory for DissolverFactory {
    fn name(&self) -> &str {
        "Dissolver"
    }

    fn description(&self) -> &str {
        "Dissolve Features by Grouping Attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(DissolverParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![AREA_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: DissolverParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::DissolverFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::DissolverFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::DissolverFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = Dissolver {
            group_by: param.group_by,
            // Default tolerance to 0.0 if not specified.
            // TODO: This default value is to not break existing behavior, but should be changed in the future once we have more unit tests.
            tolerance: param.tolerance.unwrap_or(0.0),
            attribute_accumulation: param.attribute_accumulation,
            group_map: HashMap::new(),
            group_count: 0,
            temp_dir: None,
            buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: None,
        };

        Ok(Box::new(process))
    }
}

/// # Dissolver Parameters
/// Configure how to dissolve features by grouping them based on shared attributes
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DissolverParam {
    /// # Group By Attributes
    /// List of attribute names to group features by before dissolving. Features with the same values for these attributes will be dissolved together
    group_by: Option<Vec<Attribute>>,
    /// # Tolerance
    /// Geometric tolerance. Vertices closer than this distance will be considered identical during the dissolve operation.
    tolerance: Option<f64>,
    /// # Attribute Accumulation
    /// Strategy for handling attributes when dissolving features
    #[serde(default)]
    attribute_accumulation: AttributeAccumulationStrategy,
}

pub struct Dissolver {
    group_by: Option<Vec<Attribute>>,
    tolerance: f64,
    attribute_accumulation: AttributeAccumulationStrategy,
    // Disk-backed state
    group_map: HashMap<AttributeValue, usize>,
    group_count: usize,
    temp_dir: Option<PathBuf>,
    // In-memory buffer: group_idx -> Vec<feature_json>
    buffer: HashMap<usize, Vec<String>>,
    buffer_bytes: usize,
    /// Executor ID for cache isolation, set on first process() call
    executor_id: Option<uuid::Uuid>,
}

impl std::fmt::Debug for Dissolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dissolver")
            .field("group_count", &self.group_count)
            .finish_non_exhaustive()
    }
}

impl Clone for Dissolver {
    fn clone(&self) -> Self {
        Self {
            group_by: self.group_by.clone(),
            tolerance: self.tolerance,
            attribute_accumulation: self.attribute_accumulation.clone(),
            group_map: HashMap::new(),
            group_count: 0,
            temp_dir: None,
            buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: self.executor_id,
        }
    }
}

impl Drop for Dissolver {
    fn drop(&mut self) {
        if let Some(ref dir) = self.temp_dir {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

impl Dissolver {
    fn ensure_temp_dir(&mut self) -> Result<&PathBuf, BoxedError> {
        if self.temp_dir.is_none() {
            let executor_id = self.executor_id.unwrap_or_else(uuid::Uuid::nil);
            let dir =
                engine_cache_dir(executor_id).join(format!("dissolver-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&dir)?;
            self.temp_dir = Some(dir);
        }
        Ok(self.temp_dir.as_ref().unwrap())
    }

    fn group_file_path(&self, group_idx: usize) -> PathBuf {
        self.temp_dir
            .as_ref()
            .unwrap()
            .join(format!("group_{group_idx:06}.jsonl"))
    }

    fn write_feature(&mut self, group_idx: usize, feature: &Feature) -> Result<(), BoxedError> {
        let feature_json = serde_json::to_string(feature)?;
        self.buffer_bytes += feature_json.len();
        self.buffer.entry(group_idx).or_default().push(feature_json);

        if self.buffer_bytes >= ACCUMULATOR_BUFFER_BYTE_THRESHOLD {
            self.flush_buffer()?;
        }
        Ok(())
    }

    fn flush_buffer(&mut self) -> Result<(), BoxedError> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        self.ensure_temp_dir()?;
        for (group_idx, entries) in std::mem::take(&mut self.buffer) {
            let path = self.group_file_path(group_idx);
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

    fn read_features_for_group(&self, group_idx: usize) -> Result<Vec<Feature>, BoxedError> {
        let path = self.group_file_path(group_idx);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&path)?;
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

    fn dissolve_all_groups(&mut self) -> Result<Vec<Feature>, BoxedError> {
        // Flush buffer before reading files
        self.flush_buffer()?;

        let mut dissolved = Vec::new();

        for &group_idx in self.group_map.values() {
            let features = match self.read_features_for_group(group_idx) {
                Ok(f) => f,
                Err(_) => continue,
            };

            let buffered_features_2d: Vec<Feature> = features
                .into_iter()
                .filter(|f| matches!(&f.geometry.value, GeometryValue::FlowGeometry2D(_)))
                .collect();

            if let Some(dissolved_2d) = self.dissolve_2d(buffered_features_2d) {
                dissolved.push(dissolved_2d);
            }
        }

        // Clean up all group files
        for &group_idx in self.group_map.values() {
            let path = self.group_file_path(group_idx);
            let _ = std::fs::remove_file(path);
        }

        // Reset state
        self.group_map.clear();
        self.group_count = 0;

        Ok(dissolved)
    }
}

impl Processor for Dissolver {
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

        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(_) => {
                let key = if let Some(group_by) = &self.group_by {
                    AttributeValue::Array(
                        group_by
                            .iter()
                            .filter_map(|attr| feature.attributes.get(attr).cloned())
                            .collect(),
                    )
                } else {
                    AttributeValue::Null
                };

                // If the key is new, dissolve all current groups first
                if !self.group_map.contains_key(&key) {
                    for dissolved in self.dissolve_all_groups()? {
                        fw.send(ctx.new_with_feature_and_port(dissolved, AREA_PORT.clone()));
                    }
                }

                // Get or create group index for this key
                let group_idx = if let Some(&idx) = self.group_map.get(&key) {
                    idx
                } else {
                    let idx = self.group_count;
                    self.group_map.insert(key, idx);
                    self.group_count += 1;
                    idx
                };

                self.write_feature(group_idx, feature)?;
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        for dissolved in self.dissolve_all_groups()? {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                dissolved,
                AREA_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Dissolver"
    }
}

impl Dissolver {
    fn dissolve_2d(&self, buffered_features_2d: Vec<Feature>) -> Option<Feature> {
        // Start with an empty multi-polygon
        let mut multi_polygon_2d = MultiPolygon2D::new(vec![]);

        // Apply attribute accumulation strategy
        let attrs: IndexMap<_, _> = match self.attribute_accumulation {
            AttributeAccumulationStrategy::DropAttributes => {
                // Only keep group_by attributes if specified
                if let (Some(group_by), Some(last_feature)) =
                    (&self.group_by, buffered_features_2d.last())
                {
                    group_by
                        .iter()
                        .filter_map(|attr| {
                            let value = last_feature.attributes.get(attr).cloned()?;
                            Some((attr.clone(), value))
                        })
                        .collect::<IndexMap<_, _>>()
                } else {
                    IndexMap::new()
                }
            }
            AttributeAccumulationStrategy::MergeAttributes => {
                // Merge all attributes from all features
                let mut merged_attributes = IndexMap::new();

                for feature in &buffered_features_2d {
                    for (key, value) in feature.attributes.iter() {
                        merged_attributes
                            .entry(key.clone())
                            .and_modify(|existing: &mut Vec<AttributeValue>| {
                                // Add value if it's not already in the list
                                if !existing.contains(value) {
                                    existing.push(value.clone());
                                }
                            })
                            .or_insert_with(|| vec![value.clone()]);
                    }
                }

                // Convert single-element vectors to single values
                merged_attributes
                    .into_iter()
                    .map(|(key, values)| {
                        let final_value = if values.len() == 1 {
                            values.into_iter().next().unwrap()
                        } else {
                            AttributeValue::Array(values)
                        };
                        (key, final_value)
                    })
                    .collect::<IndexMap<_, _>>()
            }
            AttributeAccumulationStrategy::UseOneFeature => {
                // Use attributes from the last feature
                if let Some(last_feature) = buffered_features_2d.last() {
                    (*last_feature.attributes).clone()
                } else {
                    IndexMap::new()
                }
            }
        };

        // Process all features uniformly
        for feature in buffered_features_2d {
            let geometry = feature.geometry.value.as_flow_geometry_2d()?;
            let mut multi_polygon = if let Some(mp) = geometry.as_multi_polygon() {
                mp.clone()
            } else if let Some(polygon) = geometry.as_polygon() {
                MultiPolygon2D::new(vec![polygon.clone()])
            } else {
                continue;
            };
            let mut vertices = multi_polygon_2d.get_vertices_mut();
            vertices.extend(multi_polygon.get_vertices_mut());
            glue_vertices_closer_than(self.tolerance, vertices);
            multi_polygon_2d = multi_polygon_2d.union(&multi_polygon);
        }

        // Only create feature if we accumulated some geometry
        if multi_polygon_2d.is_empty() {
            return None;
        }

        let geometry = Geometry {
            value: GeometryValue::FlowGeometry2D(multi_polygon_2d.into()),
            ..Default::default()
        };
        Some(Feature::new_with_attributes_and_geometry(
            attrs,
            geometry,
        ))
    }
}
