use std::borrow::Cow;

use super::{
    geometry::Geometry3D, line::Line3D, line_string::LineString3D,
    multi_line_string::MultiLineString3D, multi_point::MultiPoint3D, multi_polygon::MultiPolygon3D,
    point::Point3D, polygon::Polygon3D, rectangle::Rectangle3D, triangle::Triangle3D,
};

pub enum GeometryCow3D<'a> {
    Point(Cow<'a, Point3D<f64>>),
    Line(Cow<'a, Line3D<f64>>),
    LineString(Cow<'a, LineString3D<f64>>),
    Polygon(Cow<'a, Polygon3D<f64>>),
    MultiPoint(Cow<'a, MultiPoint3D<f64>>),
    MultiLineString(Cow<'a, MultiLineString3D<f64>>),
    MultiPolygon(Cow<'a, MultiPolygon3D<f64>>),
    Rectangle(Cow<'a, Rectangle3D<f64>>),
    Triangle(Cow<'a, Triangle3D<f64>>),
}

impl<'a> From<&'a Geometry3D<f64>> for GeometryCow3D<'a> {
    fn from(geometry: &'a Geometry3D<f64>) -> Self {
        match geometry {
            Geometry3D::Point(g) => GeometryCow3D::Point(Cow::Borrowed(g)),
            Geometry3D::Line(g) => GeometryCow3D::Line(Cow::Borrowed(g)),
            Geometry3D::LineString(g) => GeometryCow3D::LineString(Cow::Borrowed(g)),
            Geometry3D::Polygon(g) => GeometryCow3D::Polygon(Cow::Borrowed(g)),
            Geometry3D::MultiPoint(g) => GeometryCow3D::MultiPoint(Cow::Borrowed(g)),
            Geometry3D::MultiLineString(g) => GeometryCow3D::MultiLineString(Cow::Borrowed(g)),
            Geometry3D::MultiPolygon(g) => GeometryCow3D::MultiPolygon(Cow::Borrowed(g)),
            Geometry3D::Rectangle(g) => GeometryCow3D::Rectangle(Cow::Borrowed(g)),
            Geometry3D::Triangle(g) => GeometryCow3D::Triangle(Cow::Borrowed(g)),
        }
    }
}

impl<'a> From<&'a Point3D<f64>> for GeometryCow3D<'a> {
    fn from(point: &'a Point3D<f64>) -> Self {
        GeometryCow3D::Point(Cow::Borrowed(point))
    }
}

impl<'a> From<&'a LineString3D<f64>> for GeometryCow3D<'a> {
    fn from(line_string: &'a LineString3D<f64>) -> Self {
        GeometryCow3D::LineString(Cow::Borrowed(line_string))
    }
}

impl<'a> From<&'a Line3D<f64>> for GeometryCow3D<'a> {
    fn from(line: &'a Line3D<f64>) -> Self {
        GeometryCow3D::Line(Cow::Borrowed(line))
    }
}

impl<'a> From<&'a Polygon3D<f64>> for GeometryCow3D<'a> {
    fn from(polygon: &'a Polygon3D<f64>) -> Self {
        GeometryCow3D::Polygon(Cow::Borrowed(polygon))
    }
}

impl<'a> From<&'a MultiPoint3D<f64>> for GeometryCow3D<'a> {
    fn from(multi_point: &'a MultiPoint3D<f64>) -> GeometryCow3D<'a> {
        GeometryCow3D::MultiPoint(Cow::Borrowed(multi_point))
    }
}

impl<'a> From<&'a MultiLineString3D<f64>> for GeometryCow3D<'a> {
    fn from(multi_line_string: &'a MultiLineString3D<f64>) -> Self {
        GeometryCow3D::MultiLineString(Cow::Borrowed(multi_line_string))
    }
}

impl<'a> From<&'a MultiPolygon3D<f64>> for GeometryCow3D<'a> {
    fn from(multi_polygon: &'a MultiPolygon3D<f64>) -> Self {
        GeometryCow3D::MultiPolygon(Cow::Borrowed(multi_polygon))
    }
}

impl<'a> From<&'a Rectangle3D<f64>> for GeometryCow3D<'a> {
    fn from(rectangle: &'a Rectangle3D<f64>) -> Self {
        GeometryCow3D::Rectangle(Cow::Borrowed(rectangle))
    }
}

impl<'a> From<&'a Triangle3D<f64>> for GeometryCow3D<'a> {
    fn from(triangle: &'a Triangle3D<f64>) -> Self {
        GeometryCow3D::Triangle(Cow::Borrowed(triangle))
    }
}
