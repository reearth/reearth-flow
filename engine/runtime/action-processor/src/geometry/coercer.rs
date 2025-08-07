use std::collections::HashMap;

use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::geometry::Geometry3D;
use reearth_flow_geometry::types::multi_line_string::{MultiLineString2D, MultiLineString3D};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{CityGmlGeometry, Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct GeometryCoercerFactory;

impl ProcessorFactory for GeometryCoercerFactory {
    fn name(&self) -> &str {
        "GeometryCoercer"
    }

    fn description(&self) -> &str {
        "Coerces and converts feature geometries to specified target geometry types"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeometryCoercer))
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
        let coercer: GeometryCoercer = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::GeometryCoercerFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::GeometryCoercerFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::GeometryCoercerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(coercer))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum CoercerType {
    LineString,
}

/// # GeometryCoercer Parameters
///
/// Configuration for coercing geometries to specific target types.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct GeometryCoercer {
    /// Target geometry type to coerce features to (e.g., LineString)
    coercer_type: CoercerType,
}

impl Processor for GeometryCoercer {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), DEFAULT_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geos) => {
                self.handle_2d_geometry(geos, feature, geometry, &ctx, fw);
            }
            GeometryValue::FlowGeometry3D(geos) => {
                self.handle_3d_geometry(geos, feature, geometry, &ctx, fw);
            }
            GeometryValue::CityGmlGeometry(geos) => {
                self.handle_city_gml_geometry(geos, feature, geometry, &ctx, fw);
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "GeometryCoercer"
    }
}

impl GeometryCoercer {
    fn handle_2d_geometry(
        &self,
        geos: &Geometry2D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        match geos {
            Geometry2D::Polygon(polygon) => {
                let mut feature = feature.clone();
                match self.coercer_type {
                    CoercerType::LineString => {
                        let line_strings = polygon.rings().to_vec();
                        let geo = if let Some(first) = line_strings.first() {
                            if line_strings.len() == 1 {
                                Geometry2D::LineString(first.clone())
                            } else {
                                Geometry2D::MultiLineString(MultiLineString2D::new(line_strings))
                            }
                        } else {
                            return;
                        };
                        let mut geometry = geometry.clone();
                        geometry.value = GeometryValue::FlowGeometry2D(geo);
                        feature.geometry = geometry;
                    }
                }
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            Geometry2D::MultiPolygon(polygons) => {
                let mut feature = feature.clone();
                match self.coercer_type {
                    CoercerType::LineString => {
                        let mut geometries = Vec::<Geometry2D>::new();
                        for polygon in polygons.iter() {
                            let line_strings = polygon.rings().to_vec();
                            if let Some(first) = line_strings.first() {
                                let geometry = if line_strings.len() == 1 {
                                    Geometry2D::LineString(first.clone())
                                } else {
                                    Geometry2D::MultiLineString(MultiLineString2D::new(
                                        line_strings,
                                    ))
                                };
                                geometries.push(geometry);
                            }
                        }
                        let geo = if let Some(first) = geometries.first() {
                            if geometries.len() == 1 {
                                first.clone()
                            } else {
                                Geometry2D::GeometryCollection(geometries)
                            }
                        } else {
                            return;
                        };
                        let mut geometry = geometry.clone();
                        geometry.value = GeometryValue::FlowGeometry2D(geo);
                        feature.geometry = geometry;
                    }
                }
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            _ => unimplemented!(),
        }
    }

    fn handle_3d_geometry(
        &self,
        geos: &Geometry3D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        match geos {
            Geometry3D::Polygon(polygon) => {
                let mut feature = feature.clone();
                match self.coercer_type {
                    CoercerType::LineString => {
                        let line_strings = polygon.rings().to_vec();
                        let geo = if let Some(first) = line_strings.first() {
                            if line_strings.len() == 1 {
                                Geometry3D::LineString(first.clone())
                            } else {
                                Geometry3D::MultiLineString(MultiLineString3D::new(line_strings))
                            }
                        } else {
                            return;
                        };
                        let mut geometry = geometry.clone();
                        geometry.value = GeometryValue::FlowGeometry3D(geo);
                        feature.geometry = geometry;
                    }
                }
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            Geometry3D::MultiPolygon(polygons) => {
                let mut feature = feature.clone();
                match self.coercer_type {
                    CoercerType::LineString => {
                        let mut geometries = Vec::<Geometry3D>::new();
                        for polygon in polygons.iter() {
                            let line_strings = polygon.rings().to_vec();
                            if let Some(first) = line_strings.first() {
                                let geometry = if line_strings.len() == 1 {
                                    Geometry3D::LineString(first.clone())
                                } else {
                                    Geometry3D::MultiLineString(MultiLineString3D::new(
                                        line_strings,
                                    ))
                                };
                                geometries.push(geometry);
                            }
                        }
                        let geo = if let Some(first) = geometries.first() {
                            if geometries.len() == 1 {
                                first.clone()
                            } else {
                                Geometry3D::GeometryCollection(geometries)
                            }
                        } else {
                            return;
                        };
                        let mut geometry = geometry.clone();
                        geometry.value = GeometryValue::FlowGeometry3D(geo);
                        feature.geometry = geometry;
                    }
                }
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            _ => unimplemented!(),
        }
    }

    fn handle_city_gml_geometry(
        &self,
        geos: &CityGmlGeometry,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        geos.gml_geometries.iter().for_each(|geo_feature| {
            let mut geometries = Vec::<Geometry3D>::new();
            match &self.coercer_type {
                CoercerType::LineString => {
                    for polygon in geo_feature.polygons.iter() {
                        let line_strings = polygon.rings().to_vec();
                        if let Some(first) = line_strings.first() {
                            let geometry = if line_strings.len() == 1 {
                                Geometry3D::LineString(first.clone())
                            } else {
                                Geometry3D::MultiLineString(MultiLineString3D::new(line_strings))
                            };
                            geometries.push(geometry);
                        }
                    }
                    let geo = if let Some(first) = geometries.first() {
                        if geometries.len() == 1 {
                            first.clone()
                        } else {
                            Geometry3D::GeometryCollection(geometries)
                        }
                    } else {
                        return;
                    };
                    let mut geometry = geometry.clone();
                    geometry.value = GeometryValue::FlowGeometry3D(geo);
                    let mut feature = feature.clone();
                    feature.refresh_id();
                    feature.geometry = geometry;
                    fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                }
            }
        });
    }
}
