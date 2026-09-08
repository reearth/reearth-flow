//! Geometric predicates over triangular-mesh leaves.

use super::{TriangularMesh2D, TriangularMesh3D};
use crate::predicates::{self, Equal};

impl Equal for TriangularMesh2D {
    fn equal(&self, rhs: &Self, tolerance: f64) -> predicates::Result<bool> {
        // 2D is the one case where every shared edge may be cancelled: the mesh
        // lies in a single plane, so its region is recoverable from its outline
        // and no crease can hide inside it.
        use crate::predicates::equal::surface_curves_2d;

        predicates::require_same_frame(self.frame(), rhs.frame())?;
        Ok(surface_curves_2d(self)?.within(&surface_curves_2d(rhs)?, tolerance))
    }
}

impl Equal for TriangularMesh3D {
    fn equal(&self, _rhs: &Self, _tolerance: f64) -> predicates::Result<bool> {
        // TODO: replace with a topology-blind comparison — neither the triangles
        // nor the faces are believed. For every vertex of either surface, find
        // the closest point of the other (a point of the point set, so triangle
        // interiors count), through an rstar prefilter over the triangle boxes
        // and an exact point-to-triangle distance after it, returning as soon as
        // one vertex is further than the tolerance; then match the boundary
        // loops, if any, as 3D curves. Note vertex sampling alone is unsound on
        // closed surfaces: a cube and the same cube with a tunnel drilled
        // through it have every vertex of each lying on the other.
        // Panics rather than refusing: an unwritten comparison is a hole in the
        // implementation, not a shape this cannot answer for, and a silent
        // refusal would let it pass for one.
        unimplemented!("geometric equality is not implemented for TriangularMesh3D")
    }
}
