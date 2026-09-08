//! Per-leaf coordinate frame, and the axis-order and orientation conventions
//! that follow from it.
//!
//! # Axis order conventions
//!
//! When a geometry has a CRS frame, its coordinate axes are in the order declared by
//! the CRS authority. When a geometry lives in a general Euclidean frame, its axes are
//! in `(x, y[, z])` order. Otherwise, a geometry lives in a tangent plane anchored in
//! a base frame, and its axes inherited from the base frame.
//!
//! # Orientation sign
//!
//! For CRS coordinate systems, the stored axis basis is either right-handed or a
//! reflection of one (for example, a north-first order reflects an east-first one), so
//! the sign of every face normal depends on the frame.
//! [`CoordinateFrame::orientation_sign`] is the sign function that is used to determine
//! the orientation of a ring, and is given by the following rules:
//!    1. CRS frame: for geographic CRSs and projected CRSs the sign is determined by the
//!       CRS's axis order and directions. A geocentric (ECEF) CRS is right-handed in
//!       `(X, Y, Z)` order, so `+1`.
//!    2. Euclidean frame: the sign is always `+1`.
//!    3. Tangent frame: the sign is derived from the base frame.
//!
//! The canonical orientation of a ring is then defined as
//! `right_hand_rule(ring) * CoordinateFrame::orientation_sign(frame)`. This product is
//! invariant under reprojection: reordering the coordinate axes flips `right_hand_rule(ring)`
//! and `CoordinateFrame::orientation_sign(frame)` together.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// An EPSG code identifying a coordinate reference system.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(transparent)]
pub struct EpsgCode(u16);

impl EpsgCode {
    /// Wrap a raw EPSG code.
    pub const fn new(code: u16) -> Self {
        Self(code)
    }

    /// The raw EPSG code.
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl fmt::Display for EpsgCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<u16> for EpsgCode {
    fn from(code: u16) -> Self {
        Self(code)
    }
}

impl From<EpsgCode> for u16 {
    fn from(code: EpsgCode) -> Self {
        code.0
    }
}

/// The coordinate frame a geometry leaf is expressed in.
///
/// Every coordinate-bearing leaf carries its own `frame: CoordinateFrame`, so an
/// operation reads its source frame from `self` and a collection may hold
/// members in different frames.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub enum CoordinateFrame {
    /// A geographic / projected CRS identified by its EPSG code.
    Crs(EpsgCode),
    /// Bare Euclidean space with no geo-referencing.
    #[default]
    Euclidean,
    /// A 2D plane embedded in 3D, anchored in a base frame.
    Tangent(Box<TangentPlane>),
}

impl CoordinateFrame {
    /// The EPSG code of this frame, or an error if it is not a CRS frame.
    pub(crate) fn require_crs(&self) -> Result<EpsgCode> {
        match self {
            CoordinateFrame::Crs(epsg) => Ok(*epsg),
            CoordinateFrame::Euclidean => Err(Error::projection(
                "cannot reproject a Euclidean (non-georeferenced) geometry",
            )),
            CoordinateFrame::Tangent(_) => Err(Error::projection(
                "cannot reproject a Tangent-plane geometry",
            )),
        }
    }

    /// The orientation sign of this frame: `+1` when its coordinates are
    /// right-handed in canonical `(East, North[, Up])` order, `-1` when reflected.
    /// A stored winding times this sign is the canonical orientation.
    ///
    /// `Crs` frames read the sign from the CRS's declared axis directions and
    /// therefore error on an unknown CRS or one whose axes are not axis-aligned.
    /// `Euclidean` coordinates are right-handed by construction, so their sign is
    /// `+1`. A `Tangent` frame's in-plane axes are expressed in its base frame, so
    /// its sign is the base frame's: `+1` for a Euclidean base, the CRS's sign for
    /// a `Crs` base.
    pub fn orientation_sign(&self) -> Result<i8> {
        match self {
            CoordinateFrame::Crs(epsg) => crate::ops::axis_order_sign(*epsg),
            CoordinateFrame::Euclidean => Ok(1),
            CoordinateFrame::Tangent(tangent) => match tangent.base {
                BaseFrame::Crs(epsg) => crate::ops::axis_order_sign(epsg),
                BaseFrame::Euclidean => Ok(1),
            },
        }
    }

    /// This frame with its vertical axis dropped, for coordinates that have just
    /// been flattened to 2D.
    ///
    /// A `Crs` frame becomes its 2D counterpart: the horizontal component of a
    /// compound CRS (EPSG:6697 becomes EPSG:6668), the 2D form of a geographic 3D
    /// one (EPSG:4979 becomes EPSG:4326), or itself when already 2D — so the
    /// operation is idempotent. The counterpart shares the datum, axis order and
    /// units, so coordinate values are unaffected. `Euclidean` and `Tangent` have
    /// no vertical axis to shed and come back unchanged.
    ///
    /// Errors per [`FrameDemotionError`]: a frame that cannot be shown to be 2D
    /// must not be attached to 2D coordinates.
    pub fn demote_to_2d(&self) -> Result<CoordinateFrame, FrameDemotionError> {
        let CoordinateFrame::Crs(epsg) = self else {
            return Ok(self.clone());
        };
        let reason = match crate::ops::crs_demote_to_2d(*epsg) {
            Ok(crate::ops::TwoDimensionalCrs::Code(code)) => return Ok(CoordinateFrame::Crs(code)),
            Ok(crate::ops::TwoDimensionalCrs::None(why)) => {
                FrameDemotionReason::NoTwoDimensionalForm(why)
            }
            Err(why) => FrameDemotionReason::Unresolvable(why),
        };
        Err(FrameDemotionError {
            epsg: *epsg,
            reason,
        })
    }

    /// Whether coordinates in this frame are in linear (length) units, so that
    /// unit-sensitive checks (planarity, surface triangulation) are meaningful.
    /// True only for a definitely-linear frame; an angular or undeterminable
    /// `Crs` both yield false, so the affected checks are skipped rather than
    /// trusted. Use [`unit_kind`](Self::unit_kind) to tell those two apart.
    pub fn has_linear_units(&self) -> bool {
        matches!(self.unit_kind(), UnitKind::Linear)
    }

    /// Classify this frame's horizontal coordinate units. `Euclidean` and
    /// `Tangent` (in-plane metres) are linear; a `Crs` frame is linear iff its
    /// horizontal axes use a length unit (projected / geocentric) rather than an
    /// angular one (geographic degrees), and
    /// [`Undeterminable`](UnitKind::Undeterminable) when PROJ cannot classify
    /// it (e.g. an unknown code or missing PROJ data).
    pub fn unit_kind(&self) -> UnitKind {
        match self {
            CoordinateFrame::Euclidean => UnitKind::Linear,
            CoordinateFrame::Tangent(_) => UnitKind::Linear,
            CoordinateFrame::Crs(epsg) => match crate::ops::crs_is_linear(*epsg) {
                Ok(true) => UnitKind::Linear,
                Ok(false) => UnitKind::Angular,
                Err(e) => UnitKind::Undeterminable(e.to_string()),
            },
        }
    }
}

/// A CRS whose 2D counterpart could not be established, so no frame can describe
/// its coordinates once the vertical axis is dropped. Returned by
/// [`demote_to_2d`](CoordinateFrame::demote_to_2d).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameDemotionError {
    /// The CRS that could not be demoted.
    pub epsg: EpsgCode,
    /// Why it could not be.
    pub reason: FrameDemotionReason,
}

/// The two ways demoting a CRS frame fails. Kept apart because they call for
/// different fixes: the wrong CRS for the operation, versus a CRS the
/// installation cannot look up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameDemotionReason {
    NoTwoDimensionalForm(MissingTwoDimensionalForm),
    /// PROJ's own failure message, verbatim.
    Unresolvable(String),
}

/// PROJ's grounds for a resolved CRS having no 2D form, each a fixed property of
/// the CRS. The [`Display`](fmt::Display) arms are the wording users see.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissingTwoDimensionalForm {
    Geocentric,
    Unidentified,
    StillThreeDimensional,
    NoCoordinateSystem,
}

impl fmt::Display for MissingTwoDimensionalForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MissingTwoDimensionalForm::Geocentric => {
                "it is geocentric (ECEF), so its third axis is the rotation axis rather than a vertical one"
            }
            MissingTwoDimensionalForm::Unidentified => "its 2D form carries no EPSG identifier",
            MissingTwoDimensionalForm::StillThreeDimensional => {
                "its 2D form still has more than two axes"
            }
            MissingTwoDimensionalForm::NoCoordinateSystem => {
                "its 2D form reports no coordinate system"
            }
        })
    }
}

impl fmt::Display for FrameDemotionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameDemotionReason::NoTwoDimensionalForm(cause) => cause.fmt(f),
            FrameDemotionReason::Unresolvable(why) => f.write_str(why),
        }
    }
}

impl fmt::Display for FrameDemotionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EPSG:{} cannot be demoted to 2D: {}",
            self.epsg, self.reason
        )
    }
}

impl std::error::Error for FrameDemotionError {}

/// How a coordinate frame's horizontal units classify: linear (length), angular
/// (degrees), or unclassifiable. "Linear" rather than "metric" because a length
/// unit need not be metres (e.g. feet).
#[derive(Clone, Debug, PartialEq)]
pub enum UnitKind {
    /// Linear (length) units: unit-sensitive checks are meaningful.
    Linear,
    /// Angular units (geographic degrees): unit-sensitive checks are skipped.
    Angular,
    /// PROJ could not classify the CRS; carries the failure reason so a caller
    /// can surface it rather than silently treating the frame as angular.
    Undeterminable(String),
}

/// The absolute frame a [`TangentPlane`] is anchored in: exactly the non-tangent
/// [`CoordinateFrame`] frames, so a tangent plane cannot be anchored in another
/// tangent plane.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BaseFrame {
    /// A geographic / projected CRS identified by its EPSG code.
    Crs(EpsgCode),
    /// Bare Euclidean space with no geo-referencing.
    Euclidean,
}

/// A 2D Euclidean plane embedded in 3D space.
///
/// A [`CoordinateFrame::Tangent`] geometry stores in-plane `(x, y)` whose 3D position
/// is `origin + x * u + y * v`. When `base` is a geographic CRS this is the
/// local tangent (ENU) frame at `origin`, with in-plane coordinates in metres.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schema", schemars(title = "Tangent plane"))]
pub struct TangentPlane {
    /// Frame that `origin`, `u` and `v` are expressed in.
    #[cfg_attr(feature = "schema", schemars(title = "Anchor frame"))]
    pub base: BaseFrame,
    /// Plane origin, in `base`.
    #[cfg_attr(feature = "schema", schemars(title = "Plane origin"))]
    pub origin: [f64; 3],
    /// Orthonormal in-plane axis; the plane normal is the cross product of `u`
    /// and `v`.
    #[cfg_attr(feature = "schema", schemars(title = "In-plane axis U"))]
    pub u: [f64; 3],
    /// Orthonormal in-plane axis.
    #[cfg_attr(feature = "schema", schemars(title = "In-plane axis V"))]
    pub v: [f64; 3],
}

/// Why a [`TangentPlane`] could not be built from the given vectors.
#[derive(Clone, Debug, PartialEq)]
pub enum TangentPlaneError {
    /// The normal has (near) zero length.
    ZeroNormal,
    /// The requested in-plane x axis is (near) parallel to the normal, so it
    /// has no in-plane component.
    XAxisParallelToNormal,
    /// `u` and `v` are not orthonormal.
    NotOrthonormal,
}

impl core::fmt::Display for TangentPlaneError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TangentPlaneError::ZeroNormal => write!(f, "plane normal has zero length"),
            TangentPlaneError::XAxisParallelToNormal => {
                write!(f, "plane x axis is parallel to the normal")
            }
            TangentPlaneError::NotOrthonormal => {
                write!(f, "plane axes are not orthonormal")
            }
        }
    }
}

impl std::error::Error for TangentPlaneError {}

use crate::predicates::kernel::{cross3, dot3, sub3};

/// Tolerance for the orthonormality checks on plane axes.
const AXIS_TOLERANCE: f64 = 1e-9;

impl TangentPlane {
    /// The plane through `origin` with unit normal along `normal`, oriented so
    /// that `u x v` points along `normal`.
    ///
    /// `x_axis`, when given, fixes `u` as its component within the plane.
    /// Otherwise `v` is the in-plane direction closest to the base frame's third
    /// axis (up), so a vertical plane's `v` points up; for a horizontal plane
    /// `u` follows the base frame's first axis instead.
    pub fn from_normal(
        base: BaseFrame,
        origin: [f64; 3],
        normal: [f64; 3],
        x_axis: Option<[f64; 3]>,
    ) -> Result<Self, TangentPlaneError> {
        let n = unit(normal).ok_or(TangentPlaneError::ZeroNormal)?;
        let (u, v) = match x_axis {
            Some(x) => {
                let u = unit(reject(x, n)).ok_or(TangentPlaneError::XAxisParallelToNormal)?;
                (u, cross3(n, u))
            }
            None => match unit(reject([0.0, 0.0, 1.0], n)) {
                Some(v) => (cross3(v, n), v),
                None => {
                    let u = [1.0, 0.0, 0.0];
                    (u, cross3(n, u))
                }
            },
        };
        Ok(Self { base, origin, u, v })
    }

    /// The unit normal `u x v`.
    pub fn normal(&self) -> [f64; 3] {
        cross3(self.u, self.v)
    }

    /// Whether `u` and `v` are unit length and perpendicular.
    pub fn is_orthonormal(&self) -> bool {
        (dot3(self.u, self.u) - 1.0).abs() <= AXIS_TOLERANCE
            && (dot3(self.v, self.v) - 1.0).abs() <= AXIS_TOLERANCE
            && dot3(self.u, self.v).abs() <= AXIS_TOLERANCE
    }

    /// The in-plane `(x, y)` of a base-frame position: its offset from `origin`
    /// resolved along `u` and `v`, dropping the component along the normal.
    #[inline]
    pub fn project(&self, position: [f64; 3]) -> [f64; 2] {
        let d = sub3(position, self.origin);
        [dot3(d, self.u), dot3(d, self.v)]
    }

    /// The base-frame position of an in-plane `(x, y)`: `origin + x * u + y * v`.
    #[inline]
    pub fn lift(&self, [x, y]: [f64; 2]) -> [f64; 3] {
        [
            self.origin[0] + x * self.u[0] + y * self.v[0],
            self.origin[1] + x * self.u[1] + y * self.v[1],
            self.origin[2] + x * self.u[2] + y * self.v[2],
        ]
    }
}

/// `a` scaled to unit length, or `None` when it is (near) zero.
fn unit(a: [f64; 3]) -> Option<[f64; 3]> {
    let len = dot3(a, a).sqrt();
    (len > AXIS_TOLERANCE).then(|| [a[0] / len, a[1] / len, a[2] / len])
}

/// The component of `a` perpendicular to the unit vector `n`.
fn reject(a: [f64; 3], n: [f64; 3]) -> [f64; 3] {
    let along = dot3(a, n);
    [
        a[0] - along * n[0],
        a[1] - along * n[1],
        a[2] - along * n[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f64; 3], b: [f64; 3]) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-12)
    }

    #[test]
    fn from_normal_builds_a_right_handed_orthonormal_plane() {
        // A vertical plane facing +x: v is up, u = v x n = +y.
        let plane =
            TangentPlane::from_normal(BaseFrame::Euclidean, [1.0, 2.0, 3.0], [2.0, 0.0, 0.0], None)
                .unwrap();
        assert!(plane.is_orthonormal());
        assert!(close(plane.normal(), [1.0, 0.0, 0.0]));
        assert!(close(plane.u, [0.0, 1.0, 0.0]));
        assert!(close(plane.v, [0.0, 0.0, 1.0]));
        assert_eq!(plane.project([1.0, 5.0, 7.0]), [3.0, 4.0]);
        // A horizontal plane keeps the base frame's x as u; facing down flips v.
        let up = TangentPlane::from_normal(BaseFrame::Euclidean, [0.0; 3], [0.0, 0.0, 1.0], None)
            .unwrap();
        assert!(close(up.u, [1.0, 0.0, 0.0]) && close(up.v, [0.0, 1.0, 0.0]));
        let down =
            TangentPlane::from_normal(BaseFrame::Euclidean, [0.0; 3], [0.0, 0.0, -1.0], None)
                .unwrap();
        assert!(close(down.u, [1.0, 0.0, 0.0]) && close(down.v, [0.0, -1.0, 0.0]));
        assert!(close(down.normal(), [0.0, 0.0, -1.0]));
    }

    #[test]
    fn from_normal_honours_a_requested_x_axis() {
        // The requested axis is not in the plane; its in-plane component is.
        let plane = TangentPlane::from_normal(
            BaseFrame::Euclidean,
            [0.0; 3],
            [0.0, 0.0, 1.0],
            Some([0.0, 3.0, 4.0]),
        )
        .unwrap();
        assert!(plane.is_orthonormal());
        assert!(close(plane.u, [0.0, 1.0, 0.0]));
        assert!(close(plane.v, [-1.0, 0.0, 0.0]));
        assert!(close(plane.normal(), [0.0, 0.0, 1.0]));
    }

    #[test]
    fn unit_kind_classifies_frames() {
        assert_eq!(CoordinateFrame::Euclidean.unit_kind(), UnitKind::Linear);
        // EPSG:6677 is a projected CRS in metres; EPSG:4326 is geographic.
        assert_eq!(
            CoordinateFrame::Crs(EpsgCode::new(6677)).unit_kind(),
            UnitKind::Linear
        );
        assert_eq!(
            CoordinateFrame::Crs(EpsgCode::new(4326)).unit_kind(),
            UnitKind::Angular
        );
        // EPSG:1 is not a real CRS, so PROJ cannot classify it: undeterminable
        // rather than silently angular.
        assert!(matches!(
            CoordinateFrame::Crs(EpsgCode::new(1)).unit_kind(),
            UnitKind::Undeterminable(_)
        ));
        // A tangent plane's in-plane coordinates are metres regardless of its
        // base, including when anchored in a geographic CRS.
        let tangent_over_geographic = CoordinateFrame::Tangent(Box::new(TangentPlane {
            base: BaseFrame::Crs(EpsgCode::new(4326)),
            origin: [0.0, 0.0, 0.0],
            u: [1.0, 0.0, 0.0],
            v: [0.0, 1.0, 0.0],
        }));
        assert_eq!(tangent_over_geographic.unit_kind(), UnitKind::Linear);
    }

    #[test]
    fn demote_to_2d_rejects_a_crs_with_no_2d_form() {
        // A geocentric CRS's third axis is the rotation axis, so there is no
        // horizontal component to fall back to.
        let err = CoordinateFrame::Crs(EpsgCode::new(4978))
            .demote_to_2d()
            .unwrap_err();
        assert_eq!(err.epsg, EpsgCode::new(4978));
        assert_eq!(
            err.reason,
            FrameDemotionReason::NoTwoDimensionalForm(MissingTwoDimensionalForm::Geocentric)
        );
    }

    #[test]
    fn demote_to_2d_leaves_non_crs_frames_alone() {
        // Neither frame declares a vertical axis to shed: Euclidean coordinates
        // are dimensionless, and a tangent plane's are in-plane by construction.
        assert_eq!(
            CoordinateFrame::Euclidean.demote_to_2d(),
            Ok(CoordinateFrame::Euclidean)
        );
        // The base frame describes the plane, not the geometry's coordinates, so
        // a 3D base changes nothing.
        let tangent = CoordinateFrame::Tangent(Box::new(TangentPlane {
            base: BaseFrame::Crs(EpsgCode::new(6697)),
            origin: [0.0, 0.0, 0.0],
            u: [1.0, 0.0, 0.0],
            v: [0.0, 1.0, 0.0],
        }));
        assert_eq!(tangent.demote_to_2d(), Ok(tangent.clone()));
    }

    #[test]
    fn an_unresolvable_crs_is_rejected() {
        // EPSG:1 is not a real CRS, so PROJ cannot say whether it has a 2D form.
        // "Unknown" must reject rather than keep the 3D tag, and must stay
        // distinguishable from a resolved CRS that simply has no 2D form.
        let err = CoordinateFrame::Crs(EpsgCode::new(1))
            .demote_to_2d()
            .unwrap_err();
        assert_eq!(err.epsg, EpsgCode::new(1));
        assert!(matches!(err.reason, FrameDemotionReason::Unresolvable(_)));
    }

    #[test]
    fn crs_sign_follows_axis_order() {
        assert_eq!(
            CoordinateFrame::Crs(EpsgCode::new(4326))
                .orientation_sign()
                .unwrap(),
            -1
        );
        assert_eq!(
            CoordinateFrame::Crs(EpsgCode::new(3857))
                .orientation_sign()
                .unwrap(),
            1
        );
    }

    #[test]
    fn euclidean_is_right_handed() {
        assert_eq!(CoordinateFrame::Euclidean.orientation_sign().unwrap(), 1);
    }

    #[test]
    fn tangent_sign_follows_its_base() {
        let euclidean_base = CoordinateFrame::Tangent(Box::new(TangentPlane {
            base: BaseFrame::Euclidean,
            origin: [0.0, 0.0, 0.0],
            u: [1.0, 0.0, 0.0],
            v: [0.0, 1.0, 0.0],
        }));
        assert_eq!(euclidean_base.orientation_sign().unwrap(), 1);

        // EPSG:6697 is lat-first (sign -1), so a tangent plane anchored in it is
        // reflected too.
        let reflected_base = CoordinateFrame::Tangent(Box::new(TangentPlane {
            base: BaseFrame::Crs(EpsgCode::new(6697)),
            origin: [0.0, 0.0, 0.0],
            u: [1.0, 0.0, 0.0],
            v: [0.0, 1.0, 0.0],
        }));
        assert_eq!(reflected_base.orientation_sign().unwrap(), -1);
    }
}
