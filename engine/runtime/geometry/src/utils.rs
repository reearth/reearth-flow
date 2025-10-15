use nalgebra::DMatrix;
use num_traits::FromPrimitive;

use crate::{
    algorithm::{
        geo_distance_converter::coordinate_diff_to_meter,
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
        point::{Point, Point3D},
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

pub fn line_bounding_rect<T, Z>(line: Line<T, Z>) -> Rect<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
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

pub fn line_segment_distance<T, Z>(
    point: Coordinate<T, Z>,
    start: Coordinate<T, Z>,
    end: Coordinate<T, Z>,
) -> T
where
    T: CoordFloat,
    Z: CoordFloat,
{
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

pub fn point_contains_point<T, Z>(p1: Point<T, Z>, p2: Point<T, Z>) -> bool
where
    T: CoordFloat,
    Z: CoordFloat,
{
    let distance = line_euclidean_length(Line::new_(p1, p2)).to_f32().unwrap();
    approx::relative_eq!(distance, 0.0)
}

pub fn point_line_string_euclidean_distance<T, Z>(p: Point<T, Z>, l: &LineString<T, Z>) -> T
where
    T: CoordFloat,
    Z: CoordFloat,
{
    if line_string_contains_point(l, p) || l.0.is_empty() {
        return T::zero();
    }
    l.lines()
        .map(|line| line_segment_distance(p.0, line.start, line.end))
        .fold(T::max_value(), |accum, val| accum.min(val))
}

pub fn line_string_contains_point<T, Z>(line_string: &LineString<T, Z>, point: Point<T, Z>) -> bool
where
    T: CoordFloat,
    Z: CoordFloat,
{
    // LineString without points
    if line_string.0.is_empty() {
        return false;
    }
    // LineString with one point equal p
    if line_string.0.len() == 1 {
        return point_contains_point(Point::from(line_string[0]), point);
    }
    // check if point is a vertex
    if line_string.0.contains(&point.0) {
        return true;
    }
    for line in line_string.lines() {
        // This is a duplicate of the line-contains-point logic in the "intersects" module
        let tx = if line.dx() == T::zero() {
            None
        } else {
            Some((point.x() - line.start.x) / line.dx())
        };
        let ty = if line.dy() == T::zero() {
            None
        } else {
            Some((point.y() - line.start.y) / line.dy())
        };
        let contains = match (tx, ty) {
            (None, None) => {
                // Degenerate line
                point.0 == line.start
            }
            (Some(t), None) => {
                // Horizontal line
                point.y() == line.start.y && T::zero() <= t && t <= T::one()
            }
            (None, Some(t)) => {
                // Vertical line
                point.x() == line.start.x && T::zero() <= t && t <= T::one()
            }
            (Some(t_x), Some(t_y)) => {
                // All other lines
                (t_x - t_y).abs() <= T::epsilon() && T::zero() <= t_x && t_x <= T::one()
            }
        };
        if contains {
            return true;
        }
    }
    false
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

/// Check for self-intersection with 3D-aware analysis
/// This function first checks if the LineString is planar, and if so, rotates it to XY plane for accurate intersection testing
pub fn linestring_has_self_intersection_3d<
    T: GeoNum + num_traits::FromPrimitive + num_traits::ToPrimitive,
    Z: GeoNum + num_traits::FromPrimitive + num_traits::ToPrimitive,
>(
    geom: &LineString<T, Z>,
) -> bool {
    if let Some(planarity_info) = is_linestring_planar(geom, 1e-6) {
        if let Some(rotated_geom) = rotate_linestring_to_xy_plane(geom, &planarity_info) {
            return linestring_has_self_intersection(&rotated_geom);
        }
    }

    // If not planar or rotation failed, use 2D projection method (fallback)
    linestring_has_self_intersection(geom)
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

#[derive(Debug)]
pub struct PointsCoplanar {
    pub normal: Point3D<f64>,
    pub center: Point3D<f64>,
}

pub fn are_points_coplanar(
    points: Vec<nalgebra::Point3<f64>>,
    tolerance: f64,
) -> Option<PointsCoplanar> {
    let n = points.len();
    if points.len() < 3 {
        return None; // Three points or less are always on the same plane.
    }

    // Calculate the mean value of the point cloud.
    let mean: nalgebra::Vector3<f64> = points
        .iter()
        .map(|p| p.coords)
        .sum::<nalgebra::Vector3<f64>>()
        / (n as f64);

    // Calculate the covariance matrix
    let mut covariance_matrix = DMatrix::<f64>::zeros(3, 3);
    for point in points {
        let centered = point.coords - mean;
        covariance_matrix += centered * centered.transpose();
    }
    covariance_matrix /= n as f64;
    // Calculate eigenvalues and eigenvectors
    let eig = covariance_matrix.symmetric_eigen();

    // Get the smallest eigenvalue
    let min_eigenvalue = eig.eigenvalues.min();

    // Get the eigenvector corresponding to the smallest eigenvalue
    let min_eigenvalue_index = eig.eigenvalues.imin();
    let normal_vector = eig.eigenvectors.column(min_eigenvalue_index).into_owned();

    // If the smallest eigenvalue is smaller than the tolerance, it is considered flat.
    let is_planar = min_eigenvalue < tolerance;

    let normal_point = nalgebra::Point3::new(normal_vector[0], normal_vector[1], normal_vector[2]);
    let center_point = nalgebra::Point3::new(mean[0], mean[1], mean[2]);
    if is_planar {
        Some(PointsCoplanar {
            normal: Point3D::new_(normal_point.x, normal_point.y, normal_point.z),
            center: Point3D::new_(center_point.x, center_point.y, center_point.z),
        })
    } else {
        None
    }
}

pub fn remove_redundant_vertices<Z: CoordFloat>(
    line_string: &LineString<f64, Z>,
    tolerance: f64,
) -> LineString<f64, Z> {
    let mut new_coords = Vec::new();
    let coords = &line_string.coords().collect::<Vec<_>>();

    if coords.len() < 3 {
        return line_string.clone();
    }

    new_coords.push(*coords[0]);

    for i in 1..coords.len() - 1 {
        let prev = coords[i - 1];
        let curr = coords[i];
        let next = coords[i + 1];

        if !is_colinear(prev, curr, next, tolerance) {
            new_coords.push(*curr);
        }
    }

    new_coords.push(*coords[coords.len() - 1]);
    LineString::from(new_coords)
}

fn is_colinear<Z: CoordFloat>(
    p1: &Coordinate<f64, Z>,
    p2: &Coordinate<f64, Z>,
    p3: &Coordinate<f64, Z>,
    tolerance: f64,
) -> bool {
    let area =
        ((p1.x * (p2.y - p3.y)) + (p2.x * (p3.y - p1.y)) + (p3.x * (p1.y - p2.y))).abs() / 2.0;
    area < tolerance
}

/// Calculate 3D distance between two coordinates using geo-distance conversion
/// Returns None if coordinate conversion fails
pub fn calculate_geo_distance_3d<T: CoordFloat, Z: CoordFloat>(
    p1: &Coordinate<T, Z>,
    p2: &Coordinate<T, Z>,
) -> Option<f64> {
    let lng1 = p1.x.to_f64()?;
    let lat1 = p1.y.to_f64()?;
    let lng2 = p2.x.to_f64()?;
    let lat2 = p2.y.to_f64()?;

    let dlng = lng2 - lng1;
    let dlat = lat2 - lat1;
    let mid_lat = (lat1 + lat2) / 2.0;

    // Convert coordinate differences to meters
    let (dx_meters, dy_meters) = coordinate_diff_to_meter(dlng, dlat, mid_lat);

    // Handle Z coordinate (already in meters)
    let dz_meters = match (p1.z.to_f64(), p2.z.to_f64()) {
        (Some(z1), Some(z2)) => z2 - z1,
        (None, None) => 0.0, // Both missing Z is valid
        _ => return None,    // Inconsistent Z coordinate availability
    };

    Some((dx_meters * dx_meters + dy_meters * dy_meters + dz_meters * dz_meters).sqrt())
}

/// Check if a LineString is planar (all points lie on the same plane)
/// Returns Some(PointsCoplanar) if planar, None if not planar
pub fn is_linestring_planar<T, Z>(
    line_string: &LineString<T, Z>,
    tolerance: f64,
) -> Option<PointsCoplanar>
where
    T: CoordFloat + num_traits::ToPrimitive,
    Z: CoordFloat + num_traits::ToPrimitive,
{
    if line_string.0.len() < 3 {
        return None; // Less than 3 points cannot define a plane
    }

    let points: Result<Vec<nalgebra::Point3<f64>>, &str> = line_string
        .coords()
        .map(|coord| {
            Ok(nalgebra::Point3::new(
                coord.x.to_f64().ok_or("Failed to convert x coordinate")?,
                coord.y.to_f64().ok_or("Failed to convert y coordinate")?,
                coord.z.to_f64().ok_or("Failed to convert z coordinate")?,
            ))
        })
        .collect();

    let points = points.ok()?;
    are_points_coplanar(points, tolerance)
}

/// Rotate a planar LineString to lie on the XY plane (Z=0)
/// Returns the rotated LineString if successful, None if the operation fails
pub fn rotate_linestring_to_xy_plane<T, Z>(
    line_string: &LineString<T, Z>,
    planarity_info: &PointsCoplanar,
) -> Option<LineString<T, Z>>
where
    T: CoordFloat + num_traits::FromPrimitive + num_traits::ToPrimitive,
    Z: CoordFloat + num_traits::FromPrimitive + num_traits::ToPrimitive,
{
    use nalgebra::{Matrix3, Vector3};

    // Calculate rotation matrix to align surface normal with Z-axis
    let from_vector = Vector3::new(
        planarity_info.normal.x(),
        planarity_info.normal.y(),
        planarity_info.normal.z(),
    );
    let to_vector = Vector3::new(0.0, 0.0, 1.0);

    // Skip rotation if already aligned with Z-axis
    if (from_vector - to_vector).norm() < 1e-6 {
        return Some(line_string.clone());
    }

    // Calculate rotation axis and angle
    let cross_product = from_vector.cross(&to_vector);

    let (rotation_axis, rotation_angle) = if cross_product.norm() < 1e-10 {
        // Vectors are parallel - either same direction (already handled above) or opposite
        // For opposite direction, we need a 180-degree rotation around any perpendicular axis
        let perpendicular = if from_vector.x.abs() < 0.9 {
            Vector3::new(1.0, 0.0, 0.0).cross(&from_vector)
        } else {
            Vector3::new(0.0, 1.0, 0.0).cross(&from_vector)
        };
        (perpendicular.normalize(), std::f64::consts::PI)
    } else {
        (
            cross_product.normalize(),
            from_vector.dot(&to_vector).acos(),
        )
    };

    // Create rotation matrix using Rodrigues' rotation formula
    let k = rotation_axis;
    let cos_theta = rotation_angle.cos();
    let sin_theta = rotation_angle.sin();

    let rotation_matrix = Matrix3::identity() * cos_theta
        + Matrix3::from_columns(&[
            Vector3::new(0.0, -k.z, k.y),
            Vector3::new(k.z, 0.0, -k.x),
            Vector3::new(-k.y, k.x, 0.0),
        ]) * sin_theta
        + k * k.transpose() * (1.0 - cos_theta);

    // Apply rotation to each coordinate
    let rotated_coords: Result<Vec<Coordinate<T, Z>>, &str> = line_string
        .coords()
        .map(|coord| {
            let original = Vector3::new(
                coord.x.to_f64().ok_or("Failed to convert x coordinate")?,
                coord.y.to_f64().ok_or("Failed to convert y coordinate")?,
                coord.z.to_f64().ok_or("Failed to convert z coordinate")?,
            );

            let rotated = rotation_matrix * original;

            Ok(Coordinate::new__(
                T::from_f64(rotated.x).ok_or("Failed to convert rotated x coordinate")?,
                T::from_f64(rotated.y).ok_or("Failed to convert rotated y coordinate")?,
                Z::from_f64(0.0).ok_or("Failed to convert z coordinate to zero")?, // Force Z to 0 for XY plane
            ))
        })
        .collect();

    let rotated_coords = rotated_coords.ok()?;
    Some(LineString::from(rotated_coords))
}

/// Returns the circumcenter and the circumradius if the triangle is non-degenerate.
pub fn circumcenter(
    a: Coordinate<f64>,
    b: Coordinate<f64>,
    c: Coordinate<f64>,
) -> Result<(Coordinate<f64>, f64), String> {
    let t = [a, b, c];
    // Work in the triangle's plane basis: x = A + α(B−A) + β(C−A),
    // with constraints x·(B−A) = |B−A|²/2 and x·(C−A) = |C−A|²/2.
    let ab = t[1] - t[0];
    let ac = t[2] - t[0];

    let g11 = ab.dot(&ab);
    let g12 = ab.dot(&ac);
    let g22 = ac.dot(&ac);

    // det = |ab×ac|² (Gram determinant). Near-zero => collinear/degenerate.
    let det = g11 * g22 - g12 * g12;
    let eps = 1e-10 * (g11 + g22).max(1.0);
    if det.abs() < eps {
        return Err(format!("Degenerate triangle: [{a:?}, {b:?}, {c:?}]"));
    }

    let rhs1 = 0.5 * g11;
    let rhs2 = 0.5 * g22;

    let alpha = (rhs1 * g22 - rhs2 * g12) / det;
    let beta = (g11 * rhs2 - g12 * rhs1) / det;

    let center = a + ab * alpha + ac * beta;
    let radius = (center - a).norm();
    Ok((center, radius))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_intersection_3d_ok() {
        assert!(!linestring_has_self_intersection_3d(&floor_surface()));
        assert!(!linestring_has_self_intersection_3d(&roof_surface()));
        assert!(!linestring_has_self_intersection_3d(&wall_surface()));
    }

    #[test]
    fn test_self_intersection_3d_error() {
        assert!(linestring_has_self_intersection_3d(
            &floor_surface_with_self_intersection()
        ));

        assert!(linestring_has_self_intersection_3d(
            &roof_surface_with_self_intersection()
        ));

        assert!(linestring_has_self_intersection_3d(
            &wall_surface_with_self_intersection()
        ));
    }

    fn floor_surface() -> LineString<f64, f64> {
        LineString::from(vec![
            Coordinate::new__(0.0, 0.0, 0.0),
            Coordinate::new__(1.0, 0.0, 0.0),
            Coordinate::new__(1.0, 1.0, 0.0),
            Coordinate::new__(0.0, 1.0, 0.0),
            Coordinate::new__(0.0, 0.0, 0.0),
        ])
    }

    fn floor_surface_with_self_intersection() -> LineString<f64, f64> {
        LineString::from(vec![
            Coordinate::new__(0.0, 0.0, 0.0),
            Coordinate::new__(1.0, 0.0, 0.0),
            Coordinate::new__(0.0, 1.0, 0.0),
            Coordinate::new__(1.0, 1.0, 0.0),
            Coordinate::new__(0.0, 0.0, 0.0),
        ])
    }

    fn roof_surface() -> LineString<f64, f64> {
        LineString::from(vec![
            Coordinate::new__(0.0, 0.0, 8.0),
            Coordinate::new__(1.0, 0.0, 8.0),
            Coordinate::new__(1.0, 1.0, 8.0),
            Coordinate::new__(0.0, 1.0, 8.0),
            Coordinate::new__(0.0, 0.0, 8.0),
        ])
    }

    fn roof_surface_with_self_intersection() -> LineString<f64, f64> {
        LineString::from(vec![
            Coordinate::new__(0.0, 0.0, 8.0),
            Coordinate::new__(1.0, 0.0, 8.0),
            Coordinate::new__(0.0, 1.0, 8.0),
            Coordinate::new__(1.0, 1.0, 8.0),
            Coordinate::new__(0.0, 0.0, 8.0),
        ])
    }

    fn wall_surface() -> LineString<f64, f64> {
        LineString::from(vec![
            Coordinate::new__(0.0, 0.0, 0.0),
            Coordinate::new__(0.0, 1.0, 0.0),
            Coordinate::new__(0.0, 1.0, 8.0),
            Coordinate::new__(0.0, 0.0, 8.0),
            Coordinate::new__(0.0, 0.0, 0.0),
        ])
    }

    fn wall_surface_with_self_intersection() -> LineString<f64, f64> {
        LineString::from(vec![
            Coordinate::new__(0.0, 0.0, 0.0),
            Coordinate::new__(1.0, 0.0, 8.0),
            Coordinate::new__(0.0, 1.0, 8.0),
            Coordinate::new__(1.0, 0.0, 0.0),
            Coordinate::new__(0.0, 0.0, 0.0),
        ])
    }

    #[test]
    fn circum_center_test() {
        let a = Coordinate::new__(0.0, 0.0, 0.0);
        let b = Coordinate::new__(2.0, 0.0, 0.0);
        let c = Coordinate::new__(1.0, 3_f64.sqrt(), 0.0);
        let (center, radius) = circumcenter(a, b, c).unwrap();
        assert!((center.x - 1.0).abs() < 1e-10);
        assert!((center.y - 1.0 / 3_f64.sqrt()).abs() < 1e-10);
        assert!((center.z - 0.0).abs() < 1e-10);
        assert!((radius - 2.0 / (3_f64).sqrt()).abs() < 1e-10);
    }
}
