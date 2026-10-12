//! Extrude: sweep a geometry along a direction to add a dimension.
//!
//! A point becomes a segment, a line becomes an open wall, and an area (a
//! polygon, a polygon or triangle mesh) becomes a closed `Solid` or an open
//! shell. 2D input is lifted to its elevation (`0.0` when it has none) first.
//! The op does geometry only: callers resolve expressions to numbers and pass
//! an [`ExtrudeSpec`].

// Stubs filled in by Tasks 2 to 4; the `allow`s below are removed then.
#[allow(dead_code)]
mod line;
#[allow(dead_code)] // used by the sweeps in Tasks 2 to 4
mod math;
#[allow(dead_code)] // used from Tasks 3 and 4
mod place;
#[allow(dead_code)]
mod surface;
#[allow(dead_code)] // used from Tasks 3 and 4
mod weld;

#[allow(unused_imports)]
pub(crate) use line::{sweep_line, sweep_point};
#[allow(unused_imports)]
pub(crate) use surface::{sweep_surface, CapFace};

use crate::collection::Collection3D;
use crate::ops::UnsupportedOperation;
use crate::{Euclidean3DGeometry, Geometry, GeometryCollection};

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
    /// Every base vertex moves to the piece's lowest elevation.
    FlattenToLowest,
    /// Every base vertex moves to this elevation.
    FixedElevation(f64),
}

/// Where the top of a vertical sweep ends.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TopMode {
    /// Each vertex rises by this height from its base (follows a sloped base).
    ByHeight(f64),
    /// A flat top at this absolute elevation.
    ToElevation(f64),
    /// A flat top this far above the piece's lowest base vertex.
    AboveLowest(f64),
    /// A flat top this far above the piece's highest base vertex.
    AboveHighest(f64),
}

/// What an area becomes. Points and lines ignore it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ExtrudeOutput {
    /// A closed solid: bottom cap, top cap and walls.
    #[default]
    Solid,
    /// A polygon mesh of the top cap and walls.
    OpenTop,
    /// A polygon mesh of the walls only.
    WallsOnly,
}

/// Why a geometry could not be extruded. Each variant maps to one diagnostic
/// code in the action.
#[derive(Clone, Debug, PartialEq)]
pub enum ExtrudeError {
    /// The geometry type (or this type under this direction) cannot be swept.
    UnsupportedType(String),
    /// A surface that is closed, non-manifold, non-orientable, or not planar
    /// where planarity is required.
    InvalidSurface(&'static str),
    /// The sweep has no volume or area: zero length, top not above base, or a
    /// face edge-on to the direction.
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
/// `PointCloud` keep the refusing default.
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
pub fn extrude(geometry: &Geometry, spec: &ExtrudeSpec) -> Result<Geometry, ExtrudeError> {
    match geometry.clone().into_3d() {
        Geometry::None => Err(ExtrudeError::UnsupportedType(
            "the feature has no geometry".to_string(),
        )),
        Geometry::Euclidean3D(g) => Ok(Geometry::Euclidean3D(g.extrude(spec)?)),
        Geometry::GeometryCollection(c) => {
            let members = c
                .members()
                .iter()
                .map(|m| extrude(m, spec))
                .collect::<Result<Vec<_>, _>>()?;
            let collection =
                GeometryCollection::with_attributes(members, c.member_attributes().to_vec())
                    .expect("member count is unchanged by extrusion");
            Ok(Geometry::GeometryCollection(collection))
        }
        Geometry::Euclidean2D(_) => unreachable!("into_3d lifts every 2D geometry"),
    }
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
    #[allow(dead_code)] // used by the tests of Tasks 3 to 5
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
