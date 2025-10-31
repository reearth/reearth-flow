use std::collections::HashMap;

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use reearth_flow_geometry::{
    algorithm::{bool_ops::BooleanOps, tolerance::glue_vertices_closer_than},
    types::multi_polygon::MultiPolygon2D,
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

pub static AREA_PORT: Lazy<Port> = Lazy::new(|| Port::new("area"));

/// # Attribute Accumulation Strategy
/// Defines how attributes should be handled when dissolving multiple features into one
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
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
    UseOneFeature,
}

impl Default for AttributeAccumulationStrategy {
    fn default() -> Self {
        Self::UseOneFeature
    }
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
            buffer: HashMap::new(),
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

#[derive(Debug, Clone)]
pub struct Dissolver {
    group_by: Option<Vec<Attribute>>,
    tolerance: f64,
    attribute_accumulation: AttributeAccumulationStrategy,
    buffer: HashMap<AttributeValue, Vec<Feature>>,
}

impl Processor for Dissolver {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
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

                if !self.buffer.contains_key(&key) {
                    for dissolved in self.dissolve() {
                        fw.send(ctx.new_with_feature_and_port(dissolved, AREA_PORT.clone()));
                    }
                    self.buffer.clear();
                }

                self.buffer
                    .entry(key.clone())
                    .or_default()
                    .push(feature.clone());
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        for dissolved in self.dissolve() {
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
    fn dissolve(&self) -> Vec<Feature> {
        let mut dissolved = Vec::new();
        for buffer in self.buffer.values() {
            let buffered_features_2d = buffer
                .iter()
                .filter(|f| matches!(&f.geometry.value, GeometryValue::FlowGeometry2D(_)))
                .collect::<Vec<_>>();

            if let Some(dissolved_2d) = self.dissolve_2d(buffered_features_2d) {
                dissolved.push(dissolved_2d);
            }
        }
        dissolved
    }

    fn dissolve_2d(&self, buffered_features_2d: Vec<&Feature>) -> Option<Feature> {
        let multi_polygon_2d = buffered_features_2d.iter().fold(
            None,
            |multi_polygon_acc: Option<_>, feature_incoming| {
                let geometry_incoming = feature_incoming.geometry.value.as_flow_geometry_2d()?;
                let mut multi_polygon_incoming =
                    if let Some(multi_polygon) = geometry_incoming.as_multi_polygon() {
                        multi_polygon
                    } else if let Some(polygon) = geometry_incoming.as_polygon() {
                        MultiPolygon2D::new(vec![polygon])
                    } else {
                        return multi_polygon_acc;
                    };

                let mut mutli_polygon_acc = if let Some(mutli_polygon_acc) = multi_polygon_acc {
                    mutli_polygon_acc
                } else {
                    return Some(multi_polygon_incoming);
                };

                let mut vertices = mutli_polygon_acc.get_vertices_mut();
                vertices.extend(multi_polygon_incoming.get_vertices_mut());
                glue_vertices_closer_than(self.tolerance, vertices);

                let unite = multi_polygon_incoming.union(&mutli_polygon_acc);
                Some(unite)
            },
        );

        let multi_polygon_2d = multi_polygon_2d?;

        let mut feature = Feature::new();

        // Apply attribute accumulation strategy
        feature.attributes = match self.attribute_accumulation {
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
                    for (key, value) in &feature.attributes {
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
                    last_feature.attributes.clone()
                } else {
                    IndexMap::new()
                }
            }
        };

        feature.geometry.value = GeometryValue::FlowGeometry2D(multi_polygon_2d.into());
        Some(feature)
    }
}
