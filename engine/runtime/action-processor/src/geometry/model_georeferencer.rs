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

        if let Some(epsg) = find_crs_frame(&feature.geometry) {
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
            "EPSG:{epsg} does not use linear units; model coordinates are distances, so declare a projected or geocentric CRS"
        ))) as BoxedError),
        UnitKind::Undeterminable(why) => Err(Box::new(GeometryProcessorError::ModelGeoreferencer(
            format!("EPSG:{epsg} could not be classified: {why}"),
        )) as BoxedError),
    }
}

/// The EPSG code of the first `Crs`-tagged leaf found anywhere in `geometry`,
/// searched recursively through `GeometryCollection` and `Collection3D`
/// members. `None` means every leaf reachable this way is `Euclidean` (or the
/// geometry is empty / 2D) — it does **not** mean placement is safe: `Csg` and
/// a `ScaledI32`-encoded `PointCloud` segment are left for [`Place::place`]'s
/// own error handling, since it already unconditionally rejects both
/// regardless of their current frame (a `Csg` carries no frame of its own; a
/// `ScaledI32` segment cannot represent a rotation). Every leaf `Place::place`
/// *would* silently accept and overwrite — `Point`, `LineString`, `Polygon`,
/// `PolygonMesh`, `TriangularMesh`, `Solid`, and an `F64`/`F32`-encoded
/// `PointCloud` — is inspected here, at any collection depth, so re-running
/// this action on already-georeferenced geometry is caught rather than
/// silently re-transforming and overwriting the frame.
fn find_crs_frame(geometry: &Geometry) -> Option<EpsgCode> {
    match geometry {
        Geometry::Euclidean3D(g) => find_crs_frame_3d(g),
        Geometry::Euclidean2D(_) | Geometry::None => None,
        Geometry::GeometryCollection(c) => c.members().iter().find_map(find_crs_frame),
    }
}

/// [`find_crs_frame`], scoped to an already-3D-embedded geometry.
fn find_crs_frame_3d(geometry: &Euclidean3DGeometry) -> Option<EpsgCode> {
    let frame = match geometry {
        Euclidean3DGeometry::Point(p) => p.frame(),
        Euclidean3DGeometry::PointCloud(pc) => pc.frame(),
        Euclidean3DGeometry::LineString(l) => l.frame(),
        Euclidean3DGeometry::Polygon(p) => p.frame(),
        Euclidean3DGeometry::PolygonMesh(m) => m.frame(),
        Euclidean3DGeometry::TriangularMesh(m) => m.frame(),
        Euclidean3DGeometry::Solid(s) => s.frame(),
        // No frame of its own (rejected outright by `Place::place`).
        Euclidean3DGeometry::Csg(_) => return None,
        Euclidean3DGeometry::Collection(c) => {
            return c.members().iter().find_map(find_crs_frame_3d)
        }
    };
    match frame {
        CoordinateFrame::Crs(epsg) => Some(*epsg),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::collection::Collection3D;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::point_cloud::PointCloud;
    use reearth_flow_geometry::triangular_mesh::TriangularMesh3D;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Attributes;

    use crate::tests::utils::create_default_execute_context;

    fn euclidean_mesh(soup: Vec<[f64; 3]>) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(
            TriangularMesh3D::from_soup(CoordinateFrame::Euclidean, soup),
        )))
    }

    fn crs_mesh(epsg: EpsgCode, soup: Vec<[f64; 3]>) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(
            TriangularMesh3D::from_soup(CoordinateFrame::Crs(epsg), soup),
        )))
    }

    /// Drive `ModelGeoreferencer::process` end to end through a real
    /// `ExecutorContext` / `ProcessorChannelForwarder`, mirroring how
    /// `center_point_replacer.rs` and `coordinate_extractor.rs` drive their
    /// own processors in tests.
    fn run_processor(
        feature: &Feature,
        processor: &mut ModelGeoreferencer,
    ) -> (Vec<Feature>, Vec<Port>) {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let ctx = create_default_execute_context(feature);
        processor.process(ctx, &fw).unwrap();
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let features = noop.send_features.lock().unwrap().clone();
            let ports = noop.send_ports.lock().unwrap().clone();
            (features, ports)
        } else {
            unreachable!()
        }
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

    #[test]
    fn process_places_euclidean_geometry_and_forwards_to_features() {
        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            euclidean_mesh(vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]),
        );
        let mut processor = ModelGeoreferencer {
            placement: Placement::DeclareCrs { epsg_code: 4978 },
            up_axis: UpAxis::Z,
        };

        let (features, ports) = run_processor(&feature, &mut processor);

        assert_eq!(ports, vec![FEATURES_PORT.clone()]);
        let m = match &*features[0].geometry {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => m,
            other => panic!("expected a triangular mesh, got {other:?}"),
        };
        assert!(m.vertices().contains(&[1.0, 2.0, 3.0]));
        assert_eq!(*m.frame(), CoordinateFrame::Crs(EpsgCode::new(4978)));
    }

    #[test]
    fn process_rejects_an_already_georeferenced_leaf_unchanged() {
        // Regression for the already-CRS guard: a feature that arrives already
        // tagged with a CRS must not be re-rotated/re-translated and silently
        // forwarded to `features` — it must go to `rejected`, untouched.
        let geometry = crs_mesh(
            EpsgCode::new(4978),
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
        );
        let feature =
            Feature::new_with_attributes_and_geometry(Attributes::new(), geometry.clone());
        let mut processor = ModelGeoreferencer {
            placement: Placement::DeclareCrs { epsg_code: 6677 },
            up_axis: UpAxis::Y,
        };

        let (features, ports) = run_processor(&feature, &mut processor);

        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
        assert_eq!(
            *features[0].geometry, geometry,
            "geometry must be untouched"
        );
    }

    #[test]
    fn process_rejects_an_already_georeferenced_point_cloud_unchanged() {
        // The shape Finding 1 named specifically: an F64-encoded PointCloud
        // already tagged with a CRS has no public `.frame()`-having sibling
        // check to fall back on before this fix, so `Place::place` would
        // silently re-transform and overwrite its frame.
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::PointCloud(Box::new(
            PointCloud::from_positions(
                CoordinateFrame::Crs(EpsgCode::new(4978)),
                vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
            ),
        )));
        let feature =
            Feature::new_with_attributes_and_geometry(Attributes::new(), geometry.clone());
        let mut processor = ModelGeoreferencer {
            placement: Placement::DeclareCrs { epsg_code: 6677 },
            up_axis: UpAxis::Y,
        };

        let (features, ports) = run_processor(&feature, &mut processor);

        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
        assert_eq!(
            *features[0].geometry, geometry,
            "point cloud must be untouched"
        );
    }

    #[test]
    fn process_rejects_a_collection_with_an_already_georeferenced_member_unchanged() {
        // The already-CRS guard must recurse into collection members, not just
        // check the top-level geometry.
        let already_crs =
            Euclidean3DGeometry::TriangularMesh(Box::new(TriangularMesh3D::from_soup(
                CoordinateFrame::Crs(EpsgCode::new(4978)),
                vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            )));
        let still_euclidean =
            Euclidean3DGeometry::TriangularMesh(Box::new(TriangularMesh3D::from_soup(
                CoordinateFrame::Euclidean,
                vec![[9.0, 9.0, 9.0], [8.0, 8.0, 8.0], [7.0, 7.0, 7.0]],
            )));
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([
            still_euclidean,
            already_crs,
        ])));
        let feature =
            Feature::new_with_attributes_and_geometry(Attributes::new(), geometry.clone());
        let mut processor = ModelGeoreferencer {
            placement: Placement::DeclareCrs { epsg_code: 6677 },
            up_axis: UpAxis::Y,
        };

        let (features, ports) = run_processor(&feature, &mut processor);

        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
        assert_eq!(
            *features[0].geometry, geometry,
            "collection must be untouched"
        );
    }
}
