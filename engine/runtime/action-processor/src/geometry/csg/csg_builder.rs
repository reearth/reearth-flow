use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_geometry::types::{
    csg::{CSGChild, CSGOperation, CSG},
    face::Face,
    geometry::Geometry3D as FlowGeometry3D,
    solid::Solid3D,
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, REJECTED_PORT},
};
use reearth_flow_types::{
    Attribute, AttributeValue, Expr, Feature, Geometry, GeometryType, GeometryValue,
};
use rhai::Dynamic;
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
        "Constructs a Consecutive Solid Geometry (CSG) representation from a pair (Left, Right) of solid geometries. It detects union, intersection, difference (Left - Right). \
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
        ctx: NodeContext,
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

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let pair_id_attribute = if let Some(expr) = &param.pair_id_attribute {
            Some(expr_engine.compile(expr.as_ref()).map_err(|e| {
                GeometryProcessorError::CSGBuilderFactory(format!(
                    "Failed to compile pair_id_attribute expression: {e:?}"
                ))
            })?)
        } else {
            None
        };

        let processor = CSGBuilder {
            pair_id_attribute,
            left_buffer: HashMap::new(),
            right_buffer: HashMap::new(),
            create_list: param.create_list,
            list_attribute_name: param.list_attribute_name,
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
    /// Expression to evaluate the pair ID used to match features from left and right ports
    pair_id_attribute: Option<Expr>,

    /// # Create List
    /// When enabled, creates a list of attribute values from both children (left and right)
    create_list: Option<bool>,

    /// # List Attribute Name
    /// Name of the attribute to create the list from (required when create_list is true)
    list_attribute_name: Option<String>,
}

/// # CSG Builder
/// Builds a CSG tree from two solid geometries. To create a mesh from the CSG tree, use CSGEvaluator.
#[derive(Debug, Clone)]
pub struct CSGBuilder {
    pair_id_attribute: Option<rhai::AST>,
    left_buffer: HashMap<AttributeValue, Feature>,
    right_buffer: HashMap<AttributeValue, Feature>,
    create_list: Option<bool>,
    list_attribute_name: Option<String>,
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

        // Get the pair ID from the feature by evaluating the expression
        let pair_id = if let Some(expr) = &self.pair_id_attribute {
            let expr_engine = Arc::clone(&ctx.expr_engine);
            let scope = feature.new_scope(expr_engine.clone(), &None);
            match scope.eval_ast::<Dynamic>(expr) {
                Ok(value) => match value.try_into() {
                    Ok(attr_value) => attr_value,
                    Err(_) => {
                        // Failed to convert to AttributeValue, send to rejected
                        fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
                        return Ok(());
                    }
                },
                Err(_e) => {
                    // Failed to evaluate expression, send to rejected
                    fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
                    return Ok(());
                }
            }
        } else {
            // No expression configured, send to rejected
            fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
            return Ok(());
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
        for feature in self.left_buffer.values() {
            let exec_ctx = ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature.clone(),
                REJECTED_PORT.clone(),
            );
            fw.send(exec_ctx);
        }

        for feature in self.right_buffer.values() {
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
            GeometryValue::CityGmlGeometry(cg) => {
                let faces: Vec<Face> = cg
                    .gml_geometries
                    .iter()
                    .filter(|gml_geometry| gml_geometry.ty == GeometryType::Solid)
                    .flat_map(|gml_geometry| gml_geometry.polygons.clone())
                    .map(|polygon| polygon.exterior().clone().into())
                    .collect::<Vec<_>>();
                if faces.is_empty() {
                    // No solid faces found, send both to rejected
                    fw.send(ctx.new_with_feature_and_port(left_feature, REJECTED_PORT.clone()));
                    fw.send(ctx.new_with_feature_and_port(right_feature, REJECTED_PORT.clone()));
                    return Ok(());
                }
                Solid3D::new_with_faces(faces)
            }
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
            GeometryValue::CityGmlGeometry(cg) => {
                let faces: Vec<Face> = cg
                    .gml_geometries
                    .iter()
                    .filter(|gml_geometry| gml_geometry.ty == GeometryType::Solid)
                    .flat_map(|gml_geometry| gml_geometry.polygons.clone())
                    .map(|polygon| polygon.exterior().clone().into())
                    .collect::<Vec<_>>();
                if faces.is_empty() {
                    // No solid faces found, send both to rejected
                    fw.send(ctx.new_with_feature_and_port(left_feature, REJECTED_PORT.clone()));
                    fw.send(ctx.new_with_feature_and_port(right_feature, REJECTED_PORT.clone()));
                    return Ok(());
                }
                Solid3D::new_with_faces(faces)
            }
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

        // Create list attribute if enabled
        let list_attribute = if self.create_list.unwrap_or(false) {
            if let Some(ref attr_name) = self.list_attribute_name {
                // Create a list containing the entire attributes object from both features
                let mut attribute_objects = Vec::new();

                // Convert left feature's entire attributes to AttributeValue::Map
                let left_attrs: std::collections::HashMap<String, AttributeValue> = left_feature
                    .attributes
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.clone()))
                    .collect();
                attribute_objects.push(AttributeValue::Map(left_attrs));

                // Convert right feature's entire attributes to AttributeValue::Map
                let right_attrs: std::collections::HashMap<String, AttributeValue> = right_feature
                    .attributes
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.clone()))
                    .collect();
                attribute_objects.push(AttributeValue::Map(right_attrs));

                // Create the attribute with the list of attribute objects
                let attr_key = Attribute::new(attr_name.clone());
                Some((attr_key, AttributeValue::Array(attribute_objects)))
            } else {
                None
            }
        } else {
            None
        };

        // Create and send intersection CSG
        let intersection_csg = CSG::new(
            left_csg_child.clone(),
            right_csg_child.clone(),
            CSGOperation::Intersection,
        );
        let mut intersection_feature = Feature::new();
        intersection_feature.geometry = Geometry {
            epsg: left_feature.geometry.epsg,
            value: GeometryValue::FlowGeometry3D(FlowGeometry3D::CSG(Box::new(intersection_csg))),
        };

        // Add list attribute if created
        if let Some((attr_key, attr_value)) = &list_attribute {
            intersection_feature
                .attributes
                .insert(attr_key.clone(), attr_value.clone());
        }

        fw.send(ctx.new_with_feature_and_port(intersection_feature, INTERSECTION_PORT.clone()));

        // Create and send union CSG
        let union_csg = CSG::new(
            left_csg_child.clone(),
            right_csg_child.clone(),
            CSGOperation::Union,
        );
        let mut union_feature = Feature::new();
        union_feature.geometry = Geometry {
            epsg: left_feature.geometry.epsg,
            value: GeometryValue::FlowGeometry3D(FlowGeometry3D::CSG(Box::new(union_csg))),
        };

        // Add list attribute if created
        if let Some((attr_key, attr_value)) = &list_attribute {
            union_feature
                .attributes
                .insert(attr_key.clone(), attr_value.clone());
        }

        fw.send(ctx.new_with_feature_and_port(union_feature, UNION_PORT.clone()));

        // Create and send difference CSG (left - right)
        let difference_csg = CSG::new(left_csg_child, right_csg_child, CSGOperation::Difference);
        let mut difference_feature = Feature::new();
        difference_feature.geometry = Geometry {
            epsg: left_feature.geometry.epsg,
            value: GeometryValue::FlowGeometry3D(FlowGeometry3D::CSG(Box::new(difference_csg))),
        };

        // Add list attribute if created
        if let Some((attr_key, attr_value)) = &list_attribute {
            difference_feature
                .attributes
                .insert(attr_key.clone(), attr_value.clone());
        }

        fw.send(ctx.new_with_feature_and_port(difference_feature, DIFFERENCE_PORT.clone()));

        Ok(())
    }
}
