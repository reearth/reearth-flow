use std::collections::HashMap;

use nusamai_projection::{
    crs::*, ellipsoid::wgs84, etmerc::ExtendedTransverseMercatorProjection, jprect::JPRZone,
};
use reearth_flow_geometry::algorithm::{
    centroid::Centroid, transverse_mercator_proj::TransverseMercatorProjection,
};
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

const K: f64 = 0.9999;

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
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::HorizontalReprojectorFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::HorizontalReprojectorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let projection = if let Some(epsg_code) = params.epsg_code {
            if !SUPPORT_EPSG_CODE.contains(&epsg_code) {
                return Err(GeometryProcessorError::HorizontalReprojectorFactory(
                    "Unsupported EPSG code".to_string(),
                )
                .into());
            }
            let zone = JPRZone::from_epsg(epsg_code).ok_or(
                GeometryProcessorError::HorizontalReprojectorFactory(format!(
                    "Failed to create JPRZone from EPSG code: {}",
                    epsg_code,
                )),
            )?;
            Some(zone.projection())
        } else {
            None
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
    epsg_code: Option<EpsgCode>,
}

#[derive(Debug, Clone)]
pub struct HorizontalReprojector {
    epsg_code: Option<EpsgCode>,
    projection: Option<ExtendedTransverseMercatorProjection>,
}

impl Processor for HorizontalReprojector {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        match &geometry.value {
            GeometryValue::CityGmlGeometry(v) => {
                let mut feature = feature.clone();
                let mut geometry = geometry.clone();
                let Some(projection) = &self.projection else {
                    fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                    return Ok(());
                };
                geometry.epsg = self.epsg_code;
                let mut geometry_value = v.clone();
                for gml_geometry in &mut geometry_value.gml_geometries {
                    for polygon in &mut gml_geometry.polygons {
                        polygon.project_forward(projection)?;
                    }
                }
                geometry.value = GeometryValue::CityGmlGeometry(geometry_value);
                feature.geometry = geometry;
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geos) => {
                let projection = if let Some(projection) = &self.projection {
                    projection.clone()
                } else {
                    let Some(centroid) = geos.centroid() else {
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()),
                        );
                        return Ok(());
                    };
                    ExtendedTransverseMercatorProjection::new(
                        centroid.x(),
                        centroid.y(),
                        K,
                        &wgs84(),
                    )
                };
                let epsg = if let Some(epsg_code) = self.epsg_code {
                    Some(epsg_code)
                } else {
                    Some(EPSG_JGD2011_GEOGRAPHIC_2D)
                };
                let mut feature = feature.clone();
                let mut geometry = geometry.clone();
                let mut geos = geos.clone();
                geos.project_forward(&projection)?;
                geometry.value = GeometryValue::FlowGeometry2D(geos);
                geometry.epsg = epsg;
                feature.geometry = geometry;
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geos) => {
                let projection = if let Some(projection) = &self.projection {
                    projection.clone()
                } else {
                    let Some(centroid) = geos.centroid() else {
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()),
                        );
                        return Ok(());
                    };
                    ExtendedTransverseMercatorProjection::new(
                        centroid.x(),
                        centroid.y(),
                        K,
                        &wgs84(),
                    )
                };
                let epsg = if let Some(epsg_code) = self.epsg_code {
                    Some(epsg_code)
                } else {
                    Some(EPSG_JGD2011_GEOGRAPHIC_3D)
                };
                let mut feature = feature.clone();
                let mut geometry = geometry.clone();
                let mut geos = geos.clone();
                geos.project_forward(&projection)?;
                geometry.value = GeometryValue::FlowGeometry3D(geos);
                geometry.epsg = epsg;
                feature.geometry = geometry;
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()))
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
        "HorizontalReprojector"
    }
}
