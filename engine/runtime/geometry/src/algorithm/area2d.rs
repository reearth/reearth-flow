use crate::types::{
    coordnum::{CoordFloat, CoordNum},
    geometry::Geometry2D,
    geometry_collection::GeometryCollection2D,
    line::Line2D,
    line_string::LineString2D,
    multi_line_string::MultiLineString2D,
    multi_point::MultiPoint2D,
    multi_polygon::MultiPolygon2D,
    point::Point2D,
    polygon::Polygon2D,
    rect::Rect2D,
    triangle::Triangle2D,
};

use super::map_coords::MapCoords;

pub(crate) fn twice_signed_ring_area2d<T>(linestring: &LineString2D<T>) -> T
where
    T: CoordNum,
{
    // LineString with less than 3 points is empty, or a
    // single point, or is not closed.
    if linestring.0.len() < 3 {
        return T::zero();
    }

    // Above test ensures the vector has at least 2 elements.
    // We check if linestring is closed, and return 0 otherwise.
    if linestring.0.first().unwrap() != linestring.0.last().unwrap() {
        return T::zero();
    }

    // Use a reasonable shift for the line-string coords
    // to avoid numerical-errors when summing the
    // determinants.
    //
    // Note: we can't use the `Centroid` trait as it
    // requires `T: Float` and in fact computes area in the
    // implementation. Another option is to use the average
    // of the coordinates, but it is not fool-proof to
    // divide by the length of the linestring (eg. a long
    // line-string with T = u8)
    let shift = linestring.0[0];

    let mut tmp = T::zero();
    for line in linestring.lines() {
        let line = line.map_coords(|c| c - shift);
        tmp = tmp + line.determinant2d();
    }

    tmp
}

pub trait Area2D<T>
where
    T: CoordNum,
{
    fn signed_area2d(&self) -> T;

    fn unsigned_area2d(&self) -> T;
}

// Calculation of simple (no interior holes) Polygon area
pub(crate) fn get_linestring_area2d<T>(linestring: &LineString2D<T>) -> T
where
    T: CoordFloat,
{
    twice_signed_ring_area2d(linestring) / (T::one() + T::one())
}

impl<T> Area2D<T> for Point2D<T>
where
    T: CoordNum,
{
    fn signed_area2d(&self) -> T {
        T::zero()
    }

    fn unsigned_area2d(&self) -> T {
        T::zero()
    }
}

impl<T> Area2D<T> for LineString2D<T>
where
    T: CoordNum,
{
    fn signed_area2d(&self) -> T {
        T::zero()
    }

    fn unsigned_area2d(&self) -> T {
        T::zero()
    }
}

impl<T> Area2D<T> for Line2D<T>
where
    T: CoordNum,
{
    fn signed_area2d(&self) -> T {
        T::zero()
    }

    fn unsigned_area2d(&self) -> T {
        T::zero()
    }
}

/// **Note.** The implementation handles polygons whose
/// holes do not all have the same orientation. The sign of
/// the output is the same as that of the exterior shell.
impl<T> Area2D<T> for Polygon2D<T>
where
    T: CoordFloat,
{
    fn signed_area2d(&self) -> T {
        let area = get_linestring_area2d(self.exterior());

        // We could use winding order here, but that would
        // result in computing the shoelace formula twice.
        let is_negative = area < T::zero();

        let area = self.interiors().iter().fold(area.abs(), |total, next| {
            total - get_linestring_area2d(next).abs()
        });

        if is_negative {
            -area
        } else {
            area
        }
    }

    fn unsigned_area2d(&self) -> T {
        self.signed_area2d().abs()
    }
}

impl<T> Area2D<T> for MultiPoint2D<T>
where
    T: CoordNum,
{
    fn signed_area2d(&self) -> T {
        T::zero()
    }

    fn unsigned_area2d(&self) -> T {
        T::zero()
    }
}

impl<T> Area2D<T> for MultiLineString2D<T>
where
    T: CoordNum,
{
    fn signed_area2d(&self) -> T {
        T::zero()
    }

    fn unsigned_area2d(&self) -> T {
        T::zero()
    }
}

impl<T> Area2D<T> for MultiPolygon2D<T>
where
    T: CoordFloat,
{
    fn signed_area2d(&self) -> T {
        self.0
            .iter()
            .fold(T::zero(), |total, next| total + next.signed_area2d())
    }

    fn unsigned_area2d(&self) -> T {
        self.0
            .iter()
            .fold(T::zero(), |total, next| total + next.signed_area2d().abs())
    }
}

/// Because a `Rect` has no winding order, the area will always be positive.
impl<T> Area2D<T> for Rect2D<T>
where
    T: CoordNum,
{
    fn signed_area2d(&self) -> T {
        self.width() * self.height()
    }

    fn unsigned_area2d(&self) -> T {
        self.width() * self.height()
    }
}

impl<T> Area2D<T> for Triangle2D<T>
where
    T: CoordFloat,
{
    fn signed_area2d(&self) -> T {
        self.to_lines()
            .iter()
            .fold(T::zero(), |total, line| total + line.determinant2d())
            / (T::one() + T::one())
    }

    fn unsigned_area2d(&self) -> T {
        self.signed_area2d().abs()
    }
}

impl<T> Area2D<T> for Geometry2D<T>
where
    T: CoordFloat,
{
    fn signed_area2d(&self) -> T {
        match self {
            Geometry2D::Point(p) => p.signed_area2d(),
            Geometry2D::Line(l) => l.signed_area2d(),
            Geometry2D::LineString(ls) => ls.signed_area2d(),
            Geometry2D::Polygon(p) => p.signed_area2d(),
            Geometry2D::MultiPoint(mp) => mp.signed_area2d(),
            Geometry2D::MultiLineString(mls) => mls.signed_area2d(),
            Geometry2D::MultiPolygon(mp) => mp.signed_area2d(),
            Geometry2D::Rect(r) => r.signed_area2d(),
            Geometry2D::Triangle(t) => t.signed_area2d(),
            _ => unimplemented!(),
        }
    }
    fn unsigned_area2d(&self) -> T {
        match self {
            Geometry2D::Point(p) => p.unsigned_area2d(),
            Geometry2D::Line(l) => l.unsigned_area2d(),
            Geometry2D::LineString(ls) => ls.unsigned_area2d(),
            Geometry2D::Polygon(p) => p.unsigned_area2d(),
            Geometry2D::MultiPoint(mp) => mp.unsigned_area2d(),
            Geometry2D::MultiLineString(mls) => mls.unsigned_area2d(),
            Geometry2D::MultiPolygon(mp) => mp.unsigned_area2d(),
            Geometry2D::Rect(r) => r.unsigned_area2d(),
            Geometry2D::Triangle(t) => t.unsigned_area2d(),
            _ => unimplemented!(),
        }
    }
}

impl<T> Area2D<T> for GeometryCollection2D<T>
where
    T: CoordFloat,
{
    fn signed_area2d(&self) -> T {
        self.0
            .iter()
            .map(|g| g.signed_area2d())
            .fold(T::zero(), |acc, next| acc + next)
    }

    fn unsigned_area2d(&self) -> T {
        self.0
            .iter()
            .map(|g| g.unsigned_area2d())
            .fold(T::zero(), |acc, next| acc + next)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        algorithm::{area2d::Area2D, map_coords::MapCoords},
        types::{coordinate::Coordinate2D, polygon::Polygon2D, line_string::LineString2D, multi_polygon::MultiPolygon2D},
    };

    #[test]
    fn area_polygon_numerical_stability() {
        let polygon = {
            use std::f64::consts::PI;
            const NUM_VERTICES: usize = 10;
            const ANGLE_INC: f64 = 2. * PI / NUM_VERTICES as f64;

            Polygon2D::new(
                (0..NUM_VERTICES)
                    .map(|i| {
                        let angle = i as f64 * ANGLE_INC;
                        Coordinate2D::new_(angle.cos(), angle.sin())
                    })
                    .collect::<Vec<_>>()
                    .into(),
                vec![],
            )
        };

        let area = polygon.signed_area2d();

        let shift = Coordinate2D::new_(1.5e8, 1.5e8);

        let polygon = polygon.map_coords(|c| c + shift);

        let new_area = polygon.signed_area2d();
        let err = (area - new_area).abs() / area;

        assert!(err < 1e-2);
    }

    #[test]
    fn test_square_area() {
        let exterior = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(10.0, 0.0),
            Coordinate2D::new_(10.0, 10.0),
            Coordinate2D::new_(0.0, 10.0),
            Coordinate2D::new_(0.0, 0.0),
        ]);
        let polygon = Polygon2D::new(exterior, vec![]);
        
        assert_eq!(polygon.unsigned_area2d(), 100.0);
    }

    #[test]
    fn test_triangle_area() {
        let exterior = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(10.0, 0.0),
            Coordinate2D::new_(0.0, 10.0),
            Coordinate2D::new_(0.0, 0.0),
        ]);
        let polygon = Polygon2D::new(exterior, vec![]);
        
        assert_eq!(polygon.unsigned_area2d(), 50.0);
    }

    #[test]
    fn test_polygon_with_hole_area() {
        let exterior = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(20.0, 0.0),
            Coordinate2D::new_(20.0, 20.0),
            Coordinate2D::new_(0.0, 20.0),
            Coordinate2D::new_(0.0, 0.0),
        ]);
        
        let hole = LineString2D::new(vec![
            Coordinate2D::new_(5.0, 5.0),
            Coordinate2D::new_(15.0, 5.0),
            Coordinate2D::new_(15.0, 15.0),
            Coordinate2D::new_(5.0, 15.0),
            Coordinate2D::new_(5.0, 5.0),
        ]);
        
        let polygon = Polygon2D::new(exterior, vec![hole]);
        assert_eq!(polygon.unsigned_area2d(), 300.0);
    }

    #[test]
    fn test_building_footprint_area() {
        let footprint = LineString2D::new(vec![
            Coordinate2D::new_(139.7503, 35.6851),
            Coordinate2D::new_(139.7506, 35.6851),
            Coordinate2D::new_(139.7506, 35.6854),
            Coordinate2D::new_(139.7503, 35.6854),
            Coordinate2D::new_(139.7503, 35.6851),
        ]);
        
        let polygon = Polygon2D::new(footprint, vec![]);
        let area = polygon.unsigned_area2d();
        
        assert!(area > 0.0);
    }

    #[test]
    fn test_multipolygon_area() {
        let poly1_exterior = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(5.0, 0.0),
            Coordinate2D::new_(5.0, 5.0),
            Coordinate2D::new_(0.0, 5.0),
            Coordinate2D::new_(0.0, 0.0),
        ]);
        
        let poly2_exterior = LineString2D::new(vec![
            Coordinate2D::new_(10.0, 10.0),
            Coordinate2D::new_(15.0, 10.0),
            Coordinate2D::new_(15.0, 15.0),
            Coordinate2D::new_(10.0, 15.0),
            Coordinate2D::new_(10.0, 10.0),
        ]);
        
        let poly1 = Polygon2D::new(poly1_exterior, vec![]);
        let poly2 = Polygon2D::new(poly2_exterior, vec![]);
        let multipolygon = MultiPolygon2D::from(vec![poly1, poly2]);
        
        assert_eq!(multipolygon.unsigned_area2d(), 50.0);
    }
}
