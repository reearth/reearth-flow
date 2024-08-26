use num_traits::FromPrimitive;

use crate::types::{
    coordnum::CoordNum, face::Face, geometry::Geometry, geometry_collection::GeometryCollection,
    line::Line, line_string::LineString, multi_line_string::MultiLineString,
    multi_point::MultiPoint, multi_polygon::MultiPolygon, point::Point, polygon::Polygon,
    rect::Rect, solid::Solid, triangle::Triangle,
};

pub trait RemoveRepeatedPoints<T, Z>
where
    T: CoordNum + FromPrimitive,
    Z: CoordNum + FromPrimitive,
{
    /// Create a new geometry with (consecutive) repeated points removed.
    fn remove_repeated_points(&self) -> Self;
    /// Remove (consecutive) repeated points inplace.
    fn remove_repeated_points_mut(&mut self);
}

impl<T, Z> RemoveRepeatedPoints<T, Z> for MultiPoint<T, Z>
where
    T: CoordNum + FromPrimitive,
    Z: CoordNum + FromPrimitive,
{
    /// Create a MultiPoint with repeated points removed.
    fn remove_repeated_points(&self) -> Self {
        let mut points = vec![];
        for p in self.0.iter() {
            if !points.contains(p) {
                points.push(*p);
            }
        }
        MultiPoint(points)
    }

    /// Remove repeated points from a MultiPoint inplace.
    fn remove_repeated_points_mut(&mut self) {
        let mut points = vec![];
        for p in self.0.iter() {
            if !points.contains(p) {
                points.push(*p);
            }
        }
        self.0 = points;
    }
}

impl<T, Z> RemoveRepeatedPoints<T, Z> for LineString<T, Z>
where
    T: CoordNum + FromPrimitive,
    Z: CoordNum + FromPrimitive,
{
    /// Create a LineString with consecutive repeated points removed.
    fn remove_repeated_points(&self) -> Self {
        let mut coords = self.0.clone();
        coords.dedup();
        LineString(coords)
    }

    /// Remove consecutive repeated points from a LineString inplace.
    fn remove_repeated_points_mut(&mut self) {
        self.0.dedup();
    }
}

impl<T, Z> RemoveRepeatedPoints<T, Z> for Polygon<T, Z>
where
    T: CoordNum + FromPrimitive,
    Z: CoordNum + FromPrimitive,
{
    /// Create a Polygon with consecutive repeated points removed.
    fn remove_repeated_points(&self) -> Self {
        Polygon::new(
            self.exterior().remove_repeated_points(),
            self.interiors()
                .iter()
                .map(|ls| ls.remove_repeated_points())
                .collect(),
        )
    }

    /// Remove consecutive repeated points from a Polygon inplace.
    fn remove_repeated_points_mut(&mut self) {
        self.exterior_mut(|exterior| exterior.remove_repeated_points_mut());
        self.interiors_mut(|interiors| {
            for interior in interiors {
                interior.remove_repeated_points_mut();
            }
        });
    }
}

impl<T, Z> RemoveRepeatedPoints<T, Z> for MultiLineString<T, Z>
where
    T: CoordNum + FromPrimitive,
    Z: CoordNum + FromPrimitive,
{
    /// Create a MultiLineString with consecutive repeated points removed.
    fn remove_repeated_points(&self) -> Self {
        MultiLineString::new(
            self.0
                .iter()
                .map(|ls| ls.remove_repeated_points())
                .collect(),
        )
    }

    /// Remove consecutive repeated points from a MultiLineString inplace.
    fn remove_repeated_points_mut(&mut self) {
        for ls in self.0.iter_mut() {
            ls.remove_repeated_points_mut();
        }
    }
}

impl<T, Z> RemoveRepeatedPoints<T, Z> for MultiPolygon<T, Z>
where
    T: CoordNum + FromPrimitive,
    Z: CoordNum + FromPrimitive,
{
    /// Create a MultiPolygon with consecutive repeated points removed.
    fn remove_repeated_points(&self) -> Self {
        MultiPolygon::new(self.0.iter().map(|p| p.remove_repeated_points()).collect())
    }

    /// Remove consecutive repeated points from a MultiPolygon inplace.
    fn remove_repeated_points_mut(&mut self) {
        for p in self.0.iter_mut() {
            p.remove_repeated_points_mut();
        }
    }
}

// Implementation for types that are not candidate for coordinates removal
// (Point / Line / Triangle / Rect), where `remove_repeated_points` returns a clone of the geometry
// and `remove_repeated_points_mut` is a no-op.
macro_rules! impl_for_not_candidate_types {
    ($type:ident) => {
        impl<T, Z> RemoveRepeatedPoints<T, Z> for $type<T, Z>
        where
            T: CoordNum + FromPrimitive,
            Z: CoordNum + FromPrimitive,
        {
            fn remove_repeated_points(&self) -> Self {
                self.clone()
            }

            fn remove_repeated_points_mut(&mut self) {
                // no-op
            }
        }
    };
}

impl_for_not_candidate_types!(Point);
impl_for_not_candidate_types!(Rect);
impl_for_not_candidate_types!(Triangle);
impl_for_not_candidate_types!(Line);
impl_for_not_candidate_types!(Solid);
impl_for_not_candidate_types!(Face);

impl<T, Z> RemoveRepeatedPoints<T, Z> for GeometryCollection<T, Z>
where
    T: CoordNum + FromPrimitive,
    Z: CoordNum + FromPrimitive,
{
    /// Create a GeometryCollection with (consecutive) repeated points
    /// of its geometries removed.
    fn remove_repeated_points(&self) -> Self {
        GeometryCollection::from(
            self.0
                .iter()
                .map(|g| g.remove_repeated_points())
                .collect::<Vec<_>>(),
        )
    }

    /// Remove (consecutive) repeated points of its geometries from a GeometryCollection inplace.
    fn remove_repeated_points_mut(&mut self) {
        for g in self.0.iter_mut() {
            g.remove_repeated_points_mut();
        }
    }
}

impl<T, Z> RemoveRepeatedPoints<T, Z> for Geometry<T, Z>
where
    T: CoordNum + FromPrimitive,
    Z: CoordNum + FromPrimitive,
{
    /// Create a Geometry with consecutive repeated points removed.
    fn remove_repeated_points(&self) -> Self {
        match self {
            Geometry::Point(p) => Geometry::Point(p.remove_repeated_points()),
            Geometry::Line(l) => Geometry::Line(l.remove_repeated_points()),
            Geometry::LineString(ls) => Geometry::LineString(ls.remove_repeated_points()),
            Geometry::Polygon(p) => Geometry::Polygon(p.remove_repeated_points()),
            Geometry::MultiPoint(mp) => Geometry::MultiPoint(mp.remove_repeated_points()),
            Geometry::MultiLineString(mls) => {
                Geometry::MultiLineString(mls.remove_repeated_points())
            }
            Geometry::MultiPolygon(mp) => Geometry::MultiPolygon(mp.remove_repeated_points()),
            Geometry::Rect(r) => Geometry::Rect(r.remove_repeated_points()),
            Geometry::Triangle(t) => Geometry::Triangle(t.remove_repeated_points()),
            Geometry::GeometryCollection(gc) => Geometry::GeometryCollection(
                gc.iter().map(|g| g.remove_repeated_points()).collect(),
            ),
            _ => unimplemented!(),
        }
    }

    /// Remove consecutive repeated points from a Geometry inplace.
    fn remove_repeated_points_mut(&mut self) {
        match self {
            Geometry::Point(p) => p.remove_repeated_points_mut(),
            Geometry::Line(l) => l.remove_repeated_points_mut(),
            Geometry::LineString(ls) => ls.remove_repeated_points_mut(),
            Geometry::Polygon(p) => p.remove_repeated_points_mut(),
            Geometry::MultiPoint(mp) => mp.remove_repeated_points_mut(),
            Geometry::MultiLineString(mls) => mls.remove_repeated_points_mut(),
            Geometry::MultiPolygon(mp) => mp.remove_repeated_points_mut(),
            Geometry::Rect(r) => r.remove_repeated_points_mut(),
            Geometry::Triangle(t) => t.remove_repeated_points_mut(),
            Geometry::GeometryCollection(gc) => {
                for g in gc.iter_mut() {
                    g.remove_repeated_points_mut();
                }
            }
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        point,
        types::{
            coordinate::Coordinate2D, line_string::LineString2D, multi_point::MultiPoint2D,
            polygon::Polygon2D,
        },
    };

    fn make_test_mp_integer() -> MultiPoint2D<i32> {
        MultiPoint(vec![
            point!(x: 0, y: 0),
            point!(x: 1, y: 1),
            point!(x: 1, y: 1),
            point!(x: 1, y: 1),
            point!(x: 2, y: 2),
            point!(x: 0, y: 0),
        ])
    }

    fn make_result_mp_integer() -> MultiPoint2D<i32> {
        MultiPoint(vec![
            point!(x: 0, y: 0),
            point!(x: 1, y: 1),
            point!(x: 2, y: 2),
        ])
    }

    fn make_test_mp1() -> MultiPoint2D<f64> {
        MultiPoint(vec![
            point!(x: 0.,y:  0.),
            point!(x: 1.,y:  1.),
            point!(x: 1.,y:  1.),
            point!(x: 1.,y:  1.),
            point!(x: 2.,y:  2.),
            point!(x: 0.,y:  0.),
        ])
    }

    fn make_result_mp1() -> MultiPoint2D<f64> {
        MultiPoint(vec![
            point!(x: 0.,y:  0.),
            point!(x: 1.,y:  1.),
            point!(x: 2.,y:  2.),
        ])
    }

    fn make_test_line1() -> LineString2D<f64> {
        LineString2D::new(vec![
            Coordinate2D::new_(0., 0.),
            Coordinate2D::new_(1., 1.),
            Coordinate2D::new_(1., 1.),
            Coordinate2D::new_(1., 1.),
            Coordinate2D::new_(2., 2.),
            Coordinate2D::new_(2., 2.),
            Coordinate2D::new_(0., 0.),
        ])
    }

    fn make_result_line1() -> LineString2D<f64> {
        LineString2D::new(vec![
            Coordinate2D::new_(0., 0.),
            Coordinate2D::new_(1., 1.),
            Coordinate2D::new_(2., 2.),
            Coordinate2D::new_(0., 0.),
        ])
    }

    fn make_test_line2() -> LineString2D<f64> {
        LineString2D::new(vec![
            Coordinate2D::new_(10., 10.),
            Coordinate2D::new_(11., 11.),
            Coordinate2D::new_(11., 11.),
            Coordinate2D::new_(11., 11.),
            Coordinate2D::new_(12., 12.),
            Coordinate2D::new_(12., 12.),
            Coordinate2D::new_(10., 10.),
        ])
    }

    fn make_result_line2() -> LineString2D<f64> {
        LineString2D::new(vec![
            Coordinate2D::new_(10., 10.),
            Coordinate2D::new_(11., 11.),
            Coordinate2D::new_(12., 12.),
            Coordinate2D::new_(10., 10.),
        ])
    }

    fn make_test_poly1() -> Polygon2D<f64> {
        Polygon2D::new(
            LineString2D::new(vec![
                Coordinate2D::new_(0., 0.),
                Coordinate2D::new_(1., 1.),
                Coordinate2D::new_(1., 1.),
                Coordinate2D::new_(1., 1.),
                Coordinate2D::new_(0., 2.),
                Coordinate2D::new_(0., 2.),
                Coordinate2D::new_(0., 0.),
            ]),
            vec![],
        )
    }

    fn make_result_poly1() -> Polygon2D<f64> {
        Polygon2D::new(
            LineString2D::new(vec![
                Coordinate2D::new_(0., 0.),
                Coordinate2D::new_(1., 1.),
                Coordinate2D::new_(0., 2.),
                Coordinate2D::new_(0., 0.),
            ]),
            vec![],
        )
    }

    fn make_test_poly2() -> Polygon2D<f64> {
        Polygon2D::new(
            LineString2D::new(vec![
                Coordinate2D::new_(10., 10.),
                Coordinate2D::new_(11., 11.),
                Coordinate2D::new_(11., 11.),
                Coordinate2D::new_(11., 11.),
                Coordinate2D::new_(10., 12.),
                Coordinate2D::new_(10., 12.),
                Coordinate2D::new_(10., 10.),
            ]),
            vec![],
        )
    }

    fn make_result_poly2() -> Polygon2D<f64> {
        Polygon2D::new(
            LineString2D::new(vec![
                Coordinate2D::new_(10., 10.),
                Coordinate2D::new_(11., 11.),
                Coordinate2D::new_(10., 12.),
                Coordinate2D::new_(10., 10.),
            ]),
            vec![],
        )
    }

    #[test]
    fn test_remove_repeated_points_multipoint_integer() {
        let mp = make_test_mp_integer();
        let expected = make_result_mp_integer();

        assert_eq!(mp.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_multipoint() {
        let mp = make_test_mp1();
        let expected = make_result_mp1();

        assert_eq!(mp.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_linestring() {
        let ls = make_test_line1();
        let expected = make_result_line1();

        assert_eq!(ls.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_polygon() {
        let poly = make_test_poly1();
        let expected = make_result_poly1();

        assert_eq!(poly.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_multilinestring() {
        let mls = MultiLineString(vec![make_test_line1(), make_test_line2()]);

        let expected = MultiLineString(vec![make_result_line1(), make_result_line2()]);

        assert_eq!(mls.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_multipolygon() {
        let mpoly = MultiPolygon(vec![make_test_poly1(), make_test_poly2()]);

        let expected = MultiPolygon(vec![make_result_poly1(), make_result_poly2()]);

        assert_eq!(mpoly.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_mut_multipoint_integer() {
        let mut mp = make_test_mp_integer();
        mp.remove_repeated_points_mut();
        let expected = make_result_mp_integer();

        assert_eq!(mp, expected);
    }

    #[test]
    fn test_remove_repeated_points_mut_multipoint() {
        let mut mp = make_test_mp1();
        mp.remove_repeated_points_mut();
        let expected = make_result_mp1();

        assert_eq!(mp, expected);
    }

    #[test]
    fn test_remove_repeated_points_mut_linestring() {
        let mut ls = make_test_line1();
        ls.remove_repeated_points_mut();
        let expected = make_result_line1();

        assert_eq!(ls, expected);
    }

    #[test]
    fn test_remove_repeated_points_mut_polygon() {
        let mut poly = make_test_poly1();
        poly.remove_repeated_points_mut();
        let expected = make_result_poly1();

        assert_eq!(poly, expected);
    }

    #[test]
    fn test_remove_repeated_points_mut_multilinestring() {
        let mut mls = MultiLineString(vec![make_test_line1(), make_test_line2()]);
        mls.remove_repeated_points_mut();
        let expected = MultiLineString(vec![make_result_line1(), make_result_line2()]);

        assert_eq!(mls, expected);
    }

    #[test]
    fn test_remove_repeated_points_mut_multipolygon() {
        let mut mpoly = MultiPolygon(vec![make_test_poly1(), make_test_poly2()]);
        mpoly.remove_repeated_points_mut();
        let expected = MultiPolygon(vec![make_result_poly1(), make_result_poly2()]);

        assert_eq!(mpoly, expected);
    }
}
