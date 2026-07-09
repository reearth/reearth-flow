use super::{Polygon2D, Polygon3D};
use crate::coordinate::{CoordinateFrame, EpsgCode};
use crate::ops::reproject::{transform_coords_2d, transform_coords_3d};
use crate::ops::triangulation::{expand_appearance, triangulate_2d, triangulate_3d, Cache};
use crate::ops::{
    Aabb, BoundingBox, Reproject, ReprojectionCache, Triangulate, UnsupportedOperation,
};
use crate::triangular_mesh::{TriangularMesh2D, TriangularMesh3D};
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

impl Reproject for Polygon2D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        let from = self.frame.require_crs()?;
        if from != target {
            transform_coords_2d(cache, from, target, &mut self.coords, self.z.as_deref_mut())?;
            self.frame = CoordinateFrame::Crs(target);
        }
        Ok(())
    }
}

impl Reproject for Polygon3D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        let from = self.frame.require_crs()?;
        if from != target {
            transform_coords_3d(cache, from, target, &mut self.coords)?;
            self.frame = CoordinateFrame::Crs(target);
        }
        Ok(())
    }
}

impl Triangulate for Polygon2D {
    fn triangulate(&mut self, cache: &mut Cache) -> Result<Geometry, UnsupportedOperation> {
        let Cache { earcut, buffers } = cache;
        open_ring_positions(
            &self.coords,
            &self.interior_offsets,
            &mut buffers.positions,
            &mut buffers.holes,
        );
        buffers.out.clear();
        // 3V slightly over-reserves with no holes (by 6) and is exact at one hole, but under-reserves by 6(H−1) once there are ≥2 holes.
        buffers.out.reserve(3 * buffers.positions.len());

        // earcut emits triangle corner indices into the gathered ring vertices
        // (3 per triangle, each < the vertex count), so the unchecked assembly is
        // sound. The gathered `verts` is the output mesh's own pool (not scratch).
        let mut mesh = match &self.z {
            None => {
                let mut verts: Vec<[f64; 2]> = Vec::with_capacity(buffers.positions.len());
                // SAFETY: `positions` are in-range indices into `coords`.
                verts.extend(
                    buffers
                        .positions
                        .iter()
                        .map(|&i| unsafe { *self.coords.get_unchecked(i as usize) }),
                );
                triangulate_2d(earcut, &verts, &buffers.holes, &mut buffers.out);
                // SAFETY: every earcut index is `< verts.len()`; count is a multiple of 3.
                unsafe {
                    TriangularMesh2D::from_parts_unchecked(
                        self.frame.clone(),
                        verts,
                        buffers.out.len() / 3,
                        buffers.out.iter().copied(),
                    )
                }
            }
            Some(z) => {
                let mut verts: Vec<[f64; 3]> = Vec::with_capacity(buffers.positions.len());
                verts.extend(buffers.positions.iter().map(|&i| {
                    let i = i as usize;
                    // SAFETY: `positions` index `coords`, and `z` is parallel to `coords`.
                    let [x, y] = unsafe { *self.coords.get_unchecked(i) };
                    [x, y, unsafe { *z.get_unchecked(i) }]
                }));
                // Triangulate the planar (x, y) footprint; elevation rides along.
                earcut.earcut(
                    verts.iter().map(|&[x, y, _]| [x, y]),
                    &buffers.holes,
                    &mut buffers.out,
                );
                // SAFETY: every earcut index is `< verts.len()`; count is a multiple of 3.
                unsafe {
                    TriangularMesh2D::from_parts_with_elevation_unchecked(
                        self.frame.clone(),
                        verts,
                        buffers.out.len() / 3,
                        buffers.out.iter().copied(),
                    )
                }
            }
        };
        let src_corner: Vec<u32> = buffers
            .out
            .iter()
            .map(|&c| buffers.positions[c as usize])
            .collect();
        let appearance = expand_appearance(
            std::mem::take(&mut self.appearance),
            &[(buffers.out.len() / 3) as u32],
            &src_corner,
        );
        mesh.set_raw_appearance(appearance);
        Ok(Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(
            Box::new(mesh),
        )))
    }
}

impl Triangulate for Polygon3D {
    fn triangulate(&mut self, cache: &mut Cache) -> Result<Geometry, UnsupportedOperation> {
        let Cache { earcut, buffers } = cache;
        let num_outer = open_ring_positions(
            &self.coords,
            &self.interior_offsets,
            &mut buffers.positions,
            &mut buffers.holes,
        );
        let mut verts: Vec<[f64; 3]> = Vec::with_capacity(buffers.positions.len());
        // SAFETY: `positions` are in-range indices into `coords`.
        verts.extend(
            buffers
                .positions
                .iter()
                .map(|&i| unsafe { *self.coords.get_unchecked(i as usize) }),
        );
        buffers.out.clear();
        buffers.out.reserve(3 * verts.len());
        let _ = triangulate_3d(earcut, &verts, num_outer, &buffers.holes, &mut buffers.out);
        // SAFETY: every earcut index is `< verts.len()`; count is a multiple of 3.
        let mut mesh = unsafe {
            TriangularMesh3D::from_parts_unchecked(
                self.frame.clone(),
                verts,
                buffers.out.len() / 3,
                buffers.out.iter().copied(),
            )
        };
        let src_corner: Vec<u32> = buffers
            .out
            .iter()
            .map(|&c| buffers.positions[c as usize])
            .collect();
        let appearance = expand_appearance(
            std::mem::take(&mut self.appearance),
            &[(buffers.out.len() / 3) as u32],
            &src_corner,
        );
        mesh.set_raw_appearance(appearance);
        Ok(Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(
            Box::new(mesh),
        )))
    }
}

/// Walk a polygon's rings (exterior, then holes) over its flat `coords` /
/// `interior_offsets` layout, dropping each ring's closing duplicate, into the
/// reused `positions` (the open rings' positions into `coords`, exterior first)
/// and `holes` (each hole's start offset within `positions`) buffers; returns
/// the exterior vertex count. earcut closes rings implicitly, so the stored
/// closing vertex is stripped here.
fn open_ring_positions<const N: usize>(
    coords: &[[f64; N]],
    interior_offsets: &[u32],
    positions: &mut Vec<u32>,
    holes: &mut Vec<u32>,
) -> usize {
    positions.clear();
    holes.clear();

    // Strip a ring's closing duplicate, yielding the half-open `[start, end)` of
    // its distinct vertices.
    let open = |start: usize, end: usize| -> std::ops::Range<usize> {
        if end - start > 1 && coords[start] == coords[end - 1] {
            start..end - 1
        } else {
            start..end
        }
    };

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
        holes.push(positions.len() as u32);
        positions.extend(open(start, end).map(|i| i as u32));
    }

    num_outer
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;

    #[test]
    fn polygon2d_box_is_the_exterior_extent() {
        // A square exterior with an interior hole; the hole lies inside, so the
        // box is the exterior's.
        let exterior = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let hole = vec![[1.0, 1.0], [2.0, 1.0], [2.0, 2.0], [1.0, 1.0]];
        let p = Polygon2D::from_rings(CoordinateFrame::Euclidean, exterior, vec![hole]);
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
        let mut p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            square,
            Vec::<Vec<[f64; 2]>>::new(),
        );
        let g = p.triangulate(&mut Cache::new()).unwrap();
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
        let mut p = Polygon2D::from_rings(CoordinateFrame::Euclidean, exterior, vec![hole]);
        let g = p.triangulate(&mut Cache::new()).unwrap();
        let m = tri_mesh_2d(&g);
        assert_eq!(m.num_triangles(), 8);
    }

    #[test]
    fn polygon2d_preserves_elevation() {
        let g = Polygon2D::from_rings_with_elevation(
            CoordinateFrame::Euclidean,
            [
                [0.0, 0.0, 10.0],
                [4.0, 0.0, 11.0],
                [4.0, 4.0, 12.0],
                [0.0, 0.0, 10.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        )
        .triangulate(&mut Cache::new())
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
        let mut p = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            square,
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let g = p.triangulate(&mut Cache::new()).unwrap();
        let m = tri_mesh_3d(&g);
        assert_eq!(m.num_triangles(), 2);
        assert_eq!(g.bounding_box().unwrap(), p.bounding_box().unwrap());
    }

    #[test]
    fn one_cache_reused_across_calls_stays_correct() {
        // Reuse a single cache across a square, a square-with-hole, and a 3D
        // face — each must reset its scratch and produce the right result.
        let mut cache = Cache::new();
        let square = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];

        let a = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            square,
            Vec::<Vec<[f64; 2]>>::new(),
        )
        .triangulate(&mut cache)
        .unwrap();
        assert_eq!(tri_mesh_2d(&a).num_triangles(), 2);

        let hole = vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0], [1.0, 1.0]];
        let b = Polygon2D::from_rings(CoordinateFrame::Euclidean, square, vec![hole])
            .triangulate(&mut cache)
            .unwrap();
        assert_eq!(tri_mesh_2d(&b).num_triangles(), 8);

        let face3d = [
            [0.0, 0.0, 0.0],
            [0.0, 4.0, 0.0],
            [0.0, 4.0, 4.0],
            [0.0, 0.0, 4.0],
            [0.0, 0.0, 0.0],
        ];
        let c = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            face3d,
            Vec::<Vec<[f64; 3]>>::new(),
        )
        .triangulate(&mut cache)
        .unwrap();
        assert_eq!(tri_mesh_3d(&c).num_triangles(), 2);
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
        let mut p = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            line,
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let g = p.triangulate(&mut Cache::new()).unwrap();
        assert_eq!(tri_mesh_3d(&g).num_triangles(), 0);
    }

    #[test]
    fn triangulation_carries_uniform_appearance_and_regathers_uv() {
        use crate::appearance::{FaceBinding, UvSource};
        use crate::test_support::{textured, theme};

        // UV is parallel to `coords` (5 entries, last = closing dup), distinct per
        // real corner so the gather is checkable.
        let square = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let mut p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            square,
            Vec::<Vec<[f64; 2]>>::new(),
        );
        let src_uv = UvSource::Explicit(Box::new([
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
            [0.0, 0.0], // closing duplicate — never gathered
        ]));
        p.set_appearance(theme("rgb"), textured(), Some(src_uv))
            .unwrap();

        let g = p.triangulate(&mut Cache::new()).unwrap();
        let m = tri_mesh_2d(&g);
        assert_eq!(m.num_triangles(), 2);

        let app = m.appearance().as_ref().expect("appearance carried over");
        assert_eq!(app.materials().len(), 1);
        assert_eq!(*app.default_theme(), theme("rgb"));
        assert!(matches!(app.themes()[0].front, FaceBinding::Uniform(_)));

        // Every output UV is one of the real source-corner UVs (gathered, not
        // interpolated; the closing-duplicate slot is never referenced).
        let UvSource::Explicit(out_uv) = &app.themes()[0].uv_sets[0].uv else {
            panic!("expected an explicit output UV set");
        };
        assert_eq!(out_uv.len(), 6);
        let corners = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        assert!(out_uv.iter().all(|uv| corners.contains(uv)));
    }

    #[test]
    fn triangulation_passes_through_world_to_texture_uv() {
        use crate::appearance::{TexMatrix, UvSource};
        use crate::test_support::{textured, theme};

        let square = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let mut p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            square,
            Vec::<Vec<[f64; 2]>>::new(),
        );
        let matrix = TexMatrix([
            [0.25, 0.0, 0.0, 0.0],
            [0.0, 0.25, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        p.set_appearance(
            theme("rgb"),
            textured(),
            Some(UvSource::WorldToTexture(matrix)),
        )
        .unwrap();

        let g = p.triangulate(&mut Cache::new()).unwrap();
        let m = tri_mesh_2d(&g);
        let app = m.appearance().as_ref().unwrap();
        assert!(matches!(
            app.themes()[0].uv_sets[0].uv,
            UvSource::WorldToTexture(out) if out == matrix
        ));
    }

    #[test]
    fn polygon3d_box() {
        let exterior = [
            [0.0, 0.0, 1.0],
            [4.0, 0.0, 1.0],
            [4.0, 4.0, 2.0],
            [0.0, 0.0, 1.0],
        ];
        let p = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            exterior,
            Vec::<Vec<[f64; 3]>>::new(),
        );
        assert_eq!(
            p.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, 0.0, 1.0],
                max: [4.0, 4.0, 2.0]
            }
        );
    }
}
