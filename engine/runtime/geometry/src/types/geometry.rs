use core::any::type_name;
use std::convert::TryFrom;

use approx::{AbsDiffEq, RelativeEq};
use nusamai_projection::vshift::Jgd2011ToWgs84;
use serde::{Deserialize, Serialize};

use super::conversion::geojson::{
    create_geo_line_string_2d, create_geo_line_string_3d, create_geo_multi_line_string_2d,
    create_geo_multi_line_string_3d, create_geo_multi_polygon_2d, create_geo_multi_polygon_3d,
    create_geo_point_2d, create_geo_point_3d, create_geo_polygon_2d, create_geo_polygon_3d,
};
use super::coordinate::Coordinate;
use super::coordnum::{CoordFloat, CoordNum};
use super::line::Line;
use super::line_string::LineString;
use super::multi_line_string::MultiLineString;
use super::multi_point::{MultiPoint, MultiPoint2D, MultiPoint3D};
use super::multi_polygon::MultiPolygon;
use super::no_value::NoValue;
use super::point::Point;
use super::polygon::Polygon;
use super::rect::Rect;
use super::solid::Solid;
use super::traits::Elevation;
use super::triangle::Triangle;
use crate::error::Error;
use crate::types::csg::CSG;
use crate::utils::PointsCoplanar;

static EPSILON: f64 = 1e-10;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Geometry<T: CoordNum = f64, Z: CoordNum = f64> {
    CSG(Box<CSG<T, Z>>),
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
            Geometry::CSG(_) => "CSG",
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

    pub fn as_point(&self) -> Option<Point<T, Z>> {
        match self {
            Geometry::Point(p) => Some(*p),
            _ => None,
        }
    }

    pub fn as_line(&self) -> Option<Line<T, Z>> {
        match self {
            Geometry::Line(l) => Some(*l),
            _ => None,
        }
    }

    pub fn as_line_string(&self) -> Option<LineString<T, Z>> {
        match self {
            Geometry::LineString(ls) => Some(ls.clone()),
            _ => None,
        }
    }

    pub fn as_multi_line_string(&self) -> Option<MultiLineString<T, Z>> {
        match self {
            Geometry::MultiLineString(mls) => Some(mls.clone()),
            _ => None,
        }
    }

    pub fn as_polygon(&self) -> Option<Polygon<T, Z>> {
        match self {
            Geometry::Polygon(p) => Some(p.clone()),
            _ => None,
        }
    }

    pub fn as_multi_polygon(&self) -> Option<MultiPolygon<T, Z>> {
        match self {
            Geometry::MultiPolygon(mp) => Some(mp.clone()),
            _ => None,
        }
    }

    pub fn as_rect(&self) -> Option<Rect<T, Z>> {
        match self {
            Geometry::Rect(rect) => Some(*rect),
            _ => None,
        }
    }

    pub fn as_triangle(&self) -> Option<Triangle<T, Z>> {
        match self {
            Geometry::Triangle(triangle) => Some(*triangle),
            _ => None,
        }
    }

    pub fn as_solid(&self) -> Option<Solid<T, Z>> {
        match self {
            Geometry::Solid(solid) => Some(solid.clone()),
            _ => None,
        }
    }

    pub fn as_geometry_collection(&self) -> Option<Vec<Geometry<T, Z>>> {
        match self {
            Geometry::GeometryCollection(gc) => Some(gc.clone()),
            _ => None,
        }
    }

    pub fn get_all_coordinates(&self) -> Vec<Coordinate<T, Z>> {
        match self {
            Geometry::CSG(csg) => csg.get_all_vertex_coordinates(),
            Geometry::Point(p) => vec![p.0],
            Geometry::Line(l) => vec![l.start, l.end],
            Geometry::LineString(ls) => ls.0.clone(),
            Geometry::Polygon(poly) => {
                let mut coords = poly.exterior.0.clone();
                for interior in &poly.interiors {
                    coords.extend(interior.0.clone());
                }
                coords
            }
            Geometry::MultiPoint(mp) => mp.0.iter().map(|p| p.0).collect(),
            Geometry::MultiLineString(mls) => {
                let mut coords = Vec::new();
                for ls in &mls.0 {
                    coords.extend(ls.0.clone());
                }
                coords
            }
            Geometry::MultiPolygon(mp) => {
                let mut coords = Vec::new();
                for poly in &mp.0 {
                    coords.extend(poly.exterior.0.clone());
                    for interior in &poly.interiors {
                        coords.extend(interior.0.clone());
                    }
                }
                coords
            }
            Geometry::Rect(rect) => {
                vec![
                    Coordinate::new__(rect.min.x, rect.min.y, rect.min.z),
                    Coordinate::new__(rect.max.x, rect.min.y, rect.min.z),
                    Coordinate::new__(rect.min.x, rect.max.y, rect.min.z),
                    Coordinate::new__(rect.max.x, rect.max.y, rect.min.z),
                    Coordinate::new__(rect.min.x, rect.min.y, rect.max.z),
                    Coordinate::new__(rect.max.x, rect.min.y, rect.max.z),
                    Coordinate::new__(rect.min.x, rect.max.y, rect.max.z),
                    Coordinate::new__(rect.max.x, rect.max.y, rect.max.z),
                ]
            }
            Geometry::Triangle(triangle) => {
                vec![triangle.0, triangle.1, triangle.2]
            }
            Geometry::Solid(solid) => solid.get_all_vertex_coordinates(),
            Geometry::GeometryCollection(gc) => {
                let mut coords = Vec::new();
                for geom in gc {
                    coords.extend(geom.get_all_coordinates());
                }
                coords
            }
        }
    }
}

impl<T: CoordFloat, Z: CoordFloat> From<Geometry<T, Z>> for geojson::Value {
    fn from(geom: Geometry<T, Z>) -> Self {
        match geom {
            Geometry::Point(point) => point.into(),
            Geometry::Line(line) => line.into(),
            Geometry::LineString(line_string) => line_string.into(),
            Geometry::Polygon(polygon) => polygon.into(),
            Geometry::MultiPoint(multi_point) => multi_point.into(),
            Geometry::MultiLineString(multi_line_string) => multi_line_string.into(),
            Geometry::MultiPolygon(multi_point) => multi_point.into(),
            Geometry::Rect(rect) => rect.into(),
            Geometry::Triangle(triangle) => triangle.into(),
            Geometry::GeometryCollection(gc) => {
                let mut geometries = Vec::new();
                for g in gc {
                    geometries.push(g.into());
                }
                geojson::Value::GeometryCollection(geometries)
            }
            _ => unimplemented!(),
        }
    }
}

impl TryFrom<geojson::Value> for Geometry2D<f64> {
    type Error = crate::error::Error;

    fn try_from(value: geojson::Value) -> crate::error::Result<Self> {
        match value {
            geojson::Value::Point(ref point_type) => {
                Ok(Geometry2D::Point(create_geo_point_2d(point_type)))
            }
            geojson::Value::MultiPoint(ref multi_point_type) => Ok(Geometry2D::MultiPoint(
                MultiPoint2D::new(multi_point_type.iter().map(create_geo_point_2d).collect()),
            )),
            geojson::Value::LineString(ref line_string_type) => Ok(Geometry2D::LineString(
                create_geo_line_string_2d(line_string_type),
            )),
            geojson::Value::MultiLineString(ref multi_line_string_type) => {
                Ok(Geometry2D::MultiLineString(
                    create_geo_multi_line_string_2d(multi_line_string_type),
                ))
            }
            geojson::Value::Polygon(ref polygon_type) => {
                Ok(Geometry2D::Polygon(create_geo_polygon_2d(polygon_type)))
            }
            geojson::Value::MultiPolygon(ref multi_polygon_type) => Ok(Geometry2D::MultiPolygon(
                create_geo_multi_polygon_2d(multi_polygon_type),
            )),
            _ => Err(Error::mismatched_geometry("Geometry2D")),
        }
    }
}

impl TryFrom<geojson::Value> for Geometry3D<f64> {
    type Error = crate::error::Error;

    fn try_from(value: geojson::Value) -> crate::error::Result<Self> {
        match value {
            geojson::Value::Point(ref point_type) => {
                Ok(Geometry3D::Point(create_geo_point_3d(point_type)))
            }
            geojson::Value::MultiPoint(ref multi_point_type) => Ok(Geometry3D::MultiPoint(
                MultiPoint3D::new(multi_point_type.iter().map(create_geo_point_3d).collect()),
            )),
            geojson::Value::LineString(ref line_string_type) => Ok(Geometry3D::LineString(
                create_geo_line_string_3d(line_string_type),
            )),
            geojson::Value::MultiLineString(ref multi_line_string_type) => {
                Ok(Geometry3D::MultiLineString(
                    create_geo_multi_line_string_3d(multi_line_string_type),
                ))
            }
            geojson::Value::Polygon(ref polygon_type) => {
                Ok(Geometry3D::Polygon(create_geo_polygon_3d(polygon_type)))
            }
            geojson::Value::MultiPolygon(ref multi_polygon_type) => Ok(Geometry3D::MultiPolygon(
                create_geo_multi_polygon_3d(multi_polygon_type),
            )),
            _ => Err(Error::mismatched_geometry("Geometry3D")),
        }
    }
}

impl Geometry2D<f64> {
    pub fn elevation(&self) -> f64 {
        0.0
    }
}

impl Geometry3D<f64> {
    pub fn elevation(&self) -> f64 {
        match self {
            Self::CSG(csg) => csg.elevation(),
            Self::Point(p) => p.z(),
            Self::Line(l) => l.start.z,
            Self::LineString(ls) => ls.0.first().map(|c| c.z).unwrap_or(0.0),
            Self::Polygon(poly) => poly.exterior.0.first().map(|c| c.z).unwrap_or(0.0),
            Self::MultiPoint(mpoint) => mpoint.0.first().map(|p| p.z()).unwrap_or(0.0),
            Self::MultiLineString(mls) => mls
                .0
                .first()
                .map(|ls| ls.0.first().map(|c| c.z).unwrap_or(0.0))
                .unwrap_or(0.0),
            Self::MultiPolygon(mpoly) => mpoly
                .0
                .first()
                .map(|poly| poly.exterior.0.first().map(|c| c.z).unwrap_or(0.0))
                .unwrap_or(0.0),
            Self::Rect(rect) => rect.min.z,
            Self::Triangle(triangle) => triangle.0.z,
            Self::Solid(solid) => solid.elevation(),
            Self::GeometryCollection(gc) => gc.first().map(|g| g.elevation()).unwrap_or(0.0),
        }
    }

    #[inline]
    pub fn is_elevation_zero(&self) -> bool {
        match self {
            Geometry::CSG(csg) => csg.is_elevation_zero(),
            Geometry::Point(p) => p.is_elevation_zero(),
            Geometry::Line(l) => l.is_elevation_zero(),
            Geometry::LineString(ls) => ls.is_elevation_zero(),
            Geometry::Polygon(p) => p.is_elevation_zero(),
            Geometry::MultiPoint(mp) => mp.is_elevation_zero(),
            Geometry::MultiLineString(mls) => mls.is_elevation_zero(),
            Geometry::MultiPolygon(mp) => mp.is_elevation_zero(),
            Geometry::Rect(rect) => rect.is_elevation_zero(),
            Geometry::Triangle(triangle) => triangle.is_elevation_zero(),
            Geometry::Solid(solid) => solid.is_elevation_zero(),
            Geometry::GeometryCollection(gc) => gc.iter().all(|g| g.is_elevation_zero()),
        }
    }

    pub fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        match self {
            Self::CSG(csg) => csg.transform_inplace(jgd2wgs),
            Self::Point(p) => p.transform_inplace(jgd2wgs),
            Self::Line(l) => l.transform_inplace(jgd2wgs),
            Self::LineString(ls) => ls.transform_inplace(jgd2wgs),
            Self::Polygon(poly) => poly.transform_inplace(jgd2wgs),
            Self::MultiPoint(mpoint) => mpoint.transform_inplace(jgd2wgs),
            Self::MultiLineString(mls) => mls.transform_inplace(jgd2wgs),
            Self::MultiPolygon(mpoly) => mpoly.transform_inplace(jgd2wgs),
            Self::Rect(rect) => rect.transform_inplace(jgd2wgs),
            Self::Triangle(triangle) => triangle.transform_inplace(jgd2wgs),
            Self::Solid(solid) => solid.transform_inplace(jgd2wgs),
            Self::GeometryCollection(gc) => {
                for g in gc {
                    g.transform_inplace(jgd2wgs);
                }
            }
        }
    }

    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        match self {
            Self::CSG(csg) => csg.transform_offset(x, y, z),
            Self::Point(p) => p.transform_offset(x, y, z),
            Self::Line(l) => l.transform_offset(x, y, z),
            Self::LineString(ls) => ls.transform_offset(x, y, z),
            Self::Polygon(poly) => poly.transform_offset(x, y, z),
            Self::MultiPoint(mpoint) => mpoint.transform_offset(x, y, z),
            Self::MultiLineString(mls) => mls.transform_offset(x, y, z),
            Self::MultiPolygon(mpoly) => mpoly.transform_offset(x, y, z),
            Self::Rect(rect) => rect.transform_offset(x, y, z),
            Self::Triangle(triangle) => triangle.transform_offset(x, y, z),
            Self::Solid(solid) => solid.transform_offset(x, y, z),
            Self::GeometryCollection(gc) => {
                for g in gc {
                    g.transform_offset(x, y, z);
                }
            }
        }
    }
}

impl From<Geometry3D<f64>> for Geometry2D<f64> {
    fn from(geos: Geometry3D<f64>) -> Self {
        match geos {
            Geometry3D::CSG(_csg) => unreachable!(), // 2D CSG never exists
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
    pub fn are_points_coplanar(&self) -> Option<PointsCoplanar> {
        match self {
            Geometry::CSG(_csg) => unimplemented!(),
            Geometry::Point(_) => None,
            Geometry::Line(_) => None,
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
            Geometry::GeometryCollection(_) => unimplemented!(),
        }
    }
}

fn inner_type_name<T: CoordNum, Z: CoordNum>(geometry: Geometry<T, Z>) -> &'static str {
    match geometry {
        Geometry::CSG(_) => type_name::<CSG<T, Z>>(),
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
