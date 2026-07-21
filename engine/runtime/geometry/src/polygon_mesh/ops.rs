use super::{PolygonMesh2D, PolygonMesh3D, PolygonMesh3DData};
use crate::coordinate::{CoordinateFrame, EpsgCode};
use crate::index::IndexBuffer;
use crate::ops::reproject::{transform_coords_2d, transform_coords_3d};
use crate::ops::triangulation::{
    expand_appearance, triangulate_2d, triangulate_3d, Cache, Triangulated,
};
use crate::ops::{
    Aabb, BoundingBox, Reproject, ReprojectionCache, Triangulate, UnsupportedOperation,
};
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

impl Reproject for PolygonMesh2D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        let from = self.frame.require_crs()?;
        if from != target {
            transform_coords_2d(
                cache,
                from,
                target,
                &mut self.vertices,
                self.z.as_deref_mut(),
            )?;
            self.frame = CoordinateFrame::Crs(target);
        }
        Ok(())
    }
}

impl Reproject for PolygonMesh3D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        let from = self.frame.require_crs()?;
        if from != target {
            transform_coords_3d(cache, from, target, self.data.vertices_mut())?;
            self.frame = CoordinateFrame::Crs(target);
        }
        Ok(())
    }
}

impl Triangulate for PolygonMesh2D {
    fn triangulate(&mut self, cache: &mut Cache) -> Result<Geometry, UnsupportedOperation> {
        let Cache { earcut, buffers } = cache;
        decode_into(&self.face_indices, &mut buffers.face_indices);
        decode_into(&self.face_offsets, &mut buffers.face_offsets);
        decode_into(&self.interior_offsets, &mut buffers.interior_offsets);

        buffers.tris.clear();
        buffers.corner_src.clear();
        buffers.face_tris.clear();
        buffers.tris.reserve(3 * buffers.face_indices.len());

        let n = buffers.face_indices.len();
        if n != 0 {
            let n_faces = buffers.face_offsets.len() + 1;
            let mut start = 0usize;
            for fi in 0..n_faces {
                let end = buffers.face_offsets.get(fi).map_or(n, |&o| o as usize);
                build_open_rings(
                    &buffers.face_indices,
                    &buffers.interior_offsets,
                    start,
                    end,
                    &mut buffers.open_src,
                    &mut buffers.holes,
                );
                let face = &buffers.face_indices[start..end];
                buffers.verts2.clear();
                // SAFETY: face indices are validated `< vertices.len()` at construction.
                buffers
                    .verts2
                    .extend(buffers.open_src.iter().map(|&p| unsafe {
                        *self
                            .vertices
                            .get_unchecked(*face.get_unchecked(p as usize) as usize)
                    }));
                // earcut clears `out` itself, but reset explicitly so the per-face
                // count below stays correct without relying on that internal.
                buffers.out.clear();
                triangulate_2d(earcut, &buffers.verts2, &buffers.holes, &mut buffers.out);
                // Map face-local corner indices back to the shared vertex pool.
                // SAFETY: each earcut index is `< open_src.len()`.
                buffers.tris.extend(buffers.out.iter().map(|&l| unsafe {
                    *face.get_unchecked(*buffers.open_src.get_unchecked(l as usize) as usize)
                }));
                buffers.corner_src.extend(
                    buffers
                        .out
                        .iter()
                        .map(|&l| start as u32 + buffers.open_src[l as usize]),
                );
                buffers.face_tris.push((buffers.out.len() / 3) as u32);
                start = end;
            }
        }

        let appearance = expand_appearance(
            std::mem::take(&mut self.appearance),
            &buffers.face_tris,
            &buffers.corner_src,
        );
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
                        self.frame.clone(),
                        verts3,
                        triangle_count,
                        buffers.tris.iter().copied(),
                    )
                }
            }
            // SAFETY: every index is `< vertices.len()`; count is a multiple of 3.
            None => unsafe {
                TriangularMesh2D::from_parts_unchecked(
                    self.frame.clone(),
                    std::mem::take(&mut self.vertices),
                    triangle_count,
                    buffers.tris.iter().copied(),
                )
            },
        };
        mesh.set_raw_appearance(appearance);
        Ok(Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(
            Box::new(mesh),
        )))
    }
}

impl PolygonMesh3DData {
    /// Triangulate every polygon into coordinate-free triangle-mesh data,
    /// **stealing** this mesh's vertex pool, appearance and UV (see
    /// [`Triangulate`]). Also returns each source polygon's flat normal
    /// (Newell's method over that polygon's own exterior ring, not
    /// recomputed per triangle) and how many triangles that polygon was
    /// split into, in polygon order — plain, transient values, not stored on
    /// any mesh type. A caller wanting the normal for output triangle `t`
    /// sums the triangle counts until it passes `t` and reads the matching
    /// normal entry, rather than this pre-expanding to one entry per
    /// triangle itself.
    pub(crate) fn triangulate(&mut self, cache: &mut Cache) -> Triangulated<TriangularMesh3DData> {
        let Cache { earcut, buffers } = cache;
        decode_into(&self.face_indices, &mut buffers.face_indices);
        decode_into(&self.face_offsets, &mut buffers.face_offsets);
        decode_into(&self.interior_offsets, &mut buffers.interior_offsets);

        buffers.tris.clear();
        buffers.corner_src.clear();
        buffers.face_tris.clear();
        buffers.face_normals.clear();
        buffers.tris.reserve(3 * buffers.face_indices.len());

        let n = buffers.face_indices.len();
        if n != 0 {
            let n_faces = buffers.face_offsets.len() + 1;
            let mut start = 0usize;
            for fi in 0..n_faces {
                let end = buffers.face_offsets.get(fi).map_or(n, |&o| o as usize);
                build_open_rings(
                    &buffers.face_indices,
                    &buffers.interior_offsets,
                    start,
                    end,
                    &mut buffers.open_src,
                    &mut buffers.holes,
                );
                let face = &buffers.face_indices[start..end];
                let num_outer = buffers
                    .holes
                    .first()
                    .map_or(buffers.open_src.len(), |&h| h as usize);
                buffers.verts3.clear();
                // SAFETY: face indices are validated `< vertices.len()` at construction.
                buffers
                    .verts3
                    .extend(buffers.open_src.iter().map(|&p| unsafe {
                        *self
                            .vertices
                            .get_unchecked(*face.get_unchecked(p as usize) as usize)
                    }));
                // `triangulate_3d` clears `out` on its degenerate paths and earcut
                // clears it on success; reset explicitly so the per-face count
                // below holds without relying on either internal.
                buffers.out.clear();
                let face_normal = triangulate_3d(
                    earcut,
                    &buffers.verts3,
                    num_outer,
                    &buffers.holes,
                    &mut buffers.out,
                );
                if face_normal.is_some() {
                    // SAFETY: each earcut index is `< open_src.len()`.
                    buffers.tris.extend(buffers.out.iter().map(|&l| unsafe {
                        *face.get_unchecked(*buffers.open_src.get_unchecked(l as usize) as usize)
                    }));
                    buffers.corner_src.extend(
                        buffers
                            .out
                            .iter()
                            .map(|&l| start as u32 + buffers.open_src[l as usize]),
                    );
                }
                // Outside the `if`: a degenerate face records 0 triangles (its
                // `out` is empty) and a placeholder normal that's never repeated.
                buffers.face_tris.push((buffers.out.len() / 3) as u32);
                buffers
                    .face_normals
                    .push(face_normal.unwrap_or([0.0, 0.0, 1.0]));
                start = end;
            }
        }

        let appearance = expand_appearance(
            std::mem::take(&mut self.appearance),
            &buffers.face_tris,
            &buffers.corner_src,
        );
        // SAFETY: `tris` index `self.vertices` (`< len`); count is a multiple of 3.
        let mut data = unsafe {
            TriangularMesh3DData::from_parts_unchecked(
                std::mem::take(&mut self.vertices),
                buffers.tris.len() / 3,
                buffers.tris.iter().copied(),
            )
        };
        data.set_raw_appearance(appearance);
        Triangulated {
            mesh: data,
            polygon_normals: std::mem::take(&mut buffers.face_normals),
            polygon_tris: std::mem::take(&mut buffers.face_tris),
        }
    }
}

impl Triangulate for PolygonMesh3D {
    fn triangulate(&mut self, cache: &mut Cache) -> Result<Geometry, UnsupportedOperation> {
        let result = self.data.triangulate(cache);
        let mesh = TriangularMesh3D::new(self.frame.clone(), result.mesh);
        Ok(Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(
            Box::new(mesh),
        )))
    }
}

impl PolygonMesh3D {
    /// As [`Triangulate::triangulate`], but also returns each source
    /// polygon's flat normal and its output triangle count, in polygon order
    /// (see [`PolygonMesh3DData::triangulate`]), for a caller that wants to
    /// attach flat normals as a mesh attribute without this crate's geometry
    /// types ever carrying a normal field themselves.
    ///
    /// Each normal is the polygon's canonical outward normal (see
    /// [`crate::coordinate`]). Errors when the frame's orientation sign cannot
    /// be determined (e.g. an unknown or non-axis-aligned CRS).
    pub fn triangulate_with_normals(
        &mut self,
        cache: &mut Cache,
    ) -> crate::error::Result<Triangulated<TriangularMesh3D>> {
        let sign = self.frame.orientation_sign()? as f64;
        let result = self.data.triangulate(cache);
        let polygon_normals = result
            .polygon_normals
            .into_iter()
            .map(|[x, y, z]| [x * sign, y * sign, z * sign])
            .collect();
        Ok(Triangulated {
            mesh: TriangularMesh3D::new(self.frame.clone(), result.mesh),
            polygon_normals,
            polygon_tris: result.polygon_tris,
        })
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

/// Build the open-ring layout of the face spanning `[start, end)` of
/// `face_indices` for triangulation. Fills `open_src` with the face-local position
/// of each corner (each ring's closing duplicate dropped) and `holes` with the
/// start offset of each hole ring into `open_src`.
pub(crate) fn build_open_rings(
    face_indices: &[u32],
    interior_offsets: &[u32],
    start: usize,
    end: usize,
    open_src: &mut Vec<u32>,
    holes: &mut Vec<u32>,
) {
    open_src.clear();
    holes.clear();
    let mut ring_start = start;
    for &offset in interior_offsets {
        let boundary = offset as usize;
        if boundary <= start || boundary >= end {
            continue;
        }
        push_open_ring(face_indices, start, ring_start, boundary, open_src);
        holes.push(open_src.len() as u32);
        ring_start = boundary;
    }
    push_open_ring(face_indices, start, ring_start, end, open_src);
}

/// Append the face-local positions of ring `[ring_start, ring_end)` to `open_src`,
/// dropping the ring's closing vertex when it duplicates the first.
fn push_open_ring(
    face_indices: &[u32],
    start: usize,
    ring_start: usize,
    ring_end: usize,
    open_src: &mut Vec<u32>,
) {
    let open_end =
        if ring_end - ring_start >= 2 && face_indices[ring_start] == face_indices[ring_end - 1] {
            ring_end - 1
        } else {
            ring_end
        };
    open_src.extend((ring_start..open_end).map(|p| (p - start) as u32));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::{CoordinateFrame, EpsgCode};

    #[test]
    fn polygon_mesh2d_box_spans_vertex_pool() {
        let m = PolygonMesh2D::from_parts(
            CoordinateFrame::Euclidean,
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
            CoordinateFrame::Euclidean,
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
            CoordinateFrame::Euclidean,
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
            CoordinateFrame::Euclidean,
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
    fn triangulate_with_normals_orients_normal_by_frame_sign() {
        // A quad wound CCW in the z = 0 plane: its right-hand-rule normal is +Z.
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 2.0, 0.0],
            [0.0, 2.0, 0.0],
        ];
        let build = |frame| {
            PolygonMesh3D::from_parts(frame, vertices.clone(), vec![vec![0u32, 1, 2, 3]]).unwrap()
        };

        // A right-handed (Euclidean) frame keeps the raw +Z as the outward normal.
        let mut mesh = build(CoordinateFrame::Euclidean);
        assert_eq!(
            mesh.triangulate_with_normals(&mut Cache::new())
                .unwrap()
                .polygon_normals,
            vec![[0.0, 0.0, 1.0]]
        );

        // EPSG:6697 is lat-first (orientation sign -1), so the same winding is
        // canonically the opposite orientation: the normal comes out flipped,
        // without the caller reprojecting into a right-handed frame first.
        let mut mesh = build(CoordinateFrame::Crs(EpsgCode::new(6697)));
        assert_eq!(
            mesh.triangulate_with_normals(&mut Cache::new())
                .unwrap()
                .polygon_normals,
            vec![[0.0, 0.0, -1.0]]
        );
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
            CoordinateFrame::Euclidean,
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
    fn polygon_mesh3d_triangulates_holed_polygon_with_closed_rings() {
        // Built via `from_polygons`, so the exterior and hole rings are stored
        // closed; triangulation must drop each closing vertex before earcut.
        use crate::polygon::Polygon3D;

        let exterior = [
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 4.0, 0.0],
            [0.0, 4.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let hole = vec![
            [1.0, 1.0, 0.0],
            [3.0, 1.0, 0.0],
            [3.0, 3.0, 0.0],
            [1.0, 3.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let p = Polygon3D::from_rings(CoordinateFrame::Euclidean, exterior, vec![hole]);
        let mut mesh = PolygonMesh3D::from_polygons(CoordinateFrame::Euclidean, [&p]).unwrap();
        let g = mesh.triangulate(&mut Cache::new()).unwrap();
        let tm = match &g {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => m,
            _ => panic!("expected a 3D triangular mesh"),
        };
        assert_eq!(tm.num_triangles(), 8);
    }

    #[test]
    fn triangulation_expands_per_face_binding() {
        use crate::appearance::{FaceBinding, MaterialIndex};
        use crate::polygon::Polygon3D;
        use crate::test_support::{textured, theme, uv};

        let ring_a = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let ring_b = [
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [3.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
            [2.0, 0.0, 0.0],
        ];
        let mut a = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            ring_a,
            Vec::<Vec<[f64; 3]>>::new(),
        );
        a.set_appearance(theme("rgb"), textured(), Some(uv(5)))
            .unwrap();
        let mut b = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            ring_b,
            Vec::<Vec<[f64; 3]>>::new(),
        );
        b.set_appearance(theme("rgb"), textured(), Some(uv(5)))
            .unwrap();

        let mut mesh = PolygonMesh3D::from_polygons(CoordinateFrame::Euclidean, [&a, &b]).unwrap();
        assert!(matches!(
            mesh.appearance().as_ref().unwrap().themes()[0].front,
            FaceBinding::PerFace(_)
        ));

        let g = mesh.triangulate(&mut Cache::new()).unwrap();
        let tm = match &g {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => m,
            _ => panic!("expected a 3D triangular mesh"),
        };
        // Each quad -> 2 triangles; the per-face binding expands to one entry per triangle.
        assert_eq!(tm.num_triangles(), 4);
        let FaceBinding::PerFace(front) = &tm.appearance().as_ref().unwrap().themes()[0].front
        else {
            panic!("expected PerFace");
        };
        assert_eq!(
            front,
            &[
                MaterialIndex::new(0),
                MaterialIndex::new(0),
                MaterialIndex::new(1),
                MaterialIndex::new(1),
            ]
        );
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
        data.appearance = Some(Appearance::from_parts(
            vec![textured()],
            vec![ThemeBinding {
                theme: theme("rgb"),
                front: FaceBinding::Uniform(MaterialIndex::new(0).unwrap()),
                back: None,
                uv_sets: vec![UvSet {
                    side: Side::Front,
                    channel: Default::default(),
                    uv: UvSource::Explicit(Box::new(src)),
                }],
            }],
            theme("rgb"),
        ));

        let result = data.triangulate(&mut Cache::new());
        let mesh = TriangularMesh3D::new(CoordinateFrame::Euclidean, result.mesh);
        assert_eq!(mesh.num_triangles(), 2);

        let app = mesh.appearance().as_ref().unwrap();
        let UvSource::Explicit(out_uv) = &app.themes()[0].uv_sets[0].uv else {
            panic!("expected an explicit output UV set");
        };
        assert_eq!(out_uv.len(), 6);
        assert!(out_uv.iter().all(|uv| src.contains(uv)));
        assert!(matches!(
            mesh.appearance().as_ref().unwrap().themes()[0].front,
            FaceBinding::Uniform(_)
        ));
    }
}
