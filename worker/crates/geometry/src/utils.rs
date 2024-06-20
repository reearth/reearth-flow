use num_traits::FromPrimitive;

use crate::{
    algorithm::{
        intersects::Intersects,
        kernels::{Orientation, RobustKernel},
        remove_repeated_points::RemoveRepeatedPoints,
        GeoNum,
    },
    types::{
        coordinate::{Coordinate, Coordinate2D},
        coordnum::{CoordFloat, CoordNum},
        line::Line,
        line_string::LineString,
        point::Point,
        rect::Rect,
    },
};

pub fn line_string_bounding_rect<T, Z>(line_string: &LineString<T, Z>) -> Option<Rect<T, Z>>
where
    T: CoordNum,
    Z: CoordNum,
{
    get_bounding_rect(line_string.coords().cloned())
}

pub fn line_bounding_rect<T>(line: Line<T, T>) -> Rect<T, T>
where
    T: CoordNum,
{
    Rect::new(line.start, line.end)
}

pub fn get_bounding_rect<I, T, Z>(collection: I) -> Option<Rect<T, Z>>
where
    T: CoordNum,
    Z: CoordNum,
    I: IntoIterator<Item = Coordinate<T, Z>>,
{
    let mut iter = collection.into_iter();
    if let Some(pnt) = iter.next() {
        let mut xrange = (pnt.x, pnt.x);
        let mut yrange = (pnt.y, pnt.y);
        let mut zrange = (pnt.z, pnt.z);
        for pnt in iter {
            let (px, py, pz) = pnt.x_y_z();
            xrange = get_min_max(px, xrange.0, xrange.1);
            yrange = get_min_max(py, yrange.0, yrange.1);
            zrange = get_min_max(pz, zrange.0, zrange.1);
        }

        return Some(Rect::new(
            Coordinate::new__(xrange.0, yrange.0, zrange.0),
            Coordinate::new__(xrange.1, yrange.1, zrange.1),
        ));
    }
    None
}

fn get_min_max<T: PartialOrd>(p: T, min: T, max: T) -> (T, T) {
    if p > max {
        (min, p)
    } else if p < min {
        (p, max)
    } else {
        (min, max)
    }
}

pub fn line_segment_distance<T, Z, C>(point: C, start: C, end: C) -> T
where
    T: CoordFloat,
    Z: CoordFloat,
    C: Into<Coordinate<T, Z>>,
{
    let point = point.into();
    let start = start.into();
    let end = end.into();

    if start == end {
        return line_euclidean_length(Line::new_(point, start));
    }
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let r = ((point.x - start.x) * dx + (point.y - start.y) * dy) / (dx.powi(2) + dy.powi(2));
    if r <= T::zero() {
        return line_euclidean_length(Line::new_(point, start));
    }
    if r >= T::one() {
        return line_euclidean_length(Line::new_(point, end));
    }
    let s = ((start.y - point.y) * dx - (start.x - point.x) * dy) / (dx * dx + dy * dy);
    s.abs() * dx.hypot(dy)
}

pub fn line_euclidean_length<T, Z>(line: Line<T, Z>) -> T
where
    T: CoordFloat,
    Z: CoordFloat,
{
    line.dx().hypot(line.dy())
}

pub fn point_line_euclidean_distance<C, T, Z>(p: C, l: Line<T, Z>) -> T
where
    T: CoordFloat,
    Z: CoordFloat,
    C: Into<Coordinate<T, Z>>,
{
    line_segment_distance(p.into(), l.start, l.end)
}

pub fn point_contains_point<T>(p1: Point<T, T>, p2: Point<T, T>) -> bool
where
    T: CoordFloat,
{
    let distance = line_euclidean_length(Line::new_(p1, p2)).to_f32().unwrap();
    approx::relative_eq!(distance, 0.0)
}

#[macro_export]
macro_rules! geometry_delegate_impl {
    ($($a:tt)*) => { $crate::__geometry_delegate_impl_helper!{ Geometry, $($a)* } }
}

#[macro_export]
macro_rules! geometry_cow_delegate_impl {
    ($($a:tt)*) => { $crate::__geometry_delegate_impl_helper!{ GeometryCow, $($a)* } }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __geometry_delegate_impl_helper {
    (
        $enum:ident,
        $(
            $(#[$outer:meta])*
            fn $func_name: ident(&$($self_life:lifetime)?self $(, $arg_name: ident: $arg_type: ty)*) -> $return: ty;
         )+
    ) => {
            $(
                $(#[$outer])*
                fn $func_name(&$($self_life)? self, $($arg_name: $arg_type),*) -> $return {
                    match self {
                        $enum::Point(g) => g.$func_name($($arg_name),*).into(),
                        $enum::Line(g) =>  g.$func_name($($arg_name),*).into(),
                        $enum::LineString(g) => g.$func_name($($arg_name),*).into(),
                        $enum::Polygon(g) => g.$func_name($($arg_name),*).into(),
                        $enum::MultiPoint(g) => g.$func_name($($arg_name),*).into(),
                        $enum::MultiLineString(g) => g.$func_name($($arg_name),*).into(),
                        $enum::MultiPolygon(g) => g.$func_name($($arg_name),*).into(),
                        $enum::Rect(g) => g.$func_name($($arg_name),*).into(),
                        $enum::Triangle(g) => g.$func_name($($arg_name),*).into(),
                        _ => unimplemented!(),
                    }
                }
            )+
        };
}

pub fn check_coord_is_not_finite<T: CoordFloat, Z: CoordFloat>(geom: &Coordinate<T, Z>) -> bool {
    if geom.x.is_finite() || geom.y.is_finite() || geom.z.is_finite() {
        return false;
    }
    true
}

pub fn robust_2d_check_points_are_collinear<T: CoordFloat>(
    p0: &Coordinate2D<T>,
    p1: &Coordinate2D<T>,
    p2: &Coordinate2D<T>,
) -> bool {
    RobustKernel::orient(
        Coordinate::new_(p0.x, p0.y),
        Coordinate::new_(p1.x, p1.y),
        Coordinate::new_(p2.x, p2.y),
        None,
    ) == Orientation::Collinear
}

pub fn check_too_few_points<T: CoordFloat + FromPrimitive, Z: CoordFloat + FromPrimitive>(
    geom: &LineString<T, Z>,
    is_ring: bool,
) -> bool {
    let n_pts = if is_ring { 4 } else { 2 };
    if geom.remove_repeated_points().0.len() < n_pts {
        return true;
    }
    false
}

pub fn linestring_has_self_intersection<T: GeoNum, Z: GeoNum>(geom: &LineString<T, Z>) -> bool {
    for (i, line) in geom.lines().enumerate() {
        for (j, other_line) in geom.lines().enumerate() {
            if i != j
                && line.intersects(&other_line)
                && line.start != other_line.end
                && line.end != other_line.start
            {
                return true;
            }
        }
    }
    false
}
