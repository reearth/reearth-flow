use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{AttributeValue, Feature, GeometryValue};
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
        "Filter Features by Geometry Planarity"
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
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            send_feature_as_non_planar_surface(feature, &ctx, fw);
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                send_feature_as_non_planar_surface(feature, &ctx, fw);
            }
            GeometryValue::FlowGeometry2D(geometry) => {
                if geometry.are_points_coplanar() {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), PLANARITY_PORT.clone()));
                } else {
                    send_feature_as_non_planar_surface(feature, &ctx, fw);
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
                    send_feature_as_non_planar_surface(feature, &ctx, fw);
                }
            }
            GeometryValue::CityGmlGeometry(geometry) => {
                if geometry.are_points_coplanar() {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), PLANARITY_PORT.clone()));
                } else {
                    send_feature_as_non_planar_surface(feature, &ctx, fw);
                }
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "PlanarityFilter"
    }
}

fn send_feature_as_non_planar_surface(
    feature: &Feature,
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
) {
    let mut feature = feature.clone();
    feature.insert(
        "issue",
        AttributeValue::String("NonPlanarSurface".to_string()),
    );
    fw.send(ctx.new_with_feature_and_port(feature, NOT_PLANARITY_PORT.clone()));
}

#[cfg(test)]
mod tests {
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Feature;

    use super::*;
    use crate::tests::utils::create_default_execute_context;

    #[test]
    fn test_process_null_geometry() {
        let mut processor = PlanarityFilter;
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        let feature = Feature::default();
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(noop.send_ports.lock().unwrap().len(), 1);
            assert_eq!(
                noop.send_ports.lock().unwrap().first().cloned(),
                Some(NOT_PLANARITY_PORT.clone())
            );
        }
    }
}
