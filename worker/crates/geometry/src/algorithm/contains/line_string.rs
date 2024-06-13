use super::Contains;
use crate::algorithm::intersects::Intersects;
use crate::algorithm::GeoNum;
use crate::types::coordinate::Coordinate;
use crate::types::coordnum::CoordNum;
use crate::types::line::Line;
use crate::types::line_string::LineString;
use crate::types::multi_line_string::MultiLineString;
use crate::types::multi_point::MultiPoint;
use crate::types::multi_polygon::MultiPolygon;
use crate::types::point::Point;
use crate::types::polygon::Polygon;
use crate::types::rect::Rect;
use crate::types::triangle::Triangle;

// ┌────────────────────────────────┐
// │ Implementations for LineString │
// └────────────────────────────────┘

impl<T, Z> Contains<Coordinate<T, Z>> for LineString<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn contains(&self, coord: &Coordinate<T, Z>) -> bool {
        if self.0.is_empty() {
            return false;
        }

        if coord == &self.0[0] || coord == self.0.last().unwrap() {
            return self.is_closed();
        }

        self.lines()
            .enumerate()
            .any(|(i, line)| line.contains(coord) || (i > 0 && coord == &line.start))
    }
}

impl<T, Z> Contains<Point<T, Z>> for LineString<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn contains(&self, p: &Point<T, Z>) -> bool {
        self.contains(&p.0)
    }
}

impl<T, Z> Contains<Line<T, Z>> for LineString<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn contains(&self, line: &Line<T, Z>) -> bool {
        if line.start == line.end {
            return self.contains(&line.start);
        }

        // We copy the line as we may truncate the line as
        // we find partial matches.
        let mut line = *line;
        let mut first_cut = None;

        let lines_iter = self.lines();
        let num_lines = lines_iter.len();

        // We need to repeat the logic twice to handle cases
        // where the linestring starts at the middle of the line.
        for (i, segment) in self.lines().chain(lines_iter).enumerate() {
            if i >= num_lines {
                // The first loop was done. If we never cut
                // the line, or at the cut segment again, we
                // can exit now.
                if let Some(upto_i) = first_cut {
                    if i >= num_lines + upto_i {
                        break;
                    }
                } else {
                    break;
                }
            }
            // Look for a segment that intersects at least
            // one of the end points.
            let other = if segment.intersects(&line.start) {
                line.end
            } else if segment.intersects(&line.end) {
                line.start
            } else {
                continue;
            };

            // If the other end point also intersects this
            // segment, then we are done.
            let new_inside = if segment.intersects(&other) {
                return true;
            }
            // otoh, if the line contains one of the ends of
            // the segments, then we truncate the line to
            // the part outside.
            else if line.contains(&segment.start) {
                segment.start
            } else if line.contains(&segment.end) {
                segment.end
            } else {
                continue;
            };

            first_cut = first_cut.or(Some(i));
            if other == line.start {
                line.end = new_inside;
            } else {
                line.start = new_inside;
            }
        }

        false
    }
}

impl<T, Z> Contains<LineString<T, Z>> for LineString<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn contains(&self, rhs: &LineString<T, Z>) -> bool {
        rhs.lines().all(|l| self.contains(&l))
    }
}

impl<T, Z> Contains<Point<T, Z>> for MultiLineString<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
    LineString<T, Z>: Contains<Point<T, Z>>,
{
    fn contains(&self, rhs: &Point<T, Z>) -> bool {
        self.iter().any(|ls| ls.contains(rhs))
    }
}

impl_contains_from_relate!(LineString<T, Z>, [Polygon<T, Z>, MultiPoint<T, Z>, MultiLineString<T, Z>, MultiPolygon<T, Z>, Rect<T, Z>, Triangle<T, Z>]);

impl_contains_from_relate!(MultiLineString<T, Z>, [Line<T, Z>, LineString<T, Z>, Polygon<T, Z>, MultiPoint<T, Z>, MultiLineString<T, Z>, MultiPolygon<T, Z>, Rect<T, Z>, Triangle<T, Z>]);
