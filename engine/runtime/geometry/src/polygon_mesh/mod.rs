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

/// A connected, vertex-sharing polygon mesh in 2D space, with optional
/// per-vertex elevation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PolygonMesh2D {
    /// Coordinate frame these vertices are expressed in.
    pub(crate) coordinate: Coordinate,
    pub(crate) vertices: Vec<[f64; 2]>,
    /// Optional per-vertex elevation, parallel to `vertices`. INVARIANT: when
    /// `Some`, `z.len() == vertices.len()`. `None` = pure 2D.
    pub(crate) z: Option<Box<[f64]>>,
    /// All rings of all faces concatenated; each face is its exterior ring then
    /// its hole rings. Width from `vertices.len() - 1`.
    pub(crate) face_indices: IndexBuffer<1>,
    /// `len() = n_faces + 1`; face i spans
    /// `face_indices[face_offsets[i]..face_offsets[i+1]]`. Width from
    /// `face_indices.len()`.
    pub(crate) face_offsets: IndexBuffer<1>,
    /// Start in `face_indices` of each hole ring, across all faces; empty when
    /// no face has holes. Width from `face_indices.len()`.
    pub(crate) interior_offsets: IndexBuffer<1>,
    /// Geometric UV, parallel to the corner buffers; empty = no UV.
    pub(crate) uv_sets: Vec<UvSet>,
    /// Optional materials / themes / per-face binding; `None` = bare.
    pub(crate) appearance: Option<Appearance>,
}

/// A connected, vertex-sharing polygon mesh in 3D space.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PolygonMesh3D {
    /// Coordinate frame these vertices are expressed in.
    pub(crate) coordinate: Coordinate,
    pub(crate) vertices: Vec<[f64; 3]>,
    /// All rings of all faces concatenated; each face is its exterior ring then
    /// its hole rings. Width from `vertices.len() - 1`.
    pub(crate) face_indices: IndexBuffer<1>,
    /// `len() = n_faces + 1`; face i spans
    /// `face_indices[face_offsets[i]..face_offsets[i+1]]`. Width from
    /// `face_indices.len()`.
    pub(crate) face_offsets: IndexBuffer<1>,
    /// Start in `face_indices` of each hole ring, across all faces; empty when
    /// no face has holes. Width from `face_indices.len()`.
    pub(crate) interior_offsets: IndexBuffer<1>,
    /// Geometric UV, parallel to the corner buffers; empty = no UV.
    pub(crate) uv_sets: Vec<UvSet>,
    /// Optional materials / themes / per-face binding; `None` = bare.
    pub(crate) appearance: Option<Appearance>,
}
