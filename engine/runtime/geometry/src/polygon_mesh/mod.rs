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

use crate::appearance::Appearance;
use crate::coordinate::CoordinateFrame;
use crate::index::IndexBuffer;

mod constructor;
mod ops;
#[cfg(feature = "new-geometry")]
mod validation;

/// A connected, vertex-sharing polygon mesh in 2D space, with optional
/// per-vertex elevation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PolygonMesh2D {
    /// Coordinate frame these vertices are expressed in.
    frame: CoordinateFrame,
    vertices: Vec<[f64; 2]>,
    /// Optional per-vertex elevation, parallel to `vertices`. INVARIANT: when
    /// `Some`, `z.len() == vertices.len()`. `None` = pure 2D.
    z: Option<Box<[f64]>>,
    /// All rings of all faces concatenated; each face is its exterior ring then
    /// its hole rings. A valid face has the exterior wound counter-clockwise and
    /// interiors clockwise in canonical orientation (see [`crate::coordinate`]:
    /// winding is judged after applying the frame's orientation sign, not in stored
    /// coordinate order). Width from `vertices.len() - 1`.
    face_indices: IndexBuffer<1>,
    /// Internal face boundaries into `face_indices`: `len() = n_faces - 1`, no
    /// leading 0. `face_offsets[i]` is where face `i+1` begins; face `i` spans
    /// `face_indices[s..e]` with `s = if i == 0 { 0 } else { face_offsets[i-1] }`
    /// and `e = face_offsets.get(i).copied().unwrap_or(face_indices.len())`. Width
    /// from `face_indices.len() - 1`.
    face_offsets: IndexBuffer<1>,
    /// Start in `face_indices` of each hole ring, across all faces; empty when
    /// no face has holes. Width from `face_indices.len() - 1`.
    interior_offsets: IndexBuffer<1>,
    /// Optional materials / themes / per-face binding, incl. per-theme UV parallel
    /// to the corner buffers; `None` = bare.
    appearance: Option<Appearance>,
}

/// The coordinate-free data of a 3D polygon mesh: the vertex pool, CSR face
/// topology and appearance, with no frame of its own.
///
/// Shared by two hosts that each supply the frame: the standalone
/// [`PolygonMesh3D`] leaf pairs this with its own [`CoordinateFrame`], while a
/// [`Solid`](crate::solid::Solid) shell stores it directly and takes the one
/// frame from the enclosing `Solid` — so a solid and its boundaries cannot
/// disagree on a frame. Mirrors the [`Raster`](crate::appearance::Raster) /
/// [`RasterData`](crate::appearance::RasterData) split.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PolygonMesh3DData {
    vertices: Vec<[f64; 3]>,
    /// All rings of all faces concatenated; each face is its exterior ring then
    /// its hole rings. Each face's canonical outward normal is its exterior's
    /// right-hand-rule normal times the frame's orientation sign (see
    /// [`crate::coordinate`]). A valid face has exterior and interior rings wound
    /// opposite to each other. Width from `vertices.len() - 1`.
    face_indices: IndexBuffer<1>,
    /// Internal face boundaries into `face_indices`: `len() = n_faces - 1`, no
    /// leading 0. `face_offsets[i]` is where face `i+1` begins; face `i` spans
    /// `face_indices[s..e]` with `s = if i == 0 { 0 } else { face_offsets[i-1] }`
    /// and `e = face_offsets.get(i).copied().unwrap_or(face_indices.len())`. Width
    /// from `face_indices.len() - 1`.
    face_offsets: IndexBuffer<1>,
    /// Start in `face_indices` of each hole ring, across all faces; empty when
    /// no face has holes. Width from `face_indices.len() - 1`.
    interior_offsets: IndexBuffer<1>,
    /// Optional materials / themes / per-face binding, incl. per-theme UV parallel
    /// to the corner buffers; `None` = bare.
    appearance: Option<Appearance>,
}

/// A connected, vertex-sharing polygon mesh in 3D space: coordinate-free mesh
/// data plus the frame it is expressed in.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PolygonMesh3D {
    /// Coordinate frame the mesh data is expressed in.
    frame: CoordinateFrame,
    /// coordinate-free mesh data; the same form a [`Solid`](crate::solid::Solid)
    /// shell stores directly.
    data: PolygonMesh3DData,
}

impl PolygonMesh3DData {
    /// The vertex pool. Crate-internal: lets a [`Solid`](crate::solid::Solid)
    /// shell bound itself without exposing the raw layout.
    #[inline]
    pub(crate) fn vertices(&self) -> &[[f64; 3]] {
        &self.vertices
    }
}

impl PolygonMesh2D {
    /// The coordinate frame these vertices are expressed in.
    #[inline]
    pub fn frame(&self) -> &CoordinateFrame {
        &self.frame
    }

    /// The shared vertex pool.
    #[inline]
    pub fn vertices(&self) -> &[[f64; 2]] {
        &self.vertices
    }

    /// The number of faces.
    #[inline]
    pub fn num_faces(&self) -> usize {
        if self.face_indices.len() == 0 {
            0
        } else {
            self.face_offsets.len() + 1
        }
    }

    /// The CSR ring buffers `(face_indices, face_offsets, interior_offsets)`,
    /// for the predicate views to decode.
    #[inline]
    pub(crate) fn csr_buffers(&self) -> (&IndexBuffer<1>, &IndexBuffer<1>, &IndexBuffer<1>) {
        (
            &self.face_indices,
            &self.face_offsets,
            &self.interior_offsets,
        )
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

impl PolygonMesh3DData {
    /// The vertex pool, mutable.
    pub(crate) fn vertices_mut(&mut self) -> &mut [[f64; 3]] {
        &mut self.vertices
    }
}

impl PolygonMesh3D {
    /// The coordinate frame the mesh data is expressed in.
    #[inline]
    pub fn frame(&self) -> &CoordinateFrame {
        &self.frame
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
    pub fn into_data(self) -> PolygonMesh3DData {
        self.data
    }

    /// The number of faces.
    #[inline]
    pub fn num_faces(&self) -> usize {
        self.data.num_faces()
    }

    /// The shared vertex pool.
    #[inline]
    pub fn vertices(&self) -> &[[f64; 3]] {
        self.data.vertices()
    }
}

impl PolygonMesh3DData {
    /// The number of faces.
    #[inline]
    pub fn num_faces(&self) -> usize {
        if self.face_indices.len() == 0 {
            0
        } else {
            self.face_offsets.len() + 1
        }
    }
}

impl PolygonMesh3DData {
    /// Drop all back-side appearance, keeping only the front; see
    /// [`crate::appearance::make_front_only`].
    pub(crate) fn make_front_only(&mut self) {
        crate::appearance::make_front_only(&mut self.appearance);
    }
}

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
        let mut m = PolygonMesh3DData::from_parts(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [[0u32, 1, 2]],
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

    #[test]
    fn make_front_only_leaves_a_front_only_appearance_intact() {
        let mut m = PolygonMesh3DData::from_parts(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [[0u32, 1, 2]],
        )
        .unwrap();
        let theme = ThemeId(Arc::from("t"));
        m.appearance = Some(Appearance::from_parts(
            vec![bare()],
            vec![ThemeBinding {
                theme: theme.clone(),
                front: FaceBinding::Uniform(MaterialIndex::new(0).unwrap()),
                back: None,
                uv_sets: vec![uv(Side::Front)],
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
