use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::GeometryValue;
use serde_json::Value;

pub static TWO_DIMENSION_PORT: Lazy<Port> = Lazy::new(|| Port::new("2d"));
pub static THREE_DIMENSION_PORT: Lazy<Port> = Lazy::new(|| Port::new("3d"));

#[derive(Debug, Clone, Default)]
pub struct DimensionFilterFactory;

impl ProcessorFactory for DimensionFilterFactory {
    fn name(&self) -> &str {
        "DimensionFilter"
    }

    fn description(&self) -> &str {
        "Filter Features by Geometry Dimension"
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
        vec![
            TWO_DIMENSION_PORT.clone(),
            THREE_DIMENSION_PORT.clone(),
            REJECTED_PORT.clone(),
        ]
    }
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(DimensionFilter))
    }
}

#[derive(Debug, Clone)]
pub struct DimensionFilter;

impl Processor for DimensionFilter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(_) => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), TWO_DIMENSION_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geometry) => {
                if geometry.is_elevation_zero() {
                    fw.send(
                        ctx.new_with_feature_and_port(feature.clone(), TWO_DIMENSION_PORT.clone()),
                    );
                } else {
                    fw.send(
                        ctx.new_with_feature_and_port(
                            feature.clone(),
                            THREE_DIMENSION_PORT.clone(),
                        ),
                    );
                }
            }
            GeometryValue::CityGmlGeometry(geometry) => {
                if geometry.is_elevation_zero() {
                    fw.send(
                        ctx.new_with_feature_and_port(feature.clone(), TWO_DIMENSION_PORT.clone()),
                    );
                } else {
                    fw.send(
                        ctx.new_with_feature_and_port(
                            feature.clone(),
                            THREE_DIMENSION_PORT.clone(),
                        ),
                    );
                }
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "DimensionFilter"
    }
}
