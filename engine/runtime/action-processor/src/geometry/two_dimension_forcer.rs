use std::collections::HashMap;
#[cfg(not(feature = "new-geometry"))]
use std::sync::Arc;

#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::geometry::Geometry2D;
#[cfg(feature = "new-geometry")]
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::GeometryValue;
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct TwoDimensionForcerFactory;

impl ProcessorFactory for TwoDimensionForcerFactory {
    fn name(&self) -> &str {
        "Two Dimension Forcer"
    }

    fn description(&self) -> &str {
        "Force 3D Geometry to 2D by Removing Z-Coordinates"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["2d"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    #[cfg(not(feature = "new-geometry"))]
    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    #[cfg(feature = "new-geometry")]
    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(TwoDimensionForcer))
    }
}

#[derive(Debug, Clone)]
pub struct TwoDimensionForcer;

impl Processor for TwoDimensionForcer {
    // Drops the Z coordinate, re-representing 3D geometry in a 2D embedding and
    // clearing any 2.5D elevation. The coordinate frame (CRS tag) is preserved
    // verbatim — no reprojection; use the Coordinate Frame Reprojector to change
    // it. Geometry with no 2D counterpart is routed to the rejected port.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        match feature.geometry_mut().force_2d() {
            Ok(forced) => {
                *feature.geometry_mut() = forced;
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            Err(e) => {
                ctx.event_hub
                    .debug_log(Some(ctx.error_span()), format!("force 2D rejected: {e}"));
                // `feature` may be partially moved-from on a collection failure;
                // forward a pristine copy of the input to the rejected port.
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    #[cfg(feature = "new-geometry")]
    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
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
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), FEATURES_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(_) => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), FEATURES_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geos) => {
                let value: Geometry2D = geos.clone().into();
                let mut geometry = (**geometry).clone();
                geometry.value = GeometryValue::FlowGeometry2D(value);
                let mut feature = feature.clone();
                feature.geometry = Arc::new(geometry);
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            GeometryValue::CityGmlGeometry(gml) => {
                let value: Geometry2D = gml.clone().into();
                let mut geometry = (**geometry).clone();
                geometry.value = GeometryValue::FlowGeometry2D(value);
                let mut feature = feature.clone();
                feature.geometry = Arc::new(geometry);
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Two Dimension Forcer"
    }
}
