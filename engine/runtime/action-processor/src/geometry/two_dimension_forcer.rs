use std::collections::HashMap;
#[cfg(feature = "new-geometry")]
use std::collections::HashSet;
#[cfg(not(feature = "new-geometry"))]
use std::sync::Arc;

#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::coordinate::EpsgCode;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::ForceTwoDimensionError;
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
        "Removes Z-coordinates from 3D geometries to produce 2D output."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &[]
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
        Ok(Box::new(TwoDimensionForcer::default()))
    }
}

#[derive(Debug, Clone, Default)]
pub struct TwoDimensionForcer {
    /// CRSs already reported as unusable. An unusable CRS is a property of the
    /// stream rather than of one feature, so it is logged once per code.
    #[cfg(feature = "new-geometry")]
    reported_frames: HashSet<EpsgCode>,
}

impl Processor for TwoDimensionForcer {
    // Drops the Z coordinate and any 2.5D elevation, demoting the CRS tag with it
    // so it still matches the coordinates. Geometry that cannot be flattened,
    // whether by type or by CRS, goes to the rejected port.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // Forced on a copy: a failure part-way through a collection leaves the
        // geometry moved-from, and the input must stay intact for the reject port.
        let mut geometry = (*ctx.feature.geometry).clone();
        match geometry.force_2d() {
            Ok(forced) => {
                let mut feature = ctx.feature.clone();
                feature.set_geometry(forced);
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            Err(e) => {
                // An unsupported type is this port's normal business, but an
                // unusable CRS means broken data or a broken PROJ install.
                let should_warn = match &e {
                    ForceTwoDimensionError::UnsupportedFrame(frame) => {
                        self.reported_frames.insert(frame.epsg)
                    }
                    ForceTwoDimensionError::UnsupportedGeometry(_) => false,
                };
                let message = format!("force 2D rejected: {e}");
                if should_warn {
                    ctx.event_hub.warn_log(Some(ctx.error_span()), message);
                } else {
                    ctx.event_hub.debug_log(Some(ctx.error_span()), message);
                }
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            }
        }
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
