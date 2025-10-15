use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::types::geometry::Geometry3D as FlowGeometry3D;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Geometry, GeometryValue};
use serde_json::Value;

static NULL_PORT: Lazy<Port> = Lazy::new(|| Port::new("nullport"));

#[derive(Debug, Clone, Default)]
pub struct CSGEvaluatorFactory;

impl ProcessorFactory for CSGEvaluatorFactory {
    fn name(&self) -> &str {
        "CSGEvaluator"
    }

    fn description(&self) -> &str {
        "Evaluates a Constructive Solid Geometry (CSG) tree to produce a solid geometry. \
        Takes a CSG representation and computes the resulting mesh from the boolean operations."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            DEFAULT_PORT.clone(),
            NULL_PORT.clone(),
            REJECTED_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let processor = CSGEvaluator {};
        Ok(Box::new(processor))
    }
}

/// # CSG Evaluator
/// Evaluates a CSG tree to produce a solid geometry mesh
#[derive(Debug, Clone)]
pub struct CSGEvaluator {}

impl Processor for CSGEvaluator {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();

        // Extract CSG from the geometry
        let csg = match &feature.geometry.value {
            GeometryValue::FlowGeometry3D(geom) => match geom {
                FlowGeometry3D::CSG(csg) => csg.clone(),
                _ => {
                    // Not a CSG geometry, send to rejected
                    fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
                    return Ok(());
                }
            },
            _ => {
                // Not a 3D geometry, send to rejected
                fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
                return Ok(());
            }
        };

        // Evaluate the CSG to get a solid
        match csg.evaluate() {
            Ok(solid) => {
                if solid.is_void() {
                    fw.send(ctx.new_with_feature_and_port(feature, NULL_PORT.clone()));
                } else {
                    // Update the feature with the evaluated solid geometry
                    feature.geometry = Geometry {
                        epsg: feature.geometry.epsg,
                        value: GeometryValue::FlowGeometry3D(FlowGeometry3D::Solid(solid)),
                    };
                    fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                }
            }
            Err(_e) => {
                // Evaluation failed, send to rejected
                fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
            }
        }

        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "CSGEvaluator"
    }
}
