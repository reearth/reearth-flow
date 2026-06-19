//! TriangularMesh leaves.
//!
//! Vertex-pool format: the triangles are represented by three indices into a
//! shared vertex pool. The index list uses a dynamic width chosen from
//! `vertices.len() - 1` at construction.
//!
//! Comes in 2D and 3D variants. The 2D variant carries `vertices: Vec<[f64; 2]>`
//! plus an optional per-vertex elevation buffer parallel to `vertices`, matching
//! the 2D leaf convention.

use serde::{Deserialize, Serialize};

use crate::appearance::{Appearance, UvSet};
use crate::coordinate::Coordinate;
use crate::index::IndexBuffer;

/// A triangle mesh in 2D space, with optional per-vertex elevation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TriangularMesh2D {
    /// Coordinate frame these vertices are expressed in.
    coordinate: Coordinate,
    vertices: Vec<[f64; 2]>,
    /// Optional per-vertex elevation, parallel to `vertices`. INVARIANT: when
    /// `Some`, `z.len() == vertices.len()`. `None` = pure 2D.
    z: Option<Box<[f64]>>,
    /// Flat triangle index list; width from `vertices.len() - 1`.
    indices: IndexBuffer<3>,
    /// Geometric UV, parallel to the corner buffers; empty = no UV.
    uv_sets: Vec<UvSet>,
    /// Optional materials / themes / per-face binding; `None` = bare.
    appearance: Option<Appearance>,
}

/// A triangle mesh in 3D space.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TriangularMesh3D {
    /// Coordinate frame these vertices are expressed in.
    coordinate: Coordinate,
    vertices: Vec<[f64; 3]>,
    /// Flat triangle index list; width from `vertices.len() - 1`.
    indices: IndexBuffer<3>,
    /// Geometric UV, parallel to the corner buffers; empty = no UV.
    uv_sets: Vec<UvSet>,
    /// Optional materials / themes / per-face binding; `None` = bare.
    appearance: Option<Appearance>,
}

impl TriangularMesh2D {
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

impl TriangularMesh3D {
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
