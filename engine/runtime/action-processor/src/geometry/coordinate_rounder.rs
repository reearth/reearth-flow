use std::collections::HashMap;

use reearth_flow_geometry::ops::{CoordinatePrecision, RoundCoordinates};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct CoordinateRounderFactory;

impl ProcessorFactory for CoordinateRounderFactory {
    fn name(&self) -> &str {
        "Coordinate Rounder"
    }

    fn description(&self) -> &str {
        "Rounds every coordinate of a geometry to a fixed number of decimal places per axis."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CoordinateRounderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let Some(with) = with else {
            return Err(GeometryProcessorError::CoordinateRounderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let value = serde_json::to_value(with).map_err(|e| {
            GeometryProcessorError::CoordinateRounderFactory(format!(
                "Failed to serialize `with` parameter: {e}"
            ))
        })?;
        let params: CoordinateRounderParam = serde_json::from_value(value).map_err(|e| {
            GeometryProcessorError::CoordinateRounderFactory(format!(
                "Failed to deserialize `with` parameter: {e}"
            ))
        })?;
        let DecimalPlaces { x, y, z } = params.precision;
        Ok(Box::new(CoordinateRounder {
            precision: CoordinatePrecision { x, y, z },
        }))
    }
}

/// # Coordinate Rounder Parameters
/// Decimal places each axis is rounded to.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CoordinateRounderParam {
    /// # Precision
    /// Decimal places per axis. An axis left out is not rounded.
    precision: DecimalPlaces,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct DecimalPlaces {
    /// # X
    /// Decimal places the first stored coordinate is rounded to.
    #[serde(default)]
    x: Option<u32>,
    /// # Y
    /// Decimal places the second stored coordinate is rounded to.
    #[serde(default)]
    y: Option<u32>,
    /// # Z
    /// Decimal places the height is rounded to.
    #[serde(default)]
    z: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct CoordinateRounder {
    precision: CoordinatePrecision,
}

impl Processor for CoordinateRounder {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        match feature.geometry_mut().round_coordinates(&self.precision) {
            Ok(()) => fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone())),
            Err(e) => {
                ctx.event_hub
                    .debug_log(Some(ctx.error_span()), format!("coordinate rounding: {e}"));
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
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
        "Coordinate Rounder"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::create_default_execute_context;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::point_cloud::PointCloud;
    use reearth_flow_geometry::polygon::Polygon3D;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Feature;

    fn face(ring: Vec<[f64; 3]>) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                CoordinateFrame::Euclidean,
                ring,
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        )))
    }

    fn round(feature: Feature, precision: CoordinatePrecision) -> (Port, Feature) {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let ctx = create_default_execute_context(&feature);
        CoordinateRounder { precision }.process(ctx, &fw).unwrap();

        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("the forwarder is the one built above");
        };
        let ports = noop.send_ports.lock().unwrap();
        let features = noop.send_features.lock().unwrap();
        assert_eq!(ports.len(), 1);
        (ports[0].clone(), features[0].clone())
    }

    #[test]
    fn each_axis_is_rounded_to_its_own_places() {
        let (port, feature) = round(
            Feature::from(face(vec![
                [0.123, 0.0, 0.0],
                [1.0, 0.456, 0.0],
                [1.0, 1.0, 0.000_05],
                [0.123, 0.0, 0.000_04],
            ])),
            CoordinatePrecision {
                x: Some(2),
                y: None,
                z: Some(4),
            },
        );
        assert_eq!(port, *FEATURES_PORT);
        assert_eq!(
            *feature.geometry,
            face(vec![
                [0.12, 0.0, 0.0],
                [1.0, 0.456, 0.0],
                [1.0, 1.0, 0.0001],
                [0.12, 0.0, 0.0],
            ])
        );
    }

    #[test]
    fn a_feature_without_geometry_passes_through() {
        let (port, feature) = round(
            Feature::from(Geometry::None),
            CoordinatePrecision::default(),
        );
        assert_eq!(port, *FEATURES_PORT);
        assert_eq!(*feature.geometry, Geometry::None);
    }

    #[test]
    fn geometry_that_cannot_be_rounded_is_rejected() {
        let cloud = Geometry::Euclidean3D(Euclidean3DGeometry::PointCloud(Box::new(
            PointCloud::from_positions(CoordinateFrame::Euclidean, [[0.123, 0.0, 0.0]]),
        )));
        let (port, feature) = round(
            Feature::from(cloud.clone()),
            CoordinatePrecision {
                x: Some(2),
                y: None,
                z: None,
            },
        );
        assert_eq!(port, *REJECTED_PORT);
        assert_eq!(*feature.geometry, cloud);
    }
}
