use std::collections::HashMap;

use indexmap::IndexMap;
use reearth_flow_geometry::algorithm::convex_hull::ConvexHull;
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::geometry_collection::GeometryCollection;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct ConvexHullAccumulatorFactory;

impl ProcessorFactory for ConvexHullAccumulatorFactory {
    fn name(&self) -> &str {
        "ConvexHullAccumulator"
    }

    fn description(&self) -> &str {
        "Creates a convex hull based on a group of input features."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ConvexHullAccumulatorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: ConvexHullAccumulatorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::ConvexHullAccumulatorFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::ConvexHullAccumulatorFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::ConvexHullAccumulatorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = ConvexHullAccumulator {
            group_by: param.group_by,
            buffer: HashMap::new(),
        };

        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConvexHullAccumulatorParam {
    group_by: Option<Vec<Attribute>>,
}

#[derive(Debug, Clone)]
pub struct ConvexHullAccumulator {
    group_by: Option<Vec<Attribute>>,
    buffer: HashMap<AttributeValue, Vec<Feature>>,
}

impl Processor for ConvexHullAccumulator {
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
                    for hull in self.create_hull() {
                        fw.send(ctx.new_with_feature_and_port(hull, DEFAULT_PORT.clone()));
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
        for hull in self.create_hull() {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                hull,
                DEFAULT_PORT.clone(),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "ConvexHullAccumulator"
    }
}

impl ConvexHullAccumulator {
    fn create_hull(&self) -> Vec<Feature> {
        let mut hulls = Vec::new();
        for buffer in self.buffer.values() {
            let buffered_features_2d = buffer
                .iter()
                .filter(|f| matches!(&f.geometry.value, GeometryValue::FlowGeometry2D(_)))
                .collect::<Vec<_>>();
            hulls.push(self.create_hull_2d(buffered_features_2d));
        }
        hulls
    }

    fn create_hull_2d(&self, buffered_features_2d: Vec<&Feature>) -> Feature {
        let collection = GeometryCollection(
            buffered_features_2d
                .iter()
                .filter_map(|f| f.geometry.value.as_flow_geometry_2d().cloned())
                .collect::<Vec<_>>(),
        );
        let convex_hull = collection.convex_hull();

        let mut feature = Feature::new();
        if let (Some(group_by), Some(last_feature)) = (&self.group_by, buffered_features_2d.last())
        {
            feature.attributes = group_by
                .iter()
                .filter_map(|attr| {
                    let value = last_feature.attributes.get(attr).cloned()?;
                    Some((attr.clone(), value))
                })
                .collect::<IndexMap<_, _>>();
        } else {
            feature.attributes = IndexMap::new();
        }
        feature.geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::Polygon(convex_hull));
        feature
    }
}
