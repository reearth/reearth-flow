use crate::{
    algorithm::{bounding_rect::BoundingRect, intersects::Intersects},
    types::{coordinate::Coordinate, line::Line},
    utils::point_line_euclidean_distance,
};

use super::{
    kernels::{Orientation, RobustKernel},
    GeoFloat,
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LineIntersection<T: GeoFloat, Z: GeoFloat> {
    SinglePoint {
        intersection: Coordinate<T, Z>,
        is_proper: bool,
    },

    /// Overlapping Lines intersect in a line segment
    Collinear { intersection: Line<T, Z> },
}

impl<T: GeoFloat, Z: GeoFloat> LineIntersection<T, Z> {
    pub fn is_proper(&self) -> bool {
        match self {
            Self::Collinear { .. } => false,
            Self::SinglePoint { is_proper, .. } => *is_proper,
        }
    }
}

pub fn line_intersection<T, Z>(p: Line<T, Z>, q: Line<T, Z>) -> Option<LineIntersection<T, Z>>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    if !p.bounding_rect().intersects(&q.bounding_rect()) {
        return None;
    }

    let p_q1 = RobustKernel::orient(p.start, p.end, q.start, None);
    let p_q2 = RobustKernel::orient(p.start, p.end, q.end, None);
    if matches!(
        (p_q1, p_q2),
        (Orientation::Clockwise, Orientation::Clockwise)
            | (Orientation::CounterClockwise, Orientation::CounterClockwise)
    ) {
        return None;
    }

    let q_p1 = RobustKernel::orient(q.start, q.end, p.start, None);
    let q_p2 = RobustKernel::orient(q.start, q.end, p.end, None);
    if matches!(
        (q_p1, q_p2),
        (Orientation::Clockwise, Orientation::Clockwise)
            | (Orientation::CounterClockwise, Orientation::CounterClockwise)
    ) {
        return None;
    }

    if matches!(
        (p_q1, p_q2, q_p1, q_p2),
        (
            Orientation::Collinear,
            Orientation::Collinear,
            Orientation::Collinear,
            Orientation::Collinear
        )
    ) {
        return collinear_intersection(p, q);
    }

    // At this point we know that there is a single intersection point (since the lines are not
    // collinear).
    //
    // Check if the intersection is an endpoint. If it is, copy the endpoint as the
    // intersection point. Copying the point rather than computing it ensures the point has the
    // exact value, which is important for robustness. It is sufficient to simply check for an
    // endpoint which is on the other line, since at this point we know that the inputLines
    // must intersect.
    if p_q1 == Orientation::Collinear
        || p_q2 == Orientation::Collinear
        || q_p1 == Orientation::Collinear
        || q_p2 == Orientation::Collinear
    {
        // Check for two equal endpoints.
        // This is done explicitly rather than by the orientation tests below in order to improve
        // robustness.
        //
        // [An example where the orientation tests fail to be consistent is the following (where
        // the true intersection is at the shared endpoint
        // POINT (19.850257749638203 46.29709338043669)
        //
        // LINESTRING ( 19.850257749638203 46.29709338043669, 20.31970698357233 46.76654261437082 )
        // and
        // LINESTRING ( -48.51001596420236 -22.063180333403878, 19.850257749638203 46.29709338043669 )
        //
        // which used to produce the INCORRECT result: (20.31970698357233, 46.76654261437082, NaN)

        let intersection: Coordinate<T, Z>;
        // false positives for this overzealous clippy https://github.com/rust-lang/rust-clippy/issues/6747
        #[allow(clippy::suspicious_operation_groupings)]
        if p.start == q.start || p.start == q.end {
            intersection = p.start;
        } else if p.end == q.start || p.end == q.end {
            intersection = p.end;
            // Now check to see if any endpoint lies on the interior of the other segment.
        } else if p_q1 == Orientation::Collinear {
            intersection = q.start;
        } else if p_q2 == Orientation::Collinear {
            intersection = q.end;
        } else if q_p1 == Orientation::Collinear {
            intersection = p.start;
        } else {
            assert_eq!(q_p2, Orientation::Collinear);
            intersection = p.end;
        }
        Some(LineIntersection::SinglePoint {
            intersection,
            is_proper: false,
        })
    } else {
        let intersection = proper_intersection(p, q);
        Some(LineIntersection::SinglePoint {
            intersection,
            is_proper: true,
        })
    }
}

fn collinear_intersection<T: GeoFloat, Z: GeoFloat>(
    p: Line<T, Z>,
    q: Line<T, Z>,
) -> Option<LineIntersection<T, Z>> {
    fn collinear<T: GeoFloat, Z: GeoFloat>(intersection: Line<T, Z>) -> LineIntersection<T, Z> {
        LineIntersection::Collinear { intersection }
    }

    fn improper<T: GeoFloat, Z: GeoFloat>(
        intersection: Coordinate<T, Z>,
    ) -> LineIntersection<T, Z> {
        LineIntersection::SinglePoint {
            intersection,
            is_proper: false,
        }
    }

    let p_bounds = p.bounding_rect();
    let q_bounds = q.bounding_rect();
    Some(
        match (
            p_bounds.intersects(&q.start),
            p_bounds.intersects(&q.end),
            q_bounds.intersects(&p.start),
            q_bounds.intersects(&p.end),
        ) {
            (true, true, _, _) => collinear(q),
            (_, _, true, true) => collinear(p),
            (true, false, true, false) if q.start == p.start => improper(q.start),
            (true, _, true, _) => collinear(Line::new_(q.start, p.start)),
            (true, false, false, true) if q.start == p.end => improper(q.start),
            (true, _, _, true) => collinear(Line::new_(q.start, p.end)),
            (false, true, true, false) if q.end == p.start => improper(q.end),
            (_, true, true, _) => collinear(Line::new_(q.end, p.start)),
            (false, true, false, true) if q.end == p.end => improper(q.end),
            (_, true, _, true) => collinear(Line::new_(q.end, p.end)),
            _ => return None,
        },
    )
}

fn raw_line_intersection<T: GeoFloat, Z: GeoFloat>(
    p: Line<T, Z>,
    q: Line<T, Z>,
) -> Option<Coordinate<T, Z>> {
    let p_min_x = p.start.x.min(p.end.x);
    let p_min_y = p.start.y.min(p.end.y);
    let p_min_z = p.start.z.min(p.end.z);
    let p_max_x = p.start.x.max(p.end.x);
    let p_max_y = p.start.y.max(p.end.y);
    let p_max_z = p.start.z.max(p.end.z);

    let q_min_x = q.start.x.min(q.end.x);
    let q_min_y = q.start.y.min(q.end.y);
    let q_min_z = q.start.z.min(q.end.z);
    let q_max_x = q.start.x.max(q.end.x);
    let q_max_y = q.start.y.max(q.end.y);
    let q_max_z = q.start.z.max(q.end.z);

    let int_min_x = p_min_x.max(q_min_x);
    let int_max_x = p_max_x.min(q_max_x);
    let int_min_y = p_min_y.max(q_min_y);
    let int_max_y = p_max_y.min(q_max_y);
    let int_min_z = p_min_z.max(q_min_z);
    let int_max_z = p_max_z.min(q_max_z);

    let two = T::one() + T::one();
    let mid_x = (int_min_x + int_max_x) / two;
    let mid_y = (int_min_y + int_max_y) / two;
    let two = Z::one() + Z::one();
    let mid_z = (int_min_z + int_max_z) / two;

    // condition ordinate values by subtracting midpoint
    let p1x = p.start.x - mid_x;
    let p1y = p.start.y - mid_y;
    let p2x = p.end.x - mid_x;
    let p2y = p.end.y - mid_y;
    let q1x = q.start.x - mid_x;
    let q1y = q.start.y - mid_y;
    let q2x = q.end.x - mid_x;
    let q2y = q.end.y - mid_y;

    // unrolled computation using homogeneous coordinates eqn
    let px = p1y - p2y;
    let py = p2x - p1x;
    let pw = p1x * p2y - p2x * p1y;

    let qx = q1y - q2y;
    let qy = q2x - q1x;
    let qw = q1x * q2y - q2x * q1y;

    let xw = py * qw - qy * pw;
    let yw = qx * pw - px * qw;
    let w = px * qy - qx * py;

    let x_int = xw / w;
    let y_int = yw / w;

    // check for parallel lines
    if (x_int.is_nan() || x_int.is_infinite()) || (y_int.is_nan() || y_int.is_infinite()) {
        None
    } else {
        // de-condition intersection point
        Some(Coordinate::new__(x_int + mid_x, y_int + mid_y, mid_z))
    }
}

fn nearest_endpoint<T: GeoFloat, Z: GeoFloat>(p: Line<T, Z>, q: Line<T, Z>) -> Coordinate<T, Z> {
    let mut nearest_pt = p.start;
    let mut min_dist = point_line_euclidean_distance(p.start, q);

    let dist = point_line_euclidean_distance(p.end, q);
    if dist < min_dist {
        min_dist = dist;
        nearest_pt = p.end;
    }
    let dist = point_line_euclidean_distance(q.start, p);
    if dist < min_dist {
        min_dist = dist;
        nearest_pt = q.start;
    }
    let dist = point_line_euclidean_distance(q.end, p);
    if dist < min_dist {
        nearest_pt = q.end;
    }
    nearest_pt
}

fn proper_intersection<T: GeoFloat, Z: GeoFloat>(p: Line<T, Z>, q: Line<T, Z>) -> Coordinate<T, Z> {
    let mut int_pt = raw_line_intersection(p, q).unwrap_or_else(|| nearest_endpoint(p, q));
    if !(p.bounding_rect().intersects(&int_pt) && q.bounding_rect().intersects(&int_pt)) {
        int_pt = nearest_endpoint(p, q);
    }
    int_pt
}
