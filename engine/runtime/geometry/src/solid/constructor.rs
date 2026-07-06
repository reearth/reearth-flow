//! Solid constructors.
//!
//! A `Solid`'s shells are already-constructed coordinate-free meshes
//! ([`PolygonMesh3DData`] / [`TriangularMesh3DData`]); these constructors just pair
//! an exterior shell (and any interior void shells) with the frame they are
//! expressed in. The `From` impls let a mesh be passed where a [`Shell`] is
//! expected.

use crate::coordinate::CoordinateFrame;
use crate::polygon_mesh::PolygonMesh3DData;
use crate::triangular_mesh::TriangularMesh3DData;

use super::{Shell, Solid};

impl From<PolygonMesh3DData> for Shell {
    fn from(mesh: PolygonMesh3DData) -> Self {
        Shell::PolygonMesh(mesh)
    }
}

impl From<TriangularMesh3DData> for Shell {
    fn from(mesh: TriangularMesh3DData) -> Self {
        Shell::TriangularMesh(mesh)
    }
}

impl Shell {
    /// Drop back-side appearance from this shell's mesh, whichever kind it is.
    /// A solid boundary's back face is never rendered (see
    /// [`PolygonMesh3DData::make_front_only`](crate::polygon_mesh::PolygonMesh3DData)).
    fn make_front_only(&mut self) {
        match self {
            Shell::PolygonMesh(mesh) => mesh.make_front_only(),
            Shell::TriangularMesh(mesh) => mesh.make_front_only(),
        }
    }
}

impl Solid {
    /// A solid bounded by `exterior` with the given interior (void) shells, in
    /// `frame`.
    ///
    /// Every shell — exterior and interiors — is normalised to **front-side only**:
    /// a solid's boundary faces are oriented outward (or inward for voids), so the
    /// back of any boundary is the solid's interior and is never rendered. Any
    /// back-side appearance a shell mesh carried is dropped here.
    pub fn new(
        frame: CoordinateFrame,
        exterior: impl Into<Shell>,
        mut interiors: Vec<Shell>,
    ) -> Self {
        let mut exterior = exterior.into();
        exterior.make_front_only();
        for shell in &mut interiors {
            shell.make_front_only();
        }
        Self {
            frame,
            exterior,
            interiors,
        }
    }

    /// A solid bounded by a single exterior shell, with no voids.
    pub fn from_exterior(frame: CoordinateFrame, exterior: impl Into<Shell>) -> Self {
        Self::new(frame, exterior, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_shell() -> TriangularMesh3DData {
        // A degenerate stand-in is fine here: construction does not validate closure.
        TriangularMesh3DData::from_parts(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [0u32, 1, 2],
        )
        .unwrap()
    }

    #[test]
    fn from_exterior_has_no_voids() {
        let s = Solid::from_exterior(CoordinateFrame::Euclidean, cube_shell());
        assert!(matches!(s.exterior, Shell::TriangularMesh(_)));
        assert!(s.interiors.is_empty());
        assert_eq!(s.frame, CoordinateFrame::Euclidean);
    }

    #[test]
    fn new_carries_void_shells() {
        let s = Solid::new(
            CoordinateFrame::Euclidean,
            cube_shell(),
            vec![Shell::from(cube_shell())],
        );
        assert_eq!(s.interiors.len(), 1);
    }
}
