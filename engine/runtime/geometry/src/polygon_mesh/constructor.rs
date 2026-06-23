//! PolygonMesh constructors.
//!
//! Three entry points, matching how a polygon mesh's faces arrive:
//!
//! * `from_polygons` — a set of independent [`Polygon`](crate::polygon) faces, the
//!   shape a CityGML surface parses into (a flat list of face polygons, each with
//!   its own coords and holes, no shared pool). The shared vertex pool is
//!   *discovered* by deduplicating face corners (exact `f64` bits), and each
//!   polygon maps 1:1 to a mesh face — exterior then holes. Because the CityGML
//!   reader also emits each face as its own `Polygon` feature, the mesh is built
//!   from those same borrowed polygons with no extra geometry logic.
//!
//! * `from_parts` — a vertex pool already in hand plus faces given as vertex-index
//!   lists (OBJ). Exterior-only (standard OBJ has no holes); indices are validated
//!   `< vertices.len()`.
//!
//! * `from_raw_parts` — the flat CSR buffers in hand, validated for consistency.
//!
//! Rings are stored **open**: the closing vertex is implied by the face / hole
//! boundary, so `from_polygons` drops the closing duplicate that a `Polygon`
//! carries, and `from_parts` expects the open faces OBJ provides. The index widths
//! follow the type's contract: `face_indices` from the (dedup-discovered) vertex
//! count, `face_offsets` / `interior_offsets` from `face_indices.len()`.
//!
//! 3D constructors build the coordinate-free [`PolygonMesh3DData`] a
//! [`Solid`](crate::solid::Solid) shell also stores, so the frame-carrying
//! [`PolygonMesh3D`] is a thin wrapper. All meshes are built bare (no UV, no
//! appearance); attach an appearance afterwards via `appearance_mut`.

use std::collections::HashMap;

use crate::coordinate::Coordinate;
use crate::error::Error;
use crate::index::{IndexBuffer, IndexWidth};
use crate::polygon::Polygon3D;

use super::{PolygonMesh2D, PolygonMesh3D, PolygonMesh3DData};

impl PolygonMesh3DData {
    /// Build coordinate-free mesh data from independent face polygons, deduplicating
    /// their corners into a shared vertex pool. Each polygon becomes one face
    /// (exterior then holes), stored open.
    pub fn from_polygons<'a>(polygons: impl IntoIterator<Item = &'a Polygon3D>) -> Self {
        let faces = polygons
            .into_iter()
            .map(|p| std::iter::once(p.exterior()).chain(p.interiors()));
        let (vertices, face_indices, face_offsets, interior_offsets) = dedup_faces(faces);
        let (face_indices, face_offsets, interior_offsets) =
            pack_csr(vertices.len(), face_indices, face_offsets, interior_offsets);
        Self {
            vertices,
            face_indices,
            face_offsets,
            interior_offsets,
            uv_sets: Vec::new(),
            appearance: None,
        }
    }

    /// Build from a shared vertex pool and faces given as vertex-index lists
    /// (exterior rings, no holes). Validates every index is `< vertices.len()`.
    pub fn from_parts(
        vertices: Vec<[f64; 3]>,
        faces: impl IntoIterator<Item = impl IntoIterator<Item = u32>>,
    ) -> Result<Self, Error> {
        let (face_indices, face_offsets) = index_faces(vertices.len(), faces)?;
        let (face_indices, face_offsets, interior_offsets) =
            pack_csr(vertices.len(), face_indices, face_offsets, Vec::new());
        Ok(Self {
            vertices,
            face_indices,
            face_offsets,
            interior_offsets,
            uv_sets: Vec::new(),
            appearance: None,
        })
    }

    /// Build from flat CSR buffers, validating that the offsets are consistent and
    /// every index is in range.
    pub fn from_raw_parts(
        vertices: Vec<[f64; 3]>,
        face_indices: Vec<u32>,
        face_offsets: Vec<u32>,
        interior_offsets: Vec<u32>,
    ) -> Result<Self, Error> {
        validate_csr(
            vertices.len(),
            &face_indices,
            &face_offsets,
            &interior_offsets,
        )?;
        let (face_indices, face_offsets, interior_offsets) =
            pack_csr(vertices.len(), face_indices, face_offsets, interior_offsets);
        Ok(Self {
            vertices,
            face_indices,
            face_offsets,
            interior_offsets,
            uv_sets: Vec::new(),
            appearance: None,
        })
    }
}

impl PolygonMesh3D {
    /// Pair coordinate-free mesh data with the frame it is expressed in.
    pub fn new(coordinate: Coordinate, data: PolygonMesh3DData) -> Self {
        Self { coordinate, data }
    }

    /// Build from independent face polygons; see [`PolygonMesh3DData::from_polygons`].
    /// Errors if any polygon's frame differs from `coordinate`.
    pub fn from_polygons<'a>(
        coordinate: Coordinate,
        polygons: impl IntoIterator<Item = &'a Polygon3D>,
    ) -> Result<Self, Error> {
        let polygons: Vec<&Polygon3D> = polygons.into_iter().collect();
        for p in &polygons {
            if p.coordinate() != &coordinate {
                return Err(Error::invalid_geometry(
                    "polygon coordinate frame differs from the mesh frame",
                ));
            }
        }
        Ok(Self::new(
            coordinate,
            PolygonMesh3DData::from_polygons(polygons),
        ))
    }

    /// Build from a vertex pool and index-list faces; see
    /// [`PolygonMesh3DData::from_parts`].
    pub fn from_parts(
        coordinate: Coordinate,
        vertices: Vec<[f64; 3]>,
        faces: impl IntoIterator<Item = impl IntoIterator<Item = u32>>,
    ) -> Result<Self, Error> {
        Ok(Self::new(
            coordinate,
            PolygonMesh3DData::from_parts(vertices, faces)?,
        ))
    }

    /// Build from flat CSR buffers; see [`PolygonMesh3DData::from_raw_parts`].
    pub fn from_raw_parts(
        coordinate: Coordinate,
        vertices: Vec<[f64; 3]>,
        face_indices: Vec<u32>,
        face_offsets: Vec<u32>,
        interior_offsets: Vec<u32>,
    ) -> Result<Self, Error> {
        Ok(Self::new(
            coordinate,
            PolygonMesh3DData::from_raw_parts(
                vertices,
                face_indices,
                face_offsets,
                interior_offsets,
            )?,
        ))
    }
}

impl PolygonMesh2D {
    /// Build a pure-2D mesh from a vertex pool and index-list faces (no holes).
    pub fn from_parts(
        coordinate: Coordinate,
        vertices: Vec<[f64; 2]>,
        faces: impl IntoIterator<Item = impl IntoIterator<Item = u32>>,
    ) -> Result<Self, Error> {
        let (face_indices, face_offsets) = index_faces(vertices.len(), faces)?;
        let (face_indices, face_offsets, interior_offsets) =
            pack_csr(vertices.len(), face_indices, face_offsets, Vec::new());
        Ok(Self {
            coordinate,
            vertices,
            z: None,
            face_indices,
            face_offsets,
            interior_offsets,
            uv_sets: Vec::new(),
            appearance: None,
        })
    }

    /// Build a 2.5D mesh from `[x, y, z]` vertices (the `(x, y)` populate the pool,
    /// the `z` a parallel elevation buffer) and index-list faces (no holes).
    pub fn from_parts_with_elevation(
        coordinate: Coordinate,
        vertices: Vec<[f64; 3]>,
        faces: impl IntoIterator<Item = impl IntoIterator<Item = u32>>,
    ) -> Result<Self, Error> {
        let (face_indices, face_offsets) = index_faces(vertices.len(), faces)?;
        let (xy, z) = split_elevation(vertices);
        let (face_indices, face_offsets, interior_offsets) =
            pack_csr(xy.len(), face_indices, face_offsets, Vec::new());
        Ok(Self {
            coordinate,
            vertices: xy,
            z: Some(z),
            face_indices,
            face_offsets,
            interior_offsets,
            uv_sets: Vec::new(),
            appearance: None,
        })
    }

    /// Build a pure-2D mesh from flat CSR buffers.
    pub fn from_raw_parts(
        coordinate: Coordinate,
        vertices: Vec<[f64; 2]>,
        face_indices: Vec<u32>,
        face_offsets: Vec<u32>,
        interior_offsets: Vec<u32>,
    ) -> Result<Self, Error> {
        validate_csr(
            vertices.len(),
            &face_indices,
            &face_offsets,
            &interior_offsets,
        )?;
        let (face_indices, face_offsets, interior_offsets) =
            pack_csr(vertices.len(), face_indices, face_offsets, interior_offsets);
        Ok(Self {
            coordinate,
            vertices,
            z: None,
            face_indices,
            face_offsets,
            interior_offsets,
            uv_sets: Vec::new(),
            appearance: None,
        })
    }
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

/// Drop a ring's closing vertex when it duplicates the first, yielding the open
/// ring (closure is implied by the face / hole boundary in the mesh).
fn open_ring<const N: usize>(ring: &[[f64; N]]) -> &[[f64; N]] {
    if ring.len() > 1 && ring.first() == ring.last() {
        &ring[..ring.len() - 1]
    } else {
        ring
    }
}

/// Deduplicate the corners of a sequence of faces (each a sequence of rings,
/// exterior first) into a shared pool and the flat CSR buffers.
fn dedup_faces<'a, const N: usize, Faces, Rings>(
    faces: Faces,
) -> (Vec<[f64; N]>, Vec<u32>, Vec<u32>, Vec<u32>)
where
    Faces: IntoIterator<Item = Rings>,
    Rings: IntoIterator<Item = &'a [[f64; N]]>,
{
    let mut vertices: Vec<[f64; N]> = Vec::new();
    let mut seen: HashMap<[u64; N], u32> = HashMap::new();
    let mut face_indices: Vec<u32> = Vec::new();
    let mut face_offsets: Vec<u32> = Vec::new();
    let mut interior_offsets: Vec<u32> = Vec::new();
    for face in faces {
        let index_start = face_indices.len();
        let interior_start = interior_offsets.len();
        let mut is_exterior = true;
        for ring in face {
            if !is_exterior {
                interior_offsets.push(face_indices.len() as u32);
            }
            is_exterior = false;
            for &coord in open_ring(ring) {
                face_indices.push(dedup_index(&mut vertices, &mut seen, coord));
            }
        }
        if face_indices.len() == index_start {
            // Empty face contributes no geometry: drop it (and any hole offsets it
            // recorded) so `face_offsets` stays strictly increasing with no leading 0.
            interior_offsets.truncate(interior_start);
            continue;
        }
        face_offsets.push(face_indices.len() as u32);
    }
    // `face_offsets` holds only the internal boundaries — no leading 0, no trailing
    // total — so drop the last face's end (the implicit `face_indices.len()`).
    face_offsets.pop();
    (vertices, face_indices, face_offsets, interior_offsets)
}

/// Flatten index-list faces (exterior rings, no holes) into `face_indices` and
/// `face_offsets`, validating every index is `< vertex_count`.
fn index_faces(
    vertex_count: usize,
    faces: impl IntoIterator<Item = impl IntoIterator<Item = u32>>,
) -> Result<(Vec<u32>, Vec<u32>), Error> {
    let mut face_indices: Vec<u32> = Vec::new();
    let mut face_offsets: Vec<u32> = Vec::new();
    for face in faces {
        let index_start = face_indices.len();
        for index in face {
            if index as usize >= vertex_count {
                return Err(Error::invalid_geometry(format!(
                    "face index {index} is out of range for {vertex_count} vertices"
                )));
            }
            face_indices.push(index);
        }
        if face_indices.len() == index_start {
            // Empty face: contributes no ring, so record no boundary for it (keeps
            // `face_offsets` strictly increasing with no leading 0).
            continue;
        }
        face_offsets.push(face_indices.len() as u32);
    }
    // Keep only the internal boundaries (no leading 0, no trailing total).
    face_offsets.pop();
    Ok((face_indices, face_offsets))
}

/// Validate flat CSR buffers. `face_offsets` holds the `n_faces - 1` internal face
/// boundaries (no leading 0): strictly increasing, each in `1..face_indices.len()`
/// (every face non-empty). Every vertex index is `< vertex_count`. Each interior
/// offset is strictly increasing, in `1..face_indices.len()`, and not a face
/// boundary — so a hole starts strictly inside a face, after its exterior.
fn validate_csr(
    vertex_count: usize,
    face_indices: &[u32],
    face_offsets: &[u32],
    interior_offsets: &[u32],
) -> Result<(), Error> {
    for &index in face_indices {
        if index as usize >= vertex_count {
            return Err(Error::invalid_geometry(format!(
                "face index {index} is out of range for {vertex_count} vertices"
            )));
        }
    }
    let mut prev = 0u32;
    for (i, &boundary) in face_offsets.iter().enumerate() {
        if boundary <= prev || boundary as usize >= face_indices.len() {
            return Err(Error::invalid_geometry(format!(
                "face_offsets[{i}] = {boundary} must be strictly increasing and within 1..{}",
                face_indices.len()
            )));
        }
        prev = boundary;
    }
    let mut prev = 0u32;
    for (i, &offset) in interior_offsets.iter().enumerate() {
        if offset <= prev
            || offset as usize >= face_indices.len()
            || face_offsets.binary_search(&offset).is_ok()
        {
            return Err(Error::invalid_geometry(format!(
                "interior_offsets[{i}] = {offset} must be strictly increasing, within \
                 1..{}, and not coincide with a face boundary",
                face_indices.len()
            )));
        }
        prev = offset;
    }
    Ok(())
}

/// Pack the flat CSR buffers into width-erased index buffers: `face_indices` at the
/// vertex-count-derived width, the offsets at the `face_indices.len() - 1` width
/// (the largest possible offset, since the internal boundaries carry no trailing
/// total and every offset is `< face_indices.len()`).
fn pack_csr(
    vertex_count: usize,
    face_indices: Vec<u32>,
    face_offsets: Vec<u32>,
    interior_offsets: Vec<u32>,
) -> (IndexBuffer<1>, IndexBuffer<1>, IndexBuffer<1>) {
    let index_width = IndexWidth::for_value(vertex_count.saturating_sub(1) as u32);
    let offset_width = IndexWidth::for_value(face_indices.len().saturating_sub(1) as u32);
    (
        IndexBuffer::with_exact_width(
            index_width,
            Some(face_indices.len()),
            face_indices.into_iter().map(|i| [i]),
        ),
        IndexBuffer::with_exact_width(
            offset_width,
            Some(face_offsets.len()),
            face_offsets.into_iter().map(|o| [o]),
        ),
        IndexBuffer::with_exact_width(
            offset_width,
            Some(interior_offsets.len()),
            interior_offsets.into_iter().map(|o| [o]),
        ),
    )
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

    fn ones(buf: &IndexBuffer<1>) -> Vec<u32> {
        match buf {
            IndexBuffer::U8(v) => v.iter().map(|t| u32::from(t[0])).collect(),
            IndexBuffer::U16(v) => v.iter().map(|t| u32::from(t[0])).collect(),
            IndexBuffer::U32(v) => v.iter().map(|t| t[0]).collect(),
        }
    }

    fn quad(corners: [[f64; 3]; 4]) -> Polygon3D {
        Polygon3D::from_rings(Coordinate::Euclidean, corners, Vec::<Vec<[f64; 3]>>::new())
    }

    #[test]
    fn from_polygons_dedups_shared_edge() {
        // Two unit quads sharing the edge (1,0,0)-(1,1,0): 6 unique vertices.
        let a = quad([[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        let b = quad([[1., 0., 0.], [2., 0., 0.], [2., 1., 0.], [1., 1., 0.]]);
        let m = PolygonMesh3D::from_polygons(Coordinate::Euclidean, [&a, &b]).unwrap();
        assert_eq!(
            m.data.vertices,
            vec![
                [0., 0., 0.],
                [1., 0., 0.],
                [1., 1., 0.],
                [0., 1., 0.],
                [2., 0., 0.],
                [2., 1., 0.],
            ]
        );
        // Rings stored open (the closing duplicate is dropped).
        assert_eq!(ones(&m.data.face_indices), vec![0, 1, 2, 3, 1, 4, 5, 2]);
        // Internal boundary only: face 0 ends / face 1 begins at 4.
        assert_eq!(ones(&m.data.face_offsets), vec![4]);
        assert!(ones(&m.data.interior_offsets).is_empty());
    }

    #[test]
    fn from_polygons_preserves_a_hole() {
        let outer = [[0., 0., 0.], [4., 0., 0.], [4., 4., 0.], [0., 4., 0.]];
        let hole = vec![[1., 1., 0.], [2., 1., 0.], [2., 2., 0.], [1., 2., 0.]];
        let p = Polygon3D::from_rings(Coordinate::Euclidean, outer, vec![hole]);
        let m = PolygonMesh3DData::from_polygons([&p]);
        assert_eq!(m.vertices.len(), 8);
        assert_eq!(ones(&m.face_indices), vec![0, 1, 2, 3, 4, 5, 6, 7]);
        // Single face -> no internal boundaries; the hole starts at index 4.
        assert!(ones(&m.face_offsets).is_empty());
        assert_eq!(ones(&m.interior_offsets), vec![4]);
    }

    #[test]
    fn from_polygons_rejects_frame_mismatch() {
        let a = quad([[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        let err = PolygonMesh3D::from_polygons(Coordinate::Crs(4326), [&a]).unwrap_err();
        assert!(matches!(err, Error::InvalidGeometry(_)));
    }

    #[test]
    fn from_parts_builds_faces() {
        let verts = vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]];
        let faces = vec![vec![0u32, 1, 2], vec![0u32, 2, 3]];
        let m = PolygonMesh3D::from_parts(Coordinate::Euclidean, verts, faces).unwrap();
        assert_eq!(ones(&m.data.face_indices), vec![0, 1, 2, 0, 2, 3]);
        // Two faces -> a single internal boundary at 3.
        assert_eq!(ones(&m.data.face_offsets), vec![3]);
        assert!(ones(&m.data.interior_offsets).is_empty());
    }

    #[test]
    fn from_parts_rejects_out_of_range_index() {
        let verts = vec![[0., 0., 0.]];
        assert!(PolygonMesh3DData::from_parts(verts, vec![vec![0u32, 1, 2]]).is_err());
    }

    // An empty face contributes no ring, so it is skipped rather than producing a
    // duplicate (non-increasing) boundary.
    #[test]
    fn from_polygons_skips_empty_face() {
        let a = quad([[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        let empty = Polygon3D::from_rings(
            Coordinate::Euclidean,
            Vec::<[f64; 3]>::new(),
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let b = quad([[1., 0., 0.], [2., 0., 0.], [2., 1., 0.], [1., 1., 0.]]);
        let m = PolygonMesh3DData::from_polygons([&a, &empty, &b]);
        assert_eq!(m.vertices.len(), 6);
        // Two real faces -> one internal boundary, no duplicate from the empty face.
        assert_eq!(ones(&m.face_offsets), vec![4]);
    }

    #[test]
    fn from_parts_skips_empty_face() {
        let verts = vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]];
        let faces: Vec<Vec<u32>> = vec![vec![0, 1, 2], vec![], vec![0, 2, 3]];
        let m = PolygonMesh3D::from_parts(Coordinate::Euclidean, verts, faces).unwrap();
        assert_eq!(ones(&m.data.face_indices), vec![0, 1, 2, 0, 2, 3]);
        assert_eq!(ones(&m.data.face_offsets), vec![3]);
    }

    #[test]
    fn from_parts_with_elevation_splits_z() {
        let verts = vec![[0., 0., 10.], [1., 0., 11.], [0., 1., 12.]];
        let m = PolygonMesh2D::from_parts_with_elevation(
            Coordinate::Euclidean,
            verts,
            vec![vec![0u32, 1, 2]],
        )
        .unwrap();
        assert_eq!(m.vertices, vec![[0., 0.], [1., 0.], [0., 1.]]);
        assert_eq!(m.z.as_deref(), Some(&[10., 11., 12.][..]));
    }

    #[test]
    fn from_raw_parts_validates_offsets() {
        let verts = vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.]];
        // Good: a single triangular face has no internal boundaries.
        assert!(
            PolygonMesh3DData::from_raw_parts(verts.clone(), vec![0, 1, 2], vec![], vec![]).is_ok()
        );
        // Good: two faces with one internal boundary at 3.
        assert!(PolygonMesh3DData::from_raw_parts(
            verts.clone(),
            vec![0, 1, 2, 0, 1, 2],
            vec![3],
            vec![],
        )
        .is_ok());
        // A leading 0 is no longer allowed (it would mean an empty first face).
        assert!(
            PolygonMesh3DData::from_raw_parts(verts.clone(), vec![0, 1, 2], vec![0], vec![])
                .is_err()
        );
        // Equal boundaries (an empty face) are rejected.
        assert!(PolygonMesh3DData::from_raw_parts(
            verts.clone(),
            vec![0, 1, 2, 0, 1, 2],
            vec![3, 3],
            vec![],
        )
        .is_err());
        // Index out of range.
        assert!(PolygonMesh3DData::from_raw_parts(verts, vec![0, 1, 9], vec![], vec![]).is_err());
    }

    #[test]
    fn from_raw_parts_rejects_interior_on_face_boundary() {
        // Two faces, boundary at 3; an interior offset of 3 coincides with it.
        let verts = vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.]];
        assert!(
            PolygonMesh3DData::from_raw_parts(verts, vec![0, 1, 2, 0, 1, 2], vec![3], vec![3],)
                .is_err()
        );
    }
}
