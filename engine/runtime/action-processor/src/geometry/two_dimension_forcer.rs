use std::collections::HashMap;

use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::GeometryValue;
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct TwoDimensionForcerFactory;

impl ProcessorFactory for TwoDimensionForcerFactory {
    fn name(&self) -> &str {
        "TwoDimensionForcer"
    }

    fn description(&self) -> &str {
        "Forces a geometry to be two dimensional."
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
        vec![DEFAULT_PORT.clone()]
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
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(_) => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geos) => {
                let value: Geometry2D = geos.clone().into();
                let mut geometry = geometry.clone();
                geometry.value = GeometryValue::FlowGeometry2D(value);
                let mut feature = feature.clone();
                feature.geometry = geometry;
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::CityGmlGeometry(gml) => {
                let value: Geometry2D = gml.clone().into();
                let mut geometry = geometry.clone();
                geometry.value = GeometryValue::FlowGeometry2D(value);
                let mut feature = feature.clone();
                feature.geometry = geometry;
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "TwoDimensionForcer"
    }
}
