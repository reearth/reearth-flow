//! Assigns a coordinate reference system to model-space (Euclidean) 3D
//! geometry, so readers that emit local model-space coordinates (glTF, OBJ)
//! can be tagged with a real geographic frame before reaching a CRS-only sink
//! such as the Cesium 3D Tiles writer.
//!
//! `declareCrs` treats the model's coordinates as already expressed in a given
//! CRS and simply tags them (after an optional up-axis flip). `anchor` will
//! place a local model at a geographic position, aligned to the local
//! east/north/up directions there; it is not implemented yet (Task 3).

use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode, UnitKind};
use reearth_flow_geometry::ops::{Affine3, Place};
use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Code, CodeType, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

/// Output port for features whose geometry could not be placed, or that
/// arrived already tagged with a coordinate reference system.
static REJECTED_PORT: Lazy<Port> = Lazy::new(|| Port::new("rejected"));

/// The source model's up axis.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum UpAxis {
    /// # Y Up
    /// The model's vertical axis is Y, the glTF and common OBJ convention.
    #[default]
    Y,
    /// # Z Up
    /// The model's vertical axis is already Z, so no rotation is applied.
    Z,
}

/// How the model is positioned on the globe.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Placement {
    /// # Declare CRS
    /// Treats the coordinates as already being in the given coordinate reference
    /// system and tags them without moving them.
    #[serde(rename_all = "camelCase")]
    DeclareCrs {
        /// # EPSG Code
        /// EPSG code the coordinates are already expressed in. Must use linear
        /// units, because model coordinates are distances rather than angles.
        epsg_code: u16,
    },
    /// # Anchor
    /// Places a local model at a geographic position, aligning it to the local
    /// east/north/up directions there.
    #[serde(rename_all = "camelCase")]
    Anchor {
        /// # Latitude
        /// Latitude of the anchor in degrees.
        latitude: Code<{ CodeType::FlowExpr as u32 }>,
        /// # Longitude
        /// Longitude of the anchor in degrees.
        longitude: Code<{ CodeType::FlowExpr as u32 }>,
        /// # Height
        /// Height of the anchor in metres above the ellipsoid.
        #[serde(default)]
        height: Option<Code<{ CodeType::FlowExpr as u32 }>>,
        /// # Heading
        /// Rotation of the model about its vertical axis, in degrees clockwise
        /// from north.
        #[serde(default)]
        heading: Option<Code<{ CodeType::FlowExpr as u32 }>>,
    },
}

/// # Model Georeferencer Parameters
/// Controls how model-space coordinates are oriented and positioned on the globe.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModelGeoreferencerParam {
    /// # Placement
    /// How the model is positioned on the globe.
    placement: Placement,
    /// # Up Axis
    /// The source model's vertical axis.
    #[serde(default)]
    up_axis: UpAxis,
}

#[derive(Debug, Clone, Default)]
pub struct ModelGeoreferencerFactory;

impl ProcessorFactory for ModelGeoreferencerFactory {
    fn name(&self) -> &str {
        "Model Georeferencer"
    }

    fn description(&self) -> &str {
        "Assigns a coordinate reference system to model-space 3D geometry, optionally anchoring it at a given latitude and longitude."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ModelGeoreferencerParam))
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
        vec![FEATURES_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: ModelGeoreferencerParam = {
            let with = with.ok_or_else(|| {
                GeometryProcessorError::ModelGeoreferencerFactory(
                    "Missing required parameter `with`".to_string(),
                )
            })?;
            let value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::ModelGeoreferencerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::ModelGeoreferencerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        };

        Ok(Box::new(ModelGeoreferencer {
            placement: params.placement,
            up_axis: params.up_axis,
        }))
    }
}

/// Tags model-space geometry with a coordinate reference system, rotating it
/// from the source model's up axis onto Z first.
#[derive(Debug, Clone)]
pub struct ModelGeoreferencer {
    placement: Placement,
    up_axis: UpAxis,
}

impl ModelGeoreferencer {
    /// Compute the affine and target frame for `self.placement` and apply them
    /// to `feature`'s geometry.
    fn place_feature(&self, feature: &mut Feature) -> Result<(), BoxedError> {
        let (affine, target) = match &self.placement {
            Placement::DeclareCrs { epsg_code } => {
                let epsg = EpsgCode::new(*epsg_code);
                validate_declared_crs(epsg)?;
                (axis_affine(&self.up_axis), CoordinateFrame::Crs(epsg))
            }
            Placement::Anchor { .. } => {
                return Err(Box::new(GeometryProcessorError::ModelGeoreferencer(
                    "anchor placement is not implemented yet".to_string(),
                )));
            }
        };
        feature.geometry_mut().place(&affine, &target).map_err(|e| {
            Box::new(GeometryProcessorError::ModelGeoreferencer(e.to_string())) as BoxedError
        })
    }
}

impl Processor for ModelGeoreferencer {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();

        if let Some(CoordinateFrame::Crs(epsg)) = top_level_frame(&feature.geometry) {
            ctx.event_hub.warn_log(
                Some(ctx.error_span()),
                format!("feature geometry is already tagged with CRS EPSG:{epsg}; skipping"),
            );
            fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
            return Ok(());
        }

        match self.place_feature(&mut feature) {
            Ok(()) => fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone())),
            Err(e) => {
                ctx.event_hub.warn_log(
                    Some(ctx.error_span()),
                    format!("georeferencing failed: {e}"),
                );
                fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
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
        "Model Georeferencer"
    }
}

/// The rotation that brings the source model's up axis onto Z.
fn axis_affine(up_axis: &UpAxis) -> Affine3 {
    match up_axis {
        // (x, y, z) -> (x, -z, y)
        UpAxis::Y => Affine3::new(
            [[1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]],
            [0.0; 3],
        ),
        UpAxis::Z => Affine3::identity(),
    }
}

/// Reject a declared CRS whose units are angular: model coordinates are metres.
/// An undeterminable CRS (unknown code, or PROJ cannot classify it) is
/// rejected too, rather than silently treated as usable.
fn validate_declared_crs(epsg: EpsgCode) -> Result<(), BoxedError> {
    match CoordinateFrame::Crs(epsg).unit_kind() {
        UnitKind::Linear => Ok(()),
        UnitKind::Angular => Err(Box::new(GeometryProcessorError::ModelGeoreferencer(format!(
            "EPSG:{} does not use linear units; model coordinates are distances, so declare a projected or geocentric CRS",
            epsg.get()
        ))) as BoxedError),
        UnitKind::Undeterminable(why) => Err(Box::new(GeometryProcessorError::ModelGeoreferencer(
            format!("EPSG:{} could not be classified: {why}", epsg.get()),
        )) as BoxedError),
    }
}

/// The coordinate frame carried by a feature's top-level geometry, where the
/// leaf type exposes one directly. `None` for `Csg`, `PointCloud`,
/// collections, 2D geometry, and no geometry — cases this action defers to
/// [`Place::place`]'s own error handling rather than pre-checking here.
fn top_level_frame(geometry: &Geometry) -> Option<&CoordinateFrame> {
    match geometry {
        Geometry::Euclidean3D(g) => match g {
            Euclidean3DGeometry::Point(p) => Some(p.frame()),
            Euclidean3DGeometry::LineString(l) => Some(l.frame()),
            Euclidean3DGeometry::Polygon(p) => Some(p.frame()),
            Euclidean3DGeometry::PolygonMesh(m) => Some(m.frame()),
            Euclidean3DGeometry::TriangularMesh(m) => Some(m.frame()),
            Euclidean3DGeometry::Solid(s) => Some(s.frame()),
            Euclidean3DGeometry::PointCloud(_)
            | Euclidean3DGeometry::Csg(_)
            | Euclidean3DGeometry::Collection(_) => None,
        },
        Geometry::Euclidean2D(_) | Geometry::None | Geometry::GeometryCollection(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::triangular_mesh::TriangularMesh3D;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};

    fn euclidean_mesh(soup: Vec<[f64; 3]>) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(
            TriangularMesh3D::from_soup(CoordinateFrame::Euclidean, soup),
        )))
    }

    #[test]
    fn y_up_declare_crs_flips_axes_and_tags_the_frame() {
        // The exact baked coordinate from the real PLATEAU content glb, which is
        // Y-up ECEF. Flipped to Z-up it must resolve to Japan (lat ~35.9, lon ~140.1);
        // unflipped it resolves to the Indian Ocean, so this pins the axis behaviour.
        let mut geometry = euclidean_mesh(vec![
            [-3958731.9, 3736419.1, -3309830.0],
            [-3958731.9, 3736419.1, -3309830.0],
            [-3958731.9, 3736419.1, -3309830.0],
        ]);
        let affine = axis_affine(&UpAxis::Y);
        geometry
            .place(&affine, &CoordinateFrame::Crs(EpsgCode::new(4978)))
            .unwrap();

        let v = match &geometry {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => m.vertices()[0],
            other => panic!("expected a triangular mesh, got {other:?}"),
        };
        let lon = v[1].atan2(v[0]).to_degrees();
        let lat = v[2].atan2((v[0] * v[0] + v[1] * v[1]).sqrt()).to_degrees();
        assert!((lat - 35.908).abs() < 0.01, "latitude was {lat}");
        assert!((lon - 140.102).abs() < 0.01, "longitude was {lon}");
    }

    #[test]
    fn z_up_declare_crs_leaves_coordinates_unchanged() {
        let mut geometry = euclidean_mesh(vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]);
        geometry
            .place(
                &axis_affine(&UpAxis::Z),
                &CoordinateFrame::Crs(EpsgCode::new(4978)),
            )
            .unwrap();
        let m = match &geometry {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => m,
            other => panic!("expected a triangular mesh, got {other:?}"),
        };
        assert!(m.vertices().contains(&[1.0, 2.0, 3.0]));
        assert_eq!(*m.frame(), CoordinateFrame::Crs(EpsgCode::new(4978)));
    }

    #[test]
    fn declare_crs_rejects_an_angular_unit_crs() {
        // EPSG:4326 is degrees; declaring metre model coordinates as degrees is
        // meaningless, so it must be rejected rather than silently accepted.
        assert!(validate_declared_crs(EpsgCode::new(4326)).is_err());
        assert!(validate_declared_crs(EpsgCode::new(4978)).is_ok());
    }
}
