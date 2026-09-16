//! Footprints: the projection of a geometry onto a plane.
//!
//! [`Footprint`] projects a leaf's coordinates along a plane's normal onto that
//! plane into a [`FootprintSink`], which dissolves the projected faces into their
//! point-set union and assembles the result as 2D geometry in the plane's frame.
//! Curves and points project as they are.

use crate::collection::Collection2D;
use crate::coordinate::{
    BaseFrame, CoordinateFrame, FrameDemotionError, TangentPlane, TangentPlaneError, UnitKind,
};
use crate::line_string::LineString2D;
use crate::ops::UnsupportedOperation;
use crate::overlay::dissolve_shapes;
use crate::point::Point2D;
use crate::validation_next::{open_ring, signed_area_2d};
use crate::{Euclidean2DGeometry, Geometry};

/// The plane a footprint is cast onto. Every variant needs the geometry in a
/// frame with linear units.
#[derive(Clone, Debug, PartialEq)]
pub enum FootprintPlane {
    /// The horizontal plane of the geometry's own frame: coordinates keep their
    /// `(x, y)` and drop `z`, and the frame is demoted to its 2D counterpart
    /// (see [`CoordinateFrame::demote_to_2d`]).
    Horizontal,
    /// An arbitrary plane anchored in the geometry's frame: coordinates become
    /// in-plane `(x, y)` (see [`TangentPlane::project`]) and the result is
    /// tagged [`CoordinateFrame::Tangent`]. The plane's `base` must be the
    /// geometry's frame.
    Tangent(TangentPlane),
    /// The plane through `origin` with the given `normal`, anchored in the
    /// geometry's own frame; see [`TangentPlane::from_normal`] for the in-plane
    /// axes. Projects as [`FootprintPlane::Tangent`] once the base frame is read
    /// from the geometry.
    Normal {
        /// Plane origin, in the geometry's frame.
        origin: [f64; 3],
        /// Plane normal, in the geometry's frame; any non-zero length.
        normal: [f64; 3],
        /// Optional direction whose in-plane component becomes the `x` axis.
        x_axis: Option<[f64; 3]>,
    },
}

/// Why a footprint could not be computed.
#[derive(Clone, Debug, PartialEq)]
pub enum FootprintError {
    /// A leaf type with no footprint (`PointCloud`, an unevaluated `Csg`).
    Unsupported(UnsupportedOperation),
    /// The geometry has no coordinates to project, or every face projected to a
    /// degenerate area.
    Empty,
    /// The geometry's CRS has no 2D counterpart to tag a horizontal footprint
    /// with.
    Frame(FrameDemotionError),
    /// The leaves do not all project into one frame.
    MixedFrames,
    /// The tangent plane is anchored in a frame other than the geometry's.
    PlaneBaseMismatch {
        /// The plane's anchor frame.
        plane: BaseFrame,
        /// The geometry's frame.
        geometry: CoordinateFrame,
    },
    /// The geometry's frame is not in linear units. Carries the reason (angular,
    /// or why the units could not be determined).
    NonLinearFrame(String),
    /// The tangent plane is degenerate.
    Plane(TangentPlaneError),
}

impl core::fmt::Display for FootprintError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FootprintError::Unsupported(e) => e.fmt(f),
            FootprintError::Empty => write!(f, "geometry has no area, curve or point to project"),
            FootprintError::Frame(e) => e.fmt(f),
            FootprintError::MixedFrames => {
                write!(f, "geometry members do not project into one frame")
            }
            FootprintError::PlaneBaseMismatch { plane, geometry } => write!(
                f,
                "plane is anchored in {plane:?} but the geometry is in {geometry:?}"
            ),
            FootprintError::NonLinearFrame(why) => {
                write!(f, "footprint needs a frame in linear units: {why}")
            }
            FootprintError::Plane(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for FootprintError {}

impl From<UnsupportedOperation> for FootprintError {
    fn from(e: UnsupportedOperation) -> Self {
        FootprintError::Unsupported(e)
    }
}

impl From<FrameDemotionError> for FootprintError {
    fn from(e: FrameDemotionError) -> Self {
        FootprintError::Frame(e)
    }
}

impl From<TangentPlaneError> for FootprintError {
    fn from(e: TangentPlaneError) -> Self {
        FootprintError::Plane(e)
    }
}

/// Project a geometry onto a plane, feeding a [`FootprintSink`].
///
/// A leaf enters the sink with its frame, then pushes every face (rings, outer
/// first), curve and point. A 2D leaf is lifted at its elevation (`0` when it
/// has none) before projecting. Containers recurse into their members; a member
/// that cannot be projected fails the whole geometry.
#[enum_dispatch::enum_dispatch]
pub trait Footprint {
    /// Project this geometry into `sink`. The default body reports the type as
    /// unsupported.
    fn footprint(&self, sink: &mut FootprintSink<'_>) -> Result<(), FootprintError> {
        let _ = sink;
        Err(UnsupportedOperation {
            geometry: core::any::type_name::<Self>(),
            operation: "footprint",
        }
        .into())
    }
}

impl<T: Footprint + ?Sized> Footprint for Box<T> {
    fn footprint(&self, sink: &mut FootprintSink<'_>) -> Result<(), FootprintError> {
        (**self).footprint(sink)
    }
}

/// The smallest area, in the frame's linear unit squared, a projected ring must
/// have to take part in the footprint. Applied per ring.
const MIN_FACE_AREA: f64 = 1e-6;

/// The area a projected open ring encloses, signed by its winding.
fn ring_area(ring: &[[f64; 2]]) -> f64 {
    signed_area_2d(ring) / 2.0
}

/// The projection in effect once the first leaf has fixed the base frame.
enum Projection {
    /// Drop `z`.
    Horizontal,
    /// In-plane coordinates on the plane.
    Tangent(TangentPlane),
}

/// The projected parts of a geometry, accumulated leaf by leaf, then dissolved
/// by [`finish`](FootprintSink::finish).
pub struct FootprintSink<'a> {
    plane: &'a FootprintPlane,
    /// The projection, resolved by the first leaf.
    projection: Option<Projection>,
    /// The output frame, fixed by the first leaf.
    frame: Option<CoordinateFrame>,
    /// Projected faces awaiting dissolution: each a ring list, outer first,
    /// wound to Flow's convention.
    shapes: Vec<Vec<Vec<[f64; 2]>>>,
    curves: Vec<Vec<[f64; 2]>>,
    points: Vec<[f64; 2]>,
}

impl<'a> FootprintSink<'a> {
    /// An empty sink projecting onto `plane`.
    pub fn new(plane: &'a FootprintPlane) -> Self {
        Self {
            plane,
            projection: None,
            frame: None,
            shapes: Vec::new(),
            curves: Vec::new(),
            points: Vec::new(),
        }
    }

    /// Dissolve the projected faces and assemble the footprint: the single part
    /// when there is exactly one, otherwise a [`Collection2D`] of the parts
    /// (areas first, then curves, then points). A face whose exterior projected
    /// to under [`MIN_FACE_AREA`] square units (such as a wall onto the
    /// horizontal plane) was dropped on entry, so a geometry of nothing but such
    /// faces is [`FootprintError::Empty`].
    pub fn finish(self) -> Result<Geometry, FootprintError> {
        let Some(frame) = self.frame else {
            return Err(FootprintError::Empty);
        };
        let mut parts: Vec<Euclidean2DGeometry> = dissolve_shapes(self.shapes, &frame)
            .into_iter()
            .map(|polygon| Euclidean2DGeometry::Polygon(Box::new(polygon)))
            .collect();
        parts.extend(self.curves.into_iter().map(|coords| {
            Euclidean2DGeometry::LineString(LineString2D::from_coords(frame.clone(), coords))
        }));
        parts.extend(
            self.points
                .into_iter()
                .map(|p| Euclidean2DGeometry::Point(Point2D::new(frame.clone(), p))),
        );
        match parts.len() {
            0 => Err(FootprintError::Empty),
            1 => Ok(Geometry::Euclidean2D(parts.pop().expect("one part"))),
            _ => Ok(Geometry::Euclidean2D(Euclidean2DGeometry::Collection(
                Collection2D::new(parts),
            ))),
        }
    }

    /// Fix or check the projection and output frame for a leaf in `frame`. Every
    /// leaf calls this before pushing.
    pub(crate) fn enter(&mut self, frame: &CoordinateFrame) -> Result<(), FootprintError> {
        if self.projection.is_none() {
            self.projection = Some(self.resolve(frame)?);
        }
        let target = match self.projection.as_ref().expect("resolved above") {
            Projection::Horizontal => frame.demote_to_2d()?,
            Projection::Tangent(plane) => {
                let base_matches = match (&plane.base, frame) {
                    (BaseFrame::Euclidean, CoordinateFrame::Euclidean) => true,
                    (BaseFrame::Crs(a), CoordinateFrame::Crs(b)) => a == b,
                    _ => false,
                };
                if !base_matches {
                    return Err(FootprintError::PlaneBaseMismatch {
                        plane: plane.base,
                        geometry: frame.clone(),
                    });
                }
                CoordinateFrame::Tangent(Box::new(plane.clone()))
            }
        };
        match &self.frame {
            None => self.frame = Some(target),
            Some(current) if *current == target => {}
            Some(_) => return Err(FootprintError::MixedFrames),
        }
        Ok(())
    }

    /// The projection for the first leaf, in `frame`, which must be in linear
    /// units.
    fn resolve(&self, frame: &CoordinateFrame) -> Result<Projection, FootprintError> {
        match frame.unit_kind() {
            UnitKind::Linear => {}
            UnitKind::Angular => {
                return Err(FootprintError::NonLinearFrame(
                    "frame is in angular units".to_string(),
                ))
            }
            UnitKind::Undeterminable(why) => return Err(FootprintError::NonLinearFrame(why)),
        }
        Ok(match self.plane {
            FootprintPlane::Horizontal => Projection::Horizontal,
            FootprintPlane::Tangent(plane) => Projection::Tangent(plane.clone()),
            FootprintPlane::Normal {
                origin,
                normal,
                x_axis,
            } => {
                let base = match frame {
                    CoordinateFrame::Euclidean => BaseFrame::Euclidean,
                    CoordinateFrame::Crs(epsg) => BaseFrame::Crs(*epsg),
                    CoordinateFrame::Tangent(plane) => {
                        return Err(FootprintError::PlaneBaseMismatch {
                            plane: plane.base,
                            geometry: frame.clone(),
                        })
                    }
                };
                Projection::Tangent(TangentPlane::from_normal(base, *origin, *normal, *x_axis)?)
            }
        })
    }

    #[inline]
    fn project(&self, p: [f64; 3]) -> [f64; 2] {
        match self.projection.as_ref().expect("a leaf was entered") {
            Projection::Horizontal => [p[0], p[1]],
            Projection::Tangent(plane) => plane.project(p),
        }
    }

    /// Add one 3D face given its rings, outer first. A face whose projection is
    /// degenerate is dropped.
    pub(crate) fn push_face_3d<'r>(&mut self, rings: impl Iterator<Item = &'r [[f64; 3]]>) {
        let projected: Vec<Vec<[f64; 2]>> = rings
            .map(|ring| open_ring(ring).iter().map(|&p| self.project(p)).collect())
            .collect();
        self.push_face(projected);
    }

    /// Add one 2D face given its rings, outer first, lifted at `elevation`.
    pub(crate) fn push_face_2d<'r>(
        &mut self,
        rings: impl Iterator<Item = &'r [[f64; 2]]>,
        elevation: Option<f64>,
    ) {
        let z = elevation.unwrap_or(0.0);
        let projected: Vec<Vec<[f64; 2]>> = rings
            .map(|ring| {
                open_ring(ring)
                    .iter()
                    .map(|&[x, y]| self.project([x, y, z]))
                    .collect()
            })
            .collect();
        self.push_face(projected);
    }

    /// Add one face from its projected open rings, outer first, each wound to
    /// Flow's convention (outer CCW, holes CW). A ring under [`MIN_FACE_AREA`]
    /// is dropped: the whole face when it is the exterior, that hole alone
    /// otherwise.
    fn push_face(&mut self, rings: Vec<Vec<[f64; 2]>>) {
        let mut rings = rings.into_iter();
        let Some(mut outer) = rings.next() else {
            return;
        };
        let outer_area = ring_area(&outer);
        if outer_area.abs() <= MIN_FACE_AREA {
            return;
        }
        if outer_area < 0.0 {
            outer.reverse();
        }
        let mut face = Vec::with_capacity(rings.len() + 1);
        face.push(outer);
        for mut hole in rings {
            let area = ring_area(&hole);
            if area.abs() <= MIN_FACE_AREA {
                continue;
            }
            if area > 0.0 {
                hole.reverse();
            }
            face.push(hole);
        }
        self.shapes.push(face);
    }

    /// Add one 3D curve.
    pub(crate) fn push_curve_3d(&mut self, coords: &[[f64; 3]]) {
        if !coords.is_empty() {
            self.curves
                .push(coords.iter().map(|&p| self.project(p)).collect());
        }
    }

    /// Add one 2D curve lifted at `elevation`.
    pub(crate) fn push_curve_2d(&mut self, coords: &[[f64; 2]], elevation: Option<f64>) {
        let z = elevation.unwrap_or(0.0);
        if !coords.is_empty() {
            self.curves.push(
                coords
                    .iter()
                    .map(|&[x, y]| self.project([x, y, z]))
                    .collect(),
            );
        }
    }

    /// Add one 3D point.
    pub(crate) fn push_point_3d(&mut self, p: [f64; 3]) {
        self.points.push(self.project(p));
    }

    /// Add one 2D point at height `0`.
    pub(crate) fn push_point_2d(&mut self, [x, y]: [f64; 2]) {
        self.points.push(self.project([x, y, 0.0]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection3D;
    use crate::coordinate::EpsgCode;
    use crate::polygon::{Polygon2D, Polygon3D};
    use crate::predicates::test3d::{box_shell, e, g3, solid_geometry};
    use crate::solid::{Shell, Solid};
    use crate::Euclidean3DGeometry;

    /// The single polygon a footprint result must be.
    fn single_polygon(g: Geometry) -> Polygon2D {
        match g {
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(p)) => *p,
            other => panic!("expected one polygon, got {other:?}"),
        }
    }

    fn square_3d(frame: CoordinateFrame, min: [f64; 2], side: f64, z: f64, ccw: bool) -> Geometry {
        let [x, y] = min;
        let mut ring = vec![
            [x, y, z],
            [x + side, y, z],
            [x + side, y + side, z],
            [x, y + side, z],
            [x, y, z],
        ];
        if !ccw {
            ring.reverse();
        }
        let face = Polygon3D::from_rings(frame, ring, Vec::<Vec<[f64; 3]>>::new());
        g3(Euclidean3DGeometry::Polygon(Box::new(face)))
    }

    fn box_solid(frame: CoordinateFrame, min: [f64; 3], size: [f64; 3]) -> Geometry {
        let solid = Solid::from_exterior(frame, Shell::TriangularMesh(box_shell(min, size)));
        g3(solid_geometry(solid))
    }

    #[test]
    fn horizontal_footprint_of_a_box_is_its_base() {
        let solid = box_solid(
            CoordinateFrame::Crs(EpsgCode::new(6677)),
            [1.0, 2.0, 3.0],
            [4.0, 3.0, 2.0],
        );
        let footprint = single_polygon(solid.footprint_on(&FootprintPlane::Horizontal).unwrap());
        assert_eq!(
            footprint.frame(),
            &CoordinateFrame::Crs(EpsgCode::new(6677))
        );
        assert!((footprint.area() - 12.0).abs() < 1e-9);
        // EPSG:6677 stores coordinates in reflected (N, E) axis order (handedness -1),
        // so the canonically-oriented exterior has negative signed area.
        assert!(signed_area_2d(footprint.exterior()) < 0.0);
    }

    #[test]
    fn overlapping_faces_dissolve_regardless_of_winding() {
        // An upward face and a downward one (CW seen from above) overlap in a
        // 1x1 corner; the footprint is their union, not their cancellation.
        let Geometry::Euclidean3D(up) = square_3d(e(), [0.0, 0.0], 2.0, 0.0, true) else {
            unreachable!()
        };
        let Geometry::Euclidean3D(down) = square_3d(e(), [1.0, 1.0], 2.0, 5.0, false) else {
            unreachable!()
        };
        let collection = g3(Euclidean3DGeometry::Collection(Collection3D::new([
            up, down,
        ])));
        let footprint = single_polygon(
            collection
                .footprint_on(&FootprintPlane::Horizontal)
                .unwrap(),
        );
        assert!((footprint.area() - 7.0).abs() < 1e-9);
    }

    /// A 10x10 face holed by a square of `side`, wound `hole_ccw`.
    fn holed_face(side: f64, hole_ccw: bool) -> Geometry {
        let outer = vec![
            [0.0, 0.0, 4.0],
            [10.0, 0.0, 4.0],
            [10.0, 10.0, 4.0],
            [0.0, 10.0, 4.0],
            [0.0, 0.0, 4.0],
        ];
        let (lo, hi) = (5.0, 5.0 + side);
        let mut hole = vec![
            [lo, lo, 4.0],
            [hi, lo, 4.0],
            [hi, hi, 4.0],
            [lo, hi, 4.0],
            [lo, lo, 4.0],
        ];
        if !hole_ccw {
            hole.reverse();
        }
        let face = Polygon3D::from_rings(e(), outer, [hole]);
        g3(Euclidean3DGeometry::Polygon(Box::new(face)))
    }

    #[test]
    fn a_degenerate_hole_drops_only_itself() {
        let footprint = single_polygon(
            holed_face(1e-3, false)
                .footprint_on(&FootprintPlane::Horizontal)
                .unwrap(),
        );
        assert_eq!(footprint.interiors().count(), 0);
        assert!((footprint.area() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn a_hole_wound_like_its_exterior_is_still_cut_out() {
        for hole_ccw in [true, false] {
            let footprint = single_polygon(
                holed_face(2.0, hole_ccw)
                    .footprint_on(&FootprintPlane::Horizontal)
                    .unwrap(),
            );
            assert_eq!(footprint.interiors().count(), 1, "hole_ccw = {hole_ccw}");
            assert!(
                (footprint.area() - 96.0).abs() < 1e-9,
                "hole_ccw = {hole_ccw}"
            );
        }
    }

    #[test]
    fn a_face_whose_exterior_projects_to_no_area_is_dropped_whole() {
        // A wall is the everyday case; here the exterior is a sliver so the
        // hole has area, which it must not be promoted to the exterior with.
        let outer = vec![[0.0, 0.0, 0.0], [1e-9, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let hole = vec![
            [1.0, 1.0, 0.0],
            [3.0, 1.0, 0.0],
            [3.0, 3.0, 0.0],
            [1.0, 3.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let face = g3(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(e(), outer, [hole]),
        )));
        assert_eq!(
            face.footprint_on(&FootprintPlane::Horizontal),
            Err(FootprintError::Empty)
        );
    }

    #[test]
    fn a_plane_given_by_its_normal_projects_the_silhouette_in_the_geometry_frame() {
        // A plane facing +y in a projected CRS: the silhouette of a box is its
        // x-z extent.
        let epsg = EpsgCode::new(6677);
        let solid = box_solid(CoordinateFrame::Crs(epsg), [0.0; 3], [2.0, 3.0, 4.0]);
        let target = FootprintPlane::Normal {
            origin: [0.0, 0.0, 1.0],
            normal: [0.0, 5.0, 0.0],
            x_axis: None,
        };
        let footprint = single_polygon(solid.footprint_on(&target).unwrap());
        let expected =
            TangentPlane::from_normal(BaseFrame::Crs(epsg), [0.0, 0.0, 1.0], [0.0, 1.0, 0.0], None)
                .unwrap();
        assert_eq!(
            footprint.frame(),
            &CoordinateFrame::Tangent(Box::new(expected))
        );
        assert!((footprint.area() - 8.0).abs() < 1e-9);
    }

    #[test]
    fn an_angular_frame_is_rejected() {
        let face = square_3d(
            CoordinateFrame::Crs(EpsgCode::new(4979)),
            [0.0, 0.0],
            1.0,
            0.0,
            true,
        );
        assert!(matches!(
            face.footprint_on(&FootprintPlane::Horizontal),
            Err(FootprintError::NonLinearFrame(_))
        ));
    }
}
