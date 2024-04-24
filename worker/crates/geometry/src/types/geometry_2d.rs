use std::borrow::Cow;

use super::{
    geometry::Geometry2D, geometry_collection::GeometryCollection2D, line::Line2D,
    line_string::LineString2D, multi_line_string::MultiLineString2D, multi_point::MultiPoint2D,
    multi_polygon::MultiPolygon2D, point::Point2D, polygon::Polygon2D, rectangle::Rectangle2D,
    triangle::Triangle2D,
};

pub enum GeometryCow2D<'a> {
    Point(Cow<'a, Point2D<f64>>),
    Line(Cow<'a, Line2D<f64>>),
    LineString(Cow<'a, LineString2D<f64>>),
    Polygon(Cow<'a, Polygon2D<f64>>),
    MultiPoint(Cow<'a, MultiPoint2D<f64>>),
    MultiLineString(Cow<'a, MultiLineString2D<f64>>),
    MultiPolygon(Cow<'a, MultiPolygon2D<f64>>),
    GeometryCollection(Cow<'a, GeometryCollection2D<f64>>),
    Rectangle(Cow<'a, Rectangle2D<f64>>),
    Triangle(Cow<'a, Triangle2D<f64>>),
}

impl<'a> From<&'a Geometry2D<f64>> for GeometryCow2D<'a> {
    fn from(geometry: &'a Geometry2D<f64>) -> Self {
        match geometry {
            Geometry2D::Point(g) => GeometryCow2D::Point(Cow::Borrowed(g)),
            Geometry2D::Line(g) => GeometryCow2D::Line(Cow::Borrowed(g)),
            Geometry2D::LineString(g) => GeometryCow2D::LineString(Cow::Borrowed(g)),
            Geometry2D::Polygon(g) => GeometryCow2D::Polygon(Cow::Borrowed(g)),
            Geometry2D::MultiPoint(g) => GeometryCow2D::MultiPoint(Cow::Borrowed(g)),
            Geometry2D::MultiLineString(g) => GeometryCow2D::MultiLineString(Cow::Borrowed(g)),
            Geometry2D::MultiPolygon(g) => GeometryCow2D::MultiPolygon(Cow::Borrowed(g)),
            Geometry2D::GeometryCollection(g) => {
                GeometryCow2D::GeometryCollection(Cow::Borrowed(g))
            }
            Geometry2D::Rectangle(g) => GeometryCow2D::Rectangle(Cow::Borrowed(g)),
            Geometry2D::Triangle(g) => GeometryCow2D::Triangle(Cow::Borrowed(g)),
        }
    }
}

impl<'a> From<&'a Point2D<f64>> for GeometryCow2D<'a> {
    fn from(point: &'a Point2D<f64>) -> Self {
        GeometryCow2D::Point(Cow::Borrowed(point))
    }
}

impl<'a> From<&'a LineString2D<f64>> for GeometryCow2D<'a> {
    fn from(line_string: &'a LineString2D<f64>) -> Self {
        GeometryCow2D::LineString(Cow::Borrowed(line_string))
    }
}

impl<'a> From<&'a Line2D<f64>> for GeometryCow2D<'a> {
    fn from(line: &'a Line2D<f64>) -> Self {
        GeometryCow2D::Line(Cow::Borrowed(line))
    }
}

impl<'a> From<&'a Polygon2D<f64>> for GeometryCow2D<'a> {
    fn from(polygon: &'a Polygon2D<f64>) -> Self {
        GeometryCow2D::Polygon(Cow::Borrowed(polygon))
    }
}

impl<'a> From<&'a MultiPoint2D<f64>> for GeometryCow2D<'a> {
    fn from(multi_point: &'a MultiPoint2D<f64>) -> GeometryCow2D<'a> {
        GeometryCow2D::MultiPoint(Cow::Borrowed(multi_point))
    }
}

impl<'a> From<&'a MultiLineString2D<f64>> for GeometryCow2D<'a> {
    fn from(multi_line_string: &'a MultiLineString2D<f64>) -> Self {
        GeometryCow2D::MultiLineString(Cow::Borrowed(multi_line_string))
    }
}

impl<'a> From<&'a MultiPolygon2D<f64>> for GeometryCow2D<'a> {
    fn from(multi_polygon: &'a MultiPolygon2D<f64>) -> Self {
        GeometryCow2D::MultiPolygon(Cow::Borrowed(multi_polygon))
    }
}

impl<'a> From<&'a GeometryCollection2D<f64>> for GeometryCow2D<'a> {
    fn from(geometry_collection: &'a GeometryCollection2D<f64>) -> Self {
        GeometryCow2D::GeometryCollection(Cow::Borrowed(geometry_collection))
    }
}

impl<'a> From<&'a Rectangle2D<f64>> for GeometryCow2D<'a> {
    fn from(rect: &'a Rectangle2D<f64>) -> Self {
        GeometryCow2D::Rectangle(Cow::Borrowed(rect))
    }
}

impl<'a> From<&'a Triangle2D<f64>> for GeometryCow2D<'a> {
    fn from(triangle: &'a Triangle2D<f64>) -> Self {
        GeometryCow2D::Triangle(Cow::Borrowed(triangle))
    }
}
