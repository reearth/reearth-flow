//! Assigns a coordinate reference system to model-space (Euclidean) 3D
//! geometry, so readers that emit local model-space coordinates (glTF, OBJ)
//! can be tagged with a real geographic frame before reaching a CRS-only sink
//! such as the Cesium 3D Tiles writer.
//!
//! `declareCrs` treats the model's coordinates as already expressed in a given
//! CRS and simply tags them (after an optional up-axis flip). `anchor` places a
//! local model at a geographic position, aligned to the local east/north/up
//! directions there, via a local ENU basis on the WGS 84 ellipsoid.

use std::collections::HashMap;
use std::sync::Arc;

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
use reearth_flow_types::{Code, CodeType, CompiledCode, Feature};
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

/// [`Placement`], resolved for a built processor: `anchor`'s expression
/// parameters are precompiled once here (mirroring
/// `coordinate_frame_reprojector.rs`'s `BasePointSource`, which compiles its
/// `base_point` expression once in `build()` and stores the `CompiledCode`),
/// rather than re-parsed on every feature `process()` handles.
#[derive(Debug, Clone)]
enum ResolvedPlacement {
    DeclareCrs { epsg_code: u16 },
    // Boxed: `CompiledCode` is large enough that an unboxed `Anchor` variant
    // would make every `ResolvedPlacement` (including the far smaller
    // `DeclareCrs`) pay for its size (`clippy::large_enum_variant`).
    Anchor(Box<ResolvedAnchor>),
}

/// The precompiled `anchor` expression parameters, boxed out of
/// [`ResolvedPlacement::Anchor`] to keep that enum small.
#[derive(Debug, Clone)]
struct ResolvedAnchor {
    latitude: CompiledCode,
    longitude: CompiledCode,
    height: Option<CompiledCode>,
    heading: Option<CompiledCode>,
}

/// Compile an `anchor` placement expression parameter at build time. `name`
/// identifies the parameter (`"latitude"`, `"longitude"`, `"height"`, or
/// `"heading"`) in the error message on a syntax error.
fn compile_anchor_expr(
    code: &Code<{ CodeType::FlowExpr as u32 }>,
    name: &'static str,
) -> Result<CompiledCode, BoxedError> {
    code.compile().map_err(|e| {
        Box::new(GeometryProcessorError::ModelGeoreferencerFactory(format!(
            "failed to compile anchor `{name}` expression: {e:?}"
        ))) as BoxedError
    })
}

impl Placement {
    /// Resolve into [`ResolvedPlacement`], compiling `anchor`'s expression
    /// parameters.
    fn resolve(self) -> Result<ResolvedPlacement, BoxedError> {
        match self {
            Placement::DeclareCrs { epsg_code } => Ok(ResolvedPlacement::DeclareCrs { epsg_code }),
            Placement::Anchor {
                latitude,
                longitude,
                height,
                heading,
            } => Ok(ResolvedPlacement::Anchor(Box::new(ResolvedAnchor {
                latitude: compile_anchor_expr(&latitude, "latitude")?,
                longitude: compile_anchor_expr(&longitude, "longitude")?,
                height: height
                    .map(|code| compile_anchor_expr(&code, "height"))
                    .transpose()?,
                heading: heading
                    .map(|code| compile_anchor_expr(&code, "heading"))
                    .transpose()?,
            }))),
        }
    }
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
            placement: params.placement.resolve()?,
            up_axis: params.up_axis,
        }))
    }
}

/// Tags model-space geometry with a coordinate reference system, rotating it
/// from the source model's up axis onto Z first.
#[derive(Debug, Clone)]
pub struct ModelGeoreferencer {
    placement: ResolvedPlacement,
    up_axis: UpAxis,
}

impl ModelGeoreferencer {
    /// Compute the affine and target frame for `self.placement` and apply them
    /// to `feature`'s geometry. `env_vars` is threaded through to evaluate the
    /// `anchor` placement's expression parameters against `feature`.
    fn place_feature(
        &self,
        feature: &mut Feature,
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<(), BoxedError> {
        let (affine, target) = match &self.placement {
            ResolvedPlacement::DeclareCrs { epsg_code } => {
                let epsg = EpsgCode::new(*epsg_code);
                validate_declared_crs(epsg)?;
                (axis_affine(&self.up_axis), CoordinateFrame::Crs(epsg))
            }
            ResolvedPlacement::Anchor(anchor) => {
                let lat = eval_expr_f64(&anchor.latitude, "latitude", feature, env_vars.clone())?;
                let lon = eval_expr_f64(&anchor.longitude, "longitude", feature, env_vars.clone())?;
                let height_m = match &anchor.height {
                    Some(code) => eval_expr_f64(code, "height", feature, env_vars.clone())?,
                    None => 0.0,
                };
                let heading_deg = match &anchor.heading {
                    Some(code) => eval_expr_f64(code, "heading", feature, env_vars.clone())?,
                    None => 0.0,
                };
                (
                    anchor_affine(lat, lon, height_m, heading_deg, &self.up_axis),
                    CoordinateFrame::Crs(EpsgCode::new(4978)),
                )
            }
        };
        feature.geometry_mut().place(&affine, &target).map_err(|e| {
            Box::new(GeometryProcessorError::ModelGeoreferencer(e.to_string())) as BoxedError
        })
    }
}

/// Evaluate an already-compiled `anchor` placement expression parameter to an
/// `f64` against `feature`, the same way `coordinate_frame_reprojector.rs`
/// evaluates its precompiled `base_point` expression: evaluate, and treat any
/// failure -- to evaluate, or to coerce the result to a number -- as an
/// evaluation failure rather than a panic, surfaced to the caller as an error
/// so the feature is routed to the `rejected` port. `name` identifies which
/// anchor parameter (`"latitude"`, `"longitude"`, `"height"`, or `"heading"`)
/// failed, for the error message.
fn eval_expr_f64(
    code: &CompiledCode,
    name: &'static str,
    feature: &Feature,
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
) -> Result<f64, BoxedError> {
    code.eval(feature, env_vars)
        .ok()
        .and_then(|v| v.as_f64())
        .ok_or_else(|| {
            Box::new(GeometryProcessorError::ModelGeoreferencer(format!(
                "anchor `{name}` expression did not evaluate to a number"
            ))) as BoxedError
        })
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

        match self.place_feature(&mut feature, ctx.env_vars.clone()) {
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

/// The WGS 84 ellipsoid's semi-major axis, in metres.
const WGS84_A: f64 = 6378137.0;
/// The WGS 84 ellipsoid's inverse flattening.
const WGS84_INV_F: f64 = 298.257223563;

/// WGS 84 geodetic latitude/longitude/height to geocentric (ECEF) metres.
fn geodetic_to_ecef(lat_deg: f64, lon_deg: f64, height_m: f64) -> [f64; 3] {
    let f = 1.0 / WGS84_INV_F;
    let e2 = f * (2.0 - f);
    let (lat, lon) = (lat_deg.to_radians(), lon_deg.to_radians());
    let (sin_lat, cos_lat) = (lat.sin(), lat.cos());
    let (sin_lon, cos_lon) = (lon.sin(), lon.cos());
    let n = WGS84_A / (1.0 - e2 * sin_lat * sin_lat).sqrt();
    [
        (n + height_m) * cos_lat * cos_lon,
        (n + height_m) * cos_lat * sin_lon,
        (n * (1.0 - e2) + height_m) * sin_lat,
    ]
}

/// The affine placing a Z-up local model at the anchor, aligned to local
/// east/north/up and rotated `heading_deg` clockwise from north, composed after
/// the source model's axis convention.
fn anchor_affine(
    lat_deg: f64,
    lon_deg: f64,
    height_m: f64,
    heading_deg: f64,
    up_axis: &UpAxis,
) -> Affine3 {
    let (lat, lon) = (lat_deg.to_radians(), lon_deg.to_radians());
    let (sin_lat, cos_lat) = (lat.sin(), lat.cos());
    let (sin_lon, cos_lon) = (lon.sin(), lon.cos());

    let east = [-sin_lon, cos_lon, 0.0];
    let north = [-sin_lat * cos_lon, -sin_lat * sin_lon, cos_lat];
    let up = [cos_lat * cos_lon, cos_lat * sin_lon, sin_lat];

    // Heading clockwise from north: the model's +Y (north) swings toward the
    // given bearing, so its images are combinations of east and north.
    let (sin_h, cos_h) = (
        heading_deg.to_radians().sin(),
        heading_deg.to_radians().cos(),
    );
    let col_x = [
        cos_h * east[0] - sin_h * north[0],
        cos_h * east[1] - sin_h * north[1],
        cos_h * east[2] - sin_h * north[2],
    ];
    let col_y = [
        sin_h * east[0] + cos_h * north[0],
        sin_h * east[1] + cos_h * north[1],
        sin_h * east[2] + cos_h * north[2],
    ];

    // Columns are the images of the model's x, y, z basis vectors.
    let rotation = [
        [col_x[0], col_y[0], up[0]],
        [col_x[1], col_y[1], up[1]],
        [col_x[2], col_y[2], up[2]],
    ];
    let enu = Affine3::new(rotation, geodetic_to_ecef(lat_deg, lon_deg, height_m));
    enu.compose(&axis_affine(up_axis))
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

    /// Build a `ModelGeoreferencer` from a `Placement` the way the factory's
    /// `build()` does (via `Placement::resolve()`), so tests keep expressing
    /// intent in terms of the parameter-facing `Placement` even though the
    /// processor stores the precompiled `ResolvedPlacement`.
    fn processor_from(placement: Placement, up_axis: UpAxis) -> ModelGeoreferencer {
        ModelGeoreferencer {
            placement: placement.resolve().unwrap(),
            up_axis,
        }
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
        let mut processor = processor_from(Placement::DeclareCrs { epsg_code: 4978 }, UpAxis::Z);

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
        let mut processor = processor_from(Placement::DeclareCrs { epsg_code: 6677 }, UpAxis::Y);

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
        let mut processor = processor_from(Placement::DeclareCrs { epsg_code: 6677 }, UpAxis::Y);

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
        let mut processor = processor_from(Placement::DeclareCrs { epsg_code: 6677 }, UpAxis::Y);

        let (features, ports) = run_processor(&feature, &mut processor);

        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
        assert_eq!(
            *features[0].geometry, geometry,
            "collection must be untouched"
        );
    }

    /// A `Code` carrying a literal numeric flow expression, e.g. `flow_expr(35.908)`.
    fn flow_expr(value: f64) -> Code<{ CodeType::FlowExpr as u32 }> {
        Code {
            ty: CodeType::FlowExpr,
            value: format!("{value}"),
        }
    }

    #[test]
    fn geodetic_to_ecef_matches_known_reference_points() {
        // Null island: on the equator at the prime meridian, ECEF is (a, 0, 0).
        let p = geodetic_to_ecef(0.0, 0.0, 0.0);
        assert!((p[0] - 6378137.0).abs() < 1e-3, "x was {}", p[0]);
        assert!(p[1].abs() < 1e-6 && p[2].abs() < 1e-6);

        // The north pole: x and y vanish, z is the semi-minor axis b = a(1 - f).
        let n = geodetic_to_ecef(90.0, 0.0, 0.0);
        let b = 6378137.0 * (1.0 - 1.0 / 298.257223563);
        assert!(n[0].abs() < 1e-6 && n[1].abs() < 1e-6, "got {n:?}");
        assert!((n[2] - b).abs() < 1e-3, "z was {} expected {b}", n[2]);
    }

    #[test]
    fn anchor_places_the_model_origin_at_the_anchor() {
        let (lat, lon) = (35.908, 140.102);
        let a = anchor_affine(lat, lon, 0.0, 0.0, &UpAxis::Z);
        let origin = a.apply([0.0, 0.0, 0.0]);
        let expected = geodetic_to_ecef(lat, lon, 0.0);
        for i in 0..3 {
            assert!(
                (origin[i] - expected[i]).abs() < 1e-6,
                "axis {i}: {origin:?} vs {expected:?}"
            );
        }
    }

    #[test]
    fn anchor_maps_model_up_to_local_up() {
        let (lat, lon) = (35.908, 140.102);
        let a = anchor_affine(lat, lon, 0.0, 0.0, &UpAxis::Z);
        // The load-bearing property: moving 100 m along the model's Z moves
        // 100 m along the WGS 84 ellipsoid normal at (lat, lon) -- the
        // direction is exactly `[cosφcosλ, cosφsinλ, sinφ]`, componentwise.
        // (Checking only the geocentric *radius* grew by ~100 would pass for a
        // sphere's radial "up" too -- and would just as happily accept a wrong
        // implementation that used the geocentric radial direction instead of
        // the ellipsoid normal, since those coincide only at the equator and
        // poles. The direction check below is exact and catches that.)
        let base = a.apply([0.0, 0.0, 0.0]);
        let up = a.apply([0.0, 0.0, 100.0]);
        let displacement = [up[0] - base[0], up[1] - base[1], up[2] - base[2]];
        let mag = (displacement[0] * displacement[0]
            + displacement[1] * displacement[1]
            + displacement[2] * displacement[2])
            .sqrt();
        assert!((mag - 100.0).abs() < 1e-9, "displacement length was {mag}");

        let (lat_r, lon_r) = (lat.to_radians(), lon.to_radians());
        let normal = [
            lat_r.cos() * lon_r.cos(),
            lat_r.cos() * lon_r.sin(),
            lat_r.sin(),
        ];
        for i in 0..3 {
            assert!(
                (displacement[i] / 100.0 - normal[i]).abs() < 1e-9,
                "axis {i}: displacement direction {:?} vs ellipsoid normal {normal:?}",
                [
                    displacement[0] / 100.0,
                    displacement[1] / 100.0,
                    displacement[2] / 100.0
                ]
            );
        }

        // Secondary sanity check: the geocentric radius grows by very nearly,
        // but not exactly, 100 m too -- WGS 84's ellipsoid normal is radial
        // only at the equator and poles, so away from those it is tilted by
        // up to ~0.192 degrees (at +-45 deg latitude), leaving a ~5e-4 m
        // shortfall per 100 m of height at this latitude. This is not the
        // load-bearing assertion; the direction check above is.
        let r = |p: [f64; 3]| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        assert!(
            (r(up) - r(base) - 100.0).abs() < 1e-3,
            "radius grew by {}",
            r(up) - r(base)
        );
    }

    #[test]
    fn anchor_composes_the_y_up_flip_before_the_enu_placement() {
        // `UpAxis` DEFAULTS to `Y` (the glTF/OBJ convention this action exists
        // to serve), yet every other anchor test above uses `UpAxis::Z`, where
        // `axis_affine` is the identity and both composition orders
        // (`enu.compose(&axis_affine(..))` vs. `axis_affine(..).compose(&enu)`)
        // are indistinguishable. This test pins the Y-up case specifically, so
        // the composition order is actually exercised.
        let (lat, lon, height) = (35.908, 140.102, 10.0);
        let a = anchor_affine(lat, lon, height, 0.0, &UpAxis::Y);

        // (a) The model origin still lands exactly at the anchor, regardless
        // of the axis flip (a pure rotation fixes the origin).
        let origin = a.apply([0.0, 0.0, 0.0]);
        let expected = geodetic_to_ecef(lat, lon, height);
        for i in 0..3 {
            assert!(
                (origin[i] - expected[i]).abs() < 1e-6,
                "axis {i}: {origin:?} vs {expected:?}"
            );
        }

        // (b) The model's +Y (its "up", under the Y-up convention) must map
        // to local up: a point 100 m up model-Y lands 100 m along the
        // ellipsoid normal from the origin, the same exact property checked
        // for `UpAxis::Z`'s +Z in `anchor_maps_model_up_to_local_up`.
        let up_point = a.apply([0.0, 100.0, 0.0]);
        let displacement = [
            up_point[0] - origin[0],
            up_point[1] - origin[1],
            up_point[2] - origin[2],
        ];
        let mag = (displacement[0] * displacement[0]
            + displacement[1] * displacement[1]
            + displacement[2] * displacement[2])
            .sqrt();
        assert!((mag - 100.0).abs() < 1e-9, "displacement length was {mag}");

        let (lat_r, lon_r) = (lat.to_radians(), lon.to_radians());
        let normal = [
            lat_r.cos() * lon_r.cos(),
            lat_r.cos() * lon_r.sin(),
            lat_r.sin(),
        ];
        for i in 0..3 {
            assert!(
                (displacement[i] / 100.0 - normal[i]).abs() < 1e-9,
                "axis {i}: displacement direction {:?} vs ellipsoid normal {normal:?}",
                [
                    displacement[0] / 100.0,
                    displacement[1] / 100.0,
                    displacement[2] / 100.0
                ]
            );
        }
    }

    #[test]
    fn heading_rotates_the_model_about_its_up_axis() {
        let (lat, lon) = (0.0, 0.0);
        // At (0,0): east is +Y ECEF, north is +Z ECEF.
        let north = anchor_affine(lat, lon, 0.0, 0.0, &UpAxis::Z).apply([0.0, 100.0, 0.0]);
        let base = anchor_affine(lat, lon, 0.0, 0.0, &UpAxis::Z).apply([0.0, 0.0, 0.0]);
        assert!(
            (north[2] - base[2] - 100.0).abs() < 1e-6,
            "0 deg heading points north"
        );

        // 90 degrees clockwise from north is east.
        let east = anchor_affine(lat, lon, 0.0, 90.0, &UpAxis::Z).apply([0.0, 100.0, 0.0]);
        assert!(
            (east[1] - base[1] - 100.0).abs() < 1e-6,
            "90 deg heading points east"
        );
    }

    #[test]
    fn process_anchor_places_model_origin_at_the_expected_ecef_position() {
        // End-to-end through `process`: the previous review round found that a
        // unit-tested helper with untested wiring is how a real bug escapes, so
        // this drives the whole processor, not just `anchor_affine` directly.
        let (lat, lon) = (35.908, 140.102);
        let geometry = euclidean_mesh(vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]);
        let feature = Feature::new_with_attributes_and_geometry(Attributes::new(), geometry);
        let mut processor = processor_from(
            Placement::Anchor {
                latitude: flow_expr(lat),
                longitude: flow_expr(lon),
                height: None,
                heading: None,
            },
            UpAxis::Z,
        );

        let (features, ports) = run_processor(&feature, &mut processor);

        assert_eq!(ports, vec![FEATURES_PORT.clone()]);
        let m = match &*features[0].geometry {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => m,
            other => panic!("expected a triangular mesh, got {other:?}"),
        };
        let expected = geodetic_to_ecef(lat, lon, 0.0);
        for v in m.vertices() {
            for i in 0..3 {
                assert!(
                    (v[i] - expected[i]).abs() < 1e-6,
                    "axis {i}: {v:?} vs {expected:?}"
                );
            }
        }
        assert_eq!(*m.frame(), CoordinateFrame::Crs(EpsgCode::new(4978)));
    }

    #[test]
    fn process_rejects_a_feature_whose_anchor_expression_fails_to_evaluate() {
        // A syntactically valid expression that evaluates to a non-number must
        // not panic; it must route to `rejected`. (A syntax error is instead
        // caught at `build()` time now that anchor expressions are precompiled
        // -- see `anchor_with_a_malformed_expression_fails_to_resolve` below --
        // so this test exercises the remaining evaluate-time failure mode:
        // successful compile and eval, but a result that isn't a number.)
        let geometry = euclidean_mesh(vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]);
        let feature =
            Feature::new_with_attributes_and_geometry(Attributes::new(), geometry.clone());
        let mut processor = processor_from(
            Placement::Anchor {
                latitude: Code {
                    ty: CodeType::FlowExpr,
                    value: "\"not-a-number\"".to_string(),
                },
                longitude: flow_expr(140.102),
                height: None,
                heading: None,
            },
            UpAxis::Z,
        );

        let (features, ports) = run_processor(&feature, &mut processor);

        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
        assert_eq!(
            *features[0].geometry, geometry,
            "geometry must be untouched on evaluation failure"
        );
    }

    #[test]
    fn anchor_with_a_malformed_expression_fails_to_resolve() {
        // A syntax error in an anchor expression is a configuration error, not
        // per-feature data, so it must surface at `build()`/`resolve()` time
        // (mirroring `coordinate_frame_reprojector.rs`'s `base_point.compile()`
        // in `build()`), rather than compiling per feature and rejecting late.
        let result = Placement::Anchor {
            latitude: Code {
                ty: CodeType::FlowExpr,
                value: "this is not a valid expression [[[".to_string(),
            },
            longitude: flow_expr(140.102),
            height: None,
            heading: None,
        }
        .resolve();
        assert!(result.is_err());
    }

    #[test]
    fn placing_a_collection_places_every_member_and_keeps_member_attributes() {
        use reearth_flow_types::{Attribute, AttributeValue, Attributes};

        let member = |soup: Vec<[f64; 3]>| {
            Euclidean3DGeometry::TriangularMesh(Box::new(TriangularMesh3D::from_soup(
                CoordinateFrame::Euclidean,
                soup,
            )))
        };
        let mut attrs = Attributes::new();
        attrs.insert(Attribute::new("gmlId"), AttributeValue::String("a".into()));

        let collection = Collection3D::with_attributes(
            vec![
                member(vec![[1.0, 2.0, 3.0], [1.0, 2.0, 3.0], [1.0, 2.0, 3.0]]),
                member(vec![[4.0, 5.0, 6.0], [4.0, 5.0, 6.0], [4.0, 5.0, 6.0]]),
            ],
            vec![attrs.clone(), attrs],
        )
        .unwrap();
        let mut geometry = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(collection));

        let target = CoordinateFrame::Crs(EpsgCode::new(4978));
        geometry.place(&axis_affine(&UpAxis::Y), &target).unwrap();

        let Geometry::Euclidean3D(Euclidean3DGeometry::Collection(c)) = &geometry else {
            panic!("expected a collection, got {geometry:?}");
        };
        assert_eq!(c.members().len(), 2, "both members survive");
        let member_attrs = c.member_attributes();
        assert_eq!(member_attrs.len(), 2, "per-member attributes survive");
        for (i, attrs) in member_attrs.iter().enumerate() {
            // `AttributeValue` derives no `PartialEq`, so match its actual
            // value rather than compare the count alone: a wiped-then-refilled
            // slot of the right length would otherwise pass silently.
            match attrs.get(&Attribute::new("gmlId")) {
                Some(AttributeValue::String(s)) => {
                    assert_eq!(s, "a", "member {i}'s gmlId value survives placement")
                }
                other => panic!("member {i} lost its gmlId attribute: {other:?}"),
            }
        }
        for m in c.members() {
            let Euclidean3DGeometry::TriangularMesh(mesh) = m else {
                panic!("expected a mesh member, got {m:?}");
            };
            assert_eq!(*mesh.frame(), target, "every member's frame is set");
        }
    }
}
