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

#[cfg(test)]
mod test {
    use crate::{
        algorithm::line_intersection::{line_intersection, LineIntersection},
        types::{coordinate::Coordinate, line::Line},
    };

    #[test]
    fn test_central_endpoint_heuristic_failure_1() {
        let line_1 = Line::new(
            Coordinate::new_(163.81867067, -211.31840378),
            Coordinate::new_(165.9174252, -214.1665075),
        );
        let line_2 = Line::new(
            Coordinate::new_(2.84139601, -57.95412726),
            Coordinate::new_(469.59990601, -502.63851732),
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: Coordinate::new_(163.81867067, -211.31840378),
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    #[test]
    fn test_central_endpoint_heuristic_failure_2() {
        let line_1 = Line::new(
            Coordinate::new_(-58.00593335955, -1.43739086465),
            Coordinate::new_(-513.86101637525, -457.29247388035),
        );
        let line_2 = Line::new(
            Coordinate::new_(-215.22279674875, -158.65425425385),
            Coordinate::new_(-218.1208801283, -160.68343590235),
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: Coordinate::new_(-215.22279674875, -158.65425425385),
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    #[test]
    fn test_leduc_1() {
        let line_1 = Line::new(
            Coordinate::new_(305690.0434123494, 254176.46578338774),
            Coordinate::new_(305601.9999843455, 254243.19999846347),
        );
        let line_2 = Line::new(
            Coordinate::new_(305689.6153764265, 254177.33102743194),
            Coordinate::new_(305692.4999844298, 254171.4999983967),
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: Coordinate::new_(305690.0434123494, 254176.46578338774),
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    #[test]
    fn test_geos_1() {
        let line_1 = Line::new(
            Coordinate::new_(588750.7429703881, 4518950.493668233),
            Coordinate::new_(588748.2060409798, 4518933.9452804085),
        );
        let line_2 = Line::new(
            Coordinate::new_(588745.824857241, 4518940.742239175),
            Coordinate::new_(588748.2060437313, 4518933.9452791475),
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: Coordinate::new_(588748.2060416829, 4518933.945284994),
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    #[test]
    fn test_geos_2() {
        let line_1 = Line::new(
            Coordinate::new_(588743.626135934, 4518924.610969561),
            Coordinate::new_(588732.2822865889, 4518925.4314047815),
        );
        let line_2 = Line::new(
            Coordinate::new_(588739.1191384895, 4518927.235700594),
            Coordinate::new_(588731.7854614238, 4518924.578370095),
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: Coordinate::new_(588733.8306132929, 4518925.319423238),
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    #[test]
    fn test_dave_skea_case() {
        let line_1 = Line::new(
            Coordinate::new_(2089426.5233462777, 1180182.387733969),
            Coordinate::new_(2085646.6891757075, 1195618.7333999649),
        );
        let line_2 = Line::new(
            Coordinate::new_(1889281.8148903656, 1997547.0560044837),
            Coordinate::new_(2259977.3672236, 483675.17050843034),
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: Coordinate::new_(2087536.6062609926, 1187900.560566967),
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    #[test]
    fn test_cmp_5_cask_wkt() {
        let line_1 = Line::new(
            Coordinate::new_(4348433.262114629, 5552595.478385733),
            Coordinate::new_(4348440.849387404, 5552599.272022122),
        );
        let line_2 = Line::new(
            Coordinate::new_(4348433.26211463, 5552595.47838573),
            Coordinate::new_(4348440.8493874, 5552599.27202212),
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: Coordinate::new_(4348440.8493874, 5552599.27202212),
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }
}
