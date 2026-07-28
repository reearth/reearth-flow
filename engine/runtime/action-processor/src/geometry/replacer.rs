use std::collections::HashMap;
#[cfg(not(feature = "new-geometry"))]
use std::sync::Arc;

use reearth_flow_common::compress::decode;
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
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::Geometry;
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct GeometryReplacerFactory;

impl ProcessorFactory for GeometryReplacerFactory {
    fn name(&self) -> &str {
        "Geometry Replacer"
    }

    fn description(&self) -> &str {
        "Replace Feature Geometry from Attribute"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeometryReplacer))
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
        let processor: GeometryReplacer = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::GeometryReplacerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::GeometryReplacerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::GeometryReplacerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(processor))
    }
}

/// # Geometry Replacer Parameters
/// Configure which attribute contains the geometry data to replace the feature's current geometry
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeometryReplacer {
    /// # Source Attribute
    /// Name of the attribute containing the compressed geometry data to use as the new geometry
    source_attribute: Attribute,
}

#[cfg(feature = "new-geometry")]
impl GeometryReplacer {
    /// Replace the feature's geometry with the one encoded in the source
    /// attribute, and drop that attribute. A feature whose source attribute is
    /// absent or does not hold a string is left untouched.
    fn replace(&self, feature: &mut Feature) -> Result<(), BoxedError> {
        let Some(AttributeValue::String(dump)) = feature.attributes.get(&self.source_attribute)
        else {
            return Ok(());
        };
        let dump = decode(dump)?;
        let geometry: Geometry = serde_json::from_str(&dump).map_err(|e| {
            GeometryProcessorError::GeometryReplacer(format!("Failed to deserialize geometry: {e}"))
        })?;
        feature.set_geometry(geometry);
        feature.remove(&self.source_attribute);
        Ok(())
    }
}

impl Processor for GeometryReplacer {
    fn num_threads(&self) -> usize {
        2
    }

    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        self.replace(&mut feature)?;
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
        let mut feature = feature.clone();
        let Some(source) = feature.attributes.get(&self.source_attribute) else {
            fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            return Ok(());
        };
        let AttributeValue::String(dump) = source else {
            fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            return Ok(());
        };
        let dump = decode(dump)?;
        let geometry: Geometry = serde_json::from_str(&dump)?;
        feature.geometry = Arc::new(geometry);
        feature.remove(&self.source_attribute);
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
        "Geometry Replacer"
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use reearth_flow_common::compress::compress;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::line_string::LineString3D;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};

    const SOURCE: &str = "geom";

    fn replacer() -> GeometryReplacer {
        GeometryReplacer {
            source_attribute: Attribute::new(SOURCE),
        }
    }

    fn line() -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
        )))
    }

    /// A feature with no geometry carrying `geometry` encoded in the source attribute.
    fn feature_with_encoded(geometry: &Geometry) -> Feature {
        let dump = compress(&serde_json::to_string(geometry).unwrap()).unwrap();
        let mut feature = Feature::from(Geometry::None);
        feature.insert(SOURCE, AttributeValue::String(dump));
        feature
    }

    #[test]
    fn encoded_geometry_becomes_the_feature_geometry() {
        let geometry = line();
        let mut feature = feature_with_encoded(&geometry);
        replacer().replace(&mut feature).unwrap();
        assert_eq!(feature.geometry.as_ref(), &geometry);
    }

    #[test]
    fn the_source_attribute_is_consumed() {
        let mut feature = feature_with_encoded(&line());
        replacer().replace(&mut feature).unwrap();
        assert!(feature.attributes.get(&Attribute::new(SOURCE)).is_none());
    }

    #[test]
    fn existing_geometry_is_overwritten() {
        let replacement = line();
        let mut feature = feature_with_encoded(&replacement);
        feature.set_geometry(Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
            LineString3D::from_coords(CoordinateFrame::Euclidean, [[0.0, 0.0, 0.0]]),
        )));
        replacer().replace(&mut feature).unwrap();
        assert_eq!(feature.geometry.as_ref(), &replacement);
    }

    #[test]
    fn a_missing_source_attribute_leaves_the_feature_alone() {
        let mut feature = Feature::from(Geometry::None);
        replacer().replace(&mut feature).unwrap();
        assert_eq!(feature.geometry.as_ref(), &Geometry::None);
    }

    #[test]
    fn a_non_string_source_attribute_leaves_the_feature_alone() {
        let mut feature = Feature::from(Geometry::None);
        feature.insert(SOURCE, AttributeValue::Number(42.into()));
        replacer().replace(&mut feature).unwrap();
        assert_eq!(feature.geometry.as_ref(), &Geometry::None);
        assert!(feature.attributes.get(&Attribute::new(SOURCE)).is_some());
    }
}
