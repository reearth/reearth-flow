use std::collections::HashMap;

use nusamai_projection::{crs::*, etmerc::ExtendedTransverseMercatorProjection, jprect::JPRZone};
use reearth_flow_geometry::algorithm::proj::Projection;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::GeometryValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{errors::GeometryProcessorError, types::SUPPORT_EPSG_CODE};

#[derive(Debug, Clone, Default)]
pub struct ReprojectorFactory;

impl ProcessorFactory for ReprojectorFactory {
    fn name(&self) -> &str {
        "Reprojector"
    }

    fn description(&self) -> &str {
        "Reprojects the geometry of a feature to a specified coordinate system"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ReprojectorParam))
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
        let params: ReprojectorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::ReprojectorFactory(format!(
                    "Failed to serialize with: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::ReprojectorFactory(format!(
                    "Failed to deserialize with: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::ReprojectorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        if !SUPPORT_EPSG_CODE.contains(&params.epsg_code) {
            return Err(GeometryProcessorError::ReprojectorFactory(
                "Unsupported EPSG code".to_string(),
            )
            .into());
        }
        let zone = JPRZone::from_epsg(params.epsg_code).ok_or(
            GeometryProcessorError::ReprojectorFactory(format!(
                "Failed to create JPRZone from EPSG code: {}",
                params.epsg_code,
            )),
        )?;
        Ok(Box::new(Reprojector {
            epsg_code: params.epsg_code,
            projection: zone.projection(),
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReprojectorParam {
    epsg_code: EpsgCode,
}

#[derive(Debug, Clone)]
pub struct Reprojector {
    epsg_code: EpsgCode,
    projection: ExtendedTransverseMercatorProjection,
}

impl Processor for Reprojector {
    fn initialize(&mut self, _ctx: NodeContext) {}

    fn num_threads(&self) -> usize {
        5
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        if let Some(ref mut geometry) = feature.geometry {
            geometry.epsg = Some(self.epsg_code);
            match &mut geometry.value {
                GeometryValue::CityGmlGeometry(v) => {
                    for feature in &mut v.features {
                        for polygon in &mut feature.polygons {
                            polygon.projection(&self.projection)?;
                        }
                    }
                }
                GeometryValue::Null => {}
                _ => unimplemented!(),
            }
        }
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
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
        "Reprojector"
    }
}
