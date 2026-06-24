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

mod constructor;

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

/// The coordinate-free data of a 3D triangle mesh: the vertex pool, triangle
/// index list, UV and appearance, with no frame of its own.
///
/// Shared by two hosts that each supply the frame: the standalone
/// [`TriangularMesh3D`] leaf pairs this with its own [`Coordinate`], while a
/// [`Solid`](crate::solid::Solid) shell stores it directly and takes the one
/// frame from the enclosing `Solid` — so a solid and its boundaries cannot
/// disagree on a frame. Mirrors the [`Raster`](crate::appearance::Raster) /
/// [`RasterData`](crate::appearance::RasterData) split.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TriangularMesh3DData {
    vertices: Vec<[f64; 3]>,
    /// Flat triangle index list; width from `vertices.len() - 1`.
    indices: IndexBuffer<3>,
    /// Geometric UV, parallel to the corner buffers; empty = no UV.
    uv_sets: Vec<UvSet>,
    /// Optional materials / themes / per-face binding; `None` = bare.
    appearance: Option<Appearance>,
}

/// A triangle mesh in 3D space: coordinate-free mesh data plus the frame it is
/// expressed in.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TriangularMesh3D {
    /// Coordinate frame the mesh data is expressed in.
    coordinate: Coordinate,
    /// Coordinate-free mesh data; the same form a [`Solid`](crate::solid::Solid)
    /// shell stores directly.
    data: TriangularMesh3DData,
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
        &self.data.appearance
    }

    /// Mutably borrow the appearance, to set, clear, or edit it in place.
    #[inline]
    pub fn appearance_mut(&mut self) -> &mut Option<Appearance> {
        &mut self.data.appearance
    }
}

impl TriangularMesh3DData {
    /// Drop all back-side appearance, keeping only the front; see
    /// [`crate::appearance::make_front_only`].
    pub(crate) fn make_front_only(&mut self) {
        crate::appearance::make_front_only(&mut self.appearance, &mut self.uv_sets);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::appearance::{FaceBinding, MaterialIndex, Side, ThemeBinding, ThemeId, UvSource};
    use crate::test_support::bare;

    fn uv(side: Side) -> UvSet {
        UvSet {
            theme: Some(ThemeId(Arc::from("t"))),
            side,
            channel: None,
            uv: UvSource::Explicit(Box::new([])),
        }
    }

    #[test]
    fn make_front_only_drops_back_binding_and_uv() {
        let mut m = TriangularMesh3DData::from_parts(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        m.appearance = Some(Appearance {
            materials: vec![bare(), bare()],
            themes: vec![ThemeBinding {
                theme: ThemeId(Arc::from("t")),
                front: FaceBinding::Uniform(MaterialIndex::new(0).unwrap()),
                back: Some(FaceBinding::Uniform(MaterialIndex::new(1).unwrap())),
            }],
            default_theme: ThemeId(Arc::from("t")),
        });
        m.uv_sets = vec![uv(Side::Front), uv(Side::Back)];

        m.make_front_only();

        assert!(m.appearance.as_ref().unwrap().themes[0].back.is_none());
        assert_eq!(m.uv_sets.len(), 1);
        assert_eq!(m.uv_sets[0].side, Side::Front);
    }
}
