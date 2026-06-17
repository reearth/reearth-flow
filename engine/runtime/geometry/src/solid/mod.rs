//! Solid leaf.
//!
//! A `Solid` is bounded by one exterior `Shell` and any number of interior
//! `Shell`s (hollow voids). Each `Shell` is either a `PolygonMesh` or a
//! `TriangularMesh`, so a boundary that arrives as a TIN stays a triangle mesh
//! and a general one stays a polygon mesh, without forcing a single mesh kind on
//! every shell. (`Shell` rather than `Surface`, to avoid colliding with the
//! CityGML / FME `Surface` names.) Solids are 3D only; their shells are coordless
//! raw meshes and the one frame lives on the `Solid`.

use serde::{Deserialize, Serialize};

use crate::coordinate::Coordinate;
use crate::polygon_mesh::PolygonMesh3D;
use crate::triangular_mesh::TriangularMesh3D;

/// One closed boundary of a [`Solid`]: a general polygon mesh or a triangle mesh.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Shell {
    PolygonMesh(PolygonMesh3D),
    TriangularMesh(TriangularMesh3D),
}

/// A volumetric solid bounded by an exterior shell and any number of interior
/// (void) shells. Appearance lives on each shell's mesh.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Solid {
    /// Coordinate frame this solid's shells are expressed in; the shells
    /// themselves are coordless raw meshes.
    pub coordinate: Coordinate,
    pub exterior: Shell,
    /// Hollow voids.
    pub interiors: Vec<Shell>,
}
