use super::{PolygonMesh2D, PolygonMesh3D};
use crate::index::IndexBuffer;
use crate::ops::triangulation::{triangulate_2d, triangulate_3d, Cache};
use crate::ops::{Aabb, BoundingBox, Triangulate, UnsupportedOperation};
use crate::triangular_mesh::{TriangularMesh2D, TriangularMesh3D};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

impl BoundingBox for PolygonMesh2D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        Aabb::from_points_2d(self.vertices.iter().copied()).ok_or(UnsupportedOperation {
            geometry: "PolygonMesh2D",
            operation: "bounding_box",
        })
    }
}

impl BoundingBox for PolygonMesh3D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        Aabb::from_points_3d(self.data.vertices.iter().copied()).ok_or(UnsupportedOperation {
            geometry: "PolygonMesh3D",
            operation: "bounding_box",
        })
    }
}

impl Triangulate for PolygonMesh2D {
    fn triangulate(&self, cache: &mut Cache) -> Result<Geometry, UnsupportedOperation> {
        let Cache { earcut, buffers } = cache;
        decode_into(&self.face_indices, &mut buffers.face_indices);
        decode_into(&self.face_offsets, &mut buffers.face_offsets);
        decode_into(&self.interior_offsets, &mut buffers.interior_offsets);

        buffers.tris.clear();
        buffers.tris.reserve(3 * buffers.face_indices.len());

        let n = buffers.face_indices.len();
        if n != 0 {
            let n_faces = buffers.face_offsets.len() + 1;
            let mut start = 0usize;
            for fi in 0..n_faces {
                let end = buffers.face_offsets.get(fi).map_or(n, |&o| o as usize);
                face_holes(&buffers.interior_offsets, start, end, &mut buffers.holes);
                let face = &buffers.face_indices[start..end];
                buffers.verts2.clear();
                // SAFETY: face indices are validated `< vertices.len()` at construction.
                buffers.verts2.extend(
                    face.iter()
                        .map(|&gi| unsafe { *self.vertices.get_unchecked(gi as usize) }),
                );
                triangulate_2d(earcut, &buffers.verts2, &buffers.holes, &mut buffers.out);
                // Map face-local corner indices back to the shared vertex pool.
                // SAFETY: each earcut index is `< face.len()`.
                buffers.tris.extend(
                    buffers
                        .out
                        .iter()
                        .map(|&l| unsafe { *face.get_unchecked(l as usize) }),
                );
                start = end;
            }
        }

        // `tris` index the existing pool (each `< vertices.len()`) in triples.
        let mesh = match &self.z {
            Some(z) => {
                let verts3: Vec<[f64; 3]> = self
                    .vertices
                    .iter()
                    .zip(z.iter())
                    .map(|(&[x, y], &zz)| [x, y, zz])
                    .collect();
                // SAFETY: every index is `< verts3.len()`; count is a multiple of 3.
                unsafe {
                    TriangularMesh2D::from_parts_with_elevation_unchecked(
                        self.coordinate.clone(),
                        verts3,
                        buffers.tris.len() / 3,
                        buffers.tris.iter().copied(),
                    )
                }
            }
            // SAFETY: every index is `< vertices.len()`; count is a multiple of 3.
            None => unsafe {
                TriangularMesh2D::from_parts_unchecked(
                    self.coordinate.clone(),
                    self.vertices.clone(),
                    buffers.tris.len() / 3,
                    buffers.tris.iter().copied(),
                )
            },
        };
        Ok(Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(
            Box::new(mesh),
        )))
    }
}

impl Triangulate for PolygonMesh3D {
    fn triangulate(&self, cache: &mut Cache) -> Result<Geometry, UnsupportedOperation> {
        let data = &self.data;
        let Cache { earcut, buffers } = cache;
        decode_into(&data.face_indices, &mut buffers.face_indices);
        decode_into(&data.face_offsets, &mut buffers.face_offsets);
        decode_into(&data.interior_offsets, &mut buffers.interior_offsets);

        buffers.tris.clear();
        buffers.tris.reserve(3 * buffers.face_indices.len());

        let n = buffers.face_indices.len();
        if n != 0 {
            let n_faces = buffers.face_offsets.len() + 1;
            let mut start = 0usize;
            for fi in 0..n_faces {
                let end = buffers.face_offsets.get(fi).map_or(n, |&o| o as usize);
                face_holes(&buffers.interior_offsets, start, end, &mut buffers.holes);
                let face = &buffers.face_indices[start..end];
                let num_outer = buffers.holes.first().map_or(face.len(), |&h| h as usize);
                buffers.verts3.clear();
                // SAFETY: face indices are validated `< vertices.len()` at construction.
                buffers.verts3.extend(
                    face.iter()
                        .map(|&gi| unsafe { *data.vertices.get_unchecked(gi as usize) }),
                );
                if triangulate_3d(
                    earcut,
                    &buffers.verts3,
                    num_outer,
                    &buffers.holes,
                    &mut buffers.out,
                ) {
                    // SAFETY: each earcut index is `< face.len()`.
                    buffers.tris.extend(
                        buffers
                            .out
                            .iter()
                            .map(|&l| unsafe { *face.get_unchecked(l as usize) }),
                    );
                }
                start = end;
            }
        }

        // SAFETY: `tris` index `data.vertices` (`< len`); count is a multiple of 3.
        let mesh = unsafe {
            TriangularMesh3D::from_parts_unchecked(
                self.coordinate.clone(),
                data.vertices.clone(),
                buffers.tris.len() / 3,
                buffers.tris.iter().copied(),
            )
        };
        Ok(Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(
            Box::new(mesh),
        )))
    }
}

/// Decode a `<1>`-stride index buffer into the reused `out` buffer as flat
/// `u32`, widening each entry.
fn decode_into(buf: &IndexBuffer<1>, out: &mut Vec<u32>) {
    out.clear();
    match buf {
        IndexBuffer::U8(v) => out.extend(v.iter().map(|&[i]| i as u32)),
        IndexBuffer::U16(v) => out.extend(v.iter().map(|&[i]| i as u32)),
        IndexBuffer::U32(v) => out.extend(v.iter().map(|&[i]| i)),
    }
}

/// Collect, into the reused `out` buffer, the face-local start offsets of the
/// hole rings that fall strictly inside the face spanning `[start, end)` of
/// `face_indices`.
fn face_holes(interior_offsets: &[u32], start: usize, end: usize, out: &mut Vec<u32>) {
    out.clear();
    out.extend(
        interior_offsets
            .iter()
            .filter(|&&o| (o as usize) > start && (o as usize) < end)
            .map(|&o| o - start as u32),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::Coordinate;

    #[test]
    fn polygon_mesh2d_box_spans_vertex_pool() {
        let m = PolygonMesh2D::from_parts(
            Coordinate::Euclidean,
            vec![[0.0, 0.0], [3.0, 0.0], [3.0, 2.0]],
            vec![vec![0u32, 1, 2]],
        )
        .unwrap();
        assert_eq!(
            m.bounding_box().unwrap(),
            Aabb::D2 {
                min: [0.0, 0.0],
                max: [3.0, 2.0]
            }
        );
    }

    #[test]
    fn polygon_mesh3d_box_spans_vertex_pool() {
        let m = PolygonMesh3D::from_parts(
            Coordinate::Euclidean,
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 1.0], [3.0, 2.0, -1.0]],
            vec![vec![0u32, 1, 2]],
        )
        .unwrap();
        assert_eq!(
            m.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, 0.0, -1.0],
                max: [3.0, 2.0, 1.0]
            }
        );
    }

    #[test]
    fn polygon_mesh2d_triangulates_every_face() {
        // Two quads sharing edge 1-2; each quad -> 2 triangles.
        let vertices = vec![
            [0.0, 0.0],
            [2.0, 0.0],
            [2.0, 2.0],
            [0.0, 2.0],
            [4.0, 0.0],
            [4.0, 2.0],
        ];
        let mesh = PolygonMesh2D::from_parts(
            Coordinate::Euclidean,
            vertices,
            vec![vec![0u32, 1, 2, 3], vec![1, 4, 5, 2]],
        )
        .unwrap();
        let g = mesh.triangulate(&mut Cache::new()).unwrap();
        let tm = match &g {
            Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(m)) => m,
            _ => panic!("expected a 2D triangular mesh"),
        };
        assert_eq!(tm.num_triangles(), 4);
        assert_eq!(g.bounding_box().unwrap(), mesh.bounding_box().unwrap());
    }

    #[test]
    fn polygon_mesh3d_triangulates_quad_in_plane() {
        let mesh = PolygonMesh3D::from_parts(
            Coordinate::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
            ],
            vec![vec![0u32, 1, 2, 3]],
        )
        .unwrap();
        let g = mesh.triangulate(&mut Cache::new()).unwrap();
        let tm = match &g {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => m,
            _ => panic!("expected a 3D triangular mesh"),
        };
        assert_eq!(tm.num_triangles(), 2);
        assert_eq!(g.bounding_box().unwrap(), mesh.bounding_box().unwrap());
    }

    #[test]
    fn polygon_mesh3d_triangulates_face_with_hole() {
        // One face: a square exterior with a square hole, given via raw CSR
        // (interior_offsets marks where the hole ring starts in face_indices).
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 4.0, 0.0],
            [0.0, 4.0, 0.0],
            [1.0, 1.0, 0.0],
            [3.0, 1.0, 0.0],
            [3.0, 3.0, 0.0],
            [1.0, 3.0, 0.0],
        ];
        let mesh = PolygonMesh3D::from_raw_parts(
            Coordinate::Euclidean,
            vertices,
            vec![0, 1, 2, 3, 4, 5, 6, 7], // face_indices: exterior then hole
            vec![],                       // one face
            vec![4],                      // hole ring starts at index 4
        )
        .unwrap();
        let g = mesh.triangulate(&mut Cache::new()).unwrap();
        let tm = match &g {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => m,
            _ => panic!("expected a 3D triangular mesh"),
        };
        // A square ring with a square hole tessellates into 8 triangles.
        assert_eq!(tm.num_triangles(), 8);
    }
}
