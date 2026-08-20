use std::collections::HashMap;
use std::sync::Arc;

use reearth_flow_geometry::algorithm::bufferable::{buffer_polygon, Bufferable};
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::geometry::Geometry3D;
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::polygon::Polygon2D;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct BuffererFactory;

impl ProcessorFactory for BuffererFactory {
    fn name(&self) -> &str {
        "Bufferer"
    }

    fn description(&self) -> &str {
        "Creates a buffer polygon around each input geometry at a specified distance."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(Bufferer))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["spatial"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), REJECTED_PORT.clone()]
    }
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let bufferer: Bufferer = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::BuffererFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::BuffererFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::BuffererFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(bufferer))
    }
}

// TODO: add a `solid` buffer type. It needs a solid-buffering algorithm, and an
// edge-resolution control to go with it, that the geometry crate does not have yet.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
enum BufferType {
    /// # 2D Area Buffer
    /// Creates a flat polygon buffer around the input geometry, discarding any
    /// elevation it carried.
    #[serde(rename = "area2d")]
    Area2D,
}

/// # Bufferer Parameters
/// Configure the shape and extent of the buffer created around each geometry.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Bufferer {
    /// # Buffer Type
    /// Shape of buffer to create around the input geometry.
    buffer_type: BufferType,
    /// # Distance
    /// How far the buffer extends from the original geometry, in the units of
    /// the geometry's coordinate system. A negative distance contracts it.
    distance: f64,
    /// # Interpolation Angle
    /// Angular step in degrees used to approximate the rounded corners of a
    /// buffered point or curve. A smaller angle produces a smoother outline.
    /// Buffering a polygon does not use this value.
    interpolation_angle: f64,
}

impl Processor for Bufferer {
    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        match &geometry.value {
            // Nothing to buffer is not a failed buffer — pass it through.
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), FEATURES_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geos) => {
                self.handle_2d_geometry(geos, feature, geometry, &ctx, fw);
            }
            GeometryValue::FlowGeometry3D(geos) => {
                self.handle_3d_geometry(geos, feature, geometry, &ctx, fw);
            }
            GeometryValue::CityGmlGeometry(_) => {
                reject(&ctx, fw, "buffering this geometry is not supported");
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Bufferer"
    }
}

/// Route a feature the action cannot buffer to `rejected`. Emitting it on
/// `features` would leave it indistinguishable from a buffered one, and a
/// geometry this action does not handle should not panic the run.
#[cfg(not(feature = "new-geometry"))]
fn reject(ctx: &ExecutorContext, fw: &ProcessorChannelForwarder, reason: &str) {
    ctx.event_hub
        .debug_log(Some(ctx.error_span()), format!("buffer rejected: {reason}"));
    fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
}

impl Bufferer {
    #[cfg(not(feature = "new-geometry"))]
    fn handle_2d_geometry(
        &self,
        geos: &Geometry2D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        match self.buffer_type {
            BufferType::Area2D => match geos {
                Geometry2D::Point(point) => {
                    let mut feature = feature.clone();
                    let mut geometry = geometry.clone();
                    let coord = point.0;
                    geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::Polygon(
                        coord.to_polygon(self.distance, self.interpolation_angle),
                    ));
                    feature.geometry = Arc::new(geometry);
                    fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
                }
                Geometry2D::LineString(line_string) => {
                    let mut feature = feature.clone();
                    let mut geometry = geometry.clone();
                    geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::Polygon(
                        line_string.to_polygon(self.distance, self.interpolation_angle),
                    ));
                    feature.geometry = Arc::new(geometry);
                    fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
                }
                Geometry2D::Polygon(polygon) => {
                    let mut feature = feature.clone();
                    let mut geometry = geometry.clone();
                    if let Some(buffered) = buffer_polygon(polygon, self.distance) {
                        geometry.value =
                            GeometryValue::FlowGeometry2D(Geometry2D::Polygon(buffered));
                        feature.geometry = Arc::new(geometry);
                        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
                    } else {
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()),
                        );
                    }
                }
                Geometry2D::MultiPolygon(mp) => {
                    let mut feature = feature.clone();
                    let mut geometry = geometry.clone();
                    let buffered_polys: Vec<Polygon2D<f64>> =
                        mp.0.iter()
                            .filter_map(|poly| buffer_polygon(poly, self.distance))
                            .collect();
                    if buffered_polys.is_empty() {
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()),
                        );
                    } else {
                        geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::MultiPolygon(
                            reearth_flow_geometry::types::multi_polygon::MultiPolygon2D::new(
                                buffered_polys,
                            ),
                        ));
                        feature.geometry = Arc::new(geometry);
                        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
                    }
                }
                // TODO: buffer these types too — see the note on the 3D arm below.
                // They pass through unbuffered rather than going to `rejected`:
                // no standard implementation treats a geometry type as
                // un-bufferable, and a new port is unwired in existing workflows.
                _ => {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), FEATURES_PORT.clone()));
                }
            },
        }
    }

    #[cfg(not(feature = "new-geometry"))]
    fn handle_3d_geometry(
        &self,
        geos: &Geometry3D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        match self.buffer_type {
            BufferType::Area2D => match geos {
                Geometry3D::Point(point) => {
                    let mut feature = feature.clone();
                    let mut geometry = geometry.clone();
                    let coord = point.0;
                    // Convert 3D coordinate to 2D for buffering
                    let coord_2d = reearth_flow_geometry::types::coordinate::Coordinate2D {
                        x: coord.x,
                        y: coord.y,
                        z: reearth_flow_geometry::types::no_value::NoValue,
                    };
                    geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::Polygon(
                        coord_2d.to_polygon(self.distance, self.interpolation_angle),
                    ));
                    feature.geometry = Arc::new(geometry);
                    fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
                }
                Geometry3D::LineString(line_string) => {
                    let mut feature = feature.clone();
                    let mut geometry = geometry.clone();
                    let line_string: LineString2D<f64> = line_string.clone().into();
                    geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::Polygon(
                        line_string.to_polygon(self.distance, self.interpolation_angle),
                    ));
                    feature.geometry = Arc::new(geometry);
                    fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
                }
                Geometry3D::Polygon(polygon) => {
                    let mut feature = feature.clone();
                    let mut geometry = geometry.clone();
                    let polygon_2d: Polygon2D<f64> = polygon.clone().into();
                    if let Some(buffered) = buffer_polygon(&polygon_2d, self.distance) {
                        geometry.value =
                            GeometryValue::FlowGeometry2D(Geometry2D::Polygon(buffered));
                        feature.geometry = Arc::new(geometry);
                        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
                    } else {
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()),
                        );
                    }
                }
                // TODO: buffer these types too. Projecting to 2D is correct —
                // buffering is a planar operation everywhere (PostGIS: "This
                // function ignores the Z dimension. It always gives a 2D result
                // even when used on a 3D geometry") — but skipping the buffer is
                // not: JTS defines `buffer()` on the base Geometry type, so every
                // type has one. Multi-polygons in particular are buffered by the
                // 2D path above and silently are not here, which means a distance
                // tolerance is not applied to them. Fixing it moves quality-check
                // results, so it is tracked separately.
                _ => {
                    let value: Geometry2D = geos.clone().into();
                    let mut geometry = geometry.clone();
                    geometry.value = GeometryValue::FlowGeometry2D(value);
                    let mut feature = feature.clone();
                    feature.geometry = Arc::new(geometry);
                    fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
                }
            },
        }
    }
}
