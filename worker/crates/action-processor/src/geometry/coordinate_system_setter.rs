use std::collections::HashMap;

use nusamai_projection::crs::*;
use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::Geometry;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

static _SUPPORT_EPSG_CODE: Lazy<Vec<EpsgCode>> = Lazy::new(|| {
    vec![
        EPSG_WGS84_GEOGRAPHIC_2D,
        EPSG_WGS84_GEOGRAPHIC_3D,
        EPSG_WGS84_GEOCENTRIC,
        EPSG_JGD2011_GEOGRAPHIC_2D,
        EPSG_JGD2011_GEOGRAPHIC_3D,
        EPSG_JGD2011_JPRECT_I_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_II_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_III_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_IV_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_V_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_VI_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_VII_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_VIII_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_IX_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_X_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_XI_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_XII_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_XIII_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_I,
        EPSG_JGD2011_JPRECT_II,
        EPSG_JGD2011_JPRECT_III,
        EPSG_JGD2011_JPRECT_IV,
        EPSG_JGD2011_JPRECT_V,
        EPSG_JGD2011_JPRECT_VI,
        EPSG_JGD2011_JPRECT_VII,
        EPSG_JGD2011_JPRECT_VIII,
        EPSG_JGD2011_JPRECT_IX,
        EPSG_JGD2011_JPRECT_X,
        EPSG_JGD2011_JPRECT_XI,
        EPSG_JGD2011_JPRECT_XII,
        EPSG_JGD2011_JPRECT_XIII,
        EPSG_JGD2011_JPRECT_XIV,
        EPSG_JGD2011_JPRECT_XV,
        EPSG_JGD2011_JPRECT_XVI,
        EPSG_JGD2011_JPRECT_XVII,
        EPSG_JGD2011_JPRECT_XVIII,
        EPSG_JGD2011_JPRECT_XIX,
        EPSG_JGD2000_JPRECT_I,
        EPSG_JGD2000_JPRECT_II,
        EPSG_JGD2000_JPRECT_III,
        EPSG_JGD2000_JPRECT_IV,
        EPSG_JGD2000_JPRECT_V,
        EPSG_JGD2000_JPRECT_VI,
        EPSG_JGD2000_JPRECT_VII,
        EPSG_JGD2000_JPRECT_VIII,
        EPSG_JGD2000_JPRECT_IX,
        EPSG_JGD2000_JPRECT_X,
        EPSG_JGD2000_JPRECT_XI,
        EPSG_JGD2000_JPRECT_XII,
        EPSG_JGD2000_JPRECT_XIII,
        EPSG_JGD2000_JPRECT_XIV,
        EPSG_JGD2000_JPRECT_XV,
        EPSG_JGD2000_JPRECT_XVI,
        EPSG_JGD2000_JPRECT_XVII,
        EPSG_JGD2000_JPRECT_XVIII,
        EPSG_JGD2000_JPRECT_XIX,
        EPSG_TOKYO_JPRECT_I,
        EPSG_TOKYO_JPRECT_II,
        EPSG_TOKYO_JPRECT_III,
        EPSG_TOKYO_JPRECT_IV,
        EPSG_TOKYO_JPRECT_V,
        EPSG_TOKYO_JPRECT_VI,
        EPSG_TOKYO_JPRECT_VII,
        EPSG_TOKYO_JPRECT_VIII,
        EPSG_TOKYO_JPRECT_IX,
        EPSG_TOKYO_JPRECT_X,
        EPSG_TOKYO_JPRECT_XI,
        EPSG_TOKYO_JPRECT_XII,
        EPSG_TOKYO_JPRECT_XIII,
        EPSG_TOKYO_JPRECT_XIV,
        EPSG_TOKYO_JPRECT_XV,
        EPSG_TOKYO_JPRECT_XVI,
        EPSG_TOKYO_JPRECT_XVII,
        EPSG_TOKYO_JPRECT_XVIII,
        EPSG_TOKYO_JPRECT_XIX,
    ]
});

#[derive(Debug, Clone, Default)]
pub struct CoordinateSystemSetterFactory;

impl ProcessorFactory for CoordinateSystemSetterFactory {
    fn name(&self) -> &str {
        "CoordinateSystemSetter"
    }

    fn description(&self) -> &str {
        "Sets the coordinate system of a feature"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CoordinateSystemSetter))
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
        let processor: CoordinateSystemSetter = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::CoordinateSystemSetterFactory(format!(
                    "Failed to serialize with: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::CoordinateSystemSetterFactory(format!(
                    "Failed to deserialize with: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::CoordinateSystemSetterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(processor))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CoordinateSystemSetter {
    epsg_code: EpsgCode,
}

impl Processor for CoordinateSystemSetter {
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
        let mut feature = feature.clone();
        let mut geometry = if feature.geometry.is_some() {
            feature.geometry.unwrap()
        } else {
            Geometry::default()
        };
        geometry.epsg = Some(self.epsg_code);
        feature.geometry = Some(geometry);
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
        "CoordinateSystemSetter"
    }
}
