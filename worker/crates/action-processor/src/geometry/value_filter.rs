use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::GeometryValue;
use serde_json::Value;

pub static NONE_PORT: Lazy<Port> = Lazy::new(|| Port::new("none"));
pub static GEOMETRY_2D_PORT: Lazy<Port> = Lazy::new(|| Port::new("geometry2d"));
pub static GEOMETRY_3D_PORT: Lazy<Port> = Lazy::new(|| Port::new("geometry3d"));
pub static CITY_GML_PORT: Lazy<Port> = Lazy::new(|| Port::new("cityGml"));

#[derive(Debug, Clone, Default)]
pub struct GeometryValueFilterFactory;

impl ProcessorFactory for GeometryValueFilterFactory {
    fn name(&self) -> &str {
        "GeometryValueFilter"
    }

    fn description(&self) -> &str {
        "Filter geometry by value"
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
        GeometryValueFilterType::all_ports()
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(GeometryValueFilter {}))
    }
}

#[derive(Debug, Clone)]
pub enum GeometryValueFilterType {
    None,
    Geometry2D,
    Geometry3D,
    CityGmlGeometry,
}

impl GeometryValueFilterType {
    fn output_port(&self) -> Port {
        match self {
            Self::None => NONE_PORT.clone(),
            Self::Geometry2D => GEOMETRY_2D_PORT.clone(),
            Self::Geometry3D => GEOMETRY_3D_PORT.clone(),
            Self::CityGmlGeometry => CITY_GML_PORT.clone(),
        }
    }

    fn all_ports() -> Vec<Port> {
        vec![
            Self::None.output_port(),
            Self::Geometry2D.output_port(),
            Self::Geometry3D.output_port(),
            Self::CityGmlGeometry.output_port(),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct GeometryValueFilter;

impl Processor for GeometryValueFilter {
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
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), NONE_PORT.clone()));
            return Ok(());
        };
        match geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), NONE_PORT.clone()))
            }
            GeometryValue::FlowGeometry2D(_) => fw
                .send(ctx.new_with_feature_and_port(ctx.feature.clone(), GEOMETRY_2D_PORT.clone())),
            GeometryValue::FlowGeometry3D(_) => fw
                .send(ctx.new_with_feature_and_port(ctx.feature.clone(), GEOMETRY_3D_PORT.clone())),
            GeometryValue::CityGmlGeometry(_) => {
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), CITY_GML_PORT.clone()))
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
        "GeometryValueFilter"
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
    use reearth_flow_types::{Feature, Geometry};

    use crate::tests::utils::{create_default_execute_context, MockProcessorChannelForwarder};

    use super::*;

    #[test]
    fn test_filter_geometry_null() {
        let mut fw = MockProcessorChannelForwarder::default();
        let feature = Feature {
            geometry: None,
            ..Default::default()
        };
        let ctx = create_default_execute_context(&feature);
        GeometryValueFilter {}.process(ctx, &mut fw).unwrap();
        assert_eq!(fw.send_port, NONE_PORT.clone());
    }

    #[test]
    fn test_filter_geometry_none() {
        let mut fw = MockProcessorChannelForwarder::default();
        let feature = Feature {
            geometry: Some(Geometry {
                value: GeometryValue::None,
                ..Default::default()
            }),
            ..Default::default()
        };
        let ctx = create_default_execute_context(&feature);
        GeometryValueFilter {}.process(ctx, &mut fw).unwrap();
        assert_eq!(fw.send_port, NONE_PORT.clone());
    }

    #[test]
    fn test_filter_geometry_2d() {
        let mut fw = MockProcessorChannelForwarder::default();
        let feature = Feature {
            geometry: Some(Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::Point(Default::default())),
                ..Default::default()
            }),
            ..Default::default()
        };
        let ctx = create_default_execute_context(&feature);
        GeometryValueFilter {}.process(ctx, &mut fw).unwrap();
        assert_eq!(fw.send_port, GEOMETRY_2D_PORT.clone());
    }

    #[test]
    fn test_filter_geometry_3d() {
        let mut fw = MockProcessorChannelForwarder::default();
        let feature = Feature {
            geometry: Some(Geometry {
                value: GeometryValue::FlowGeometry3D(Geometry3D::Point(Default::default())),
                ..Default::default()
            }),
            ..Default::default()
        };
        let ctx = create_default_execute_context(&feature);
        GeometryValueFilter {}.process(ctx, &mut fw).unwrap();
        assert_eq!(fw.send_port, GEOMETRY_3D_PORT.clone());
    }

    #[test]
    fn test_filter_geometry_citygml() {
        let mut fw = MockProcessorChannelForwarder::default();
        let feature = Feature {
            geometry: Some(Geometry {
                value: GeometryValue::CityGmlGeometry(Default::default()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let ctx = create_default_execute_context(&feature);
        GeometryValueFilter {}.process(ctx, &mut fw).unwrap();
        assert_eq!(fw.send_port, CITY_GML_PORT.clone());
    }
    // Add more tests for other scenarios...
}
