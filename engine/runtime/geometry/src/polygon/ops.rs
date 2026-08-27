use super::{Polygon2D, Polygon3D};
use crate::coordinate::{CoordinateFrame, EpsgCode};
use crate::ops::reproject::{transform_coords_2d, transform_coords_3d};
use crate::ops::triangulation::{expand_appearance, triangulate_2d, triangulate_3d, Cache};
use crate::ops::{
    lift_coords, Aabb, BoundingBox, Reproject, ReprojectionCache, Triangulate, UnsupportedOperation,
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

impl Polygon2D {
    /// Move the face out, leaving an empty husk.
    fn take(&mut self) -> Self {
        Self {
            frame: std::mem::take(&mut self.frame),
            coords: std::mem::take(&mut self.coords),
            interior_offsets: std::mem::take(&mut self.interior_offsets),
            z: self.z.take(),
            appearance: self.appearance.take(),
        }
    }

    /// The leaf moved out and wrapped as a [`Geometry`].
    fn take_geometry(&mut self) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(self.take())))
    }
}

impl Polygon3D {
    /// Move the face out, leaving an empty husk.
    fn take(&mut self) -> Self {
        Self {
            frame: std::mem::take(&mut self.frame),
            coords: std::mem::take(&mut self.coords),
            interior_offsets: std::mem::take(&mut self.interior_offsets),
            appearance: self.appearance.take(),
        }
    }

    /// The leaf moved out and wrapped as a [`Geometry`].
    fn take_geometry(&mut self) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(self.take())))
    }
}

impl Reproject for Polygon2D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        let from = self.frame.require_crs()?;
        if from == target {
            return Ok(self.take_geometry());
        }
        if self.z.is_some() {
            return self.take().into_3d().reproject(target, cache);
        }
        let mut p = self.take();
        transform_coords_2d(cache, from, target, &mut p.coords)?;
        p.frame = CoordinateFrame::Crs(target);
        Ok(Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(
            Box::new(p),
        )))
    }
}

impl Reproject for Polygon3D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        let from = self.frame.require_crs()?;
        let mut p = self.take();
        if from != target {
            transform_coords_3d(cache, from, target, &mut p.coords)?;
            p.frame = CoordinateFrame::Crs(target);
        }
        Ok(Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(
            Box::new(p),
        )))
    }
}

use crate::ops::{plan_frame_step, translate_2d, translate_3d, ConvertFrame, FrameStep, Translate};

impl Translate for Polygon2D {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        translate_2d(&mut self.coords, &mut self.z, delta);
        Ok(())
    }
}

impl Translate for Polygon3D {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        translate_3d(&mut self.coords, delta);
        Ok(())
    }
}

impl ConvertFrame for Polygon2D {
    fn convert_frame(
        &mut self,
        target: &CoordinateFrame,
        base_point: Option<[f64; 3]>,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        match plan_frame_step(&self.frame, target, base_point)? {
            FrameStep::Noop => Ok(self.take_geometry()),
            FrameStep::Reproject(to) => self.reproject(to, cache),
            FrameStep::Translate(offset, frame) => {
                self.translate(offset)?;
                self.frame = frame;
                Ok(self.take_geometry())
            }
        }
    }
}

impl ConvertFrame for Polygon3D {
    fn convert_frame(
        &mut self,
        target: &CoordinateFrame,
        base_point: Option<[f64; 3]>,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        match plan_frame_step(&self.frame, target, base_point)? {
            FrameStep::Noop => Ok(self.take_geometry()),
            FrameStep::Reproject(to) => self.reproject(to, cache),
            FrameStep::Translate(offset, frame) => {
                self.translate(offset)?;
                self.frame = frame;
                Ok(self.take_geometry())
            }
        }
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
        let mut mesh = unsafe {
            match self.z {
                None => TriangularMesh2D::from_parts_unchecked(
                    self.frame.clone(),
                    verts,
                    buffers.out.len() / 3,
                    buffers.out.iter().copied(),
                ),
                Some(elevation) => TriangularMesh2D::from_parts_at_elevation_unchecked(
                    self.frame.clone(),
                    verts,
                    buffers.out.len() / 3,
                    buffers.out.iter().copied(),
                    elevation,
                ),
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

use crate::ops::{ForceTwoDimension, ForceTwoDimensionError};

impl ForceTwoDimension for Polygon2D {
    fn force_2d(&mut self) -> Result<Euclidean2DGeometry, ForceTwoDimensionError> {
        let frame = self.frame.demote_to_2d()?;
        self.z = None; // drop any 2.5D elevation; rings and appearance carry over
        Ok(Euclidean2DGeometry::Polygon(Box::new(Polygon2D {
            frame,
            coords: std::mem::take(&mut self.coords),
            interior_offsets: std::mem::take(&mut self.interior_offsets),
            z: None,
            appearance: self.appearance.take(),
        })))
    }
}

impl ForceTwoDimension for Polygon3D {
    fn force_2d(&mut self) -> Result<Euclidean2DGeometry, ForceTwoDimensionError> {
        // Only the coordinate embedding and the frame's dimensionality change;
        // ring offsets and appearance (including per-corner UV, which is not
        // indexed by Z) carry over.
        let frame = self.frame.demote_to_2d()?;
        let coords = std::mem::take(&mut self.coords)
            .iter()
            .map(|&[x, y, _]| [x, y])
            .collect();
        Ok(Euclidean2DGeometry::Polygon(Box::new(Polygon2D {
            frame,
            coords,
            interior_offsets: std::mem::take(&mut self.interior_offsets),
            z: None,
            appearance: self.appearance.take(),
        })))
    }
}

impl Polygon2D {
    /// The 3D counterpart of this leaf, with every coordinate placed at the
    /// elevation the leaf lies at, or at `0.0` when it carries none.
    pub(crate) fn into_3d(self) -> Polygon3D {
        Polygon3D {
            frame: self.frame,
            coords: lift_coords(self.coords.iter(), self.z).into_boxed_slice(),
            interior_offsets: self.interior_offsets,
            appearance: self.appearance,
        }
    }
}

use crate::ops::{
    emit_face_2d, emit_face_3d, CountHoles, ExtractHoles, ExtractedPart, RemoveAppearance,
};

// One offset per interior ring, so the count is the offsets' length.
impl CountHoles for Polygon2D {
    fn count_holes(&self) -> usize {
        self.interior_offsets.len()
    }
}

impl CountHoles for Polygon3D {
    fn count_holes(&self) -> usize {
        self.interior_offsets.len()
    }
}

// A face with no exterior ring bounds no area, so it is not area geometry to
// take apart — the one case where a polygon itself is rejected.
impl ExtractHoles for Polygon2D {
    fn extract_holes(
        &self,
        emit: &mut dyn FnMut(Geometry, ExtractedPart),
    ) -> Result<(), UnsupportedOperation> {
        if emit_face_2d(self, emit) {
            Ok(())
        } else {
            Err(UnsupportedOperation {
                geometry: "Polygon2D",
                operation: "extract_holes",
            })
        }
    }
}

impl ExtractHoles for Polygon3D {
    fn extract_holes(
        &self,
        emit: &mut dyn FnMut(Geometry, ExtractedPart),
    ) -> Result<(), UnsupportedOperation> {
        if emit_face_3d(self, emit) {
            Ok(())
        } else {
            Err(UnsupportedOperation {
                geometry: "Polygon3D",
                operation: "extract_holes",
            })
        }
    }
}

impl RemoveAppearance for Polygon2D {
    fn remove_appearance(&mut self) {
        *self.appearance_mut() = None;
    }
}

impl RemoveAppearance for Polygon3D {
    fn remove_appearance(&mut self) {
        *self.appearance_mut() = None;
    }
}

use crate::ops::coerce::{push_face_lines_2d, push_face_lines_3d, unchanged, wrap_2d, wrap_3d};
use crate::ops::{Coerce, CoercionTarget};

impl Coerce for Polygon2D {
    fn coerce(
        &mut self,
        target: CoercionTarget,
        cache: &mut Cache,
    ) -> Result<Geometry, UnsupportedOperation> {
        match target {
            CoercionTarget::Polygon => Err(unchanged::<Self>()),
            CoercionTarget::TriangularMesh => self.triangulate(cache),
            CoercionTarget::LineString => {
                let mut lines = Vec::new();
                push_face_lines_2d(self, &mut lines);
                wrap_2d(lines).ok_or_else(unchanged::<Self>)
            }
        }
    }
}

impl Coerce for Polygon3D {
    fn coerce(
        &mut self,
        target: CoercionTarget,
        cache: &mut Cache,
    ) -> Result<Geometry, UnsupportedOperation> {
        match target {
            CoercionTarget::Polygon => Err(unchanged::<Self>()),
            CoercionTarget::TriangularMesh => self.triangulate(cache),
            CoercionTarget::LineString => {
                let mut lines = Vec::new();
                push_face_lines_3d(self, &mut lines);
                wrap_3d(lines).ok_or_else(unchanged::<Self>)
            }
        }
    }
}

#[cfg(feature = "new-geometry")]
use crate::ops::{Footprint, FootprintError, FootprintSink};

#[cfg(feature = "new-geometry")]
impl Footprint for Polygon2D {
    fn footprint(&self, sink: &mut FootprintSink<'_>) -> Result<(), FootprintError> {
        sink.enter(self.frame())?;
        sink.push_face_2d(
            std::iter::once(self.exterior()).chain(self.interiors()),
            self.elevation(),
        );
        Ok(())
    }
}

#[cfg(feature = "new-geometry")]
impl Footprint for Polygon3D {
    fn footprint(&self, sink: &mut FootprintSink<'_>) -> Result<(), FootprintError> {
        sink.enter(self.frame())?;
        sink.push_face_3d(std::iter::once(self.exterior()).chain(self.interiors()));
        Ok(())
    }
}

#[cfg(feature = "new-geometry")]
use crate::ops::area::polygon_3d_surface_area;
#[cfg(feature = "new-geometry")]
use crate::ops::Area;

#[cfg(feature = "new-geometry")]
impl Area for Polygon2D {
    /// A 2D face has no elevation to slope, so its area is simply its planar
    /// area.
    fn surface_area(&self) -> Result<f64, UnsupportedOperation> {
        Ok(self.area())
    }
}

#[cfg(feature = "new-geometry")]
impl Area for Polygon3D {
    fn surface_area(&self) -> Result<f64, UnsupportedOperation> {
        Ok(polygon_3d_surface_area(self))
    }
}

use crate::ops::boundary::{unsupported as unbounded, Boundary, ExtractBoundary};

// A face is bounded by its own rings, exterior first, each kept verbatim. A face
// with no exterior ring encloses nothing, so there is nothing to bound. That is
// the one case where a face itself has no boundary to give.
impl ExtractBoundary for Polygon2D {
    fn extract_boundary(&self) -> Result<Boundary, UnsupportedOperation> {
        let mut lines = Vec::new();
        push_face_lines_2d(self, &mut lines);
        wrap_2d(lines)
            .map(Boundary::Bounded)
            .ok_or_else(unbounded::<Self>)
    }
}

impl ExtractBoundary for Polygon3D {
    fn extract_boundary(&self) -> Result<Boundary, UnsupportedOperation> {
        let mut lines = Vec::new();
        push_face_lines_3d(self, &mut lines);
        wrap_3d(lines)
            .map(Boundary::Bounded)
            .ok_or_else(unbounded::<Self>)
    }
}

#[cfg(feature = "new-geometry")]
use crate::ops::Elevation;

#[cfg(feature = "new-geometry")]
impl Elevation for Polygon2D {
    fn elevation(&self) -> Option<f64> {
        self.z
    }
}

#[cfg(feature = "new-geometry")]
impl Elevation for Polygon3D {
    /// The exterior ring's first vertex; the holes lie inside it and are not
    /// reached.
    fn elevation(&self) -> Option<f64> {
        self.exterior().first().map(|c| c[2])
    }
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
        let g = Polygon2D::from_rings_at_elevation(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 0.0]],
            Vec::<Vec<[f64; 2]>>::new(),
            10.0,
        )
        .triangulate(&mut Cache::new())
        .unwrap();
        // A 2.5D polygon stays a 2D mesh, tessellated at the same one elevation.
        assert!(matches!(g, Geometry::Euclidean2D(_)));
        let m = tri_mesh_2d(&g);
        assert_eq!(m.num_triangles(), 1);
        assert_eq!(m.elevation(), Some(10.0));
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

#[cfg(feature = "new-geometry")]
mod grid_impl {
    //! `DivideByGrid` for the two polygon leaves.
    //!
    //! A polygon's rings are stored **closed** when well-formed (first ==
    //! last); `ops::grid`'s half-plane clip assumes **open** rings, so the
    //! closing duplicate is stripped on the way in, the same way
    //! `open_ring_positions` (above, in this file) strips it for
    //! triangulation. On the way out, rings are carried through verbatim --
    //! neither re-closed nor re-wound -- matching how `ExtractHoles`
    //! (`ops/hole.rs`) treats algorithmically split rings elsewhere in this
    //! crate.
    //!
    //! Appearance is rebuilt per piece rather than cloned, because a piece's
    //! corner count generally differs from its source's. `Corner` (from
    //! `ops::grid`) carries exactly one UV channel through the clip, so only
    //! the default theme's front, default-channel `Explicit` UV can be
    //! threaded this way; see [`rebuild_appearance`] for what happens to any
    //! other UV set the source appearance carries.

    use super::{Polygon2D, Polygon3D};
    use crate::appearance::{Appearance, ChannelId, Side, ThemeBinding, UvSet, UvSource};
    use crate::ops::grid::{
        clip_to_window, faces_area_xy, CellCoverage, Corner, DivideByGrid, Face, GridCell,
        GridDivideError, GridSpec,
    };
    use crate::ops::{Aabb, BoundingBox};
    use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

    /// The polygon's default-theme, front-side, default-channel UV, gathered
    /// into a flat array parallel to its corner buffer when that UV is
    /// `Explicit` -- the one UV channel a [`Corner`] can carry through the
    /// clip. `WorldToTexture` UV is positional, not per-corner, so there is
    /// nothing to gather; [`rebuild_appearance`] carries it through unchanged
    /// instead.
    fn explicit_uv(appearance: &Option<Appearance>) -> Option<Vec<[f64; 2]>> {
        match appearance.as_ref()?.default_uv()? {
            UvSource::Explicit(coords) => Some(coords.to_vec()),
            UvSource::WorldToTexture(_) => None,
        }
    }

    /// Rebuild an appearance for one clipped piece, whose corner count
    /// generally differs from its source's.
    ///
    /// The default theme's front, default-channel `Explicit` UV -- the one
    /// [`explicit_uv`] threaded through the clip as each [`Corner`]'s `uv` --
    /// is replaced by `gathered_uv`, already parallel to the piece's own
    /// corner buffer (or dropped, if the clip did not yield a UV for every
    /// corner). Every *other* `Explicit` UV set the source might carry -- a
    /// distinct theme, a back side, a non-default channel -- cannot be
    /// re-derived this way, since `Corner` only carries one UV channel
    /// through the clip; rather than leave it at its stale, now-mismatched
    /// length (an invariant violation that could panic a later consumer,
    /// e.g. `Triangulate`'s UV re-gather), it is dropped. `WorldToTexture`
    /// needs no adjustment -- it is positional, not per-corner -- and carries
    /// over as-is regardless of theme.
    fn rebuild_appearance(
        src: &Option<Appearance>,
        gathered_uv: Option<&[[f64; 2]]>,
    ) -> Option<Appearance> {
        let app = src.as_ref()?;
        let default_theme = app.default_theme().clone();
        let (materials, themes, _) = app.clone().into_parts();

        let themes = themes
            .into_iter()
            .map(|theme| {
                let ThemeBinding {
                    theme: theme_id,
                    front,
                    back,
                    uv_sets,
                } = theme;
                let is_default_theme = theme_id == default_theme;
                let uv_sets = uv_sets
                    .into_iter()
                    .filter_map(|uv_set| {
                        let is_default_slot = is_default_theme
                            && uv_set.side == Side::Front
                            && uv_set.channel == ChannelId::default();
                        match &uv_set.uv {
                            UvSource::WorldToTexture(_) => Some(uv_set),
                            UvSource::Explicit(_) if is_default_slot => {
                                let coords: Box<[[f64; 2]]> = gathered_uv?.into();
                                Some(UvSet {
                                    uv: UvSource::Explicit(coords),
                                    ..uv_set
                                })
                            }
                            UvSource::Explicit(_) => None,
                        }
                    })
                    .collect();
                ThemeBinding {
                    theme: theme_id,
                    front,
                    back,
                    uv_sets,
                }
            })
            .collect();

        Some(Appearance::from_parts(materials, themes, default_theme))
    }

    macro_rules! divide_polygon {
        ($ty:ident, $dim:literal, $module:ident, $wrap:path) => {
            mod $module {
                use super::*;

                impl DivideByGrid for $ty {
                    fn divide_by_grid(
                        &self,
                        grid: &GridSpec,
                        emit: &mut dyn FnMut(GridCell, CellCoverage, Geometry),
                    ) -> Result<(), GridDivideError> {
                        let (min, max) = match self.bounding_box()? {
                            Aabb::D2 { min, max } => (min, max),
                            Aabb::D3 { min, max } => ([min[0], min[1]], [max[0], max[1]]),
                        };

                        let uv = explicit_uv(self.appearance());
                        let rings = rings_of(self, uv.as_deref());
                        if rings.is_empty() {
                            return Err(GridDivideError::Empty);
                        }

                        let (lo, hi) = grid.cell_range(min, max);

                        // Row-major, so output order is defined and reproducible.
                        for row in lo.row..=hi.row {
                            for col in lo.col..=hi.col {
                                let cell = GridCell { row, col };
                                let window = grid.window(cell);
                                let faces = clip_to_window(rings.clone(), &window);
                                if faces.is_empty() {
                                    continue;
                                }
                                let coverage =
                                    CellCoverage::from_area(faces_area_xy(&faces), window.area());
                                let geom = rebuild(self, faces);
                                emit(cell, coverage, geom);
                            }
                        }
                        Ok(())
                    }
                }

                /// Every ring (exterior, then holes) as open `Corner` rings.
                ///
                /// A well-formed ring is stored closed (first == last); the
                /// closing duplicate is stripped here. The cursor still
                /// advances by each ring's full *stored* length (closing
                /// duplicate included), so a later ring's UV indices stay
                /// aligned with `coords` -- `uv`'s own layout.
                fn rings_of(p: &$ty, uv: Option<&[[f64; 2]]>) -> Vec<Vec<Corner<$dim>>> {
                    let mut out = Vec::new();
                    let mut cursor = 0usize;
                    for ring in std::iter::once(p.exterior()).chain(p.interiors()) {
                        let open_len = if ring.len() >= 2 && ring[0] == ring[ring.len() - 1] {
                            ring.len() - 1
                        } else {
                            ring.len()
                        };
                        if open_len >= 3 {
                            let corners = ring[..open_len]
                                .iter()
                                .enumerate()
                                .map(|(i, pos)| Corner {
                                    pos: *pos,
                                    uv: uv.and_then(|u| u.get(cursor + i).copied()),
                                })
                                .collect::<Vec<_>>();
                            out.push(corners);
                        }
                        cursor += ring.len();
                    }
                    out
                }

                /// Rebuild one clipped face as a polygon in the source's frame.
                fn build_one(src: &$ty, face: Face<$dim>) -> $ty {
                    let mut rings = face.rings.into_iter();
                    let exterior_corners = rings.next().unwrap_or_default();
                    let interior_corners: Vec<Vec<Corner<$dim>>> = rings.collect();

                    // Every corner must carry a UV for the gather to be
                    // meaningful; `Option<Vec<_>>: FromIterator<Option<_>>`
                    // short-circuits to `None` the moment one is missing.
                    let gathered_uv: Option<Vec<[f64; 2]>> = std::iter::once(&exterior_corners)
                        .chain(interior_corners.iter())
                        .flat_map(|ring| ring.iter())
                        .map(|c| c.uv)
                        .collect();

                    let exterior: Vec<[f64; $dim]> =
                        exterior_corners.iter().map(|c| c.pos).collect();
                    let interiors: Vec<Vec<[f64; $dim]>> = interior_corners
                        .iter()
                        .map(|r| r.iter().map(|c| c.pos).collect())
                        .collect();

                    let mut poly = $ty::from_rings(src.frame().clone(), exterior, interiors);
                    *poly.appearance_mut() =
                        rebuild_appearance(src.appearance(), gathered_uv.as_deref());
                    poly
                }

                /// Rebuild the clipped faces as geometry. Several faces
                /// become a collection; one stays a face, so the common case
                /// does not gain a wrapper.
                fn rebuild(src: &$ty, faces: Vec<Face<$dim>>) -> Geometry {
                    $wrap(faces.into_iter().map(|face| build_one(src, face)).collect())
                }
            }
        };
    }

    divide_polygon!(Polygon2D, 2, polygon_2d_grid, wrap_2d);
    divide_polygon!(Polygon3D, 3, polygon_3d_grid, wrap_3d);

    fn wrap_2d(mut built: Vec<Polygon2D>) -> Geometry {
        if built.len() == 1 {
            return Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(built.remove(0))));
        }
        Geometry::Euclidean2D(Euclidean2DGeometry::Collection(
            crate::collection::Collection2D::new(
                built
                    .into_iter()
                    .map(|p| Euclidean2DGeometry::Polygon(Box::new(p))),
            ),
        ))
    }

    fn wrap_3d(mut built: Vec<Polygon3D>) -> Geometry {
        if built.len() == 1 {
            return Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(built.remove(0))));
        }
        Geometry::Euclidean3D(Euclidean3DGeometry::Collection(
            crate::collection::Collection3D::new(
                built
                    .into_iter()
                    .map(|p| Euclidean3DGeometry::Polygon(Box::new(p))),
            ),
        ))
    }
}
