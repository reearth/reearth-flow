//! TriangularMesh constructors.
//!
//! Two entry points, matching how a triangle mesh's buffers actually arrive:
//!
//! * `from_parts` — the vertex pool is already in hand (glTF / OBJ accessors,
//!   earcut output over an input ring, any algorithm that tracks its own
//!   indices). The index width is fixed by the vertex count — the largest index a
//!   valid mesh can hold is `vertices.len() - 1` — so it is computed once, with no
//!   scan of the index values, and the triangles are packed straight into that
//!   exact width. Indices arrive as a flat `u32` stream (the glTF / earcut shape)
//!   and are grouped into triples; the checked form validates that every index is
//!   in range and the count is a multiple of three.
//!
//! * `from_soup` — a flat stream of triangle-corner coordinates with no sharing
//!   (STL, marching-cubes-style emitters). The vertex pool is *discovered* by
//!   deduplicating corners, so the final count — hence the index width — is not
//!   known up front; the indices grow through [`IndexBuffer::from_indices`], which
//!   starts at `u8` and widens only as the pool crosses a width boundary, keeping
//!   the common small mesh narrow.
//!
//! 3D constructors build the coordinate-free [`TriangularMesh3DData`] that a
//! [`Solid`](crate::solid::Solid) shell also stores, so the frame-carrying
//! [`TriangularMesh3D`] is a thin wrapper. All meshes are built bare (no UV, no
//! appearance); attach an appearance afterwards via `appearance_mut`.

use std::collections::HashMap;

use crate::coordinate::Coordinate;
use crate::error::Error;
use crate::index::{IndexBuffer, IndexWidth};

use super::{TriangularMesh2D, TriangularMesh3D, TriangularMesh3DData};

impl TriangularMesh3DData {
    /// Build coordinate-free mesh data from a vertex pool and a flat `u32` index
    /// stream. Validates that the index count is a multiple of three and every
    /// index is `< vertices.len()`; the width is taken from the vertex count.
    pub fn from_parts(
        vertices: Vec<[f64; 3]>,
        indices: impl IntoIterator<Item = u32>,
    ) -> Result<Self, Error> {
        let width = index_width_for(vertices.len());
        let triangles = triangles_checked(indices, vertices.len())?;
        Ok(Self {
            vertices,
            indices: pack_checked(width, triangles),
            uv_sets: Vec::new(),
            appearance: None,
        })
    }

    /// Build mesh data without validating indices, filling the index buffer by the
    /// fast uninitialized path.
    ///
    /// # Safety
    /// The caller must ensure every index is `< vertices.len()` (an out-of-range
    /// index is silently truncated into the vertex-count-derived width) and that
    /// the flat `indices` stream yields exactly `3 * triangle_count` items.
    #[allow(unused)] // TODO: remove this after the migration is complete at which point this will be used.
    pub(crate) unsafe fn from_parts_unchecked(
        vertices: Vec<[f64; 3]>,
        triangle_count: usize,
        indices: impl IntoIterator<Item = u32>,
    ) -> Self {
        let width = index_width_for(vertices.len());
        Self {
            indices: IndexBuffer::from_exact_unchecked(
                width,
                triangle_count,
                group_triples(indices),
            ),
            vertices,
            uv_sets: Vec::new(),
            appearance: None,
        }
    }

    /// Build mesh data from a triangle soup: a flat stream of corner coordinates,
    /// three per triangle, deduplicated into a shared vertex pool.
    pub fn from_soup(iter: impl IntoIterator<Item = [f64; 3]>) -> Self {
        let (vertices, indices) = soup_buffers::<3>(iter);
        Self {
            vertices,
            indices,
            uv_sets: Vec::new(),
            appearance: None,
        }
    }
}

impl TriangularMesh3D {
    /// Pair coordinate-free mesh data with the frame it is expressed in.
    pub fn new(coordinate: Coordinate, data: TriangularMesh3DData) -> Self {
        Self { coordinate, data }
    }

    /// Build from a vertex pool and a flat `u32` index stream; see
    /// [`TriangularMesh3DData::from_parts`].
    pub fn from_parts(
        coordinate: Coordinate,
        vertices: Vec<[f64; 3]>,
        indices: impl IntoIterator<Item = u32>,
    ) -> Result<Self, Error> {
        Ok(Self::new(
            coordinate,
            TriangularMesh3DData::from_parts(vertices, indices)?,
        ))
    }

    /// Build without validating indices; see
    /// [`TriangularMesh3DData::from_parts_unchecked`].
    ///
    /// # Safety
    /// Same contract as [`TriangularMesh3DData::from_parts_unchecked`].
    #[allow(unused)] // TODO: remove this after the migration is complete at which point this will be used.
    pub unsafe fn from_parts_unchecked(
        coordinate: Coordinate,
        vertices: Vec<[f64; 3]>,
        triangle_count: usize,
        indices: impl IntoIterator<Item = u32>,
    ) -> Self {
        Self::new(
            coordinate,
            TriangularMesh3DData::from_parts_unchecked(vertices, triangle_count, indices),
        )
    }

    /// Build from a triangle soup; see [`TriangularMesh3DData::from_soup`].
    pub fn from_soup(coordinate: Coordinate, iter: impl IntoIterator<Item = [f64; 3]>) -> Self {
        Self::new(coordinate, TriangularMesh3DData::from_soup(iter))
    }
}

impl TriangularMesh2D {
    /// Build a pure-2D mesh from a vertex pool and a flat `u32` index stream.
    /// Validates the index count and range; the width is taken from the vertex
    /// count.
    pub fn from_parts(
        coordinate: Coordinate,
        vertices: Vec<[f64; 2]>,
        indices: impl IntoIterator<Item = u32>,
    ) -> Result<Self, Error> {
        let width = index_width_for(vertices.len());
        let triangles = triangles_checked(indices, vertices.len())?;
        Ok(Self {
            coordinate,
            vertices,
            z: None,
            indices: pack_checked(width, triangles),
            uv_sets: Vec::new(),
            appearance: None,
        })
    }

    /// Build a 2.5D mesh from `[x, y, z]` vertices: the `(x, y)` populate the
    /// vertex pool and the `z` the parallel elevation buffer.
    pub fn from_parts_with_elevation(
        coordinate: Coordinate,
        vertices: Vec<[f64; 3]>,
        indices: impl IntoIterator<Item = u32>,
    ) -> Result<Self, Error> {
        let width = index_width_for(vertices.len());
        let triangles = triangles_checked(indices, vertices.len())?;
        let (xy, z) = split_elevation(vertices);
        Ok(Self {
            coordinate,
            vertices: xy,
            z: Some(z),
            indices: pack_checked(width, triangles),
            uv_sets: Vec::new(),
            appearance: None,
        })
    }

    /// Build a pure-2D mesh without validating indices.
    ///
    /// # Safety
    /// Same contract as [`TriangularMesh3DData::from_parts_unchecked`].
    pub unsafe fn from_parts_unchecked(
        coordinate: Coordinate,
        vertices: Vec<[f64; 2]>,
        triangle_count: usize,
        indices: impl IntoIterator<Item = u32>,
    ) -> Self {
        let width = index_width_for(vertices.len());
        Self {
            coordinate,
            indices: IndexBuffer::from_exact_unchecked(
                width,
                triangle_count,
                group_triples(indices),
            ),
            vertices,
            z: None,
            uv_sets: Vec::new(),
            appearance: None,
        }
    }

    /// Build a pure-2D mesh from a triangle soup of `[x, y]` corners.
    pub fn from_soup(coordinate: Coordinate, iter: impl IntoIterator<Item = [f64; 2]>) -> Self {
        let (vertices, indices) = soup_buffers::<2>(iter);
        Self {
            coordinate,
            vertices,
            z: None,
            indices,
            uv_sets: Vec::new(),
            appearance: None,
        }
    }
}

/// Narrowest index width that can address `vertex_count` vertices (largest index
/// `vertex_count - 1`); `IndexWidth::U8` for the empty mesh.
fn index_width_for(vertex_count: usize) -> IndexWidth {
    IndexWidth::for_value(vertex_count.saturating_sub(1) as u32)
}

/// Group a flat index stream into triangles, validating that the count is a
/// multiple of three and every index addresses a vertex `< vertex_count`.
fn triangles_checked(
    indices: impl IntoIterator<Item = u32>,
    vertex_count: usize,
) -> Result<Vec<[u32; 3]>, Error> {
    let mut it = indices.into_iter();
    let mut triangles = Vec::with_capacity(it.size_hint().0 / 3);
    while let Some(a) = it.next() {
        let (Some(b), Some(c)) = (it.next(), it.next()) else {
            return Err(Error::invalid_geometry(
                "triangle index count is not a multiple of three",
            ));
        };
        for index in [a, b, c] {
            if index as usize >= vertex_count {
                return Err(Error::invalid_geometry(format!(
                    "triangle index {index} is out of range for {vertex_count} vertices"
                )));
            }
        }
        triangles.push([a, b, c]);
    }
    Ok(triangles)
}

/// Group a flat index stream into triangles, dropping a trailing partial triple.
/// No validation — for the unchecked path only.
fn group_triples(indices: impl IntoIterator<Item = u32>) -> impl Iterator<Item = [u32; 3]> {
    let mut it = indices.into_iter();
    std::iter::from_fn(move || Some([it.next()?, it.next()?, it.next()?]))
}

/// Pack already-range-checked triangles into the vertex-count-derived `width`.
/// Every index was validated `< vertex_count`, so it fits `width` (derived from
/// `vertex_count - 1`) and `with_exact_width` never panics here.
fn pack_checked(width: IndexWidth, triangles: Vec<[u32; 3]>) -> IndexBuffer<3> {
    IndexBuffer::with_exact_width(width, Some(triangles.len()), triangles)
}

/// Index of `coord` in `vertices`, inserting it (and assigning the next index) on
/// first sight. Dedup is on exact `f64` bits.
fn dedup_index<const N: usize>(
    vertices: &mut Vec<[f64; N]>,
    seen: &mut HashMap<[u64; N], u32>,
    coord: [f64; N],
) -> u32 {
    *seen.entry(coord.map(f64::to_bits)).or_insert_with(|| {
        let index = vertices.len() as u32;
        vertices.push(coord);
        index
    })
}

/// Deduplicate a flat triangle-corner stream into a vertex pool and a grown index
/// buffer, shared by the 2D and 3D soup constructors. A trailing partial triangle
/// is dropped.
fn soup_buffers<const N: usize>(
    iter: impl IntoIterator<Item = [f64; N]>,
) -> (Vec<[f64; N]>, IndexBuffer<3>) {
    let mut vertices: Vec<[f64; N]> = Vec::new();
    let mut seen: HashMap<[u64; N], u32> = HashMap::new();
    let mut src = iter.into_iter();
    let vertices_ref = &mut vertices;
    let seen_ref = &mut seen;
    let triangles = std::iter::from_fn(move || {
        let a = src.next()?;
        let b = src.next()?;
        let c = src.next()?;
        Some([
            dedup_index(vertices_ref, seen_ref, a),
            dedup_index(vertices_ref, seen_ref, b),
            dedup_index(vertices_ref, seen_ref, c),
        ])
    });
    let indices = IndexBuffer::from_indices(triangles);
    (vertices, indices)
}

/// Split a 3D vertex buffer into its 2D footprint and a parallel elevation buffer.
fn split_elevation(vertices: Vec<[f64; 3]>) -> (Vec<[f64; 2]>, Box<[f64]>) {
    let mut xy = Vec::with_capacity(vertices.len());
    let mut z = Vec::with_capacity(vertices.len());
    for [x, y, elevation] in vertices {
        xy.push([x, y]);
        z.push(elevation);
    }
    (xy, z.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triples(buf: &IndexBuffer<3>) -> Vec<[u32; 3]> {
        match buf {
            IndexBuffer::U8(v) => v.iter().map(|t| (*t).map(u32::from)).collect(),
            IndexBuffer::U16(v) => v.iter().map(|t| (*t).map(u32::from)).collect(),
            IndexBuffer::U32(v) => v.iter().map(|t| (*t).map(u32::from)).collect(),
        }
    }

    #[test]
    fn from_parts_builds_validated_mesh() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let m = TriangularMesh3D::from_parts(Coordinate::Euclidean, verts, [0u32, 1, 2]).unwrap();
        assert_eq!(m.data.vertices.len(), 3);
        assert_eq!(m.data.indices.width(), IndexWidth::U8);
        assert_eq!(triples(&m.data.indices), vec![[0, 1, 2]]);
    }

    #[test]
    fn from_parts_width_follows_vertex_count_not_indices() {
        // 300 vertices -> u16, even though the one triangle uses only small indices.
        let verts: Vec<[f64; 3]> = (0..300).map(|i| [i as f64, 0.0, 0.0]).collect();
        let m = TriangularMesh3DData::from_parts(verts, [0u32, 1, 2]).unwrap();
        assert_eq!(m.indices.width(), IndexWidth::U16);
    }

    #[test]
    fn from_parts_rejects_out_of_range_index() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let err = TriangularMesh3DData::from_parts(verts, [0u32, 1, 3]).unwrap_err();
        assert!(matches!(err, Error::InvalidGeometry(_)));
    }

    #[test]
    fn from_parts_rejects_non_multiple_of_three() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        assert!(TriangularMesh3DData::from_parts(verts, [0u32, 1]).is_err());
    }

    #[test]
    fn from_soup_dedups_shared_vertices() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let d = [1.0, 1.0, 0.0];
        // Two triangles sharing edge b-c: a b c, b c d -> 4 unique vertices.
        let m = TriangularMesh3DData::from_soup([a, b, c, b, c, d]);
        assert_eq!(m.vertices, vec![a, b, c, d]);
        assert_eq!(triples(&m.indices), vec![[0, 1, 2], [1, 2, 3]]);
        assert_eq!(m.indices.width(), IndexWidth::U8);
    }

    #[test]
    fn from_soup_drops_trailing_partial_triangle() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let d = [1.0, 1.0, 0.0];
        let e = [2.0, 2.0, 0.0];
        // One full triangle plus two leftover corners: d and e are never consumed
        // into a triple, so they are not added to the pool.
        let m = TriangularMesh3DData::from_soup([a, b, c, d, e]);
        assert_eq!(triples(&m.indices), vec![[0, 1, 2]]);
        assert_eq!(m.vertices, vec![a, b, c]);
    }

    #[test]
    fn from_parts_with_elevation_splits_z() {
        let verts = vec![[0.0, 0.0, 10.0], [1.0, 0.0, 11.0], [0.0, 1.0, 12.0]];
        let m =
            TriangularMesh2D::from_parts_with_elevation(Coordinate::Euclidean, verts, [0u32, 1, 2])
                .unwrap();
        assert_eq!(m.vertices, vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
        assert_eq!(m.z.as_deref(), Some(&[10.0, 11.0, 12.0][..]));
    }

    #[test]
    fn from_parts_unchecked_matches_checked() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let checked = TriangularMesh3DData::from_parts(verts.clone(), [0u32, 1, 2]).unwrap();
        let unchecked =
            unsafe { TriangularMesh3DData::from_parts_unchecked(verts, 1, [0u32, 1, 2]) };
        assert_eq!(checked, unchecked);
    }
}
