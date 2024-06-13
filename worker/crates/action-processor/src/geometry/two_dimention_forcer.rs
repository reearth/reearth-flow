use std::collections::HashMap;

use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct TwoDimentionForcerFactory;

impl ProcessorFactory for TwoDimentionForcerFactory {
    fn name(&self) -> &str {
        "TwoDimentionForcer"
    }

    fn description(&self) -> &str {
        "Forces a geometry to be two dimentional."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(TwoDimentionForcer))
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
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let processor: TwoDimentionForcer = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::TwoDimentionForcerFactory(format!(
                    "Failed to serialize with: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::TwoDimentionForcerFactory(format!(
                    "Failed to deserialize with: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::TwoDimentionForcerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(processor))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TwoDimentionForcer {
    output_attribute: Attribute,
}

impl Processor for TwoDimentionForcer {
    fn initialize(&mut self, _ctx: NodeContext) {}

    fn num_threads(&self) -> usize {
        5
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let Some(geometry) = &feature.geometry else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::Null => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(_) => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(_) => unimplemented!(),
            GeometryValue::CityGmlGeometry(gml) => {
                let value: Geometry2D = gml.clone().into();
                let mut geometry = geometry.clone();
                geometry.value = GeometryValue::FlowGeometry2D(value);
                let mut feature = feature.clone();
                feature.geometry = Some(geometry);
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
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
        "TwoDimentionForcer"
    }
}
