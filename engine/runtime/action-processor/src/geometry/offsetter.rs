use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct OffsetterFactory;

impl ProcessorFactory for OffsetterFactory {
    fn name(&self) -> &str {
        "Offsetter"
    }

    fn description(&self) -> &str {
        "Adds offsets to the feature's coordinates."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(OffsetterParam))
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
        let params: OffsetterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::OffsetterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::OffsetterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::OffsetterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(Offsetter { params }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OffsetterParam {
    offset_x: Option<f64>,
    offset_y: Option<f64>,
    offset_z: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct Offsetter {
    params: OffsetterParam,
}

impl Processor for Offsetter {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        let geometry_value = feature.geometry.value.clone();
        let epsg = feature.geometry.epsg;
        match geometry_value {
            GeometryValue::CityGmlGeometry(mut geos) => {
                geos.transform_offset(
                    self.params.offset_x.unwrap_or(0f64),
                    self.params.offset_y.unwrap_or(0f64),
                    self.params.offset_z.unwrap_or(0f64),
                );
                feature.geometry = Geometry {
                    epsg,
                    value: GeometryValue::CityGmlGeometry(geos),
                };
            }
            GeometryValue::FlowGeometry3D(mut geos) => {
                geos.transform_offset(
                    self.params.offset_x.unwrap_or(0f64),
                    self.params.offset_y.unwrap_or(0f64),
                    self.params.offset_z.unwrap_or(0f64),
                );
                feature.geometry = Geometry {
                    epsg,
                    value: GeometryValue::FlowGeometry3D(geos),
                };
            }
            GeometryValue::None | GeometryValue::FlowGeometry2D(..) => {}
        }
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Offsetter"
    }
}
