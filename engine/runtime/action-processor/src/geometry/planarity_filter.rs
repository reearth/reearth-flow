use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{AttributeValue, GeometryValue};
use serde_json::Value;

pub static PLANARITY_PORT: Lazy<Port> = Lazy::new(|| Port::new("planarity"));
pub static NOT_PLANARITY_PORT: Lazy<Port> = Lazy::new(|| Port::new("notplanarity"));

#[derive(Debug, Clone, Default)]
pub struct PlanarityFilterFactory;

impl ProcessorFactory for PlanarityFilterFactory {
    fn name(&self) -> &str {
        "PlanarityFilter"
    }

    fn description(&self) -> &str {
        "Filter geometry by type"
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
        vec![PLANARITY_PORT.clone(), NOT_PLANARITY_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let process = PlanarityFilter {};
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct PlanarityFilter;

impl Processor for PlanarityFilter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), NOT_PLANARITY_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), NOT_PLANARITY_PORT.clone()))
            }
            GeometryValue::FlowGeometry2D(geometry) => {
                if geometry.are_points_coplanar() {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), PLANARITY_PORT.clone()));
                } else {
                    fw.send(
                        ctx.new_with_feature_and_port(feature.clone(), NOT_PLANARITY_PORT.clone()),
                    );
                }
            }
            GeometryValue::FlowGeometry3D(geometry) => {
                let result = geometry.are_points_coplanar();
                if let Some(result) = result {
                    let mut feature = feature.clone();
                    let mut insert_number = |key: &str, value: f64| {
                        feature.insert(
                            key.to_string(),
                            AttributeValue::Number(
                                serde_json::Number::from_f64(value)
                                    .unwrap_or_else(|| serde_json::Number::from(0)),
                            ),
                        );
                    };
                    insert_number("surfaceNormalX", result.normal.x());
                    insert_number("surfaceNormalY", result.normal.y());
                    insert_number("surfaceNormalZ", result.normal.z());
                    insert_number("pointOnSurfaceX", result.center.x());
                    insert_number("pointOnSurfaceY", result.center.y());
                    insert_number("pointOnSurfaceZ", result.center.z());
                    fw.send(ctx.new_with_feature_and_port(feature, PLANARITY_PORT.clone()));
                } else {
                    fw.send(
                        ctx.new_with_feature_and_port(feature.clone(), NOT_PLANARITY_PORT.clone()),
                    );
                }
            }
            GeometryValue::CityGmlGeometry(geometry) => {
                if geometry.are_points_coplanar() {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), PLANARITY_PORT.clone()));
                } else {
                    fw.send(
                        ctx.new_with_feature_and_port(feature.clone(), NOT_PLANARITY_PORT.clone()),
                    );
                }
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
        "PlanarityFilter"
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_types::Feature;

    use super::*;
    use crate::tests::utils::{create_default_execute_context, MockProcessorChannelForwarder};

    #[test]
    fn test_process_null_geometry() {
        let mut processor = PlanarityFilter;
        let mut fw = MockProcessorChannelForwarder::default();

        let feature = Feature::default();
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &mut fw).unwrap();

        assert_eq!(fw.send_port, NOT_PLANARITY_PORT.clone());
    }
}
