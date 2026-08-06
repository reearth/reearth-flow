//! CSR face-topology decoding shared by validation and flattening.
//!
//! A polygon mesh stores every face's rings concatenated in a single flat index
//! buffer (see [`super`]). These helpers walk that layout back into per-face
//! rings, either as raw vertex indices or as gathered coordinates, and rebuild
//! each face as a standalone [`Polygon2D`] / [`Polygon3D`].
//!
//! [`FaceVisit`] is the public, read-only view of that walk: a caller outside
//! this crate — a CityGML sink, say — gets each face's rings as coordinates
//! together with the half-open range of corner positions each ring occupies in
//! the mesh's corner buffer. The range is what makes per-corner appearance data
//! usable: a theme's UV array is parallel to the corner buffer, so a ring's UV
//! is exactly that slice of it. No mutable access to the CSR layout is exposed,
//! and `csr_buffers()` stays crate-private.

use std::ops::Range;

use crate::coordinate::CoordinateFrame;
use crate::index::IndexBuffer;
use crate::polygon::{Polygon2D, Polygon3D};

use super::{PolygonMesh2D, PolygonMesh3D};

/// One ring of one face, gathered from the shared vertex pool.
///
/// `corners` is the ring's half-open `[start, end)` range of positions in the
/// mesh's corner buffer — the same indexing a theme's per-corner UV array uses
/// — so `corners.len() == coords.len()`.
#[derive(Debug, Clone)]
pub struct FaceRing<'a> {
    pub coords: &'a [[f64; 3]],
    pub corners: Range<usize>,
    pub is_exterior: bool,
}

/// One face of a mesh: its index in face order, its exterior ring, and its hole
/// rings.
#[derive(Debug)]
pub struct FaceVisit<'a> {
    pub face: usize,
    pub exterior: FaceRing<'a>,
    pub interiors: &'a [FaceRing<'a>],
}

/// Decode the CSR face topology and invoke `f` once per face ring (each face's
/// exterior ring, then its hole rings), passing the ring's vertex indices, the
/// ring's `[start, end)` range in the corner buffer, and whether it is an
/// exterior ring (vs. a hole).
///
/// The flat index buffer is streamed rather than collected, and each ring is
/// materialized into a single buffer reused across rings, so nothing allocated
/// here scales with the corner count. Only the small per-face offset lists (one
/// entry per face / per hole) are collected.
pub(crate) fn for_each_ring(
    face_indices: &IndexBuffer<1>,
    face_offsets: &IndexBuffer<1>,
    interior_offsets: &IndexBuffer<1>,
    mut f: impl FnMut(&[u32], Range<usize>, bool),
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
            f(&ring, ring_start..h, is_exterior);
            ring_start = h;
            is_exterior = false;
            hole += 1;
        }
        ring.clear();
        ring.extend(indices.by_ref().take(end - ring_start));
        f(&ring, ring_start..end, is_exterior);
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
        |ring, _corners, is_exterior| {
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

/// Hand one face's gathered rings to `f` as a [`FaceVisit`].
///
/// `coords` holds the face's rings concatenated and `rings` says where each one
/// sits in it, exterior first; both buffers are reused across faces, so the only
/// thing allocated per face is the small ring-view list (one entry per ring).
fn visit_face(
    face: usize,
    coords: &[[f64; 3]],
    rings: &[(Range<usize>, Range<usize>, bool)],
    f: &mut impl FnMut(FaceVisit<'_>),
) {
    let views: Vec<FaceRing<'_>> = rings
        .iter()
        .map(|(span, corners, is_exterior)| FaceRing {
            coords: &coords[span.clone()],
            corners: corners.clone(),
            is_exterior: *is_exterior,
        })
        .collect();
    // `for_each_ring` emits a face's exterior ring before its holes, and never
    // emits a face with no rings at all, so the split always succeeds.
    let Some((exterior, interiors)) = views.split_first() else {
        return;
    };
    f(FaceVisit {
        face,
        exterior: exterior.clone(),
        interiors,
    });
}

impl super::PolygonMesh3DData {
    /// Invoke `f` once per face, in face order, with that face's exterior ring
    /// and hole rings as coordinates plus each ring's `[start, end)` range in
    /// the mesh's corner buffer.
    ///
    /// Read-only: nothing here hands out the CSR buffers or lets a caller edit
    /// the mesh. Rings are yielded exactly as stored — a polygon-sourced mesh
    /// stores them closed, an index-sourced one stores them as given — so a
    /// consumer that needs closed rings closes them itself.
    pub fn for_each_face(&self, mut f: impl FnMut(FaceVisit<'_>)) {
        let (face_indices, face_offsets, interior_offsets) = self.csr_buffers();
        let vertices = self.vertices();
        let mut coords: Vec<[f64; 3]> = Vec::new();
        let mut rings: Vec<(Range<usize>, Range<usize>, bool)> = Vec::new();
        let mut face = 0usize;
        for_each_ring(
            face_indices,
            face_offsets,
            interior_offsets,
            |ring, corners, is_exterior| {
                if is_exterior && !rings.is_empty() {
                    visit_face(face, &coords, &rings, &mut f);
                    face += 1;
                    coords.clear();
                    rings.clear();
                }
                let start = coords.len();
                coords.extend(ring.iter().map(|&i| vertices[i as usize]));
                rings.push((start..coords.len(), corners, is_exterior));
            },
        );
        if !rings.is_empty() {
            visit_face(face, &coords, &rings, &mut f);
        }
    }
}

impl crate::triangular_mesh::TriangularMesh3DData {
    /// Invoke `f` once per triangle, in triangle order. A triangle has no hole
    /// rings and occupies corners `3i..3i + 3`, which is the same indexing a
    /// theme's per-corner UV array uses.
    ///
    /// Triangles are stored open — three corners, the first not repeated — so a
    /// consumer that needs a closed ring closes it itself.
    pub fn for_each_face(&self, mut f: impl FnMut(FaceVisit<'_>)) {
        let vertices = self.vertices();
        for (face, triangle) in self.triangles().enumerate() {
            let coords = triangle.map(|i| vertices[i as usize]);
            f(FaceVisit {
                face,
                exterior: FaceRing {
                    coords: &coords,
                    corners: 3 * face..3 * face + 3,
                    is_exterior: true,
                },
                interiors: &[],
            });
        }
    }
}

impl PolygonMesh2D {
    /// Invoke `f` once per face with that face rebuilt as a standalone bare
    /// [`Polygon2D`] in the mesh's frame. Faces are streamed rather than
    /// collected. Appearance is not carried onto them; the mesh's elevation is.
    pub(crate) fn for_each_face_polygon(&self, mut f: impl FnMut(Polygon2D)) {
        let (face_indices, face_offsets, interior_offsets) = self.csr_buffers();
        let frame = self.frame();
        let elevation = self.elevation();
        for_each_face_coords(
            self.vertices(),
            face_indices,
            face_offsets,
            interior_offsets,
            |rings| f(polygon_2d_from_rings(frame, rings, elevation)),
        );
    }
}

impl PolygonMesh3D {
    /// Invoke `f` once per face with that face rebuilt as a standalone bare
    /// [`Polygon3D`] in the mesh's frame. Faces are streamed rather than
    /// collected. Appearance is not carried onto them.
    pub(crate) fn for_each_face_polygon(&self, f: impl FnMut(Polygon3D)) {
        self.data().for_each_face_polygon(self.frame(), f);
    }
}

impl super::PolygonMesh3DData {
    /// As [`PolygonMesh3D::for_each_face_polygon`], but with the frame supplied
    /// by the caller — the form a [`Solid`](crate::solid::Solid) shell needs,
    /// since a shell holds mesh data and takes its frame from the solid.
    pub(crate) fn for_each_face_polygon(
        &self,
        frame: &CoordinateFrame,
        mut f: impl FnMut(Polygon3D),
    ) {
        let (face_indices, face_offsets, interior_offsets) = self.csr_buffers();
        for_each_face_coords(
            self.vertices(),
            face_indices,
            face_offsets,
            interior_offsets,
            |rings| f(polygon_3d_from_rings(frame, rings)),
        );
    }
}

/// Build a [`Polygon2D`] from a face's rings (exterior first, then holes), at the
/// host mesh's `elevation`.
fn polygon_2d_from_rings(
    frame: &CoordinateFrame,
    rings: &[Vec<[f64; 2]>],
    elevation: Option<f64>,
) -> Polygon2D {
    let exterior = rings
        .first()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .copied();
    let interiors = rings.iter().skip(1).map(|hole| hole.iter().copied());
    match elevation {
        None => Polygon2D::from_rings(frame.clone(), exterior, interiors),
        Some(elevation) => {
            Polygon2D::from_rings_at_elevation(frame.clone(), exterior, interiors, elevation)
        }
    }
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

    // The public read-only visitor.

    /// One visited face, flattened into owned values so the assertions do not
    /// have to live inside the callback.
    #[derive(Debug, PartialEq)]
    struct Visited {
        face: usize,
        rings: Vec<(Vec<[f64; 3]>, std::ops::Range<usize>, bool)>,
    }

    fn visit(mesh: &PolygonMesh3D) -> Vec<Visited> {
        let mut out = Vec::new();
        mesh.for_each_face(|v| {
            let mut rings = vec![(
                v.exterior.coords.to_vec(),
                v.exterior.corners.clone(),
                v.exterior.is_exterior,
            )];
            rings.extend(
                v.interiors
                    .iter()
                    .map(|r| (r.coords.to_vec(), r.corners.clone(), r.is_exterior)),
            );
            out.push(Visited {
                face: v.face,
                rings,
            });
        });
        out
    }

    /// Two triangles sharing edge 1-2, stored as index faces (so the rings are
    /// open, three corners each).
    fn two_triangles() -> PolygonMesh3D {
        PolygonMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![vec![0u32, 1, 2], vec![1, 3, 2]],
        )
        .unwrap()
    }

    /// One square face with one square hole, given as raw CSR.
    fn square_with_hole() -> PolygonMesh3D {
        PolygonMesh3D::from_raw_parts(
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
        .unwrap()
    }

    /// Faces come out in face order, once each, numbered from zero.
    #[test]
    fn for_each_face_visits_every_face_in_order() {
        let mesh = two_triangles();
        let visited = visit(&mesh);

        assert_eq!(visited.len(), mesh.num_faces());
        assert_eq!(
            visited.iter().map(|v| v.face).collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert_eq!(visited[0].rings[0].0[0], [0.0, 0.0, 0.0]);
        assert_eq!(visited[1].rings[0].0[1], [1.0, 1.0, 0.0]);
    }

    /// A face with a hole yields exactly one exterior ring, first, and one
    /// interior ring after it.
    #[test]
    fn for_each_face_separates_the_exterior_ring_from_the_hole() {
        let visited = visit(&square_with_hole());

        assert_eq!(visited.len(), 1);
        assert_eq!(visited[0].rings.len(), 2);
        assert!(visited[0].rings[0].2, "the first ring is the exterior");
        assert!(!visited[0].rings[1].2, "the second ring is the hole");
        assert_eq!(visited[0].rings[0].0.len(), 4);
        assert_eq!(
            visited[0].rings[1].0,
            vec![
                [1.0, 1.0, 0.0],
                [3.0, 1.0, 0.0],
                [3.0, 3.0, 0.0],
                [1.0, 3.0, 0.0]
            ]
        );
    }

    /// Corner ranges are what a per-corner UV array is sliced by, so they must
    /// tile the whole corner buffer: contiguous, in order, non-overlapping, and
    /// as long as the ring they belong to.
    #[test]
    fn corner_ranges_tile_the_corner_buffer_exactly() {
        for mesh in [two_triangles(), square_with_hole()] {
            let visited = visit(&mesh);
            let mut next = 0usize;
            let mut total = 0usize;
            for face in &visited {
                for (coords, corners, _) in &face.rings {
                    assert_eq!(corners.start, next, "ranges are contiguous and in order");
                    assert_eq!(corners.len(), coords.len(), "one corner per coordinate");
                    next = corners.end;
                    total += corners.len();
                }
            }
            assert_eq!(total, next, "no gaps and no overlaps");
        }
    }

    /// A triangle mesh has no holes and lands on the fixed `3i..3i+3` corner
    /// ranges its UV arrays are laid out by.
    #[test]
    fn a_triangular_mesh_yields_three_corners_per_face() {
        let mesh = crate::triangular_mesh::TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            [0u32, 1, 2, 1, 3, 2],
        )
        .unwrap();

        let mut seen = Vec::new();
        mesh.for_each_face(|v| {
            assert!(v.interiors.is_empty(), "a triangle has no hole rings");
            assert!(v.exterior.is_exterior);
            seen.push((v.face, v.exterior.coords.to_vec(), v.exterior.corners));
        });

        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0].2, 0..3);
        assert_eq!(seen[1].2, 3..6);
        assert_eq!(
            seen[1].1,
            vec![[1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]]
        );
    }

    /// A solid's exterior and interior shells are visited through the same
    /// entry point but as separate meshes, so the two can never be confused.
    #[test]
    fn a_solids_shells_are_visited_separately() {
        use crate::solid::{Shell, Solid};

        let outer = PolygonMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 4.0, 0.0]],
            vec![vec![0u32, 1, 2]],
        )
        .unwrap();
        let inner = PolygonMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[1.0, 1.0, 1.0], [2.0, 1.0, 1.0], [1.0, 2.0, 1.0]],
            vec![vec![0u32, 1, 2]],
        )
        .unwrap();
        let solid = Solid::new(
            CoordinateFrame::Euclidean,
            Shell::PolygonMesh(outer.into_data()),
            vec![Shell::PolygonMesh(inner.into_data())],
        );

        let corners = |shell: &Shell| {
            let mut out = Vec::new();
            shell.for_each_face(|v| out.push(v.exterior.coords[0]));
            out
        };

        assert_eq!(corners(solid.exterior()), vec![[0.0, 0.0, 0.0]]);
        assert_eq!(solid.interiors().len(), 1);
        assert_eq!(corners(&solid.interiors()[0]), vec![[1.0, 1.0, 1.0]]);
    }

    #[test]
    fn for_each_face_of_an_empty_mesh_visits_nothing() {
        let mesh =
            PolygonMesh3D::from_parts(CoordinateFrame::Euclidean, vec![], Vec::<Vec<u32>>::new())
                .unwrap();
        assert!(visit(&mesh).is_empty());
    }
}
