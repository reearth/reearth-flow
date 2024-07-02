use crate::types::{coordinate::Coordinate, line_string::LineString};

use super::{
    kernels::{Orientation, RobustKernel},
    GeoNum,
};

pub trait IsConvex {
    /// Test and get the orientation if the shape is convex.
    /// Tests for strict convexity if `allow_collinear`, and
    /// only accepts a specific orientation if provided.
    ///
    /// The return value is `None` if either:
    ///
    /// 1. the shape is not convex
    ///
    /// 1. the shape is not strictly convex, and
    ///    `allow_collinear` is false
    ///
    /// 1. an orientation is specified, and some three
    ///    consecutive vertices where neither collinear, nor
    ///    in the specified orientation.
    ///
    /// In all other cases, the return value is the
    /// orientation of the shape, or `Orientation::Collinear`
    /// if all the vertices are on the same line.
    ///
    /// **Note.** This predicate is not equivalent to
    /// `is_collinear` as this requires that the input is
    /// closed.
    fn convex_orientation(
        &self,
        allow_collinear: bool,
        specific_orientation: Option<Orientation>,
    ) -> Option<Orientation>;

    /// Test if the shape is convex.
    fn is_convex(&self) -> bool {
        self.convex_orientation(true, None).is_some()
    }

    /// Test if the shape is convex, and oriented
    /// counter-clockwise.
    fn is_ccw_convex(&self) -> bool {
        self.convex_orientation(true, Some(Orientation::CounterClockwise))
            .is_some()
    }

    /// Test if the shape is convex, and oriented clockwise.
    fn is_cw_convex(&self) -> bool {
        self.convex_orientation(true, Some(Orientation::Clockwise))
            .is_some()
    }

    /// Test if the shape is strictly convex.
    fn is_strictly_convex(&self) -> bool {
        self.convex_orientation(false, None).is_some()
    }

    /// Test if the shape is strictly convex, and oriented
    /// counter-clockwise.
    fn is_strictly_ccw_convex(&self) -> bool {
        self.convex_orientation(false, Some(Orientation::CounterClockwise))
            == Some(Orientation::CounterClockwise)
    }

    /// Test if the shape is strictly convex, and oriented
    /// clockwise.
    fn is_strictly_cw_convex(&self) -> bool {
        self.convex_orientation(false, Some(Orientation::Clockwise)) == Some(Orientation::Clockwise)
    }

    /// Test if the shape lies on a line.
    fn is_collinear(&self) -> bool;
}

impl<T: GeoNum, Z: GeoNum> IsConvex for LineString<T, Z> {
    fn convex_orientation(
        &self,
        allow_collinear: bool,
        specific_orientation: Option<Orientation>,
    ) -> Option<Orientation> {
        if !self.is_closed() || self.0.is_empty() {
            None
        } else {
            is_convex_shaped(&self.0[1..], allow_collinear, specific_orientation)
        }
    }

    fn is_collinear(&self) -> bool {
        self.0.is_empty()
            || is_convex_shaped(&self.0[1..], true, Some(Orientation::Collinear)).is_some()
    }
}

fn is_convex_shaped<T, Z>(
    coords: &[Coordinate<T, Z>],
    allow_collinear: bool,
    specific_orientation: Option<Orientation>,
) -> Option<Orientation>
where
    T: GeoNum,
    Z: GeoNum,
{
    let n = coords.len();

    let orientation_at = |i: usize| {
        let coord = coords[i];
        let next = coords[(i + 1) % n];
        let nnext = coords[(i + 2) % n];
        (i, RobustKernel::orient(coord, next, nnext, None))
    };

    let find_first_non_collinear = (0..n).map(orientation_at).find_map(|(i, orientation)| {
        match orientation {
            Orientation::Collinear => {
                // If collinear accepted, we skip, otherwise
                // stop.
                if allow_collinear {
                    None
                } else {
                    Some((i, orientation))
                }
            }
            _ => Some((i, orientation)),
        }
    });

    let (i, first_non_collinear) = if let Some((i, orientation)) = find_first_non_collinear {
        match orientation {
            Orientation::Collinear => {
                // Only happens if !allow_collinear
                assert!(!allow_collinear);
                return None;
            }
            _ => (i, orientation),
        }
    } else {
        // Empty or everything collinear, and allowed.
        return Some(Orientation::Collinear);
    };

    // If a specific orientation is expected, accept only that.
    if let Some(req_orientation) = specific_orientation {
        if req_orientation != first_non_collinear {
            return None;
        }
    }

    // Now we have a fixed orientation expected at the rest
    // of the coords. Loop to check everything matches it.
    if ((i + 1)..n)
        .map(orientation_at)
        .all(|(_, orientation)| match orientation {
            Orientation::Collinear => allow_collinear,
            orientation => orientation == first_non_collinear,
        })
    {
        Some(first_non_collinear)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{algorithm::is_convex::IsConvex, line_string};

    #[test]
    fn test_corner_cases() {
        // This is just tested to ensure there is no panic
        // due to out-of-index access
        let one = line_string![(x: 0., y: 0.)];
        assert!(one.is_collinear());
        assert!(one.is_convex());
        assert!(one.is_cw_convex());
        assert!(one.is_ccw_convex());
        assert!(one.is_strictly_convex());
        assert!(!one.is_strictly_ccw_convex());
        assert!(!one.is_strictly_cw_convex());
    }
}
