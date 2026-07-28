use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use serde_json::Value;

#[cfg(not(feature = "new-geometry"))]
use std::sync::Arc;

#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::Geometry;

#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::Geometry;
#[cfg(feature = "new-geometry")]
use reearth_flow_types::Feature;

#[derive(Debug, Clone, Default)]
pub struct GeometryRemoverFactory;

impl ProcessorFactory for GeometryRemoverFactory {
    fn name(&self) -> &str {
        "Geometry Remover"
    }

    fn description(&self) -> &str {
        "Discards a feature's geometry, keeping its attributes."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
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
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(GeometryRemover))
    }
}

#[derive(Debug, Clone)]
pub struct GeometryRemover;

#[cfg(feature = "new-geometry")]
impl GeometryRemover {
    /// A copy of `feature` with no geometry.
    fn remove(&self, feature: &Feature) -> Feature {
        let mut feature = feature.clone();
        feature.set_geometry(Geometry::None);
        feature
    }
}

impl Processor for GeometryRemover {
    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        feature.geometry = Arc::new(Geometry::default());
        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
        Ok(())
    }

    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = self.remove(&ctx.feature);
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
        "Geometry Remover"
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::point::Point3D;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
    use reearth_flow_types::{Attribute, AttributeValue};

    #[test]
    fn the_geometry_is_gone_and_the_attributes_remain() {
        let point = Point3D::new(CoordinateFrame::Euclidean, [1.0, 2.0, 3.0]);
        let mut feature = Feature::from(Geometry::Euclidean3D(Euclidean3DGeometry::Point(point)));
        feature.insert(Attribute::new("name"), AttributeValue::String("a".into()));

        let removed = GeometryRemover.remove(&feature);
        assert_eq!(*removed.geometry, Geometry::None);
        assert_eq!(
            removed.attributes.get(&Attribute::new("name")),
            Some(&AttributeValue::String("a".into()))
        );
        assert_eq!(removed.id, feature.id);
    }

    #[test]
    fn a_feature_that_has_no_geometry_is_unaffected() {
        let feature = Feature::from(Geometry::None);
        assert_eq!(*GeometryRemover.remove(&feature).geometry, Geometry::None);
    }
}
