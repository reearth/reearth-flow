use std::collections::HashMap;

use nusamai_projection::{crs::*, etmerc::ExtendedTransverseMercatorProjection, jprect::JPRZone};
use once_cell::sync::Lazy;
use reearth_flow_geometry::algorithm::transverse_mercator_proj::TransverseMercatorProjection;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::GeometryValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

pub(super) static SUPPORT_HORIZONTAL_EPSG_CODE: Lazy<Vec<EpsgCode>> = Lazy::new(|| {
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
pub struct HorizontalReprojectorFactory;

impl ProcessorFactory for HorizontalReprojectorFactory {
    fn name(&self) -> &str {
        "HorizontalReprojector"
    }

    fn description(&self) -> &str {
        "Reprojects the geometry of a feature to a specified coordinate system"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(HorizontalReprojectorParam))
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
        let params: HorizontalReprojectorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::HorizontalReprojectorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::HorizontalReprojectorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::HorizontalReprojectorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let projection = {
            if !SUPPORT_HORIZONTAL_EPSG_CODE.contains(&params.epsg_code) {
                return Err(GeometryProcessorError::HorizontalReprojectorFactory(
                    "Unsupported EPSG code".to_string(),
                )
                .into());
            }
            let zone = JPRZone::from_epsg(params.epsg_code).ok_or(
                GeometryProcessorError::HorizontalReprojectorFactory(format!(
                    "Failed to create JPRZone from EPSG code: {}",
                    params.epsg_code,
                )),
            )?;
            zone.projection()
        };
        Ok(Box::new(HorizontalReprojector {
            epsg_code: params.epsg_code,
            projection,
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HorizontalReprojectorParam {
    epsg_code: EpsgCode,
}

#[derive(Debug, Clone)]
pub struct HorizontalReprojector {
    epsg_code: EpsgCode,
    projection: ExtendedTransverseMercatorProjection,
}

impl Processor for HorizontalReprojector {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        match &geometry.value {
            GeometryValue::CityGmlGeometry(v) => {
                let mut feature = feature.clone();
                let mut geometry = geometry.clone();
                let projection = &self.projection;
                let mut geometry_value = v.clone();
                for gml_geometry in &mut geometry_value.gml_geometries {
                    for polygon in &mut gml_geometry.polygons {
                        polygon.project_forward(projection, true)?;
                    }
                }
                geometry.value = GeometryValue::CityGmlGeometry(geometry_value);
                geometry.epsg = Some(self.epsg_code);
                feature.geometry = geometry;
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geos) => {
                let mut feature = feature.clone();
                let mut geometry = geometry.clone();
                let projection = &self.projection;
                let mut geos = geos.clone();
                geos.project_forward(projection, true)?;
                geometry.value = GeometryValue::FlowGeometry2D(geos);
                geometry.epsg = Some(self.epsg_code);
                feature.geometry = geometry;
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geos) => {
                let mut feature = feature.clone();
                let mut geometry = geometry.clone();
                let projection = &self.projection;
                let mut geos = geos.clone();
                geos.project_forward(projection, true)?;
                geometry.value = GeometryValue::FlowGeometry3D(geos);
                geometry.epsg = Some(self.epsg_code);
                feature.geometry = geometry;
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()))
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "HorizontalReprojector"
    }
}
