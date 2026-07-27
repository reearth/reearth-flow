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

    /// The frame a geometry must carry once it is re-represented in a pure 2D
    /// embedding: a frame whose dimensionality matches the coordinates'.
    ///
    /// A `Crs` frame is demoted to its 2D counterpart — the horizontal component
    /// of a compound CRS (EPSG:6697 becomes EPSG:6668), the 2D form of a
    /// geographic 3D one (EPSG:4979 becomes EPSG:4326) — and an already-2D CRS
    /// maps to itself, so the operation is idempotent. Coordinate values are
    /// unaffected: the counterpart shares the datum, the axis order and the
    /// units, only the vertical axis is gone. `Euclidean` and `Tangent` carry no
    /// vertical axis to shed and are returned unchanged.
    ///
    /// Errors when the CRS has no 2D counterpart, and equally when PROJ cannot
    /// resolve the CRS at all: a frame that cannot be shown to be 2D must not be
    /// attached to 2D coordinates. [`FrameDemotionError`] keeps the two cases
    /// apart, since they call for different fixes.
    pub fn demote_to_2d(&self) -> std::result::Result<CoordinateFrame, FrameDemotionError> {
        let CoordinateFrame::Crs(epsg) = self else {
            return Ok(self.clone());
        };
        let reason = match crate::ops::crs_demote_to_2d(*epsg) {
            Ok(crate::ops::TwoDimensionalCrs::Code(code)) => return Ok(CoordinateFrame::Crs(code)),
            Ok(crate::ops::TwoDimensionalCrs::None(why)) => {
                FrameDemotionReason::NoTwoDimensionalForm(why)
            }
            Err(e) => FrameDemotionReason::Unresolvable(e.to_string()),
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

/// The two ways demoting a CRS frame fails. They call for different fixes — one
/// is the wrong CRS for the operation, the other is a CRS the installation
/// cannot look up — so they are kept apart rather than flattened to one message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameDemotionReason {
    /// PROJ resolved the CRS and it has no 2D form. Carries the specific reason.
    NoTwoDimensionalForm(&'static str),
    /// PROJ could not resolve the CRS, leaving its dimensionality unknown.
    /// Carries PROJ's failure message.
    Unresolvable(String),
}

impl fmt::Display for FrameDemotionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.reason {
            FrameDemotionReason::NoTwoDimensionalForm(why) => {
                write!(f, "EPSG:{} has no 2D counterpart: {why}", self.epsg)
            }
            FrameDemotionReason::Unresolvable(why) => write!(
                f,
                "EPSG:{} cannot be demoted to 2D because PROJ could not resolve it: {why}",
                self.epsg
            ),
        }
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TangentPlane {
    /// Frame that `origin`, `u` and `v` are expressed in.
    pub base: BaseFrame,
    /// Plane origin, in `base`.
    pub origin: [f64; 3],
    /// Orthonormal in-plane axis; the plane normal is the cross product of `u`
    /// and `v`.
    pub u: [f64; 3],
    /// Orthonormal in-plane axis.
    pub v: [f64; 3],
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn demote_to_2d_drops_the_vertical_axis() {
        let demote = |code: u16| CoordinateFrame::Crs(EpsgCode::new(code)).demote_to_2d();
        // Compound -> its horizontal component; geographic 3D -> its 2D form.
        assert_eq!(demote(6697), Ok(CoordinateFrame::Crs(EpsgCode::new(6668))));
        assert_eq!(demote(4979), Ok(CoordinateFrame::Crs(EpsgCode::new(4326))));
        // A CRS with no vertical axis maps to itself, which is what makes forcing
        // a geometry to 2D twice stable.
        assert_eq!(demote(6668), Ok(CoordinateFrame::Crs(EpsgCode::new(6668))));
        assert_eq!(demote(6677), Ok(CoordinateFrame::Crs(EpsgCode::new(6677))));
    }

    #[test]
    fn demote_to_2d_rejects_a_crs_with_no_2d_form() {
        // A geocentric CRS's third axis is the rotation axis, so there is no
        // horizontal component to fall back to.
        let err = CoordinateFrame::Crs(EpsgCode::new(4978))
            .demote_to_2d()
            .unwrap_err();
        assert_eq!(err.epsg, EpsgCode::new(4978));
        assert!(matches!(
            err.reason,
            FrameDemotionReason::NoTwoDimensionalForm(_)
        ));
        assert!(
            err.to_string().contains("has no 2D counterpart"),
            "unexpected message: {err}"
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
        // A tangent plane stays put even when anchored in a 3D CRS: the base
        // frame describes the plane, not the geometry's coordinates.
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
        // "Unknown" must not become "keep the 3D tag": that is exactly the
        // mismatch demotion exists to prevent. Reported apart from a
        // resolved-but-2D-less CRS because the fix differs.
        let err = CoordinateFrame::Crs(EpsgCode::new(1))
            .demote_to_2d()
            .unwrap_err();
        assert_eq!(err.epsg, EpsgCode::new(1));
        assert!(matches!(err.reason, FrameDemotionReason::Unresolvable(_)));
        assert!(
            err.to_string().contains("PROJ could not resolve it"),
            "unexpected message: {err}"
        );
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
