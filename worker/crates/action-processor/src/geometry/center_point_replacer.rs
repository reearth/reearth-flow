use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::algorithm::centroid::Centroid;
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Feature, Geometry, GeometryValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub static POINT_PORT: Lazy<Port> = Lazy::new(|| Port::new("point"));

#[derive(Debug, Clone, Default)]
pub struct CenterPointReplacerFactory;

impl ProcessorFactory for CenterPointReplacerFactory {
    fn name(&self) -> &str {
        "CenterPointReplacer"
    }

    fn description(&self) -> &str {
        "Replaces the geometry of the feature with a point that is either in the center of the feature's bounding box, at the center of mass of the feature, or somewhere guaranteed to be inside the feature's area."
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
        vec![POINT_PORT.clone(), REJECTED_PORT.clone()]
    }
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(CenterPointReplacer))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CenterPointReplacer;

impl Processor for CenterPointReplacer {
    fn initialize(&mut self, _ctx: NodeContext) {}

    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let Some(geometry) = &feature.geometry else {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::Null => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geos) => {
                self.handle_2d_geometry(geos, feature, geometry, &ctx, fw);
            }
            GeometryValue::FlowGeometry3D(geos) => {
                self.handle_3d_geometry(geos, feature, geometry, &ctx, fw);
            }
            GeometryValue::CityGmlGeometry(_) => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()))
            }
        }
        Ok(())
    }

    fn finish(
        &self,
        _ctx: NodeContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "CenterPointReplacer"
    }
}

impl CenterPointReplacer {
    fn handle_2d_geometry(
        &self,
        geos: &Geometry2D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) {
        let Some(centroid) = geos.centroid() else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return;
        };
        let feature = feature.clone();
        let mut geometry = geometry.clone();
        geometry.value = GeometryValue::FlowGeometry2D(centroid.into());
        fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
    }

    fn handle_3d_geometry(
        &self,
        geos: &Geometry3D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) {
        let Some(centroid) = geos.centroid() else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return;
        };
        let feature = feature.clone();
        let mut geometry = geometry.clone();
        geometry.value = GeometryValue::FlowGeometry3D(centroid.into());
        fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
    }
}
