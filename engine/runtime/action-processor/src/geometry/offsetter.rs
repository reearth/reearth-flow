use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(not(feature = "new-geometry"))]
use std::sync::Arc;

#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{Geometry, GeometryValue};

#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::Translate;
#[cfg(feature = "new-geometry")]
use reearth_flow_types::Feature;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct OffsetterFactory;

impl ProcessorFactory for OffsetterFactory {
    fn name(&self) -> &str {
        "Offsetter"
    }

    fn description(&self) -> &str {
        "Shifts every geometry coordinate by a fixed distance along each axis."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(OffsetterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["coordinate-system", "3d"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
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
        #[cfg(not(feature = "new-geometry"))]
        let process = Offsetter { params };
        #[cfg(feature = "new-geometry")]
        let process = Offsetter {
            delta: params.delta(),
        };
        Ok(Box::new(process))
    }
}

/// # Offsetter Parameters
/// Distances added to every geometry coordinate, one per axis.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OffsetterParam {
    /// # X Offset
    /// Distance added to every X coordinate; defaults to zero.
    offset_x: Option<f64>,
    /// # Y Offset
    /// Distance added to every Y coordinate; defaults to zero.
    offset_y: Option<f64>,
    /// # Z Offset
    /// Distance added to every Z coordinate; defaults to zero.
    offset_z: Option<f64>,
}

#[cfg(feature = "new-geometry")]
impl OffsetterParam {
    /// The configured offsets as a translation vector, with an absent axis
    /// taken as zero.
    fn delta(&self) -> [f64; 3] {
        [
            self.offset_x.unwrap_or(0f64),
            self.offset_y.unwrap_or(0f64),
            self.offset_z.unwrap_or(0f64),
        ]
    }
}

#[cfg(not(feature = "new-geometry"))]
#[derive(Debug, Clone)]
pub struct Offsetter {
    params: OffsetterParam,
}

/// Shifts a feature's geometry by a fixed translation vector.
#[cfg(feature = "new-geometry")]
#[derive(Debug, Clone)]
pub struct Offsetter {
    delta: [f64; 3],
}

#[cfg(feature = "new-geometry")]
impl Offsetter {
    /// A copy of `feature` with its geometry shifted by the configured delta.
    fn offset(&self, feature: &Feature) -> Result<Feature, reearth_flow_geometry::error::Error> {
        let mut feature = feature.clone();
        feature.geometry_mut().translate(self.delta)?;
        Ok(feature)
    }
}

impl Processor for Offsetter {
    fn num_threads(&self) -> usize {
        2
    }

    #[cfg(not(feature = "new-geometry"))]
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
                feature.geometry = Arc::new(Geometry {
                    epsg,
                    value: GeometryValue::CityGmlGeometry(geos),
                });
            }
            GeometryValue::FlowGeometry3D(mut geos) => {
                geos.transform_offset(
                    self.params.offset_x.unwrap_or(0f64),
                    self.params.offset_y.unwrap_or(0f64),
                    self.params.offset_z.unwrap_or(0f64),
                );
                feature.geometry = Arc::new(Geometry {
                    epsg,
                    value: GeometryValue::FlowGeometry3D(geos),
                });
            }
            GeometryValue::None | GeometryValue::FlowGeometry2D(..) => {}
        }
        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
        Ok(())
    }

    /// Shift the feature's geometry by the configured offsets. Every geometry
    /// type supports translation, so a failure here is a node error rather than
    /// a per-feature rejection.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = self.offset(&ctx.feature).map_err(|e| {
            GeometryProcessorError::Offsetter(format!("Failed to offset geometry: {e}"))
        })?;
        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
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
        "Offsetter"
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::line_string::{LineString2D, LineString3D};
    use reearth_flow_geometry::ops::{Aabb, BoundingBox};
    use reearth_flow_geometry::point_cloud::PointCloud;
    use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

    fn offsetter(delta: [f64; 3]) -> Offsetter {
        Offsetter { delta }
    }

    fn offset(delta: [f64; 3], geometry: Geometry) -> Geometry {
        let feature = Feature::from(geometry);
        let shifted = offsetter(delta).offset(&feature).unwrap();
        (*shifted.geometry).clone()
    }

    #[test]
    fn every_axis_of_a_three_dimensional_geometry_shifts() {
        let line = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
        );
        let shifted = offset(
            [10.0, 20.0, 30.0],
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(line)),
        );
        let Geometry::Euclidean3D(Euclidean3DGeometry::LineString(line)) = shifted else {
            panic!("expected a 3D line string");
        };
        assert_eq!(line.coords(), [[11.0, 22.0, 33.0], [14.0, 25.0, 36.0]]);
    }

    #[test]
    fn a_two_dimensional_geometry_shifts_in_the_plane() {
        let line = LineString2D::from_coords(CoordinateFrame::Euclidean, [[1.0, 2.0], [3.0, 4.0]]);
        let shifted = offset(
            [10.0, 20.0, 30.0],
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(line)),
        );
        let Geometry::Euclidean2D(Euclidean2DGeometry::LineString(line)) = shifted else {
            panic!("expected a 2D line string");
        };
        assert_eq!(line.coords(), [[11.0, 22.0], [13.0, 24.0]]);
        assert_eq!(line.elevation(), None);
    }

    #[test]
    fn a_two_and_a_half_dimensional_geometry_shifts_its_elevation() {
        let line = LineString2D::from_coords_at_elevation(
            CoordinateFrame::Euclidean,
            [[1.0, 2.0], [3.0, 4.0]],
            5.0,
        );
        let shifted = offset(
            [10.0, 20.0, 30.0],
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(line)),
        );
        let Geometry::Euclidean2D(Euclidean2DGeometry::LineString(line)) = shifted else {
            panic!("expected a 2D line string");
        };
        assert_eq!(line.coords(), [[11.0, 22.0], [13.0, 24.0]]);
        assert_eq!(line.elevation(), Some(35.0));
    }

    #[test]
    fn an_absent_offset_leaves_its_axis_alone() {
        let params: OffsetterParam = serde_json::from_value(serde_json::json!({
            "offsetZ": 0.003,
        }))
        .unwrap();
        assert_eq!(params.delta(), [0.0, 0.0, 0.003]);
    }

    #[test]
    fn a_feature_without_geometry_passes_through() {
        assert_eq!(offset([1.0, 2.0, 3.0], Geometry::None), Geometry::None);
    }

    #[test]
    fn every_sample_of_a_point_cloud_shifts() {
        let cloud = PointCloud::from_positions(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
        );
        let shifted = offset(
            [1.0, 2.0, 3.0],
            Geometry::Euclidean3D(Euclidean3DGeometry::PointCloud(Box::new(cloud))),
        );
        assert_eq!(
            shifted.bounding_box().unwrap(),
            Aabb::D3 {
                min: [1.0, 2.0, 3.0],
                max: [2.0, 3.0, 4.0]
            }
        );
    }
}
