//! Solid leaf.
//!
//! A `Solid` is bounded by one exterior `Shell` and any number of interior
//! `Shell`s (hollow voids). Each `Shell` is either a `PolygonMesh` or a
//! `TriangularMesh`, so a boundary that arrives as a TIN stays a triangle mesh
//! and a general one stays a polygon mesh, without forcing a single mesh kind on
//! every shell. (`Shell` rather than `Surface`, to avoid colliding with the
//! CityGML `Surface` name.) Solids are 3D only; their shells are coordless
//! raw meshes and the one frame lives on the `Solid`.

use serde::{Deserialize, Serialize};

use crate::coordinate::CoordinateFrame;
use crate::polygon_mesh::PolygonMesh3DData;
use crate::triangular_mesh::TriangularMesh3DData;

mod constructor;
mod ops;
#[cfg(feature = "new-geometry")]
mod validation;

/// One closed boundary of a [`Solid`]: a general polygon mesh or a triangle
/// mesh, stored as coordinate-free mesh data so the boundary cannot carry a
/// frame of its own — its frame is the `Solid`'s.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Shell {
    PolygonMesh(PolygonMesh3DData),
    TriangularMesh(TriangularMesh3DData),
}

impl Shell {
    /// The shell's vertex pool, regardless of mesh kind.
    #[inline]
    pub(crate) fn vertices(&self) -> &[[f64; 3]] {
        match self {
            Shell::PolygonMesh(d) => d.vertices(),
            Shell::TriangularMesh(d) => d.vertices(),
        }
    }

    /// The number of boundary faces, regardless of mesh kind.
    #[inline]
    pub fn num_faces(&self) -> usize {
        match self {
            Shell::PolygonMesh(d) => d.num_faces(),
            Shell::TriangularMesh(d) => d.num_triangles(),
        }
    }
}

/// A volumetric solid bounded by an exterior shell and any number of interior
/// (void) shells. Appearance lives on each shell's mesh.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Solid {
    /// Coordinate frame this solid's shells are expressed in; the shells
    /// themselves are coordless raw meshes.
    frame: CoordinateFrame,
    exterior: Shell,
    /// Hollow voids.
    interiors: Vec<Shell>,
}

impl Solid {
    /// The exterior boundary shell.
    #[inline]
    pub fn exterior(&self) -> &Shell {
        &self.exterior
    }

    /// The interior (void) boundary shells.
    #[inline]
    pub fn interiors(&self) -> &[Shell] {
        &self.interiors
    }
}
