use earcut::Earcut;

use super::{Polygon2D, Polygon3D};
use crate::ops::{Aabb, BoundingBox, Triangulate, UnsupportedOperation};
use crate::triangular_mesh::{TriangularMesh2D, TriangularMesh3D};
use crate::triangulation::{triangulate_2d, triangulate_3d};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

impl BoundingBox for Polygon2D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        // `coords` is every ring (exterior then holes) concatenated; the holes
        // lie inside the exterior, so the box over all of them equals the box
        // over the exterior alone.
        Aabb::from_points_2d(self.coords.iter().copied()).ok_or(UnsupportedOperation {
            geometry: "Polygon2D",
            operation: "bounding_box",
        })
    }
}

impl BoundingBox for Polygon3D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        Aabb::from_points_3d(self.coords.iter().copied()).ok_or(UnsupportedOperation {
            geometry: "Polygon3D",
            operation: "bounding_box",
        })
    }
}

impl Triangulate for Polygon2D {
    fn triangulate(&self) -> Result<Geometry, UnsupportedOperation> {
        let (positions, hole_indices, _num_outer) =
            open_ring_positions(&self.coords, &self.interior_offsets);
        let verts: Vec<[f64; 2]> = positions.iter().map(|&i| self.coords[i as usize]).collect();
        let mut earcut = Earcut::new();
        let mut indices = Vec::new();
        triangulate_2d(&mut earcut, &verts, &hole_indices, &mut indices);

        // earcut emits triangle corner indices into `verts` (3 per triangle, all
        // in range), so assembling the mesh from them cannot fail.
        let mesh = match &self.z {
            Some(z) => {
                let verts3: Vec<[f64; 3]> = positions
                    .iter()
                    .map(|&i| {
                        let [x, y] = self.coords[i as usize];
                        [x, y, z[i as usize]]
                    })
                    .collect();
                TriangularMesh2D::from_parts_with_elevation(
                    self.coordinate.clone(),
                    verts3,
                    indices,
                )
            }
            None => TriangularMesh2D::from_parts(self.coordinate.clone(), verts, indices),
        }
        .expect("earcut indices are in range and a multiple of three");
        Ok(Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(
            Box::new(mesh),
        )))
    }
}

impl Triangulate for Polygon3D {
    fn triangulate(&self) -> Result<Geometry, UnsupportedOperation> {
        let (positions, hole_indices, num_outer) =
            open_ring_positions(&self.coords, &self.interior_offsets);
        let verts: Vec<[f64; 3]> = positions.iter().map(|&i| self.coords[i as usize]).collect();
        let mut earcut = Earcut::new();
        let mut buf2d = Vec::new();
        let mut indices = Vec::new();
        triangulate_3d(
            &mut earcut,
            &verts,
            num_outer,
            &hole_indices,
            &mut buf2d,
            &mut indices,
        );
        let mesh = TriangularMesh3D::from_parts(self.coordinate.clone(), verts, indices)
            .expect("earcut indices are in range and a multiple of three");
        Ok(Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(
            Box::new(mesh),
        )))
    }
}

/// Walk a polygon's rings (exterior, then holes) over its flat `coords` /
/// `interior_offsets` layout, dropping each ring's closing duplicate, and return:
/// the positions into `coords` that make up the open rings (exterior first), the
/// start offset of each hole within that position list, and the exterior vertex
/// count. earcut closes rings implicitly, so the stored closing vertex is
/// stripped here.
fn open_ring_positions<const N: usize>(
    coords: &[[f64; N]],
    interior_offsets: &[u32],
) -> (Vec<u32>, Vec<u32>, usize) {
    // Strip a ring's closing duplicate, yielding the half-open `[start, end)` of
    // its distinct vertices.
    let open = |start: usize, end: usize| -> std::ops::Range<usize> {
        if end - start > 1 && coords[start] == coords[end - 1] {
            start..end - 1
        } else {
            start..end
        }
    };

    let mut positions: Vec<u32> = Vec::with_capacity(coords.len());
    let mut hole_indices: Vec<u32> = Vec::new();

    let first_hole = interior_offsets
        .first()
        .map_or(coords.len(), |&o| o as usize);
    positions.extend(open(0, first_hole).map(|i| i as u32));
    let num_outer = positions.len();

    for j in 0..interior_offsets.len() {
        let start = interior_offsets[j] as usize;
        let end = interior_offsets
            .get(j + 1)
            .map_or(coords.len(), |&o| o as usize);
        hole_indices.push(positions.len() as u32);
        positions.extend(open(start, end).map(|i| i as u32));
    }

    (positions, hole_indices, num_outer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::Coordinate;

    #[test]
    fn polygon2d_box_is_the_exterior_extent() {
        // A square exterior with an interior hole; the hole lies inside, so the
        // box is the exterior's.
        let exterior = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let hole = vec![[1.0, 1.0], [2.0, 1.0], [2.0, 2.0], [1.0, 1.0]];
        let p = Polygon2D::from_rings(Coordinate::Euclidean, exterior, vec![hole]);
        assert_eq!(
            p.bounding_box().unwrap(),
            Aabb::D2 {
                min: [0.0, 0.0],
                max: [4.0, 4.0]
            }
        );
    }

    fn tri_mesh_2d(g: &Geometry) -> &TriangularMesh2D {
        match g {
            Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(m)) => m,
            _ => panic!("expected a 2D triangular mesh, got {g:?}"),
        }
    }

    fn tri_mesh_3d(g: &Geometry) -> &TriangularMesh3D {
        match g {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => m,
            _ => panic!("expected a 3D triangular mesh, got {g:?}"),
        }
    }

    #[test]
    fn polygon2d_square_triangulates_to_two_triangles() {
        let square = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let p = Polygon2D::from_rings(Coordinate::Euclidean, square, Vec::<Vec<[f64; 2]>>::new());
        let g = p.triangulate().unwrap();
        let m = tri_mesh_2d(&g);
        assert_eq!(m.num_triangles(), 2);
        // The mesh covers the same extent as the polygon.
        assert_eq!(g.bounding_box().unwrap(), p.bounding_box().unwrap());
    }

    #[test]
    fn polygon2d_with_hole_triangulates() {
        // A 4-vertex square with a 4-vertex square hole: earcut yields 8 triangles.
        let exterior = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let hole = vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0], [1.0, 1.0]];
        let p = Polygon2D::from_rings(Coordinate::Euclidean, exterior, vec![hole]);
        let g = p.triangulate().unwrap();
        let m = tri_mesh_2d(&g);
        assert_eq!(m.num_triangles(), 8);
    }

    #[test]
    fn polygon2d_preserves_elevation() {
        let g = Polygon2D::from_rings_with_elevation(
            Coordinate::Euclidean,
            [
                [0.0, 0.0, 10.0],
                [4.0, 0.0, 11.0],
                [4.0, 4.0, 12.0],
                [0.0, 0.0, 10.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        )
        .triangulate()
        .unwrap();
        // A 2.5D polygon stays a 2D mesh (the elevation rides along in the z buffer).
        assert!(matches!(g, Geometry::Euclidean2D(_)));
        assert_eq!(tri_mesh_2d(&g).num_triangles(), 1);
    }

    #[test]
    fn polygon3d_square_triangulates_in_its_plane() {
        // A square in the x = 0 plane: earcut projects it and yields two triangles.
        let square = [
            [0.0, 0.0, 0.0],
            [0.0, 4.0, 0.0],
            [0.0, 4.0, 4.0],
            [0.0, 0.0, 4.0],
            [0.0, 0.0, 0.0],
        ];
        let p = Polygon3D::from_rings(Coordinate::Euclidean, square, Vec::<Vec<[f64; 3]>>::new());
        let g = p.triangulate().unwrap();
        let m = tri_mesh_3d(&g);
        assert_eq!(m.num_triangles(), 2);
        assert_eq!(g.bounding_box().unwrap(), p.bounding_box().unwrap());
    }

    #[test]
    fn polygon3d_degenerate_yields_no_triangles() {
        // Three collinear points cannot define a plane: no triangles, but still a mesh.
        let line = [
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [2.0, 2.0, 2.0],
            [0.0, 0.0, 0.0],
        ];
        let p = Polygon3D::from_rings(Coordinate::Euclidean, line, Vec::<Vec<[f64; 3]>>::new());
        let g = p.triangulate().unwrap();
        assert_eq!(tri_mesh_3d(&g).num_triangles(), 0);
    }

    #[test]
    fn polygon3d_box() {
        let exterior = [
            [0.0, 0.0, 1.0],
            [4.0, 0.0, 1.0],
            [4.0, 4.0, 2.0],
            [0.0, 0.0, 1.0],
        ];
        let p = Polygon3D::from_rings(Coordinate::Euclidean, exterior, Vec::<Vec<[f64; 3]>>::new());
        assert_eq!(
            p.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, 0.0, 1.0],
                max: [4.0, 4.0, 2.0]
            }
        );
    }
}
