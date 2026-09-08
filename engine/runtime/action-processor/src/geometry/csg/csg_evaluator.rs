use std::collections::HashMap;
#[cfg(not(feature = "new-geometry"))]
use std::sync::Arc;

use once_cell::sync::Lazy;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::geometry::Geometry3D as FlowGeometry3D;
// The kernel's default tolerance is named in both geometry worlds, so this
// import is not gated even though the boolean itself is.
use reearth_flow_geometry::csg;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::geometry::errors::GeometryProcessorError;

static EMPTY_PORT: Lazy<Port> = Lazy::new(|| Port::new("empty"));

#[derive(Debug, Clone, Default)]
pub struct CSGEvaluatorFactory;

impl ProcessorFactory for CSGEvaluatorFactory {
    fn name(&self) -> &str {
        "CSG Evaluator"
    }

    fn description(&self) -> &str {
        "Computes the solid a Constructive Solid Geometry tree describes, replacing the tree \
         with the result. Operands must be closed, outward-wound solids in a projected \
         coordinate reference, since the vertex tolerance is a distance; a tree whose result \
         encloses no volume leaves on the empty port."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CSGEvaluatorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["spatial", "3d"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            FEATURES_PORT.clone(),
            EMPTY_PORT.clone(),
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
        let params: CSGEvaluatorParam = match with {
            Some(with) => {
                let value: Value = serde_json::to_value(with).map_err(|e| {
                    GeometryProcessorError::CSGEvaluatorFactory(format!(
                        "Failed to serialize `with` parameter: {e}"
                    ))
                })?;
                serde_json::from_value(value).map_err(|e| {
                    GeometryProcessorError::CSGEvaluatorFactory(format!(
                        "Failed to deserialize `with` parameter: {e}"
                    ))
                })?
            }
            None => CSGEvaluatorParam::default(),
        };

        Ok(Box::new(CSGEvaluator {
            tolerance: params.tolerance,
        }))
    }
}

/// # CSG Evaluator Parameters
/// Sets how closely the operands' vertices must line up for the boolean to
/// treat them as one point.
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CSGEvaluatorParam {
    /// # Tolerance
    /// Distance below which a vertex counts as lying on a cutting plane and two
    /// vertices count as one, in the unit of the operands' coordinate
    /// reference. Defaults to a distance small enough that only near-identical
    /// vertices merge.
    #[serde(default = "default_tolerance")]
    pub tolerance: f64,
}

/// The evaluator's own default is the kernel's, named rather than copied so the
/// advertised default cannot drift from the one actually applied.
fn default_tolerance() -> f64 {
    csg::DEFAULT_TOLERANCE
}

/// # CSG Evaluator
/// Evaluates a boolean tree into the solid it denotes.
#[derive(Debug, Clone)]
pub struct CSGEvaluator {
    tolerance: f64,
}

impl Processor for CSGEvaluator {
    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        let tolerance = self.tolerance;

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
        match csg.evaluate(tolerance) {
            Ok(solid) => {
                if solid.is_void() {
                    fw.send(ctx.new_with_feature_and_port(feature, EMPTY_PORT.clone()));
                } else {
                    // Update the feature with the evaluated solid geometry
                    feature.geometry = Arc::new(Geometry {
                        epsg: feature.geometry.epsg,
                        value: GeometryValue::FlowGeometry3D(FlowGeometry3D::Solid(solid)),
                    });
                    fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
                }
            }
            Err(_e) => {
                // Evaluation failed, send to rejected
                fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
            }
        }

        Ok(())
    }

    /// Evaluate the feature's boolean tree into a solid: the result goes to
    /// the features port, a result enclosing no volume to the empty port, and a
    /// feature without a boolean tree, or one whose evaluation fails, to
    /// rejected.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        let tolerance = self.tolerance;

        let Geometry::Euclidean3D(Euclidean3DGeometry::Csg(csg)) = feature.geometry.as_ref() else {
            fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
            return Ok(());
        };

        match csg.evaluate(tolerance) {
            Ok(Some(solid)) => {
                *feature.geometry_mut() =
                    Geometry::Euclidean3D(Euclidean3DGeometry::Solid(Box::new(solid)));
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            Ok(None) => {
                fw.send(ctx.new_with_feature_and_port(feature, EMPTY_PORT.clone()));
            }
            Err(_) => {
                fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
            }
        }

        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "CSG Evaluator"
    }
}
