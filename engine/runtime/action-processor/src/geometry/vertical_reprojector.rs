use std::collections::HashMap;

use nusamai_projection::vshift::Jgd2011ToWgs84;
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
pub struct VerticalReprojectorFactory;

impl ProcessorFactory for VerticalReprojectorFactory {
    fn name(&self) -> &str {
        "VerticalReprojector"
    }

    fn description(&self) -> &str {
        "Reprojects the geometry of a feature to a specified coordinate system"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(VerticalReprojectorParam))
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
        let params: VerticalReprojectorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::VerticalReprojectorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::VerticalReprojectorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::VerticalReprojectorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let reprojector = match params.reprojector_type {
            VerticalReprojectorType::Jgd2011ToWgs84 => Jgd2011ToWgs84::new(),
        };

        Ok(Box::new(VerticalReprojector { reprojector }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum VerticalReprojectorType {
    Jgd2011ToWgs84,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerticalReprojectorParam {
    reprojector_type: VerticalReprojectorType,
}

#[derive(Debug, Clone)]
pub struct VerticalReprojector {
    reprojector: Jgd2011ToWgs84,
}

impl Processor for VerticalReprojector {
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
                geos.transform_inplace(&self.reprojector);
                feature.geometry = Geometry {
                    epsg,
                    value: GeometryValue::CityGmlGeometry(geos),
                };
            }
            GeometryValue::FlowGeometry3D(mut geos) => {
                geos.transform_inplace(&self.reprojector);
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
        "VerticalReprojector"
    }
}
