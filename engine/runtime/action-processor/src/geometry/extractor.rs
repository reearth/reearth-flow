use std::collections::HashMap;

use reearth_flow_common::compress::compress;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::Geometry;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
#[cfg(feature = "new-geometry")]
use reearth_flow_types::Feature;
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct GeometryExtractorFactory;

impl ProcessorFactory for GeometryExtractorFactory {
    fn name(&self) -> &str {
        "Geometry Extractor"
    }

    fn description(&self) -> &str {
        "Extract Geometry Data to Attribute"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeometryExtractor))
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
        let processor: GeometryExtractor = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::GeometryExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::GeometryExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::GeometryExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(processor))
    }
}

/// # Geometry Extractor Parameters
/// Configure where to store the extracted geometry data as a compressed attribute
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeometryExtractor {
    /// # Output Attribute
    /// Name of the attribute where the extracted geometry data will be stored as compressed JSON
    output_attribute: Attribute,
}

#[cfg(feature = "new-geometry")]
impl GeometryExtractor {
    /// Store the feature's geometry in the output attribute, as JSON compressed
    /// with zstd and base64-encoded. A feature carrying no geometry is left
    /// untouched, so no attribute is written for it.
    fn extract(&self, feature: &mut Feature) -> Result<(), BoxedError> {
        if matches!(feature.geometry.as_ref(), Geometry::None) {
            return Ok(());
        }
        let dump = serde_json::to_string(feature.geometry.as_ref()).map_err(|e| {
            GeometryProcessorError::GeometryExtractor(format!("Failed to serialize geometry: {e}"))
        })?;
        let dump = compress(&dump)?;
        feature.insert(&self.output_attribute, AttributeValue::String(dump));
        Ok(())
    }
}

impl Processor for GeometryExtractor {
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        self.extract(&mut feature)?;
        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
        Ok(())
    }

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
        let mut feature = feature.clone();
        let value = serde_json::to_value(geometry).map_err(|e| {
            GeometryProcessorError::GeometryExtractor(format!("Failed to serialize geometry: {e}"))
        })?;
        let dump = serde_json::to_string(&value)?;
        let dump = compress(&dump)?;
        feature.insert(&self.output_attribute, AttributeValue::String(dump));
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
        "Geometry Extractor"
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use reearth_flow_common::compress::decode;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::line_string::LineString3D;
    use reearth_flow_geometry::polygon::Polygon2D;
    use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry};

    const OUTPUT: &str = "geom";

    fn extractor() -> GeometryExtractor {
        GeometryExtractor {
            output_attribute: Attribute::new(OUTPUT),
        }
    }

    /// Decode the attribute the extractor wrote back into a geometry.
    fn decoded(feature: &Feature) -> Geometry {
        let Some(AttributeValue::String(dump)) = feature.attributes.get(&Attribute::new(OUTPUT))
        else {
            panic!("no encoded geometry in `{OUTPUT}`");
        };
        serde_json::from_str(&decode(dump).unwrap()).unwrap()
    }

    fn round_trips(geometry: Geometry) {
        let mut feature = Feature::from(geometry.clone());
        extractor().extract(&mut feature).unwrap();
        assert_eq!(decoded(&feature), geometry);
    }

    #[test]
    fn three_dimensional_geometry_round_trips() {
        let line = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
        );
        round_trips(Geometry::Euclidean3D(Euclidean3DGeometry::LineString(line)));
    }

    #[test]
    fn two_dimensional_geometry_round_trips() {
        let polygon = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]],
            vec![vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 1.0]]],
        );
        round_trips(Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(
            Box::new(polygon),
        )));
    }

    #[test]
    fn geometry_is_left_on_the_feature() {
        let line = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
        );
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::LineString(line));
        let mut feature = Feature::from(geometry.clone());
        extractor().extract(&mut feature).unwrap();
        assert_eq!(feature.geometry.as_ref(), &geometry);
    }

    #[test]
    fn a_feature_without_geometry_gets_no_attribute() {
        let mut feature = Feature::from(Geometry::None);
        extractor().extract(&mut feature).unwrap();
        assert!(feature.attributes.get(&Attribute::new(OUTPUT)).is_none());
    }
}
