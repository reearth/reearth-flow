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

use crate::appearance::Appearance;
use crate::coordinate::CoordinateFrame;
use crate::index::IndexBuffer;

mod constructor;
mod ops;

/// A triangle mesh in 2D space, with optional per-vertex elevation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TriangularMesh2D {
    /// Coordinate frame these vertices are expressed in.
    frame: CoordinateFrame,
    vertices: Vec<[f64; 2]>,
    /// Optional per-vertex elevation, parallel to `vertices`. INVARIANT: when
    /// `Some`, `z.len() == vertices.len()`. `None` = pure 2D.
    z: Option<Box<[f64]>>,
    /// Flat triangle index list; width from `vertices.len() - 1`.
    indices: IndexBuffer<3>,
    /// Optional materials / themes / per-face binding, incl. per-theme UV parallel
    /// to the corner buffers; `None` = bare.
    appearance: Option<Appearance>,
}

/// The coordinate-free data of a 3D triangle mesh: the vertex pool, triangle
/// index list and appearance, with no frame of its own.
///
/// Shared by two hosts that each supply the frame: the standalone
/// [`TriangularMesh3D`] leaf pairs this with its own [`CoordinateFrame`], while a
/// [`Solid`](crate::solid::Solid) shell stores it directly and takes the one
/// frame from the enclosing `Solid` — so a solid and its boundaries cannot
/// disagree on a frame. Mirrors the [`Raster`](crate::appearance::Raster) /
/// [`RasterData`](crate::appearance::RasterData) split.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TriangularMesh3DData {
    vertices: Vec<[f64; 3]>,
    /// Flat triangle index list; width from `vertices.len() - 1`.
    indices: IndexBuffer<3>,
    /// Optional materials / themes / per-face binding, incl. per-theme UV parallel
    /// to the corner buffers; `None` = bare.
    appearance: Option<Appearance>,
}

/// A triangle mesh in 3D space: coordinate-free mesh data plus the frame it is
/// expressed in.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TriangularMesh3D {
    /// Coordinate frame the mesh data is expressed in.
    frame: CoordinateFrame,
    /// coordinate-free mesh data; the same form a [`Solid`](crate::solid::Solid)
    /// shell stores directly.
    data: TriangularMesh3DData,
}

impl TriangularMesh3DData {
    /// The vertex pool. Crate-internal: lets a [`Solid`](crate::solid::Solid)
    /// shell bound itself without exposing the raw layout.
    #[inline]
    pub(crate) fn vertices(&self) -> &[[f64; 3]] {
        &self.vertices
    }
}

impl TriangularMesh2D {
    /// The number of triangles in the mesh.
    #[inline]
    pub fn num_triangles(&self) -> usize {
        self.indices.len()
    }

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

impl TriangularMesh3DData {
    /// The vertex pool, mutable.
    pub(crate) fn vertices_mut(&mut self) -> &mut [[f64; 3]] {
        &mut self.vertices
    }
}

impl TriangularMesh3D {
    /// The number of triangles in the mesh.
    #[inline]
    pub fn num_triangles(&self) -> usize {
        self.data.indices.len()
    }

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

    /// Consume the mesh, yielding its coordinate-free data for use as a
    /// [`Solid`](crate::solid::Solid) shell.
    #[inline]
    pub fn into_data(self) -> TriangularMesh3DData {
        self.data
    }

    /// The shared vertex pool.
    #[inline]
    pub fn vertices(&self) -> &[[f64; 3]] {
        self.data.vertices()
    }
}

impl TriangularMesh3DData {
    /// The number of triangles.
    #[inline]
    pub fn num_triangles(&self) -> usize {
        self.indices.len()
    }

    /// Drop all back-side appearance, keeping only the front; see
    /// [`crate::appearance::make_front_only`].
    pub(crate) fn make_front_only(&mut self) {
        crate::appearance::make_front_only(&mut self.appearance);
    }
}

// Tessellation is defined only for `Polygon` / `PolygonMesh`.
crate::unsupported!(TriangularMesh2D: Triangulate);
crate::unsupported!(TriangularMesh3D: Triangulate);

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::appearance::{
        ChannelId, FaceBinding, MaterialIndex, Side, ThemeBinding, ThemeId, UvSet, UvSource,
    };
    use crate::test_support::bare;

    fn uv(side: Side) -> UvSet {
        UvSet {
            side,
            channel: ChannelId::default(),
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
        let theme = ThemeId(Arc::from("t"));
        m.appearance = Some(Appearance::from_parts(
            vec![bare(), bare()],
            vec![ThemeBinding {
                theme: theme.clone(),
                front: FaceBinding::Uniform(MaterialIndex::new(0).unwrap()),
                back: Some(FaceBinding::Uniform(MaterialIndex::new(1).unwrap())),
                uv_sets: vec![uv(Side::Front), uv(Side::Back)],
            }],
            theme,
        ));

        m.make_front_only();

        let app = m.appearance.as_ref().unwrap();
        assert!(app.themes()[0].back.is_none());
        assert_eq!(app.themes()[0].uv_sets.len(), 1);
        assert_eq!(app.themes()[0].uv_sets[0].side, Side::Front);
    }
}
