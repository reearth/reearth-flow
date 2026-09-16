//! Ring winding, in canonical orientation.
//!
//! A ring's raw signed area does not by itself say which way it winds: in a
//! frame whose stored axis basis is a reflection of a right-handed one — every
//! `(northing, easting)` CRS, Japan's Plane Rectangular systems among them —
//! a ring drawn counter-clockwise on the ground is stored with a negative
//! shoelace. Canonical orientation folds the frame's
//! [`orientation_sign`](crate::coordinate::CoordinateFrame::orientation_sign)
//! into that sign, so the answer describes the ring on the ground rather than
//! the order its numbers happen to be written in, and agrees with the
//! convention each leaf's validation already applies.

use crate::coordinate::CoordinateFrame;
use crate::validation_next::signed_area_2d;

/// Which way a ring winds, once the frame's orientation sign is folded in.
///
/// Flow's convention is counter-clockwise: a valid exterior ring is
/// [`CounterClockwise`](RingWinding::CounterClockwise) and each of its holes
/// [`Clockwise`](RingWinding::Clockwise).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RingWinding {
    Clockwise,
    CounterClockwise,
    /// Zero signed area: a ring collapsed to a line or a point, which winds
    /// neither way. Not an error — a degenerate ring genuinely has no winding.
    Degenerate,
}

/// The canonical winding of a 2D ring in `frame`.
///
/// The ring is measured as if closed, so a ring stored open winds the same way
/// as the same ring stored with its first vertex repeated. A frame whose sign
/// cannot be resolved — an unknown CRS, or one whose axes are not axis-aligned
/// — falls back to `+1`, matching `overlay::output_direction`.
pub fn ring_winding_2d(frame: &CoordinateFrame, ring: &[[f64; 2]]) -> RingWinding {
    let area = signed_area_2d(ring) * f64::from(frame.orientation_sign().unwrap_or(1));
    if area > 0.0 {
        RingWinding::CounterClockwise
    } else if area < 0.0 {
        RingWinding::Clockwise
    } else {
        RingWinding::Degenerate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::EpsgCode;

    /// The unit triangle, counter-clockwise when read as `(x, y)`.
    const CCW: [[f64; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]];

    #[test]
    fn a_euclidean_frame_reads_the_shoelace_directly() {
        let frame = CoordinateFrame::Euclidean;
        let mut cw = CCW;
        cw.reverse();
        assert_eq!(ring_winding_2d(&frame, &CCW), RingWinding::CounterClockwise);
        assert_eq!(ring_winding_2d(&frame, &cw), RingWinding::Clockwise);
    }

    /// EPSG:6677 (JGD2011 Plane Rectangular CS IX) declares its axes as
    /// `(northing, easting)`, so the stored shoelace is inverted: coordinates
    /// written counter-clockwise as numbers describe a clockwise ring on the
    /// ground. Folding the frame sign in is the whole point of this module.
    #[test]
    fn a_northing_easting_frame_inverts_the_stored_sign() {
        let frame = CoordinateFrame::Crs(EpsgCode::new(6677));
        assert_eq!(ring_winding_2d(&frame, &CCW), RingWinding::Clockwise);
    }

    #[test]
    fn a_collinear_ring_winds_neither_way() {
        let flat = [[0.0, 0.0], [1.0, 1.0], [2.0, 2.0], [0.0, 0.0]];
        assert_eq!(
            ring_winding_2d(&CoordinateFrame::Euclidean, &flat),
            RingWinding::Degenerate
        );
    }
}
