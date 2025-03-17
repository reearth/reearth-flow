use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::{
    algorithm::{bool_ops::BooleanOps, coordinate_meter_converter::meter_to_coordinate_diff},
    types::{
        coordinate::{Coordinate, Coordinate2D},
        line_string::{LineString2D, LineString3D},
        multi_polygon::{MultiPolygon2D, MultiPolygon3D},
        polygon::{Polygon2D, Polygon3D},
    },
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

pub static FOOTPRINT_PORT: Lazy<Port> = Lazy::new(|| Port::new("footprint"));

#[derive(Debug, Clone, Default)]
pub struct SurfaceFootprintReplacerFactory;

impl ProcessorFactory for SurfaceFootprintReplacerFactory {
    fn name(&self) -> &str {
        "SurfaceFootprintReplacer"
    }

    fn description(&self) -> &str {
        "Replace the geometry with its footprint"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(SurfaceFootprintReplacerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FOOTPRINT_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: SurfaceFootprintReplacerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::SurfaceFootprintReplacerFactory(format!(
                    "Failed to serialize 'with' parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::SurfaceFootprintReplacerFactory(format!(
                    "Failed to deserialize 'with' parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::SurfaceFootprintReplacerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        enum ShadowMode {
            OverheadShadow,
            DropShadow,
        }

        let shadow_mode = match param.shadow_mode.as_deref() {
            Some("OverheadShadow") => ShadowMode::OverheadShadow,
            Some("DropShadow") => ShadowMode::DropShadow,
            Some(_) => {
                return Err(GeometryProcessorError::SurfaceFootprintReplacerFactory(
                    "Invalid shadow mode".to_string(),
                )
                .into())
            }
            None => ShadowMode::OverheadShadow,
        };

        let unit_light_direction_opt = {
            if let Some(light_direction) = param.light_direction {
                if light_direction[2] == 0.0 {
                    return Err(GeometryProcessorError::SurfaceFootprintReplacerFactory(
                        "Invalid light direction (z cannot be 0)".to_string(),
                    )
                    .into());
                }
                let norm = (light_direction[0].powi(2)
                    + light_direction[1].powi(2)
                    + light_direction[2].powi(2))
                .sqrt();

                Some([
                    light_direction[0] / norm,
                    light_direction[1] / norm,
                    light_direction[2] / norm,
                ])
            } else {
                None
            }
        };

        let unit_light_direction = match shadow_mode {
            ShadowMode::OverheadShadow => [0.0, 0.0, 1.0],
            ShadowMode::DropShadow => {
                if let Some(unit_light_direction) = unit_light_direction_opt {
                    unit_light_direction
                } else {
                    return Err(GeometryProcessorError::SurfaceFootprintReplacerFactory(
                        "Missing light direction".to_string(),
                    )
                    .into());
                }
            }
        };

        let elevation = match shadow_mode {
            ShadowMode::OverheadShadow | ShadowMode::DropShadow => {
                if let Some(elevation) = param.elevation {
                    elevation
                } else {
                    return Err(GeometryProcessorError::SurfaceFootprintReplacerFactory(
                        "Missing elevation".to_string(),
                    )
                    .into());
                }
            }
        };

        let process = SurfaceFootprintReplacer {
            elevation,
            unit_light_direction,
        };

        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceFootprintReplacerParam {
    shadow_mode: Option<String>,
    elevation: Option<f64>,
    light_direction: Option<[f64; 3]>,
}

#[derive(Debug, Clone)]
pub struct SurfaceFootprintReplacer {
    elevation: f64,
    unit_light_direction: [f64; 3],
}

impl Processor for SurfaceFootprintReplacer {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(_) => {
                if let Some(footprint) = self.create_footprint_3d(feature) {
                    fw.send(ctx.new_with_feature_and_port(footprint, FOOTPRINT_PORT.clone()));
                } else {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                }
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(&self, _: NodeContext, _: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "SurfaceFootprintReplacer"
    }
}

impl SurfaceFootprintReplacer {
    fn create_footprint_3d(&self, buffered_feature_3d: &Feature) -> Option<Feature> {
        let geom = buffered_feature_3d.geometry.value.as_flow_geometry_3d()?;

        let polygons = match geom {
            reearth_flow_geometry::types::geometry::Geometry3D::Polygon(poly) => vec![poly.clone()],
            reearth_flow_geometry::types::geometry::Geometry3D::MultiPolygon(mpoly) => {
                mpoly.0.clone()
            }
            _ => return None,
        };

        if polygons.is_empty() {
            return None;
        }

        let mut projected_polygons = Vec::new();

        // project
        for polygon in polygons {
            let mut projected_exterior = Vec::new();
            for point in polygon.exterior().coords() {
                let projected_point =
                    project_point_to_elevation(point, self.elevation, self.unit_light_direction);
                projected_exterior.push(Coordinate2D::new_(projected_point.x, projected_point.y));
            }
            let projected_polygon = Polygon2D::new(LineString2D::new(projected_exterior), vec![]);
            projected_polygons.push(projected_polygon);
        }

        // union
        let combined_polygons =
            projected_polygons
                .iter()
                .fold(None, |acc: Option<MultiPolygon2D<f64>>, polygon| {
                    let multi_polygon = MultiPolygon2D::new(vec![polygon.clone()]);
                    if let Some(acc) = acc {
                        Some(acc.union(&multi_polygon))
                    } else {
                        Some(multi_polygon)
                    }
                })?;

        // 2d -> 3d
        let combined_polygons = combined_polygons
            .iter()
            .map(|polygon| {
                let coords = polygon
                    .exterior()
                    .iter()
                    .map(|point| Coordinate::new__(point.x, point.y, self.elevation))
                    .collect::<Vec<_>>();
                Polygon3D::new(LineString3D::new(coords), vec![])
            })
            .collect::<Vec<_>>();

        let mut feature = buffered_feature_3d.clone();
        feature.geometry.value = GeometryValue::FlowGeometry3D(
            reearth_flow_geometry::types::geometry::Geometry3D::MultiPolygon(MultiPolygon3D::new(
                combined_polygons,
            )),
        );

        Some(feature)
    }
}

fn project_point_to_elevation(
    point: &Coordinate<f64, f64>,
    elevation: f64,
    light_dir: [f64; 3],
) -> Coordinate<f64, f64> {
    let height = elevation - point.z;

    let offset_meter_x = height * light_dir[0] / light_dir[2];
    let offset_meter_y = height * light_dir[1] / light_dir[2];
    let (offset_lng, offset_lat) = meter_to_coordinate_diff(offset_meter_x, offset_meter_y);

    Coordinate::new__(point.x + offset_lng, point.y + offset_lat, elevation)
}
