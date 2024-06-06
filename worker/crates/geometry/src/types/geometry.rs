use core::any::type_name;
use std::convert::TryFrom;

use approx::{AbsDiffEq, RelativeEq};
use serde::{Deserialize, Serialize};

use super::coordnum::CoordNum;
use super::line::Line;
use super::line_string::LineString;
use super::multi_line_string::MultiLineString;
use super::multi_point::MultiPoint;
use super::multi_polygon::MultiPolygon;
use super::no_value::NoValue;
use super::point::Point;
use super::polygon::Polygon;
use super::rectangle::Rectangle;
use super::solid::Solid;
use super::triangle::Triangle;
use crate::error::Error;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, Hash)]
pub enum Geometry<T: CoordNum = f64, Z: CoordNum = f64> {
    Point(Point<T, Z>),
    Line(Line<T, Z>),
    LineString(LineString<T, Z>),
    Polygon(Polygon<T, Z>),
    MultiPoint(MultiPoint<T, Z>),
    MultiLineString(MultiLineString<T, Z>),
    MultiPolygon(MultiPolygon<T, Z>),
    Rectangle(Rectangle<T, Z>),
    Triangle(Triangle<T, Z>),
    Solid(Solid<T, Z>),
    GeometryCollection(Vec<Geometry<T, Z>>),
}

pub type Geometry2D<T = f64> = Geometry<T, NoValue>;
pub type Geometry3D<T = f64> = Geometry<T, T>;

impl<T: CoordNum, Z: CoordNum> From<Point<T, Z>> for Geometry<T, Z> {
    fn from(x: Point<T, Z>) -> Self {
        Self::Point(x)
    }
}

impl<T: CoordNum, Z: CoordNum> From<Line<T, Z>> for Geometry<T, Z> {
    fn from(x: Line<T, Z>) -> Self {
        Self::Line(x)
    }
}
impl<T: CoordNum, Z: CoordNum> From<LineString<T, Z>> for Geometry<T, Z> {
    fn from(x: LineString<T, Z>) -> Self {
        Self::LineString(x)
    }
}
impl<T: CoordNum, Z: CoordNum> From<Polygon<T, Z>> for Geometry<T, Z> {
    fn from(x: Polygon<T, Z>) -> Self {
        Self::Polygon(x)
    }
}
impl<T: CoordNum, Z: CoordNum> From<MultiPoint<T, Z>> for Geometry<T, Z> {
    fn from(x: MultiPoint<T, Z>) -> Self {
        Self::MultiPoint(x)
    }
}
impl<T: CoordNum, Z: CoordNum> From<MultiLineString<T, Z>> for Geometry<T, Z> {
    fn from(x: MultiLineString<T, Z>) -> Self {
        Self::MultiLineString(x)
    }
}
impl<T: CoordNum, Z: CoordNum> From<MultiPolygon<T, Z>> for Geometry<T, Z> {
    fn from(x: MultiPolygon<T, Z>) -> Self {
        Self::MultiPolygon(x)
    }
}

impl<T: CoordNum, Z: CoordNum> From<Rectangle<T, Z>> for Geometry<T, Z> {
    fn from(x: Rectangle<T, Z>) -> Self {
        Self::Rectangle(x)
    }
}

impl<T: CoordNum, Z: CoordNum> From<Triangle<T, Z>> for Geometry<T, Z> {
    fn from(x: Triangle<T, Z>) -> Self {
        Self::Triangle(x)
    }
}

impl<T: CoordNum, Z: CoordNum> From<Solid<T, Z>> for Geometry<T, Z> {
    fn from(x: Solid<T, Z>) -> Self {
        Self::Solid(x)
    }
}

macro_rules! try_from_geometry_impl {
    ($($type: ident),+ $(,)? ) => {
        $(
        /// Convert a Geometry enum into its inner type.
        ///
        /// Fails if the enum case does not match the type you are trying to convert it to.
        impl<T: CoordNum, Z: CoordNum> TryFrom<Geometry<T, Z>> for $type<T, Z> {
            type Error = Error;

            fn try_from(geom: Geometry<T, Z>) -> Result<Self, Self::Error> {
                match geom {
                    Geometry::$type(g) => Ok(g),
                    other => Err(Error::mismatched_geometry(inner_type_name(other)) )
                }
            }
        }
        )+
    }
}

try_from_geometry_impl!(
    Point,
    Line,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    Rectangle,
    Triangle,
);

fn inner_type_name<T: CoordNum, Z: CoordNum>(geometry: Geometry<T, Z>) -> &'static str {
    match geometry {
        Geometry::Point(_) => type_name::<Point<T, Z>>(),
        Geometry::Line(_) => type_name::<Line<T, Z>>(),
        Geometry::LineString(_) => type_name::<LineString<T, Z>>(),
        Geometry::Polygon(_) => type_name::<Polygon<T, Z>>(),
        Geometry::MultiPoint(_) => type_name::<MultiPoint<T, Z>>(),
        Geometry::MultiLineString(_) => type_name::<MultiLineString<T, Z>>(),
        Geometry::MultiPolygon(_) => type_name::<MultiPolygon<T, Z>>(),
        Geometry::Rectangle(_) => type_name::<Rectangle<T, Z>>(),
        Geometry::Triangle(_) => type_name::<Triangle<T, Z>>(),
        Geometry::Solid(_) => type_name::<Solid<T, Z>>(),
        Geometry::GeometryCollection(_) => type_name::<Vec<Geometry<T, Z>>>(),
    }
}

impl<T> RelativeEq for Geometry<T, T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum + RelativeEq,
{
    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        match (self, other) {
            (Geometry::Point(g1), Geometry::Point(g2)) => g1.relative_eq(g2, epsilon, max_relative),
            (Geometry::Line(g1), Geometry::Line(g2)) => g1.relative_eq(g2, epsilon, max_relative),
            (Geometry::LineString(g1), Geometry::LineString(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::Polygon(g1), Geometry::Polygon(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::MultiPoint(g1), Geometry::MultiPoint(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::MultiLineString(g1), Geometry::MultiLineString(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::MultiPolygon(g1), Geometry::MultiPolygon(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::Rectangle(g1), Geometry::Rectangle(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::Triangle(g1), Geometry::Triangle(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (_, _) => false,
        }
    }
}

impl<T: AbsDiffEq<Epsilon = T> + CoordNum> AbsDiffEq for Geometry<T, T> {
    type Epsilon = T;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        match (self, other) {
            (Geometry::Point(g1), Geometry::Point(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::Line(g1), Geometry::Line(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::LineString(g1), Geometry::LineString(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::Polygon(g1), Geometry::Polygon(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::MultiPoint(g1), Geometry::MultiPoint(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::MultiLineString(g1), Geometry::MultiLineString(g2)) => {
                g1.abs_diff_eq(g2, epsilon)
            }
            (Geometry::MultiPolygon(g1), Geometry::MultiPolygon(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::Rectangle(g1), Geometry::Rectangle(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::Triangle(g1), Geometry::Triangle(g2)) => g1.abs_diff_eq(g2, epsilon),
            (_, _) => false,
        }
    }
}
