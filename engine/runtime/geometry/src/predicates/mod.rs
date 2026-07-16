//! Binary geometric predicates and their supporting robust kernels.
//!
//! The predicates are available as free functions
//! with internal pair-matching, over lightweight coordinate **views** (see
//! [`view`]) so a `Polygon`, a `PolygonMesh` face, and a `TriangularMesh`
//! triangle all feed the same [`kernel`] with zero copying.
//!
//! Available predicates, all 2D (collections are point-set unions of their
//! members; every leaf pair takes a bounding-box quick reject first, and the
//! segment-crossing searches switch from a direct scan to an rstar-indexed
//! sweep above a size threshold, with identical answers):
//!
//! - [`intersects()`]: whether two geometries share at least one point.
//! - [`contains()`] / [`covers`]: split-based containment with OGC semantics;
//!   see [`contains`](contains()) for the algorithm.
//! - [`point_position_2d`]: coordinate vs. geometry classification
//!   (`Inside` / `OnBoundary` / `Outside`), with exact shared-edge and
//!   surrounded-vertex refinement on meshes.
//! - [`relate()`]: the full DE-9IM [`IntersectionMatrix`], from which every
//!   named predicate (touches, crosses, overlaps, ...) and arbitrary DE-9IM
//!   patterns can be read; meshes relate as their dissolved face union.
//!
//! The 2D leaves' optional per-vertex elevation is ignored throughout. The
//! constructed counterparts, boolean overlay, line clipping, segment
//! intersection points, live in [`overlay`](crate::overlay) over the same
//! views and kernel. Ray casting and 3D pairs are not yet supported.

pub mod contains;
mod edge_set;
pub mod intersects;
pub mod kernel;
pub mod position;
pub mod relate;
pub mod view;

pub use contains::{contains, covers};
pub use intersects::intersects;
pub use kernel::CoordPos;
pub use position::point_position_2d;
pub use relate::{relate, Dimensions, IntersectionMatrix};

use crate::coordinate::CoordinateFrame;
use crate::Geometry;
use view::Leaf2D;

/// Why a binary predicate or overlay could not be evaluated.
///
/// Richer than [`UnsupportedOperation`](crate::ops::UnsupportedOperation): a
/// binary op fails not only on an unsupported type but on operands the policy
/// refuses to mix. Frame reprojection and dimension promotion are the caller's
/// explicit steps, so the kernel reports the mismatch rather than guessing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PredicateError {
    /// The two operands are in different coordinate frames. Reproject one to the
    /// other's frame first (via [`Reproject`](crate::ops::Reproject)); the
    /// predicates never reproject implicitly.
    MixedFrames,
    /// A 2D-embedded operand was paired with a 3D-embedded one. There is no
    /// implicit promotion; the only cross-dimension path is an explicit
    /// XY-projection opt-in exposed by the individual predicates.
    CrossDimension,
    /// The operation is not defined for this ordered pair of concrete geometry
    /// types.
    UnsupportedPair {
        /// The left operand's concrete type name.
        left: &'static str,
        /// The right operand's concrete type name.
        right: &'static str,
    },
}

impl core::fmt::Display for PredicateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PredicateError::MixedFrames => {
                write!(f, "operands are in different coordinate frames")
            }
            PredicateError::CrossDimension => {
                write!(
                    f,
                    "cannot pair a 2D-embedded operand with a 3D-embedded one"
                )
            }
            PredicateError::UnsupportedPair { left, right } => {
                write!(f, "operation is not defined for `{left}` and `{right}`")
            }
        }
    }
}

impl std::error::Error for PredicateError {}

/// The result of a binary predicate.
pub type Result<T> = core::result::Result<T, PredicateError>;

/// Require both operands to be in the **same** coordinate frame, returning
/// [`PredicateError::MixedFrames`] otherwise. Every binary op runs this before
/// touching coordinates; reprojection stays the caller's explicit step.
pub fn require_same_frame(a: &CoordinateFrame, b: &CoordinateFrame) -> Result<()> {
    if a == b {
        Ok(())
    } else {
        Err(PredicateError::MixedFrames)
    }
}

/// Flatten both operands of a binary 2D operation into their 2D leaves under
/// the shared dimension policy: a 2D × 3D pair is
/// [`CrossDimension`](PredicateError::CrossDimension), a purely 3D pair is
/// [`UnsupportedPair`](PredicateError::UnsupportedPair). `Geometry::None` and
/// empty collections flatten to no leaves.
pub(crate) fn flatten_2d_pair<'a>(
    a: &'a Geometry,
    b: &'a Geometry,
) -> Result<(Vec<Leaf2D<'a>>, Vec<Leaf2D<'a>>)> {
    let (a_leaves, a_3d) = contains::flatten_geometry(a);
    let (b_leaves, b_3d) = contains::flatten_geometry(b);
    match (a_3d, b_3d) {
        (None, None) => Ok((a_leaves, b_leaves)),
        (Some(left), Some(right)) if a_leaves.is_empty() && b_leaves.is_empty() => {
            Err(PredicateError::UnsupportedPair { left, right })
        }
        _ => Err(PredicateError::CrossDimension),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::EpsgCode;

    #[test]
    fn same_frame_passes() {
        let a = CoordinateFrame::Crs(EpsgCode::new(4326));
        let b = CoordinateFrame::Crs(EpsgCode::new(4326));
        assert!(require_same_frame(&a, &b).is_ok());
    }

    #[test]
    fn different_frame_is_mixed_frames() {
        let a = CoordinateFrame::Crs(EpsgCode::new(4326));
        let b = CoordinateFrame::Euclidean;
        assert_eq!(require_same_frame(&a, &b), Err(PredicateError::MixedFrames));
    }

    #[test]
    fn error_display_is_descriptive() {
        assert!(PredicateError::MixedFrames.to_string().contains("frame"));
        assert!(PredicateError::CrossDimension.to_string().contains("2D"));
        let up = PredicateError::UnsupportedPair {
            left: "Point2D",
            right: "Solid",
        };
        assert!(up.to_string().contains("Point2D") && up.to_string().contains("Solid"));
    }
}
