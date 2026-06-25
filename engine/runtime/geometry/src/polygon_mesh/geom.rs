use earcut::Earcut;

use super::{PolygonMesh2D, PolygonMesh3D};
use crate::index::IndexBuffer;
use crate::ops::{Aabb, BoundingBox, Triangulate, UnsupportedOperation};
use crate::triangular_mesh::{TriangularMesh2D, TriangularMesh3D};
use crate::triangulation::{triangulate_2d, triangulate_3d};
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
    fn triangulate(&self) -> Result<Geometry, UnsupportedOperation> {
        let face_indices = decode(&self.face_indices);
        let face_offsets = decode(&self.face_offsets);
        let interior_offsets = decode(&self.interior_offsets);

        let mut earcut = Earcut::new();
        let mut verts = Vec::new();
        let mut local = Vec::new();
        let mut tris: Vec<u32> = Vec::new();

        for_each_face(
            &face_indices,
            &face_offsets,
            &interior_offsets,
            |face, holes| {
                verts.clear();
                verts.extend(face.iter().map(|&gi| self.vertices[gi as usize]));
                triangulate_2d(&mut earcut, &verts, holes, &mut local);
                // Map face-local corner indices back to the shared vertex pool.
                tris.extend(local.iter().map(|&l| face[l as usize]));
            },
        );

        // `tris` index the existing pool (each `< vertices.len()`) in triples, so
        // assembling the mesh cannot fail.
        let mesh = match &self.z {
            Some(z) => {
                let verts3: Vec<[f64; 3]> = self
                    .vertices
                    .iter()
                    .zip(z.iter())
                    .map(|(&[x, y], &zz)| [x, y, zz])
                    .collect();
                TriangularMesh2D::from_parts_with_elevation(self.coordinate.clone(), verts3, tris)
            }
            None => {
                TriangularMesh2D::from_parts(self.coordinate.clone(), self.vertices.clone(), tris)
            }
        }
        .expect("face indices are in range and triangles come in triples");
        Ok(Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(
            Box::new(mesh),
        )))
    }
}

impl Triangulate for PolygonMesh3D {
    fn triangulate(&self) -> Result<Geometry, UnsupportedOperation> {
        let data = &self.data;
        let face_indices = decode(&data.face_indices);
        let face_offsets = decode(&data.face_offsets);
        let interior_offsets = decode(&data.interior_offsets);

        let mut earcut = Earcut::new();
        let mut buf2d = Vec::new();
        let mut verts = Vec::new();
        let mut local = Vec::new();
        let mut tris: Vec<u32> = Vec::new();

        for_each_face(
            &face_indices,
            &face_offsets,
            &interior_offsets,
            |face, holes| {
                let num_outer = holes.first().map_or(face.len(), |&h| h as usize);
                verts.clear();
                verts.extend(face.iter().map(|&gi| data.vertices[gi as usize]));
                if triangulate_3d(
                    &mut earcut,
                    &verts,
                    num_outer,
                    holes,
                    &mut buf2d,
                    &mut local,
                ) {
                    tris.extend(local.iter().map(|&l| face[l as usize]));
                }
            },
        );

        let mesh =
            TriangularMesh3D::from_parts(self.coordinate.clone(), data.vertices.clone(), tris)
                .expect("face indices are in range and triangles come in triples");
        Ok(Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(
            Box::new(mesh),
        )))
    }
}

/// Decode a `<1>`-stride index buffer to a flat `u32` list, widening each entry.
fn decode(buf: &IndexBuffer<1>) -> Vec<u32> {
    match buf {
        IndexBuffer::U8(v) => v.iter().map(|&[i]| i as u32).collect(),
        IndexBuffer::U16(v) => v.iter().map(|&[i]| i as u32).collect(),
        IndexBuffer::U32(v) => v.iter().map(|&[i]| i).collect(),
    }
}

/// Drive the CSR face traversal, invoking `f(face, hole_starts)` once per face.
/// `face` is the face's contiguous global vertex indices (exterior ring then any
/// hole rings, all stored open); `hole_starts` are the offsets within `face`
/// where each hole begins.
fn for_each_face(
    face_indices: &[u32],
    face_offsets: &[u32],
    interior_offsets: &[u32],
    mut f: impl FnMut(&[u32], &[u32]),
) {
    if face_indices.is_empty() {
        return;
    }
    let n = face_indices.len();
    let n_faces = face_offsets.len() + 1;
    let mut start = 0usize;
    for fi in 0..n_faces {
        let end = face_offsets.get(fi).map_or(n, |&o| o as usize);
        // Hole-ring starts that fall strictly inside this face, made face-local.
        let holes: Vec<u32> = interior_offsets
            .iter()
            .filter(|&&o| (o as usize) > start && (o as usize) < end)
            .map(|&o| o - start as u32)
            .collect();
        f(&face_indices[start..end], &holes);
        start = end;
    }
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
        let g = mesh.triangulate().unwrap();
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
        let g = mesh.triangulate().unwrap();
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
        let g = mesh.triangulate().unwrap();
        let tm = match &g {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => m,
            _ => panic!("expected a 3D triangular mesh"),
        };
        // A square ring with a square hole tessellates into 8 triangles.
        assert_eq!(tm.num_triangles(), 8);
    }
}
