use super::{PolygonMesh2D, PolygonMesh3D, PolygonMesh3DData};
use crate::index::IndexBuffer;
use crate::ops::triangulation::{
    require_uniform_bindings, retarget_uv, triangulate_2d, triangulate_3d, Cache,
};
use crate::ops::{Aabb, BoundingBox, Triangulate, UnsupportedOperation};
use crate::triangular_mesh::{TriangularMesh2D, TriangularMesh3D, TriangularMesh3DData};
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
    fn triangulate(&mut self, cache: &mut Cache) -> Result<Geometry, UnsupportedOperation> {
        require_uniform_bindings(self.appearance(), "PolygonMesh2D")?;

        let Cache { earcut, buffers } = cache;
        decode_into(&self.face_indices, &mut buffers.face_indices);
        decode_into(&self.face_offsets, &mut buffers.face_offsets);
        decode_into(&self.interior_offsets, &mut buffers.interior_offsets);

        buffers.tris.clear();
        buffers.corner_src.clear();
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
                buffers
                    .corner_src
                    .extend(buffers.out.iter().map(|&l| start as u32 + l));
                start = end;
            }
        }

        let uv_sets = std::mem::take(&mut self.uv_sets)
            .into_iter()
            .map(|uv| retarget_uv(uv, &buffers.corner_src))
            .collect();
        let appearance = std::mem::take(&mut self.appearance);
        let triangle_count = buffers.tris.len() / 3;
        // `tris` index the existing pool (each `< vertices.len()`) in triples.
        let mut mesh = match std::mem::take(&mut self.z) {
            Some(z) => {
                let verts3: Vec<[f64; 3]> = std::mem::take(&mut self.vertices)
                    .into_iter()
                    .zip(z)
                    .map(|([x, y], zz)| [x, y, zz])
                    .collect();
                // SAFETY: every index is `< verts3.len()`; count is a multiple of 3.
                unsafe {
                    TriangularMesh2D::from_parts_with_elevation_unchecked(
                        self.coordinate.clone(),
                        verts3,
                        triangle_count,
                        buffers.tris.iter().copied(),
                    )
                }
            }
            // SAFETY: every index is `< vertices.len()`; count is a multiple of 3.
            None => unsafe {
                TriangularMesh2D::from_parts_unchecked(
                    self.coordinate.clone(),
                    std::mem::take(&mut self.vertices),
                    triangle_count,
                    buffers.tris.iter().copied(),
                )
            },
        };
        mesh.set_raw_appearance(uv_sets, appearance);
        Ok(Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(
            Box::new(mesh),
        )))
    }
}

impl PolygonMesh3DData {
    /// Triangulate every face into coordinate-free triangle-mesh data, **stealing**
    /// this mesh's vertex pool, appearance and UV (see [`Triangulate`]). Fails
    /// (leaving `self` untouched) if any binding is non-uniform.
    pub(crate) fn triangulate(
        &mut self,
        cache: &mut Cache,
    ) -> Result<TriangularMesh3DData, UnsupportedOperation> {
        require_uniform_bindings(&self.appearance, "PolygonMesh3D")?;

        let Cache { earcut, buffers } = cache;
        decode_into(&self.face_indices, &mut buffers.face_indices);
        decode_into(&self.face_offsets, &mut buffers.face_offsets);
        decode_into(&self.interior_offsets, &mut buffers.interior_offsets);

        buffers.tris.clear();
        buffers.corner_src.clear();
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
                        .map(|&gi| unsafe { *self.vertices.get_unchecked(gi as usize) }),
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
                    buffers
                        .corner_src
                        .extend(buffers.out.iter().map(|&l| start as u32 + l));
                }
                start = end;
            }
        }

        let uv_sets = std::mem::take(&mut self.uv_sets)
            .into_iter()
            .map(|uv| retarget_uv(uv, &buffers.corner_src))
            .collect();
        let appearance = std::mem::take(&mut self.appearance);
        // SAFETY: `tris` index `self.vertices` (`< len`); count is a multiple of 3.
        let mut data = unsafe {
            TriangularMesh3DData::from_parts_unchecked(
                std::mem::take(&mut self.vertices),
                buffers.tris.len() / 3,
                buffers.tris.iter().copied(),
            )
        };
        data.set_raw_appearance(uv_sets, appearance);
        Ok(data)
    }
}

impl Triangulate for PolygonMesh3D {
    fn triangulate(&mut self, cache: &mut Cache) -> Result<Geometry, UnsupportedOperation> {
        let data = self.data.triangulate(cache)?;
        let mesh = TriangularMesh3D::new(self.coordinate.clone(), data);
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
        let mut mesh = PolygonMesh2D::from_parts(
            Coordinate::Euclidean,
            vertices,
            vec![vec![0u32, 1, 2, 3], vec![1, 4, 5, 2]],
        )
        .unwrap();
        let mesh_box = mesh.bounding_box().unwrap();
        let g = mesh.triangulate(&mut Cache::new()).unwrap();
        let tm = match &g {
            Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(m)) => m,
            _ => panic!("expected a 2D triangular mesh"),
        };
        assert_eq!(tm.num_triangles(), 4);
        assert_eq!(g.bounding_box().unwrap(), mesh_box);
    }

    #[test]
    fn polygon_mesh3d_triangulates_quad_in_plane() {
        let mut mesh = PolygonMesh3D::from_parts(
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
        let mesh_box = mesh.bounding_box().unwrap();
        let g = mesh.triangulate(&mut Cache::new()).unwrap();
        let tm = match &g {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => m,
            _ => panic!("expected a 3D triangular mesh"),
        };
        assert_eq!(tm.num_triangles(), 2);
        assert_eq!(g.bounding_box().unwrap(), mesh_box);
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
        let mut mesh = PolygonMesh3D::from_raw_parts(
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

    #[test]
    fn triangulation_rejects_non_uniform_binding() {
        use crate::appearance::FaceBinding;
        use crate::polygon::Polygon3D;
        use crate::test_support::{textured, theme, uv};

        // Welding appearance-carrying polygons yields a `PerFace` binding (one
        // entry per face), which tessellation does not support yet.
        let ring = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let mut a = Polygon3D::from_rings(Coordinate::Euclidean, ring, Vec::<Vec<[f64; 3]>>::new());
        a.set_appearance(theme("rgb"), textured(), Some(uv(5)))
            .unwrap();
        let mut b = Polygon3D::from_rings(Coordinate::Euclidean, ring, Vec::<Vec<[f64; 3]>>::new());
        b.set_appearance(theme("rgb"), textured(), Some(uv(5)))
            .unwrap();

        let mut mesh = PolygonMesh3D::from_polygons(Coordinate::Euclidean, [&a, &b]).unwrap();
        assert!(matches!(
            mesh.appearance().as_ref().unwrap().themes[0].front,
            FaceBinding::PerFace(_)
        ));
        let err = mesh.triangulate(&mut Cache::new()).unwrap_err();
        assert!(err.operation.contains("uniform"));
        // `self` is left intact on the error path.
        assert!(mesh.bounding_box().is_ok());
    }

    #[test]
    fn triangulation_regathers_uv_per_corner_for_uniform_mesh() {
        use crate::appearance::{
            Appearance, FaceBinding, MaterialIndex, Side, ThemeBinding, UvSet, UvSource,
        };
        use crate::test_support::{textured, theme};

        // A uniform-bound mesh is only reachable by direct construction (welding
        // always yields PerFace).
        let mut data = PolygonMesh3DData::from_parts(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
            ],
            vec![vec![0u32, 1, 2, 3]],
        )
        .unwrap();
        let src = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        data.uv_sets = vec![UvSet {
            theme: Some(theme("rgb")),
            side: Side::Front,
            channel: Default::default(),
            uv: UvSource::Explicit(Box::new(src)),
        }];
        data.appearance = Some(Appearance {
            materials: vec![textured()],
            themes: vec![ThemeBinding {
                theme: theme("rgb"),
                front: FaceBinding::Uniform(MaterialIndex::new(0).unwrap()),
                back: None,
            }],
            default_theme: theme("rgb"),
        });

        let out = data.triangulate(&mut Cache::new()).unwrap();
        let mesh = TriangularMesh3D::new(Coordinate::Euclidean, out);
        assert_eq!(mesh.num_triangles(), 2);

        let UvSource::Explicit(out_uv) = &mesh.uv_sets()[0].uv else {
            panic!("expected an explicit output UV set");
        };
        assert_eq!(out_uv.len(), 6);
        assert!(out_uv.iter().all(|uv| src.contains(uv)));
        assert!(matches!(
            mesh.appearance().as_ref().unwrap().themes[0].front,
            FaceBinding::Uniform(_)
        ));
    }
}
