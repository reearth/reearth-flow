//! Extrude: sweep a geometry along a direction to add a dimension.
//!
//! A point becomes a segment, a line becomes an open wall, and an area (a
//! polygon, or a polygon or triangle mesh) becomes a closed `Solid` or an open
//! shell. An area whose rings cross themselves or each other is rejected.
//! 2D input is lifted to its elevation (`0.0` when it has none) first.
//! The whole input is one feature: flat base and top modes use its lowest and
//! highest vertex across every piece and member.
//! The op does geometry only: callers resolve expressions to numbers and pass
//! an [`ExtrudeSpec`].

mod line;
mod math;
mod place;
mod surface;
mod weld;

pub(crate) use line::{sweep_line, sweep_point};
pub(crate) use surface::{sweep_surface, CapFace};

use crate::collection::Collection3D;
use crate::ops::{BoundingBox, UnsupportedOperation};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection};

/// What to sweep along and what to produce.
#[derive(Clone, Debug, PartialEq)]
pub struct ExtrudeSpec {
    pub direction: ExtrudeDirection,
    pub output: ExtrudeOutput,
}

/// The direction of the sweep.
#[derive(Clone, Debug, PartialEq)]
pub enum ExtrudeDirection {
    /// Along the frame's Z axis, with a base and a top rule.
    Vertical { base: BaseMode, top: TopMode },
    /// By `vector`; when `length` is set the vector is normalized and scaled
    /// to it (a negative length reverses it).
    Vector {
        vector: [f64; 3],
        length: Option<f64>,
    },
    /// Along the front normal of a planar surface by `distance`.
    SurfaceNormal { distance: f64 },
}

/// Where the base of a vertical sweep sits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BaseMode {
    /// Each vertex keeps its own elevation.
    KeepElevation,
    /// Every base vertex moves to the feature's lowest elevation.
    FlattenToLowest,
    /// Every base vertex moves to this elevation.
    FixedElevation(f64),
}

/// Where the top of a vertical sweep ends.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TopMode {
    /// Each vertex rises by this height from its base (follows a sloped base).
    ByHeight(f64),
    /// A flat top at this absolute elevation, which must lie above every base
    /// vertex.
    ToElevation(f64),
    /// A flat top this far above the feature's lowest base vertex.
    AboveLowest(f64),
    /// A flat top this far above the feature's highest base vertex.
    AboveHighest(f64),
}

/// What an area becomes. Points and lines ignore it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ExtrudeOutput {
    /// A closed solid: bottom cap, top cap and walls.
    #[default]
    Solid,
    /// A polygon mesh of the top cap and walls, open at the bottom.
    OpenBottom,
    /// A polygon mesh of the walls only.
    WallsOnly,
}

/// Why a geometry could not be extruded. Each variant maps to one diagnostic
/// code in the action.
#[derive(Clone, Debug, PartialEq)]
pub enum ExtrudeError {
    /// The geometry type (or this type under this direction) cannot be swept.
    UnsupportedType(String),
    /// A surface with a ring that crosses itself or another ring of its face,
    /// or one that is closed, non-manifold, non-orientable, or not planar
    /// where planarity is required.
    InvalidSurface(&'static str),
    /// The sweep has no volume or area: zero length, a flat top not above
    /// every base vertex, a face edge-on to the direction, or an edge that
    /// placement collapses.
    Degenerate(&'static str),
    /// The coordinate frame cannot support this sweep: unknown axes or units,
    /// or angular units for a non-vertical direction.
    UnsupportedFrame(String),
}

impl core::fmt::Display for ExtrudeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ExtrudeError::UnsupportedType(why) => write!(f, "cannot extrude: {why}"),
            ExtrudeError::InvalidSurface(why) => write!(f, "invalid surface: {why}"),
            ExtrudeError::Degenerate(why) => write!(f, "degenerate extrusion: {why}"),
            ExtrudeError::UnsupportedFrame(why) => write!(f, "unsupported frame: {why}"),
        }
    }
}

impl std::error::Error for ExtrudeError {}

impl From<UnsupportedOperation> for ExtrudeError {
    fn from(e: UnsupportedOperation) -> Self {
        ExtrudeError::UnsupportedType(e.to_string())
    }
}

/// Sweep a 3D geometry. Implemented by every 3D leaf; `Solid`, `Csg` and
/// `PointCloud` opt out via `unsupported!` and so return the refusing default.
///
/// Called directly, the feature-relative modes (`FlattenToLowest`,
/// `AboveLowest`, `AboveHighest`) fall back to each connected piece's own
/// extremes; [`extrude`] resolves them over the whole feature first.
#[enum_dispatch::enum_dispatch]
pub trait Extrude {
    fn extrude(&self, spec: &ExtrudeSpec) -> Result<Euclidean3DGeometry, ExtrudeError> {
        let _ = spec;
        Err(UnsupportedOperation {
            geometry: core::any::type_name::<Self>(),
            operation: "extrude",
        }
        .into())
    }
}

impl<T: Extrude + ?Sized> Extrude for Box<T> {
    fn extrude(&self, spec: &ExtrudeSpec) -> Result<Euclidean3DGeometry, ExtrudeError> {
        (**self).extrude(spec)
    }
}

/// Extrude any geometry: lift 2D to 3D, then sweep. A collection with any
/// member that cannot be extruded fails as a whole.
///
/// The geometry is one feature: `FlattenToLowest`, `AboveLowest` and
/// `AboveHighest` use the lowest and highest vertex of the whole geometry,
/// not of each connected piece or member. A 2D area has no stored normal, so
/// `SurfaceNormal` sweeps it along `+Z`.
pub fn extrude(geometry: &Geometry, spec: &ExtrudeSpec) -> Result<Geometry, ExtrudeError> {
    extrude_member(geometry, &per_feature(geometry, spec))
}

/// `spec` with its feature-relative vertical modes resolved to constants
/// from the feature's z range. Unchanged when no such mode is used or the
/// geometry has no vertices.
fn per_feature(geometry: &Geometry, spec: &ExtrudeSpec) -> ExtrudeSpec {
    let ExtrudeDirection::Vertical { base, top } = spec.direction else {
        return spec.clone();
    };
    let relative_top = matches!(top, TopMode::AboveLowest(_) | TopMode::AboveHighest(_));
    if base != BaseMode::FlattenToLowest && !relative_top {
        return spec.clone();
    }
    let Ok(aabb) = geometry.clone().into_3d().bounding_box() else {
        return spec.clone();
    };
    let (min, max) = aabb.corners_3d();
    let (zmin, zmax) = (min[2], max[2]);
    let (base_lo, base_hi) = match base {
        BaseMode::KeepElevation => (zmin, zmax),
        BaseMode::FlattenToLowest => (zmin, zmin),
        BaseMode::FixedElevation(z) => (z, z),
    };
    let base = match base {
        BaseMode::FlattenToLowest => BaseMode::FixedElevation(zmin),
        other => other,
    };
    let top = match top {
        TopMode::AboveLowest(h) => TopMode::ToElevation(base_lo + h),
        TopMode::AboveHighest(h) => TopMode::ToElevation(base_hi + h),
        other => other,
    };
    ExtrudeSpec {
        direction: ExtrudeDirection::Vertical { base, top },
        output: spec.output,
    }
}

fn extrude_member(geometry: &Geometry, spec: &ExtrudeSpec) -> Result<Geometry, ExtrudeError> {
    match geometry {
        Geometry::None => Err(ExtrudeError::UnsupportedType(
            "the feature has no geometry".to_string(),
        )),
        Geometry::Euclidean2D(g) => Ok(Geometry::Euclidean3D(extrude_2d(g, spec)?)),
        Geometry::Euclidean3D(g) => Ok(Geometry::Euclidean3D(g.extrude(spec)?)),
        Geometry::GeometryCollection(c) => {
            let members = c
                .members()
                .iter()
                .map(|m| extrude_member(m, spec))
                .collect::<Result<Vec<_>, _>>()?;
            let collection =
                GeometryCollection::with_attributes(members, c.member_attributes().to_vec())
                    .expect("member count is unchanged by extrusion");
            Ok(Geometry::GeometryCollection(collection))
        }
    }
}

/// Lift a 2D geometry and sweep it. A 2D area's normal is `+Z`, whatever
/// its winding, so `SurfaceNormal` becomes a `+Z` vector of that length
/// (still subject to the linear-units check). Points and lines keep
/// `SurfaceNormal` and so are refused: they have no normal.
fn extrude_2d(
    geometry: &Euclidean2DGeometry,
    spec: &ExtrudeSpec,
) -> Result<Euclidean3DGeometry, ExtrudeError> {
    if let Euclidean2DGeometry::Collection(c) = geometry {
        let members = c
            .members()
            .iter()
            .map(|m| extrude_2d(m, spec))
            .collect::<Result<Vec<_>, _>>()?;
        let collection = Collection3D::with_attributes(members, c.member_attributes().to_vec())
            .expect("member count is unchanged by extrusion");
        return Ok(Euclidean3DGeometry::Collection(collection));
    }
    let areal = matches!(
        geometry,
        Euclidean2DGeometry::Polygon(_)
            | Euclidean2DGeometry::PolygonMesh(_)
            | Euclidean2DGeometry::TriangularMesh(_)
    );
    let spec = match spec.direction {
        ExtrudeDirection::SurfaceNormal { distance } if areal => ExtrudeSpec {
            direction: ExtrudeDirection::Vector {
                vector: [0.0, 0.0, 1.0],
                length: Some(distance),
            },
            output: spec.output,
        },
        _ => spec.clone(),
    };
    geometry.clone().into_3d().extrude(&spec)
}

impl Extrude for Collection3D {
    fn extrude(&self, spec: &ExtrudeSpec) -> Result<Euclidean3DGeometry, ExtrudeError> {
        let members = self
            .members()
            .iter()
            .map(|m| m.extrude(spec))
            .collect::<Result<Vec<_>, _>>()?;
        let collection = Collection3D::with_attributes(members, self.member_attributes().to_vec())
            .expect("member count is unchanged by extrusion");
        Ok(Euclidean3DGeometry::Collection(collection))
    }
}

#[cfg(test)]
pub(crate) mod test_util {
    use super::*;
    use crate::validation_next::{validate, ValidationResult};

    pub(crate) fn vertical(base: BaseMode, top: TopMode) -> ExtrudeSpec {
        ExtrudeSpec {
            direction: ExtrudeDirection::Vertical { base, top },
            output: ExtrudeOutput::Solid,
        }
    }

    pub(crate) fn up(h: f64) -> ExtrudeSpec {
        vertical(BaseMode::KeepElevation, TopMode::ByHeight(h))
    }

    /// Every validation check on `g` ended in `Success` or `Skipped`.
    pub(crate) fn assert_valid(g: &Geometry) {
        for (check, result) in validate(g) {
            assert!(
                matches!(
                    result,
                    ValidationResult::Success | ValidationResult::Skipped(_)
                ),
                "{check} failed: {result:?} on {g:?}"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_util::*;
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::point_cloud::PointCloud;

    #[test]
    fn no_geometry_is_unsupported() {
        assert!(matches!(
            extrude(&Geometry::None, &up(1.0)),
            Err(ExtrudeError::UnsupportedType(_))
        ));
    }

    #[test]
    fn empty_collection_stays_empty() {
        let empty = Geometry::GeometryCollection(GeometryCollection::new([]));
        assert_eq!(extrude(&empty, &up(1.0)).unwrap(), empty);
    }

    #[test]
    fn point_cloud_is_unsupported() {
        let cloud = PointCloud::from_positions(CoordinateFrame::Euclidean, vec![[0.0, 0.0, 0.0]]);
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::PointCloud(Box::new(cloud)));
        assert!(matches!(
            extrude(&g, &up(1.0)),
            Err(ExtrudeError::UnsupportedType(_))
        ));
    }
}
