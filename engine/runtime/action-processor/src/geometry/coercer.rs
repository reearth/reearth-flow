use std::sync::Arc;

use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::geometry::Geometry3D;
use reearth_flow_geometry::types::multi_line_string::{MultiLineString2D, MultiLineString3D};
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_geometry::types::triangular_mesh::TriangularMesh;
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
use std::collections::HashMap;

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
enum CoerceTarget {
    LineString,
    Polygon,
    TriangularMesh,
}

/// # GeometryCoercer Parameters
///
/// Configuration for coercing geometries to specific target types.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct GeometryCoercer {
    /// Target geometry type to coerce features to (e.g., LineString)
    target_type: CoerceTarget,
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
                self.handle_2d_geometry(geos, feature, geometry, &ctx, fw)?;
            }
            GeometryValue::FlowGeometry3D(geos) => {
                self.handle_3d_geometry(geos, feature, geometry, &ctx, fw)?;
            }
            GeometryValue::CityGmlGeometry(geos) => {
                self.handle_city_gml_geometry(geos, feature, geometry, &ctx, fw)?;
            }
        }
        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
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
    ) -> Result<(), String> {
        match geos {
            Geometry2D::LineString(line_string) => {
                let mut feature = feature.clone();
                match self.target_type {
                    CoerceTarget::LineString => {
                        // Already a LineString, no conversion needed
                        // Keep as is
                    }
                    CoerceTarget::Polygon => {
                        // Check if the LineString is closed (first point equals last point)
                        if line_string.0.len() >= 4 && line_string.0.first() == line_string.0.last()
                        {
                            // It's closed, convert to a Polygon with this as the exterior ring
                            let polygon = Polygon2D::new(line_string.clone(), vec![]);
                            let mut geometry = geometry.clone();
                            geometry.value =
                                GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon));
                            feature.geometry = Arc::new(geometry);
                        } else {
                            return Err(
                                "Cannot convert to Polygon: LineString is not closed".to_string()
                            );
                        }
                    }
                    CoerceTarget::TriangularMesh => Err("Not supported".to_string())?,
                }
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            Geometry2D::Polygon(polygon) => {
                let mut feature = feature.clone();
                match self.target_type {
                    CoerceTarget::LineString => {
                        let line_strings = polygon.rings().to_vec();
                        let geo = if let Some(first) = line_strings.first() {
                            if line_strings.len() == 1 {
                                Geometry2D::LineString(first.clone())
                            } else {
                                Geometry2D::MultiLineString(MultiLineString2D::new(line_strings))
                            }
                        } else {
                            return Ok(());
                        };
                        let mut geometry = geometry.clone();
                        geometry.value = GeometryValue::FlowGeometry2D(geo);
                        feature.geometry = Arc::new(geometry);
                    }
                    CoerceTarget::Polygon => {
                        // Already a polygon, no conversion needed
                        // Keep as is
                    }
                    CoerceTarget::TriangularMesh => Err("Not supported".to_string())?,
                }
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            Geometry2D::MultiPolygon(polygons) => {
                let mut feature = feature.clone();
                match self.target_type {
                    CoerceTarget::LineString => {
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
                            return Ok(());
                        };
                        let mut geometry = geometry.clone();
                        geometry.value = GeometryValue::FlowGeometry2D(geo);
                        feature.geometry = Arc::new(geometry);
                    }
                    CoerceTarget::Polygon => {
                        // Already MultiPolygon, no direct conversion to single Polygon
                        // Keep as is or convert to GeometryCollection if there's one polygon
                    }
                    CoerceTarget::TriangularMesh => Err("Not supported".to_string())?,
                }
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            _ => return Err("Not supported".to_string()), // Not supported
        }
        Ok(())
    }

    fn handle_3d_geometry(
        &self,
        geos: &Geometry3D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), String> {
        match geos {
            Geometry3D::LineString(line_string) => {
                let mut feature = feature.clone();
                match self.target_type {
                    CoerceTarget::LineString => {
                        // Already a LineString, no conversion needed
                        // Keep as is
                    }
                    CoerceTarget::Polygon => {
                        // Check if the LineString is closed (first point equals last point)
                        if line_string.0.len() >= 4 && line_string.0.first() == line_string.0.last()
                        {
                            // It's closed, convert to a Polygon with this as the exterior ring
                            let polygon = Polygon3D::new(line_string.clone(), vec![]);
                            let mut geometry = geometry.clone();
                            geometry.value =
                                GeometryValue::FlowGeometry3D(Geometry3D::Polygon(polygon));
                            feature.geometry = Arc::new(geometry);
                        } else {
                            return Err(
                                "Cannot convert to Polygon: LineString is not closed".to_string()
                            );
                        }
                    }
                    CoerceTarget::TriangularMesh => {
                        return Err("not supported".to_string())?;
                    }
                }
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            Geometry3D::Polygon(polygon) => {
                let mut feature = feature.clone();
                match self.target_type {
                    CoerceTarget::LineString => {
                        let line_strings = polygon.rings().to_vec();
                        let geo = if let Some(first) = line_strings.first() {
                            if line_strings.len() == 1 {
                                Geometry3D::LineString(first.clone())
                            } else {
                                Geometry3D::MultiLineString(MultiLineString3D::new(line_strings))
                            }
                        } else {
                            return Ok(());
                        };
                        let mut geometry = geometry.clone();
                        geometry.value = GeometryValue::FlowGeometry3D(geo);
                        feature.geometry = Arc::new(geometry);
                    }
                    CoerceTarget::Polygon => {
                        // Already a polygon, no conversion needed
                        // Keep as is
                    }
                    CoerceTarget::TriangularMesh => {
                        let faces = polygon.rings();
                        let triangular_mesh = TriangularMesh::<f64, f64>::from_faces(&faces, None)?;
                        let mut geometry = geometry.clone();
                        geometry.value = GeometryValue::FlowGeometry3D(Geometry3D::TriangularMesh(
                            triangular_mesh,
                        ));
                        feature.geometry = Arc::new(geometry);
                    }
                }
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            Geometry3D::MultiPolygon(polygons) => {
                let mut feature = feature.clone();
                match self.target_type {
                    CoerceTarget::LineString => {
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
                            return Ok(());
                        };
                        let mut geometry = geometry.clone();
                        geometry.value = GeometryValue::FlowGeometry3D(geo);
                        feature.geometry = Arc::new(geometry);
                    }
                    CoerceTarget::Polygon => {
                        // Already MultiPolygon, no direct conversion to single Polygon
                    }
                    CoerceTarget::TriangularMesh => {
                        let faces: Vec<_> = polygons.iter().flat_map(|p| p.rings()).collect();
                        let triangular_mesh = TriangularMesh::<f64, f64>::from_faces(&faces, None)?;
                        let mut geometry = geometry.clone();
                        geometry.value = GeometryValue::FlowGeometry3D(Geometry3D::TriangularMesh(
                            triangular_mesh,
                        ));
                        feature.geometry = Arc::new(geometry);
                    }
                }
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            _ => return Err("Not supported".to_string()), // Not supported
        };
        Ok(())
    }

    fn handle_city_gml_geometry(
        &self,
        geos: &CityGmlGeometry,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), String> {
        for geo_feature in geos.gml_geometries.iter() {
            let mut geometries = Vec::<Geometry3D>::new();
            match &self.target_type {
                CoerceTarget::LineString => {
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
                        return Ok(());
                    };
                    let mut geometry = geometry.clone();
                    geometry.value = GeometryValue::FlowGeometry3D(geo);
                    let mut feature = feature.clone();
                    feature.refresh_id();
                    feature.geometry = Arc::new(geometry);
                    fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                }
                CoerceTarget::Polygon => {
                    // For CityGML, we already have polygons, so we just pass them through
                    for polygon in geo_feature.polygons.iter() {
                        geometries.push(Geometry3D::Polygon(polygon.clone()));
                    }
                    let geo = if let Some(first) = geometries.first() {
                        if geometries.len() == 1 {
                            first.clone()
                        } else {
                            Geometry3D::GeometryCollection(geometries)
                        }
                    } else {
                        return Ok(());
                    };
                    let mut geometry = geometry.clone();
                    geometry.value = GeometryValue::FlowGeometry3D(geo);
                    let mut feature = feature.clone();
                    feature.refresh_id();
                    feature.geometry = Arc::new(geometry);
                    fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                }
                CoerceTarget::TriangularMesh => {
                    for polygon in geo_feature.polygons.iter() {
                        let triangular_mesh = TriangularMesh::<f64, f64>::try_from_polygons(
                            vec![polygon.clone()],
                            None,
                        )?;
                        geometries.push(Geometry3D::TriangularMesh(triangular_mesh));
                    }
                    let geo = if let Some(first) = geometries.first() {
                        if geometries.len() == 1 {
                            first.clone()
                        } else {
                            Geometry3D::GeometryCollection(geometries)
                        }
                    } else {
                        return Ok(());
                    };
                    let mut geometry = geometry.clone();
                    geometry.value = GeometryValue::FlowGeometry3D(geo);
                    let mut feature = feature.clone();
                    feature.refresh_id();
                    feature.geometry = Arc::new(geometry);
                    fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                }
            }
        }
        Ok(())
    }
}
