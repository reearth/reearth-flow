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
use super::rect::Rect;
use super::solid::Solid;
use super::triangle::Triangle;
use crate::error::Error;

static EPSILON: f64 = 1e-10;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, Hash)]
pub enum Geometry<T: CoordNum = f64, Z: CoordNum = f64> {
    Point(Point<T, Z>),
    Line(Line<T, Z>),
    LineString(LineString<T, Z>),
    Polygon(Polygon<T, Z>),
    MultiPoint(MultiPoint<T, Z>),
    MultiLineString(MultiLineString<T, Z>),
    MultiPolygon(MultiPolygon<T, Z>),
    Rect(Rect<T, Z>),
    Triangle(Triangle<T, Z>),
    Solid(Solid<T, Z>),
    GeometryCollection(Vec<Geometry<T, Z>>),
}

pub type Geometry2D<T = f64> = Geometry<T, NoValue>;
pub type Geometry3D<T = f64> = Geometry<T, T>;

impl<T: CoordNum, Z: CoordNum> Geometry<T, Z> {
    pub fn name(&self) -> &'static str {
        match self {
            Geometry::Point(_) => "Point",
            Geometry::Line(_) => "Line",
            Geometry::LineString(_) => "LineString",
            Geometry::Polygon(_) => "Polygon",
            Geometry::MultiPoint(_) => "MultiPoint",
            Geometry::MultiLineString(_) => "MultiLineString",
            Geometry::MultiPolygon(_) => "MultiPolygon",
            Geometry::Rect(_) => "Rect",
            Geometry::Triangle(_) => "Triangle",
            Geometry::Solid(_) => "Solid",
            Geometry::GeometryCollection(_) => "GeometryCollection",
        }
    }
}

impl From<Geometry3D<f64>> for Geometry2D<f64> {
    fn from(geos: Geometry3D<f64>) -> Self {
        match geos {
            Geometry3D::Point(p) => Geometry2D::Point(p.into()),
            Geometry3D::Line(l) => Geometry2D::Line(l.into()),
            Geometry3D::LineString(ls) => Geometry2D::LineString(ls.into()),
            Geometry3D::Polygon(p) => Geometry2D::Polygon(p.into()),
            Geometry3D::MultiPoint(mp) => Geometry2D::MultiPoint(mp.into()),
            Geometry3D::MultiLineString(mls) => Geometry2D::MultiLineString(mls.into()),
            Geometry3D::MultiPolygon(mp) => Geometry2D::MultiPolygon(mp.into()),
            Geometry3D::Rect(rect) => Geometry2D::Rect(rect.into()),
            Geometry3D::Triangle(triangle) => Geometry2D::Triangle(triangle.into()),
            Geometry3D::Solid(solid) => Geometry2D::Solid(solid.into()),
            Geometry3D::GeometryCollection(gc) => {
                let mut new_gc = Vec::new();
                for g in gc {
                    new_gc.push(g.into());
                }
                Geometry2D::GeometryCollection(new_gc)
            }
        }
    }
}

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

impl<T: CoordNum, Z: CoordNum> From<Rect<T, Z>> for Geometry<T, Z> {
    fn from(x: Rect<T, Z>) -> Self {
        Self::Rect(x)
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
    Rect,
    Triangle,
);

impl Geometry2D<f64> {
    pub fn are_points_coplanar(&self) -> bool {
        true
    }
}

impl Geometry3D<f64> {
    pub fn are_points_coplanar(&self) -> bool {
        match self {
            Geometry::Point(_) => true,
            Geometry::Line(_) => true,
            Geometry::LineString(ls) => {
                crate::utils::are_points_coplanar(ls.clone().into(), EPSILON)
            }
            Geometry::Polygon(polygon) => {
                crate::utils::are_points_coplanar(polygon.clone().into(), EPSILON)
            }
            Geometry::MultiPoint(mpolygon) => {
                crate::utils::are_points_coplanar(mpolygon.clone().into(), EPSILON)
            }
            Geometry::MultiLineString(mls) => {
                crate::utils::are_points_coplanar(mls.clone().into(), EPSILON)
            }
            Geometry::MultiPolygon(mpolygon) => {
                crate::utils::are_points_coplanar(mpolygon.clone().into(), EPSILON)
            }
            Geometry::Rect(rect) => crate::utils::are_points_coplanar((*rect).into(), EPSILON),
            Geometry::Triangle(_) => unimplemented!(),
            Geometry::Solid(_) => unimplemented!(),
            Geometry::GeometryCollection(gc) => {
                gc.iter().all(|geometry| geometry.are_points_coplanar())
            }
        }
    }
}

fn inner_type_name<T: CoordNum, Z: CoordNum>(geometry: Geometry<T, Z>) -> &'static str {
    match geometry {
        Geometry::Point(_) => type_name::<Point<T, Z>>(),
        Geometry::Line(_) => type_name::<Line<T, Z>>(),
        Geometry::LineString(_) => type_name::<LineString<T, Z>>(),
        Geometry::Polygon(_) => type_name::<Polygon<T, Z>>(),
        Geometry::MultiPoint(_) => type_name::<MultiPoint<T, Z>>(),
        Geometry::MultiLineString(_) => type_name::<MultiLineString<T, Z>>(),
        Geometry::MultiPolygon(_) => type_name::<MultiPolygon<T, Z>>(),
        Geometry::Rect(_) => type_name::<Rect<T, Z>>(),
        Geometry::Triangle(_) => type_name::<Triangle<T, Z>>(),
        Geometry::Solid(_) => type_name::<Solid<T, Z>>(),
        Geometry::GeometryCollection(_) => type_name::<Vec<Geometry<T, Z>>>(),
    }
}

pub fn all_type_names() -> Vec<String> {
    [
        "Point",
        "Line",
        "LineString",
        "Polygon",
        "MultiPoint",
        "MultiLineString",
        "MultiPolygon",
        "Rect",
        "Triangle",
        "Solid",
        "GeometryCollection",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect()
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
            (Geometry::Triangle(g1), Geometry::Triangle(g2)) => g1.abs_diff_eq(g2, epsilon),
            (_, _) => false,
        }
    }
}
