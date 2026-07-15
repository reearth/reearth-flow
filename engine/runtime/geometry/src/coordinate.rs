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
//! For CRS coordinate systems, a north-first order is a reflection of an east-first one,
//! so the sign of every face normal depends on the frame.
//! [`CoordinateFrame::orientation_sign`] is the sign function that is used to determine
//! the orientation of a ring, and is given by the following rules:
//!    1. CRS frame: the sign is determined by the CRS's axis order and directions.
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
