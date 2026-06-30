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

use nusamai_projection::crs::EpsgCode;
use serde::{Deserialize, Serialize};

use crate::appearance::{Appearance, UvSet};
use crate::coordinate::Coordinate;
use crate::error::Result;
use crate::index::IndexBuffer;
use crate::ops::reproject::{transform_coords_2d, transform_coords_3d, Transformer};

mod constructor;
mod ops;

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
    /// from `face_indices.len() - 1`.
    face_offsets: IndexBuffer<1>,
    /// Start in `face_indices` of each hole ring, across all faces; empty when
    /// no face has holes. Width from `face_indices.len() - 1`.
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
    /// from `face_indices.len() - 1`.
    face_offsets: IndexBuffer<1>,
    /// Start in `face_indices` of each hole ring, across all faces; empty when
    /// no face has holes. Width from `face_indices.len() - 1`.
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

impl PolygonMesh3DData {
    /// The vertex pool. Crate-internal: lets a [`Solid`](crate::solid::Solid)
    /// shell bound itself without exposing the raw layout.
    #[inline]
    pub(crate) fn vertices(&self) -> &[[f64; 3]] {
        &self.vertices
    }
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

    /// Reproject the vertex pool to `target` (EPSG), reading the source CRS from
    /// the frame. Indices are unaffected; elevation, when present, is transformed.
    pub(crate) fn reproject(&mut self, target: EpsgCode, cache: &mut Transformer) -> Result<()> {
        let from = self.coordinate.require_crs()?;
        if from != target {
            transform_coords_2d(
                cache,
                from,
                target,
                &mut self.vertices,
                self.z.as_deref_mut(),
            )?;
            self.coordinate = Coordinate::Crs(target);
        }
        Ok(())
    }
}

impl PolygonMesh3DData {
    /// The vertex pool, mutable. Frameless mesh data shared with `Solid` shells.
    pub(crate) fn vertices_mut(&mut self) -> &mut [[f64; 3]] {
        &mut self.vertices
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

    /// Reproject the vertex pool to `target` (EPSG), reading the source CRS from
    /// the frame. Indices are unaffected.
    pub(crate) fn reproject(&mut self, target: EpsgCode, cache: &mut Transformer) -> Result<()> {
        let from = self.coordinate.require_crs()?;
        if from != target {
            transform_coords_3d(cache, from, target, self.data.vertices_mut())?;
            self.coordinate = Coordinate::Crs(target);
        }
        Ok(())
    }
}

impl PolygonMesh3DData {
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
    use crate::appearance::{
        ChannelId, FaceBinding, MaterialIndex, Side, ThemeBinding, ThemeId, UvSource,
    };
    use crate::test_support::bare;

    fn uv(side: Side) -> UvSet {
        UvSet {
            theme: Some(ThemeId(Arc::from("t"))),
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

    #[test]
    fn make_front_only_is_a_noop_when_already_front() {
        let mut m = PolygonMesh3DData::from_parts(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [[0u32, 1, 2]],
        )
        .unwrap();
        m.uv_sets = vec![uv(Side::Front)];
        m.make_front_only();
        assert_eq!(m.uv_sets.len(), 1);
        assert!(m.appearance.is_none());
    }
}
