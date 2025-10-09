use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::types::{
    csg::{CSGChild, CSGOperation, CSG},
    geometry::Geometry3D as FlowGeometry3D,
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::errors::GeometryProcessorError;

static LEFT_PORT: Lazy<Port> = Lazy::new(|| Port::new("left"));
static RIGHT_PORT: Lazy<Port> = Lazy::new(|| Port::new("right"));
static INTERSECTION_PORT: Lazy<Port> = Lazy::new(|| Port::new("intersection"));
static UNION_PORT: Lazy<Port> = Lazy::new(|| Port::new("union"));
static DIFFERENCE_PORT: Lazy<Port> = Lazy::new(|| Port::new("difference"));

#[derive(Debug, Clone, Default)]
pub struct CSGBuilderFactory;

impl ProcessorFactory for CSGBuilderFactory {
    fn name(&self) -> &str {
        "CSGBuilder"
    }

    fn description(&self) -> &str {
        "Constructs a Constructive Solid Geometry (CSG) representation from a pair (Left, Right) of solid geometries. It detects union, intersection, difference (Left - Right). \
        It however does not compute the resulting geometry, but outputs the CSG tree structure. To evaluate the CSG tree into a solid geometry, use CSGEvaluator."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CSGBuilderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![LEFT_PORT.clone(), RIGHT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            INTERSECTION_PORT.clone(),
            UNION_PORT.clone(),
            DIFFERENCE_PORT.clone(),
            REJECTED_PORT.clone(),
        ]
    }
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: CSGBuilderParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::CSGBuilderFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::CSGBuilderFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::CSGBuilderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let processor = CSGBuilder {
            pair_id_attribute: param.pair_id_attribute,
            left_buffer: HashMap::new(),
            right_buffer: HashMap::new(),
        };
        Ok(Box::new(processor))
    }
}

/// # CSG Builder Parameters
/// Configure how the CSG builder pairs features from left and right ports
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct CSGBuilderParam {
    /// # Pair ID Attribute
    /// The name of the attribute that contains the pair ID used to match features from left and right ports
    pair_id_attribute: Attribute,
}

/// # CSG Builder
/// Builds a CSG tree from two solid geometries. To create a mesh from the CSG tree, use CSGEvaluator.
#[derive(Debug, Clone)]
pub struct CSGBuilder {
    pair_id_attribute: Attribute,
    left_buffer: HashMap<AttributeValue, Feature>,
    right_buffer: HashMap<AttributeValue, Feature>,
}

impl Processor for CSGBuilder {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = ctx.feature.clone();
        let port = ctx.port.clone();

        // Get the pair ID from the feature
        let pair_id = match feature.attributes.get(&self.pair_id_attribute) {
            Some(id) => id.clone(),
            None => {
                // Feature doesn't have the pair ID attribute, send to rejected
                fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
                return Ok(());
            }
        };

        // Check which port the feature came from and process accordingly
        if port == *LEFT_PORT {
            // Check if we already have a matching right feature
            if let Some(right_feature) = self.right_buffer.remove(&pair_id) {
                // We have a pair! Create CSG objects for all three operations
                self.create_and_send_csg(feature, right_feature, fw, &ctx)?;
            } else {
                // Store in left buffer waiting for its pair
                self.left_buffer.insert(pair_id, feature);
            }
        } else if port == *RIGHT_PORT {
            // Check if we already have a matching left feature
            if let Some(left_feature) = self.left_buffer.remove(&pair_id) {
                // We have a pair! Create CSG objects for all three operations
                self.create_and_send_csg(left_feature, feature, fw, &ctx)?;
            } else {
                // Store in right buffer waiting for its pair
                self.right_buffer.insert(pair_id, feature);
            }
        } else {
            // Unknown port, send to rejected
            fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
        }

        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        // Send all unpaired features to the rejected port
        for (_, feature) in &self.left_buffer {
            let exec_ctx = ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature.clone(),
                REJECTED_PORT.clone(),
            );
            fw.send(exec_ctx);
        }

        for (_, feature) in &self.right_buffer {
            let exec_ctx = ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature.clone(),
                REJECTED_PORT.clone(),
            );
            fw.send(exec_ctx);
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "CSGBuilder"
    }
}

impl CSGBuilder {
    fn create_and_send_csg(
        &self,
        left_feature: Feature,
        right_feature: Feature,
        fw: &ProcessorChannelForwarder,
        ctx: &ExecutorContext,
    ) -> Result<(), BoxedError> {
        // Extract solid geometries from both features
        let left_solid = match &left_feature.geometry.value {
            GeometryValue::FlowGeometry3D(geom) => match geom {
                FlowGeometry3D::Solid(solid) => solid.clone(),
                _ => {
                    // Not a solid geometry, send both to rejected
                    fw.send(ctx.new_with_feature_and_port(left_feature, REJECTED_PORT.clone()));
                    fw.send(ctx.new_with_feature_and_port(right_feature, REJECTED_PORT.clone()));
                    return Ok(());
                }
            },
            _ => {
                // Not a 3D geometry, send both to rejected
                fw.send(ctx.new_with_feature_and_port(left_feature, REJECTED_PORT.clone()));
                fw.send(ctx.new_with_feature_and_port(right_feature, REJECTED_PORT.clone()));
                return Ok(());
            }
        };

        let right_solid = match &right_feature.geometry.value {
            GeometryValue::FlowGeometry3D(geom) => match geom {
                FlowGeometry3D::Solid(solid) => solid.clone(),
                _ => {
                    // Not a solid geometry, send both to rejected
                    fw.send(ctx.new_with_feature_and_port(left_feature, REJECTED_PORT.clone()));
                    fw.send(ctx.new_with_feature_and_port(right_feature, REJECTED_PORT.clone()));
                    return Ok(());
                }
            },
            _ => {
                // Not a 3D geometry, send both to rejected
                fw.send(ctx.new_with_feature_and_port(left_feature, REJECTED_PORT.clone()));
                fw.send(ctx.new_with_feature_and_port(right_feature, REJECTED_PORT.clone()));
                return Ok(());
            }
        };

        // Create CSGChild from solids
        let left_csg_child = CSGChild::Solid(left_solid);
        let right_csg_child = CSGChild::Solid(right_solid);

        // Create and send intersection CSG
        let intersection_csg = CSG::new(
            left_csg_child.clone(),
            right_csg_child.clone(),
            CSGOperation::Intersection,
        );
        let mut intersection_feature = left_feature.clone();
        intersection_feature.geometry = Geometry {
            epsg: left_feature.geometry.epsg,
            value: GeometryValue::FlowGeometry3D(FlowGeometry3D::CSG(Box::new(intersection_csg))),
        };
        fw.send(ctx.new_with_feature_and_port(intersection_feature, INTERSECTION_PORT.clone()));

        // Create and send union CSG
        let union_csg = CSG::new(
            left_csg_child.clone(),
            right_csg_child.clone(),
            CSGOperation::Union,
        );
        let mut union_feature = left_feature.clone();
        union_feature.geometry = Geometry {
            epsg: left_feature.geometry.epsg,
            value: GeometryValue::FlowGeometry3D(FlowGeometry3D::CSG(Box::new(union_csg))),
        };
        fw.send(ctx.new_with_feature_and_port(union_feature, UNION_PORT.clone()));

        // Create and send difference CSG (left - right)
        let difference_csg = CSG::new(left_csg_child, right_csg_child, CSGOperation::Difference);
        let mut difference_feature = left_feature.clone();
        difference_feature.geometry = Geometry {
            epsg: left_feature.geometry.epsg,
            value: GeometryValue::FlowGeometry3D(FlowGeometry3D::CSG(Box::new(difference_csg))),
        };
        fw.send(ctx.new_with_feature_and_port(difference_feature, DIFFERENCE_PORT.clone()));

        Ok(())
    }
}
