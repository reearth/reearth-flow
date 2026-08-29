use std::collections::HashMap;

#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::Elevation;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
#[cfg(feature = "new-geometry")]
use reearth_flow_types::Feature;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::GeometryValue;
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct ElevationExtractorFactory;

impl ProcessorFactory for ElevationExtractorFactory {
    fn name(&self) -> &str {
        "Elevation Extractor"
    }

    fn description(&self) -> &str {
        "Extracts the elevation of a geometry's first vertex into an attribute. A geometry \
         carrying no elevation passes through without it."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ElevationExtractorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["3d"]
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
        let params: ElevationExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::ElevationExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::ElevationExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::ElevationExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(ElevationExtractor {
            output_attribute: params.output_attribute,
        }))
    }
}

/// # Elevation Extractor Parameters
/// Sets where the extracted elevation is stored.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ElevationExtractorParam {
    /// # Output Attribute
    /// Attribute the elevation is written to. It is left unwritten when the geometry carries no
    /// elevation.
    output_attribute: Attribute,
}

#[derive(Debug, Clone)]
pub struct ElevationExtractor {
    output_attribute: Attribute,
}

#[cfg(feature = "new-geometry")]
impl ElevationExtractor {
    /// Writes the elevation of the geometry's representative vertex into
    /// `feature`. A geometry with no elevation to read leaves it untouched.
    /// Returns the elevation that had no JSON form, if any.
    fn extract(&self, feature: &mut Feature) -> Option<f64> {
        let elevation = feature.geometry.elevation()?;
        match serde_json::Number::from_f64(elevation) {
            Some(number) => {
                feature.insert(&self.output_attribute, AttributeValue::Number(number));
                None
            }
            None => Some(elevation),
        }
    }
}

impl Processor for ElevationExtractor {
    fn num_threads(&self) -> usize {
        2
    }

    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        // A geometry carrying no elevation has none to extract. That is nothing
        // to do rather than a failure, so the feature passes through untouched —
        // but without the attribute. Writing zero, as this used to, invents a
        // value indistinguishable from a real sea-level one. The unified geometry
        // model draws the same line: a planar leaf's `elevation()` is `None`.
        let elevation = match &geometry.value {
            GeometryValue::None | GeometryValue::FlowGeometry2D(_) => None,
            GeometryValue::FlowGeometry3D(geometry) => Some(geometry.elevation()),
            GeometryValue::CityGmlGeometry(geometry) => Some(geometry.elevation()),
        };

        let feature = match elevation {
            Some(elevation) => {
                let number = serde_json::Number::from_f64(elevation).ok_or(
                    GeometryProcessorError::ElevationExtractor(
                        "Failed to convert elevation to number".to_string(),
                    ),
                )?;
                let mut feature = feature.clone();
                feature.insert(&self.output_attribute, AttributeValue::Number(number));
                feature
            }
            None => feature.clone(),
        };
        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
        Ok(())
    }

    /// Attach the elevation of the geometry's representative vertex to the
    /// feature. Every feature leaves by `features`, with or without the
    /// attribute: a geometry with no elevation to read is nothing to do rather
    /// than a failure.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        if let Some(elevation) = self.extract(&mut feature) {
            ctx.event_hub.debug_log(
                Some(ctx.error_span()),
                format!("elevation {elevation} is not a finite number"),
            );
        }
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
        "Elevation Extractor"
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::line_string::{LineString2D, LineString3D};
    use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

    /// The feature `process` would forward for `geometry`, and the non-finite
    /// elevation it would report, if any.
    fn run(geometry: Geometry) -> (Feature, Option<f64>) {
        let extractor = ElevationExtractor {
            output_attribute: Attribute::new("elevation"),
        };
        let mut feature = Feature::from(geometry);
        let not_finite = extractor.extract(&mut feature);
        (feature, not_finite)
    }

    fn attribute(feature: &Feature) -> Option<f64> {
        match feature.attributes.get(&Attribute::new("elevation"))? {
            AttributeValue::Number(n) => n.as_f64(),
            other => panic!("expected a number, got {other:?}"),
        }
    }

    fn line_3d(coords: [[f64; 3]; 2]) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            coords,
        )))
    }

    #[test]
    fn an_elevation_is_written_under_the_configured_attribute() {
        let (feature, not_finite) = run(line_3d([[0.0, 0.0, 12.5], [1.0, 0.0, -3.0]]));
        assert_eq!(attribute(&feature), Some(12.5));
        assert_eq!(not_finite, None);
    }

    #[test]
    fn a_geometry_without_an_elevation_gets_no_attribute() {
        let plane = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords(CoordinateFrame::Euclidean, [[0.0, 0.0], [1.0, 0.0]]),
        ));
        for (label, geometry) in [
            ("no geometry", Geometry::None),
            ("planar line string", plane),
        ] {
            let (feature, not_finite) = run(geometry);
            assert_eq!(attribute(&feature), None, "{label}");
            assert_eq!(not_finite, None, "{label}");
        }
    }

    #[test]
    fn a_non_finite_elevation_is_reported_and_gets_no_attribute() {
        for z in [f64::NAN, f64::INFINITY] {
            let (feature, not_finite) = run(line_3d([[0.0, 0.0, z], [1.0, 0.0, 0.0]]));
            assert_eq!(attribute(&feature), None, "z = {z}");
            assert!(!not_finite.unwrap().is_finite(), "z = {z}");
        }
    }
}
