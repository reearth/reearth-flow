use std::collections::HashMap;

#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::algorithm::hole::HoleCounter as _;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::CountHoles;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::GeometryValue;
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct HoleCounterFactory;

impl ProcessorFactory for HoleCounterFactory {
    fn name(&self) -> &str {
        "Hole Counter"
    }

    fn description(&self) -> &str {
        "Counts the holes in every face of a feature's geometry and stores the total in an \
         attribute. A geometry that cannot carry a hole, and a feature with no geometry, both \
         count as zero."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(HoleCounterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: HoleCounterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::HoleCounterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::HoleCounterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::HoleCounterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(HoleCounter {
            output_attribute: params.output_attribute,
        }))
    }
}

/// # Hole Counter Parameters
/// Where the total number of holes is stored on each feature.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HoleCounterParam {
    /// # Output Attribute
    /// Attribute the count is written to, as a number. It is set on every feature, so a
    /// geometry with no holes records zero.
    output_attribute: Attribute,
}

#[derive(Debug, Clone)]
pub struct HoleCounter {
    output_attribute: Attribute,
}

impl Processor for HoleCounter {
    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), FEATURES_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), FEATURES_PORT.clone()))
            }
            GeometryValue::FlowGeometry2D(geometry) => {
                let mut feature = feature.clone();
                feature.attributes_mut().insert(
                    self.output_attribute.clone(),
                    AttributeValue::Number(geometry.hole_count().into()),
                );
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geometry) => {
                let mut feature = feature.clone();
                feature.attributes_mut().insert(
                    self.output_attribute.clone(),
                    AttributeValue::Number(geometry.hole_count().into()),
                );
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            GeometryValue::CityGmlGeometry(geometry) => {
                let mut feature = feature.clone();
                feature.attributes_mut().insert(
                    self.output_attribute.clone(),
                    AttributeValue::Number(geometry.hole_count().into()),
                );
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
        }
        Ok(())
    }

    /// Attach the number of interior rings the geometry's faces carry to the
    /// feature. Counting is total: a geometry that cannot hold a hole — and an
    /// absent one — yields zero rather than being rejected, so every feature
    /// leaves with the attribute set.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let count = ctx.feature.geometry.count_holes();
        let mut feature = ctx.feature.clone();
        feature.attributes_mut().insert(
            self.output_attribute.clone(),
            AttributeValue::Number(count.into()),
        );
        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
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
        "Hole Counter"
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use crate::tests::utils::create_default_execute_context;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::point::Point3D;
    use reearth_flow_geometry::polygon::Polygon3D;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Feature;

    const SQUARE: [[f64; 3]; 5] = [
        [0.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [4.0, 4.0, 0.0],
        [0.0, 4.0, 0.0],
        [0.0, 0.0, 0.0],
    ];

    /// A square face carrying `n` holes.
    fn face_with_holes(n: usize) -> Geometry {
        let holes: Vec<_> = (0..n)
            .map(|i| {
                let x = 1.0 + i as f64 * 1.5;
                vec![
                    [x, 1.0, 0.0],
                    [x + 1.0, 1.0, 0.0],
                    [x + 1.0, 2.0, 0.0],
                    [x, 2.0, 0.0],
                    [x, 1.0, 0.0],
                ]
            })
            .collect();
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(CoordinateFrame::Euclidean, SQUARE, holes),
        )))
    }

    /// Run the processor over `feature`, returning the single feature it forwards.
    fn count(feature: Feature) -> Feature {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let ctx = create_default_execute_context(&feature);
        HoleCounter {
            output_attribute: Attribute::new("holeCount"),
        }
        .process(ctx, &fw)
        .unwrap();

        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("the forwarder is the one built above");
        };
        let ports = noop.send_ports.lock().unwrap();
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], *FEATURES_PORT);
        let features = noop.send_features.lock().unwrap();
        assert_eq!(features.len(), 1);
        features[0].clone()
    }

    fn hole_count(feature: &Feature) -> Option<&AttributeValue> {
        feature.attributes.get(&Attribute::new("holeCount"))
    }

    #[test]
    fn a_face_with_holes_reports_their_number() {
        let feature = count(Feature::from(face_with_holes(2)));
        assert_eq!(
            hole_count(&feature),
            Some(&AttributeValue::Number(2.into()))
        );
    }

    #[test]
    fn a_face_without_holes_reports_zero() {
        let feature = count(Feature::from(face_with_holes(0)));
        assert_eq!(
            hole_count(&feature),
            Some(&AttributeValue::Number(0.into()))
        );
    }

    #[test]
    fn a_geometry_that_cannot_carry_a_hole_reports_zero() {
        let point = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::Euclidean,
            [1.0, 2.0, 3.0],
        )));
        let feature = count(Feature::from(point));
        assert_eq!(
            hole_count(&feature),
            Some(&AttributeValue::Number(0.into()))
        );
    }

    /// Unlike the legacy processor, which passes a feature with no geometry
    /// through untouched, the attribute is always set: a geometry with nothing to
    /// count records zero, which is what a counting operation is expected to do.
    #[test]
    fn a_feature_without_geometry_reports_zero() {
        let feature = count(Feature::from(Geometry::None));
        assert_eq!(
            hole_count(&feature),
            Some(&AttributeValue::Number(0.into()))
        );
    }

    #[test]
    fn existing_attributes_and_the_feature_id_are_preserved() {
        let mut input = Feature::from(face_with_holes(1));
        input
            .attributes_mut()
            .insert(Attribute::new("gmlId"), AttributeValue::String("x".into()));
        let id = input.id;

        let feature = count(input);
        assert_eq!(feature.id, id);
        assert_eq!(
            feature.attributes.get(&Attribute::new("gmlId")),
            Some(&AttributeValue::String("x".into()))
        );
        assert_eq!(
            hole_count(&feature),
            Some(&AttributeValue::Number(1.into()))
        );
    }
}
