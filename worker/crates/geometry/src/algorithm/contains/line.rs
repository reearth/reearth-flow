use crate::{
    algorithm::{intersects::Intersects, GeoNum},
    types::{coordinate::Coordinate, line::Line, line_string::LineString, point::Point},
};

use super::Contains;

impl<T, Z> Contains<Coordinate<T, Z>> for Line<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn contains(&self, coord: &Coordinate<T, Z>) -> bool {
        if self.start == self.end {
            &self.start == coord
        } else {
            coord != &self.start && coord != &self.end && self.intersects(coord)
        }
    }
}

impl<T, Z> Contains<Point<T, Z>> for Line<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn contains(&self, p: &Point<T, Z>) -> bool {
        self.contains(&p.0)
    }
}

impl<T, Z> Contains<Line<T, Z>> for Line<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn contains(&self, line: &Line<T, Z>) -> bool {
        if line.start == line.end {
            self.contains(&line.start)
        } else {
            self.intersects(&line.start) && self.intersects(&line.end)
        }
    }
}

impl<T, Z> Contains<LineString<T, Z>> for Line<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn contains(&self, linestring: &LineString<T, Z>) -> bool {
        // Empty linestring has no interior, and not
        // contained in anything.
        if linestring.0.is_empty() {
            return false;
        }

        let first = linestring.0.first().unwrap();
        let mut all_equal = true;

        let all_intersects = linestring.0.iter().all(|c| {
            if c != first {
                all_equal = false;
            }
            self.intersects(c)
        });

        all_intersects && (!all_equal || self.contains(first))
    }
}
