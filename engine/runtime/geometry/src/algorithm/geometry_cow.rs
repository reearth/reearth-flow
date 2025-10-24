use std::borrow::Cow;

use crate::types::{
    coordnum::CoordNum, geometry::Geometry, geometry_collection::GeometryCollection, line::Line,
    line_string::LineString, multi_line_string::MultiLineString, multi_point::MultiPoint,
    multi_polygon::MultiPolygon, point::Point, polygon::Polygon, rect::Rect, triangle::Triangle,
};

#[derive(PartialEq, Debug)]
pub(crate) enum GeometryCow<'a, T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    Point(Cow<'a, Point<T, Z>>),
    Line(Cow<'a, Line<T, Z>>),
    LineString(Cow<'a, LineString<T, Z>>),
    Polygon(Cow<'a, Polygon<T, Z>>),
    MultiPoint(Cow<'a, MultiPoint<T, Z>>),
    MultiLineString(Cow<'a, MultiLineString<T, Z>>),
    MultiPolygon(Cow<'a, MultiPolygon<T, Z>>),
    GeometryCollection(Cow<'a, GeometryCollection<T, Z>>),
    Rect(Cow<'a, Rect<T, Z>>),
    Triangle(Cow<'a, Triangle<T, Z>>),
}

impl<'a, T: CoordNum, Z: CoordNum> From<&'a Geometry<T, Z>> for GeometryCow<'a, T, Z> {
    fn from(geometry: &'a Geometry<T, Z>) -> Self {
        match geometry {
            Geometry::Point(g) => GeometryCow::Point(Cow::Borrowed(g)),
            Geometry::Line(g) => GeometryCow::Line(Cow::Borrowed(g)),
            Geometry::LineString(g) => GeometryCow::LineString(Cow::Borrowed(g)),
            Geometry::Polygon(g) => GeometryCow::Polygon(Cow::Borrowed(g)),
            Geometry::MultiPoint(g) => GeometryCow::MultiPoint(Cow::Borrowed(g)),
            Geometry::MultiLineString(g) => GeometryCow::MultiLineString(Cow::Borrowed(g)),
            Geometry::MultiPolygon(g) => GeometryCow::MultiPolygon(Cow::Borrowed(g)),
            Geometry::Rect(g) => GeometryCow::Rect(Cow::Borrowed(g)),
            Geometry::Triangle(g) => GeometryCow::Triangle(Cow::Borrowed(g)),
            _ => unimplemented!(),
        }
    }
}

impl<'a, T: CoordNum, Z: CoordNum> From<&'a Point<T, Z>> for GeometryCow<'a, T, Z> {
    fn from(point: &'a Point<T, Z>) -> Self {
        GeometryCow::Point(Cow::Borrowed(point))
    }
}

impl<'a, T: CoordNum, Z: CoordNum> From<&'a LineString<T, Z>> for GeometryCow<'a, T, Z> {
    fn from(line_string: &'a LineString<T, Z>) -> Self {
        GeometryCow::LineString(Cow::Borrowed(line_string))
    }
}

impl<'a, T: CoordNum, Z: CoordNum> From<&'a Line<T, Z>> for GeometryCow<'a, T, Z> {
    fn from(line: &'a Line<T, Z>) -> Self {
        GeometryCow::Line(Cow::Borrowed(line))
    }
}

impl<'a, T: CoordNum, Z: CoordNum> From<&'a Polygon<T, Z>> for GeometryCow<'a, T, Z> {
    fn from(polygon: &'a Polygon<T, Z>) -> Self {
        GeometryCow::Polygon(Cow::Borrowed(polygon))
    }
}

impl<'a, T: CoordNum, Z: CoordNum> From<&'a MultiPoint<T, Z>> for GeometryCow<'a, T, Z> {
    fn from(multi_point: &'a MultiPoint<T, Z>) -> GeometryCow<'a, T, Z> {
        GeometryCow::MultiPoint(Cow::Borrowed(multi_point))
    }
}

impl<'a, T: CoordNum, Z: CoordNum> From<&'a MultiLineString<T, Z>> for GeometryCow<'a, T, Z> {
    fn from(multi_line_string: &'a MultiLineString<T, Z>) -> Self {
        GeometryCow::MultiLineString(Cow::Borrowed(multi_line_string))
    }
}

impl<'a, T: CoordNum, Z: CoordNum> From<&'a MultiPolygon<T, Z>> for GeometryCow<'a, T, Z> {
    fn from(multi_polygon: &'a MultiPolygon<T, Z>) -> Self {
        GeometryCow::MultiPolygon(Cow::Borrowed(multi_polygon))
    }
}

impl<'a, T: CoordNum, Z: CoordNum> From<&'a GeometryCollection<T, Z>> for GeometryCow<'a, T, Z> {
    fn from(geometry_collection: &'a GeometryCollection<T, Z>) -> Self {
        GeometryCow::GeometryCollection(Cow::Borrowed(geometry_collection))
    }
}

impl<'a, T: CoordNum, Z: CoordNum> From<&'a Rect<T, Z>> for GeometryCow<'a, T, Z> {
    fn from(rect: &'a Rect<T, Z>) -> Self {
        GeometryCow::Rect(Cow::Borrowed(rect))
    }
}

impl<'a, T: CoordNum, Z: CoordNum> From<&'a Triangle<T, Z>> for GeometryCow<'a, T, Z> {
    fn from(triangle: &'a Triangle<T, Z>) -> Self {
        GeometryCow::Triangle(Cow::Borrowed(triangle))
    }
}
