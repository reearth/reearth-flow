use std::collections::HashMap;

use reearth_flow_geometry::algorithm::hole::HoleCounter as HoleCounterAlgorithm;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct HoleCounterFactory;

impl ProcessorFactory for HoleCounterFactory {
    fn name(&self) -> &str {
        "HoleCounter"
    }

    fn description(&self) -> &str {
        "Counts the number of holes in a geometry and adds it as an attribute."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(HoleCounterParam))
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
        let params: HoleCounterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::HoleCounterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::HoleCounterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::HoleCounterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(HoleCounter {
            output_attribute: params.output_attribute,
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HoleCounterParam {
    output_attribute: Attribute,
}

#[derive(Debug, Clone)]
pub struct HoleCounter {
    output_attribute: Attribute,
}

impl Processor for HoleCounter {
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
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()))
            }
            GeometryValue::FlowGeometry2D(geometry) => {
                let mut feature = feature.clone();
                feature.attributes.insert(
                    self.output_attribute.clone(),
                    AttributeValue::Number(geometry.hole_count().into()),
                );
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geometry) => {
                let mut feature = feature.clone();
                feature.attributes.insert(
                    self.output_attribute.clone(),
                    AttributeValue::Number(geometry.hole_count().into()),
                );
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::CityGmlGeometry(geometry) => {
                let mut feature = feature.clone();
                feature.attributes.insert(
                    self.output_attribute.clone(),
                    AttributeValue::Number(geometry.hole_count().into()),
                );
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "HoleCounter"
    }
}
