//! PolygonMesh leaves.
//!
//! CSR (compressed-sparse-row) layout: every face's vertex indices are
//! concatenated into a single flat `face_indices` array, with `face_offsets`
//! marking where each face begins. A face may carry holes: its rings (exterior
//! first, then holes) sit contiguously within its range, and `interior_offsets`
//! marks each hole ring's start, mirroring `Polygon` one level up.
//!
//! Comes in 2D and 3D variants. The 2D variant carries `vertices: Vec<[f64; 2]>`
//! plus an optional per-vertex elevation buffer parallel to `vertices`, matching
//! the 2D leaf convention.

use serde::{Deserialize, Serialize};

use crate::appearance::{Appearance, UvSet};
use crate::coordinate::Coordinate;
use crate::index::IndexBuffer;

mod constructor;

/// A connected, vertex-sharing polygon mesh in 2D space, with optional
/// per-vertex elevation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PolygonMesh2D {
    /// Coordinate frame these vertices are expressed in.
    coordinate: Coordinate,
    vertices: Vec<[f64; 2]>,
    /// Optional per-vertex elevation, parallel to `vertices`. INVARIANT: when
    /// `Some`, `z.len() == vertices.len()`. `None` = pure 2D.
    z: Option<Box<[f64]>>,
    /// All rings of all faces concatenated; each face is its exterior ring then
    /// its hole rings. Width from `vertices.len() - 1`.
    face_indices: IndexBuffer<1>,
    /// Internal face boundaries into `face_indices`: `len() = n_faces - 1`, no
    /// leading 0. `face_offsets[i]` is where face `i+1` begins; face `i` spans
    /// `face_indices[s..e]` with `s = if i == 0 { 0 } else { face_offsets[i-1] }`
    /// and `e = face_offsets.get(i).copied().unwrap_or(face_indices.len())`. Width
    /// from `face_indices.len()`.
    face_offsets: IndexBuffer<1>,
    /// Start in `face_indices` of each hole ring, across all faces; empty when
    /// no face has holes. Width from `face_indices.len()`.
    interior_offsets: IndexBuffer<1>,
    /// Geometric UV, parallel to the corner buffers; empty = no UV.
    uv_sets: Vec<UvSet>,
    /// Optional materials / themes / per-face binding; `None` = bare.
    appearance: Option<Appearance>,
}

/// The coordinate-free data of a 3D polygon mesh: the vertex pool, CSR face
/// topology, UV and appearance, with no frame of its own.
///
/// Shared by two hosts that each supply the frame: the standalone
/// [`PolygonMesh3D`] leaf pairs this with its own [`Coordinate`], while a
/// [`Solid`](crate::solid::Solid) shell stores it directly and takes the one
/// frame from the enclosing `Solid` — so a solid and its boundaries cannot
/// disagree on a frame. Mirrors the [`Raster`](crate::appearance::Raster) /
/// [`RasterData`](crate::appearance::RasterData) split.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PolygonMesh3DData {
    vertices: Vec<[f64; 3]>,
    /// All rings of all faces concatenated; each face is its exterior ring then
    /// its hole rings. Width from `vertices.len() - 1`.
    face_indices: IndexBuffer<1>,
    /// Internal face boundaries into `face_indices`: `len() = n_faces - 1`, no
    /// leading 0. `face_offsets[i]` is where face `i+1` begins; face `i` spans
    /// `face_indices[s..e]` with `s = if i == 0 { 0 } else { face_offsets[i-1] }`
    /// and `e = face_offsets.get(i).copied().unwrap_or(face_indices.len())`. Width
    /// from `face_indices.len()`.
    face_offsets: IndexBuffer<1>,
    /// Start in `face_indices` of each hole ring, across all faces; empty when
    /// no face has holes. Width from `face_indices.len()`.
    interior_offsets: IndexBuffer<1>,
    /// Geometric UV, parallel to the corner buffers; empty = no UV.
    uv_sets: Vec<UvSet>,
    /// Optional materials / themes / per-face binding; `None` = bare.
    appearance: Option<Appearance>,
}

/// A connected, vertex-sharing polygon mesh in 3D space: coordinate-free mesh
/// data plus the frame it is expressed in.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PolygonMesh3D {
    /// Coordinate frame the mesh data is expressed in.
    coordinate: Coordinate,
    /// Coordinate-free mesh data; the same form a [`Solid`](crate::solid::Solid)
    /// shell stores directly.
    data: PolygonMesh3DData,
}

impl PolygonMesh2D {
    /// Borrow the appearance, if any.
    #[inline]
    pub fn appearance(&self) -> &Option<Appearance> {
        &self.appearance
    }

    /// Mutably borrow the appearance, to set, clear, or edit it in place.
    #[inline]
    pub fn appearance_mut(&mut self) -> &mut Option<Appearance> {
        &mut self.appearance
    }
}

impl PolygonMesh3D {
    /// Borrow the appearance, if any.
    #[inline]
    pub fn appearance(&self) -> &Option<Appearance> {
        &self.data.appearance
    }

    /// Mutably borrow the appearance, to set, clear, or edit it in place.
    #[inline]
    pub fn appearance_mut(&mut self) -> &mut Option<Appearance> {
        &mut self.data.appearance
    }
}
