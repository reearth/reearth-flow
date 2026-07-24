//! CSR face-topology decoding shared by validation and flattening.
//!
//! A polygon mesh stores every face's rings concatenated in a single flat index
//! buffer (see [`super`]). These helpers walk that layout back into per-face
//! rings, either as raw vertex indices or as gathered coordinates, and rebuild
//! each face as a standalone [`Polygon2D`] / [`Polygon3D`].

use crate::coordinate::CoordinateFrame;
use crate::index::IndexBuffer;
use crate::polygon::{Polygon2D, Polygon3D};

use super::{PolygonMesh2D, PolygonMesh3D};

/// Decode the CSR face topology and invoke `f` once per face ring (each face's
/// exterior ring, then its hole rings), passing the ring's vertex indices and
/// whether it is an exterior ring (vs. a hole).
///
/// The flat index buffer is streamed rather than collected, and each ring is
/// materialized into a single buffer reused across rings, so nothing allocated
/// here scales with the corner count. Only the small per-face offset lists (one
/// entry per face / per hole) are collected.
pub(crate) fn for_each_ring(
    face_indices: &IndexBuffer<1>,
    face_offsets: &IndexBuffer<1>,
    interior_offsets: &IndexBuffer<1>,
    mut f: impl FnMut(&[u32], bool),
) {
    let n = face_indices.len();
    if n == 0 {
        return;
    }
    let face_ends: Vec<usize> = face_offsets.iter_u32().map(|[i]| i as usize).collect();
    let holes: Vec<usize> = interior_offsets.iter_u32().map(|[i]| i as usize).collect();
    let n_faces = face_ends.len() + 1;
    let mut indices = face_indices.iter_u32().map(|[i]| i);
    let mut ring: Vec<u32> = Vec::new();
    let mut start = 0usize;
    // `interior_offsets` are strictly increasing, and faces are visited in order,
    // so a single moving cursor walks the holes once across the whole mesh.
    let mut hole = 0usize;
    for face in 0..n_faces {
        let end = face_ends.get(face).copied().unwrap_or(n);
        // Hole rings of this face begin at the interior offsets inside (start, end);
        // the exterior ring runs up to the first hole (or the face end).
        let mut ring_start = start;
        let mut is_exterior = true;
        while hole < holes.len() && holes[hole] <= start {
            hole += 1;
        }
        while hole < holes.len() && holes[hole] < end {
            let h = holes[hole];
            ring.clear();
            ring.extend(indices.by_ref().take(h - ring_start));
            f(&ring, is_exterior);
            ring_start = h;
            is_exterior = false;
            hole += 1;
        }
        ring.clear();
        ring.extend(indices.by_ref().take(end - ring_start));
        f(&ring, is_exterior);
        start = end;
    }
}

/// The `[f64; N]` coordinates of one ring, gathered from the shared vertex pool.
pub(crate) fn ring_coords<const N: usize>(vertices: &[[f64; N]], ring: &[u32]) -> Vec<[f64; N]> {
    ring.iter().map(|&i| vertices[i as usize]).collect()
}

/// Decode the CSR face topology and invoke `f` once per face with that face's
/// ring coordinates, exterior first, then the face's holes.
pub(crate) fn for_each_face_coords<const N: usize>(
    vertices: &[[f64; N]],
    face_indices: &IndexBuffer<1>,
    face_offsets: &IndexBuffer<1>,
    interior_offsets: &IndexBuffer<1>,
    mut f: impl FnMut(&[Vec<[f64; N]>]),
) {
    let mut face: Vec<Vec<[f64; N]>> = Vec::new();
    for_each_ring(
        face_indices,
        face_offsets,
        interior_offsets,
        |ring, is_exterior| {
            if is_exterior && !face.is_empty() {
                f(&face);
                face.clear();
            }
            face.push(ring_coords(vertices, ring));
        },
    );
    if !face.is_empty() {
        f(&face);
    }
}

impl PolygonMesh2D {
    /// Invoke `f` once per face with that face rebuilt as a standalone bare
    /// [`Polygon2D`] in the mesh's frame. Faces are streamed rather than
    /// collected. Per-vertex elevation and appearance are not carried onto them.
    pub(crate) fn for_each_face_polygon(&self, mut f: impl FnMut(Polygon2D)) {
        let (face_indices, face_offsets, interior_offsets) = self.csr_buffers();
        let frame = self.frame();
        for_each_face_coords(
            self.vertices(),
            face_indices,
            face_offsets,
            interior_offsets,
            |rings| f(polygon_2d_from_rings(frame, rings)),
        );
    }
}

impl PolygonMesh3D {
    /// Invoke `f` once per face with that face rebuilt as a standalone bare
    /// [`Polygon3D`] in the mesh's frame. Faces are streamed rather than
    /// collected. Appearance is not carried onto them.
    pub(crate) fn for_each_face_polygon(&self, mut f: impl FnMut(Polygon3D)) {
        let data = self.data();
        let (face_indices, face_offsets, interior_offsets) = data.csr_buffers();
        let frame = self.frame();
        for_each_face_coords(
            data.vertices(),
            face_indices,
            face_offsets,
            interior_offsets,
            |rings| f(polygon_3d_from_rings(frame, rings)),
        );
    }
}

/// Build a [`Polygon2D`] from a face's rings (exterior first, then holes).
fn polygon_2d_from_rings(frame: &CoordinateFrame, rings: &[Vec<[f64; 2]>]) -> Polygon2D {
    let exterior = rings
        .first()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .copied();
    let interiors = rings.iter().skip(1).map(|hole| hole.iter().copied());
    Polygon2D::from_rings(frame.clone(), exterior, interiors)
}

/// Build a [`Polygon3D`] from a face's rings (exterior first, then holes).
fn polygon_3d_from_rings(frame: &CoordinateFrame, rings: &[Vec<[f64; 3]>]) -> Polygon3D {
    let exterior = rings
        .first()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .copied();
    let interiors = rings.iter().skip(1).map(|hole| hole.iter().copied());
    Polygon3D::from_rings(frame.clone(), exterior, interiors)
}

#[cfg(test)]
mod tests {
    use crate::coordinate::CoordinateFrame;
    use crate::polygon::{Polygon2D, Polygon3D};
    use crate::polygon_mesh::{PolygonMesh2D, PolygonMesh3D};

    fn faces_2d(mesh: &PolygonMesh2D) -> Vec<Polygon2D> {
        let mut out = Vec::new();
        mesh.for_each_face_polygon(|p| out.push(p));
        out
    }

    fn faces_3d(mesh: &PolygonMesh3D) -> Vec<Polygon3D> {
        let mut out = Vec::new();
        mesh.for_each_face_polygon(|p| out.push(p));
        out
    }

    #[test]
    fn faces_as_polygons_3d_recovers_each_face() {
        // Two triangles sharing edge 1-2.
        let mesh = PolygonMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![vec![0u32, 1, 2], vec![1, 3, 2]],
        )
        .unwrap();
        let polygons = faces_3d(&mesh);
        assert_eq!(polygons.len(), 2);
        assert_eq!(polygons[0].exterior().len(), 3);
        assert_eq!(polygons[0].exterior()[0], [0.0, 0.0, 0.0]);
        assert_eq!(polygons[1].exterior()[1], [1.0, 1.0, 0.0]);
    }

    #[test]
    fn faces_as_polygons_3d_preserves_a_hole() {
        // One square face with one square hole, given as raw CSR.
        let mesh = PolygonMesh3D::from_raw_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [4.0, 4.0, 0.0],
                [0.0, 4.0, 0.0],
                [1.0, 1.0, 0.0],
                [3.0, 1.0, 0.0],
                [3.0, 3.0, 0.0],
                [1.0, 3.0, 0.0],
            ],
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![],
            vec![4],
        )
        .unwrap();
        let polygons = faces_3d(&mesh);
        assert_eq!(polygons.len(), 1);
        assert_eq!(polygons[0].exterior().len(), 4);
        assert_eq!(polygons[0].interiors().count(), 1);
    }

    #[test]
    fn faces_as_polygons_2d_recovers_each_face() {
        let mesh = PolygonMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
            vec![vec![0u32, 1, 2, 3]],
        )
        .unwrap();
        let polygons = faces_2d(&mesh);
        assert_eq!(polygons.len(), 1);
        assert_eq!(polygons[0].exterior().len(), 4);
    }

    #[test]
    fn faces_as_polygons_of_empty_mesh_is_empty() {
        let mesh =
            PolygonMesh3D::from_parts(CoordinateFrame::Euclidean, vec![], Vec::<Vec<u32>>::new())
                .unwrap();
        assert!(faces_3d(&mesh).is_empty());
    }
}
