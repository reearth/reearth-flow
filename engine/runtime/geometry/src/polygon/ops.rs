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

#[cfg(feature = "new-geometry")]
impl crate::predicates::Equal for Polygon2D {
    fn equal(
        &self,
        rhs: &Self,
        tolerance: crate::predicates::Tolerance,
    ) -> crate::predicates::Result<bool> {
        use crate::predicates::equal::{pair_off, ring_curves_2d};

        crate::predicates::require_same_frame(self.frame(), rhs.frame())?;
        // Exterior against exterior, holes against holes; see `Polygon3D`.
        let (here, there) = (self.elevation(), rhs.elevation());
        if !ring_curves_2d(self.exterior(), here)
            .within(&ring_curves_2d(rhs.exterior(), there), tolerance.distance)
        {
            return Ok(false);
        }
        let ours: Vec<_> = self.interiors().map(|r| ring_curves_2d(r, here)).collect();
        let theirs: Vec<_> = rhs.interiors().map(|r| ring_curves_2d(r, there)).collect();
        pair_off(&ours, &theirs, |a, b| Ok(a.within(b, tolerance.distance)))
    }
}

#[cfg(feature = "new-geometry")]
impl crate::predicates::Equal for Polygon3D {
    fn equal(
        &self,
        rhs: &Self,
        tolerance: crate::predicates::Tolerance,
    ) -> crate::predicates::Result<bool> {
        use crate::predicates::equal::{pair_off, Curves};

        // Reprojection stays the caller's explicit step.
        crate::predicates::require_same_frame(self.frame(), rhs.frame())?;
        // Exterior against exterior, holes against holes. Weighing all the rings
        // together as one bag of curves would make a face equal to its
        // ring-inverted twin — the invalid face whose exterior is the other's
        // hole — because the two trace the very same curves.
        if !Curves::from_ring(self.exterior())
            .within(&Curves::from_ring(rhs.exterior()), tolerance.distance)
        {
            return Ok(false);
        }
        // The supporting planes need no separate test: exteriors that stay
        // within `distance` of one another already pin the planes together.
        let ours: Vec<Curves> = self.interiors().map(Curves::from_ring).collect();
        let theirs: Vec<Curves> = rhs.interiors().map(Curves::from_ring).collect();
        pair_off(&ours, &theirs, |a, b| Ok(a.within(b, tolerance.distance)))
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod equal_tests {
    use super::*;
    use crate::collection::Collection3D;
    use crate::predicates::{Equal, PredicateError, Tolerance};
    use crate::GeometryCollection;

    fn tolerance() -> Tolerance {
        Tolerance {
            distance: 1e-9,
            coplanarity: 1e-6,
        }
    }

    fn ring(lo: f64, hi: f64) -> Vec<[f64; 3]> {
        vec![
            [lo, lo, 0.0],
            [hi, lo, 0.0],
            [hi, hi, 0.0],
            [lo, hi, 0.0],
            [lo, lo, 0.0],
        ]
    }

    fn face(exterior: Vec<[f64; 3]>, interiors: Vec<Vec<[f64; 3]>>) -> Polygon3D {
        Polygon3D::from_rings(CoordinateFrame::Euclidean, exterior, interiors)
    }

    #[test]
    fn a_face_is_not_equal_to_its_ring_inverted_twin() {
        let big = ring(0.0, 10.0);
        let small = ring(3.0, 7.0);
        let donut = face(big.clone(), vec![small.clone()]);
        // Invalid, but it reaches the engine: the exterior lies inside the hole.
        let inverted = face(small.clone(), vec![big.clone()]);

        assert!(!donut.equal(&inverted, tolerance()).unwrap());
        assert!(!inverted.equal(&donut, tolerance()).unwrap());
        // Neither is the plain square that traces only one of the two rings.
        assert!(!donut.equal(&face(big, vec![]), tolerance()).unwrap());
        assert!(!donut.equal(&face(small, vec![]), tolerance()).unwrap());
    }

    #[test]
    fn a_face_keeps_its_identity_under_a_vertex_added_on_an_edge() {
        let donut = face(ring(0.0, 10.0), vec![ring(3.0, 7.0)]);
        let mut split = ring(0.0, 10.0);
        split.insert(1, [3.0, 0.0, 0.0]);
        let resplit = face(split, vec![ring(3.0, 7.0)]);

        assert!(donut.equal(&resplit, tolerance()).unwrap());
        assert!(resplit.equal(&donut, tolerance()).unwrap());
    }

    #[test]
    fn holes_pair_off_in_any_order() {
        let big = ring(0.0, 10.0);
        let one = face(big.clone(), vec![ring(1.0, 2.0), ring(5.0, 6.0)]);
        let other = face(big, vec![ring(5.0, 6.0), ring(1.0, 2.0)]);

        assert!(one.equal(&other, tolerance()).unwrap());
    }

    #[test]
    fn a_face_is_not_equal_to_one_with_a_hole_it_lacks() {
        let big = ring(0.0, 10.0);
        assert!(!face(big.clone(), vec![ring(3.0, 7.0)])
            .equal(&face(big, vec![]), tolerance())
            .unwrap());
    }

    #[test]
    fn faces_in_different_frames_are_refused_rather_than_answered() {
        // Reprojection is the caller's explicit step, so a frame mismatch is a
        // question this cannot answer — not a pair of different shapes.
        let outline = ring(0.0, 10.0);
        let here = face(outline.clone(), vec![]);
        let elsewhere = Polygon3D::from_rings(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            outline,
            Vec::<Vec<[f64; 3]>>::new(),
        );

        assert_eq!(
            here.equal(&elsewhere, tolerance()),
            Err(PredicateError::MixedFrames)
        );
    }

    #[test]
    fn a_2d_and_a_3d_geometry_are_refused_rather_than_answered() {
        let flat = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                CoordinateFrame::Euclidean,
                vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]],
                Vec::<Vec<[f64; 2]>>::new(),
            ),
        )));
        let solid = Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(face(
            ring(0.0, 1.0),
            vec![],
        ))));

        assert_eq!(
            flat.equal(&solid, tolerance()),
            Err(PredicateError::CrossDimension)
        );
        assert_eq!(
            solid.equal(&flat, tolerance()),
            Err(PredicateError::CrossDimension)
        );
    }

    fn wrap(face: Polygon3D) -> Euclidean3DGeometry {
        Euclidean3DGeometry::Polygon(Box::new(face))
    }

    fn bag(members: Vec<Euclidean3DGeometry>) -> Euclidean3DGeometry {
        Euclidean3DGeometry::Collection(Collection3D::new(members))
    }

    #[test]
    fn a_collection_denoting_one_geometry_is_that_geometry() {
        let bare = wrap(face(ring(0.0, 1.0), vec![]));
        let wrapped = bag(vec![wrap(face(ring(0.0, 1.0), vec![]))]);
        // Both readings of a one-member collection agree, so it is answered.
        assert!(bare.equal(&wrapped, tolerance()).unwrap());
        assert!(wrapped.equal(&bare, tolerance()).unwrap());
        // And nesting is descended to reach it.
        let nested = bag(vec![bag(vec![wrap(face(ring(0.0, 1.0), vec![]))])]);
        assert!(bare.equal(&nested, tolerance()).unwrap());
    }

    #[test]
    fn a_collection_denoting_two_geometries_is_refused() {
        let two = bag(vec![
            wrap(face(ring(0.0, 1.0), vec![])),
            wrap(face(ring(5.0, 6.0), vec![])),
        ]);
        let bare = wrap(face(ring(0.0, 1.0), vec![]));

        assert_eq!(
            bare.equal(&two, tolerance()),
            Err(PredicateError::Unsupported {
                geometry: "Collection3D"
            })
        );
        // Refused from either side, and against another collection too.
        assert_eq!(
            two.equal(&bare, tolerance()),
            Err(PredicateError::Unsupported {
                geometry: "Collection3D"
            })
        );
    }

    #[test]
    fn collections_denoting_nothing_occupy_the_same_nothing() {
        let empty = bag(vec![]);
        let nested_empty = bag(vec![bag(vec![])]);
        let something = wrap(face(ring(0.0, 1.0), vec![]));

        assert!(empty.equal(&nested_empty, tolerance()).unwrap());
        assert!(!empty.equal(&something, tolerance()).unwrap());
        assert!(!something.equal(&empty, tolerance()).unwrap());
    }

    #[test]
    fn a_heterogeneous_collection_denoting_one_geometry_is_that_geometry() {
        let bare = Geometry::Euclidean3D(wrap(face(ring(0.0, 1.0), vec![])));
        let wrapped = Geometry::GeometryCollection(GeometryCollection::new([bare.clone()]));

        assert!(bare.equal(&wrapped, tolerance()).unwrap());

        let two = Geometry::GeometryCollection(GeometryCollection::new([
            bare.clone(),
            Geometry::Euclidean3D(wrap(face(ring(5.0, 6.0), vec![]))),
        ]));
        assert_eq!(
            bare.equal(&two, tolerance()),
            Err(PredicateError::Unsupported {
                geometry: "GeometryCollection"
            })
        );
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
