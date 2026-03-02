use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;
use reearth_flow_geometry::algorithm::convex_hull::quick_hull_2d;
use reearth_flow_geometry::algorithm::coords_iter::CoordsIter;
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::line_string::LineString;
use reearth_flow_geometry::types::polygon::Polygon;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, Geometry, GeometryValue};
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
        "Generate Convex Hull Polygons from Grouped Features"
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

/// # ConvexHullAccumulator Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConvexHullAccumulatorParam {
    /// # Group By Attributes
    /// Attributes used to group features before creating convex hulls - each group gets its own hull
    group_by: Option<Vec<Attribute>>,
}

#[derive(Debug, Clone)]
pub struct ConvexHullAccumulator {
    group_by: Option<Vec<Attribute>>,
    buffer: HashMap<AttributeValue, GroupBuffer>,
}

#[derive(Debug, Clone)]
struct GroupBuffer {
    common_attr: IndexMap<Attribute, AttributeValue>,
    geometries: Vec<Arc<Geometry>>,
}

impl GroupBuffer {
    fn new(common_attr: IndexMap<Attribute, AttributeValue>) -> Self {
        Self {
            common_attr,
            geometries: Vec::new(),
        }
    }
}

impl Processor for ConvexHullAccumulator {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        if ctx.feature.geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        match &ctx.feature.geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(_) => {
                let key = if let Some(group_by) = &self.group_by {
                    let attrs = group_by
                        .iter()
                        .filter_map(|attr| ctx.feature.attributes.get(attr).cloned())
                        .collect();
                    AttributeValue::Array(attrs)
                } else {
                    AttributeValue::Null
                };

                if !self.buffer.contains_key(&key) {
                    for hull in self.create_hull() {
                        fw.send(ctx.new_with_feature_and_port(hull, DEFAULT_PORT.clone()));
                    }
                    self.buffer.clear();

                    let common_attr = if let Some(group_by) = &self.group_by {
                        let vals = key.as_vec().unwrap();
                        group_by.iter().cloned().zip(vals).collect()
                    } else {
                        IndexMap::new()
                    };
                    self.buffer
                        .insert(key.clone(), GroupBuffer::new(common_attr));
                }

                self.buffer
                    .entry(key)
                    .and_modify(|b| b.geometries.push(ctx.feature.geometry));
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
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
    fn create_hull(&mut self) -> Vec<Feature> {
        let mut hulls = Vec::new();
        for buffer in std::mem::take(&mut self.buffer).into_values() {
            hulls.push(Self::create_hull_2d(buffer));
        }
        hulls
    }

    fn create_hull_2d(buffer: GroupBuffer) -> Feature {
        let mut collection = buffer
            .geometries
            .into_iter()
            .flat_map(|g| {
                g.value
                    .as_flow_geometry_2d()
                    .unwrap()
                    .coords_iter()
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let convex_hull = quick_hull_2d(&mut collection);
        let convex_hull = Polygon::new(LineString::new(convex_hull.0), Vec::new());

        let geom = GeometryValue::FlowGeometry2D(Geometry2D::Polygon(convex_hull));
        let geom = Geometry {
            value: geom,
            ..Default::default()
        };
        Feature::new_with_attributes_and_geometry(buffer.common_attr, geom)
    }
}
