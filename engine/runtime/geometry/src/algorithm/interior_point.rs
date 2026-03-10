use std::collections::BinaryHeap;

use crate::types::{
    coordinate::Coordinate,
    coordnum::CoordNumT,
    line_string::LineString,
    no_value::NoValue,
    point::{Point, Point2D, Point3D},
    polygon::{Polygon, Polygon2D, Polygon3D},
};

use super::{bounding_rect::BoundingRect, centroid::Centroid, GeoFloat};

/// Computes a point guaranteed to be inside a polygon, using the polylabel
/// (pole of inaccessibility) algorithm. The result is the point inside the
/// polygon that is farthest from the polygon boundary.
pub trait InteriorPoint {
    type Output;

    fn interior_point(&self) -> Self::Output;
}

impl<T: GeoFloat> InteriorPoint for Polygon2D<T> {
    type Output = Option<Point2D<T>>;

    fn interior_point(&self) -> Self::Output {
        polylabel_2d(self, T::from(1e-6).unwrap_or_else(T::epsilon))
    }
}

impl<T: GeoFloat + CoordNumT> InteriorPoint for Polygon3D<T> {
    type Output = Option<Point3D<T>>;

    fn interior_point(&self) -> Self::Output {
        polylabel_3d(self, T::from(1e-6).unwrap_or_else(T::epsilon))
    }
}

fn point_to_segment_distance_sq<T: GeoFloat>(px: T, py: T, ax: T, ay: T, bx: T, by: T) -> T {
    let dx = bx - ax;
    let dy = by - ay;
    let len_sq = dx * dx + dy * dy;

    if len_sq.is_zero() {
        let ex = px - ax;
        let ey = py - ay;
        return ex * ex + ey * ey;
    }

    let t = ((px - ax) * dx + (py - ay) * dy) / len_sq;
    let t = if t < T::zero() {
        T::zero()
    } else if t > T::one() {
        T::one()
    } else {
        t
    };

    let proj_x = ax + t * dx;
    let proj_y = ay + t * dy;
    let ex = px - proj_x;
    let ey = py - proj_y;
    ex * ex + ey * ey
}

fn min_dist_to_polygon_boundary<T: GeoFloat>(px: T, py: T, polygon: &Polygon<T, NoValue>) -> T {
    let mut min_sq = T::infinity();

    for ring in std::iter::once(polygon.exterior()).chain(polygon.interiors().iter()) {
        let coords = &ring.0;
        for i in 0..coords.len().saturating_sub(1) {
            let d = point_to_segment_distance_sq(
                px,
                py,
                coords[i].x,
                coords[i].y,
                coords[i + 1].x,
                coords[i + 1].y,
            );
            if d < min_sq {
                min_sq = d;
            }
        }
    }

    min_sq.sqrt()
}

#[derive(Clone)]
struct Cell<T: GeoFloat> {
    x: T,
    y: T,
    half_size: T,
    distance: T,
    max_potential: T,
}

impl<T: GeoFloat> Cell<T> {
    fn new(x: T, y: T, half_size: T, polygon: &Polygon<T, NoValue>) -> Self {
        let distance = if point_inside_polygon(x, y, polygon) {
            min_dist_to_polygon_boundary(x, y, polygon)
        } else {
            -min_dist_to_polygon_boundary(x, y, polygon)
        };
        let max_potential = distance + half_size * T::from(std::f64::consts::SQRT_2).unwrap();
        Self {
            x,
            y,
            half_size,
            distance,
            max_potential,
        }
    }
}

impl<T: GeoFloat> PartialEq for Cell<T> {
    fn eq(&self, other: &Self) -> bool {
        self.max_potential == other.max_potential
    }
}

impl<T: GeoFloat> Eq for Cell<T> {}

impl<T: GeoFloat> PartialOrd for Cell<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: GeoFloat> Ord for Cell<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.max_potential
            .partial_cmp(&other.max_potential)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

fn point_inside_polygon<T: GeoFloat>(x: T, y: T, polygon: &Polygon<T, NoValue>) -> bool {
    let mut inside = ray_cast_inside(x, y, polygon.exterior());
    for interior in polygon.interiors() {
        if ray_cast_inside(x, y, interior) {
            inside = !inside;
        }
    }
    inside
}

fn ray_cast_inside<T: GeoFloat>(x: T, y: T, ring: &LineString<T, NoValue>) -> bool {
    let mut inside = false;
    let coords = &ring.0;
    let n = coords.len();
    let mut j = n.wrapping_sub(1);
    for i in 0..n {
        let yi = coords[i].y;
        let yj = coords[j].y;
        if ((yi > y) != (yj > y))
            && (x < (coords[j].x - coords[i].x) * (y - yi) / (yj - yi) + coords[i].x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn polylabel_2d<T: GeoFloat>(polygon: &Polygon2D<T>, precision: T) -> Option<Point2D<T>> {
    let rect = polygon.bounding_rect()?;
    let min_x = rect.min.x;
    let min_y = rect.min.y;
    let max_x = rect.max.x;
    let max_y = rect.max.y;
    let width = max_x - min_x;
    let height = max_y - min_y;
    let cell_size = if width > height { width } else { height };
    let two = T::one() + T::one();

    if cell_size.is_zero() {
        return Some(Point::from(Coordinate::new_(min_x, min_y)));
    }

    let mut half = cell_size / two;
    let mut heap = BinaryHeap::new();

    // Cover polygon with initial cells
    let mut x = min_x;
    while x < max_x {
        let mut y = min_y;
        while y < max_y {
            heap.push(Cell::new(x + half, y + half, half, polygon));
            y = y + cell_size;
        }
        x = x + cell_size;
    }

    // Start with centroid as initial best guess
    let centroid_cell = if let Some(c) = polygon.centroid() {
        Cell::new(c.x(), c.y(), T::zero(), polygon)
    } else {
        Cell::new(
            (min_x + max_x) / two,
            (min_y + max_y) / two,
            T::zero(),
            polygon,
        )
    };

    // Also try bounding box center
    let bbox_cell = Cell::new(
        (min_x + max_x) / two,
        (min_y + max_y) / two,
        T::zero(),
        polygon,
    );

    let mut best = if centroid_cell.distance > bbox_cell.distance {
        centroid_cell
    } else {
        bbox_cell
    };

    while let Some(cell) = heap.pop() {
        if cell.distance > best.distance {
            best = cell.clone();
        }

        if cell.max_potential - best.distance <= precision {
            break;
        }

        half = cell.half_size / two;
        let cx = cell.x;
        let cy = cell.y;
        heap.push(Cell::new(cx - half, cy - half, half, polygon));
        heap.push(Cell::new(cx + half, cy - half, half, polygon));
        heap.push(Cell::new(cx - half, cy + half, half, polygon));
        heap.push(Cell::new(cx + half, cy + half, half, polygon));
    }

    Some(Point::from(Coordinate::new_(best.x, best.y)))
}

fn polylabel_3d<T: GeoFloat + CoordNumT>(
    polygon: &Polygon3D<T>,
    precision: T,
) -> Option<Point3D<T>> {
    // Project to 2D (use x,y), compute polylabel, then interpolate Z
    let exterior_2d: LineString<T, NoValue> = LineString::new(
        polygon
            .exterior()
            .0
            .iter()
            .map(|c| Coordinate::new_(c.x, c.y))
            .collect(),
    );
    let interiors_2d: Vec<LineString<T, NoValue>> = polygon
        .interiors()
        .iter()
        .map(|ring| LineString::new(ring.0.iter().map(|c| Coordinate::new_(c.x, c.y)).collect()))
        .collect();
    let poly_2d = Polygon::new(exterior_2d, interiors_2d);

    let pt_2d = polylabel_2d(&poly_2d, precision)?;

    // Interpolate Z from the exterior ring vertices (inverse distance weighting)
    let z = interpolate_z(pt_2d.x(), pt_2d.y(), polygon.exterior());
    Some(Point::new_(pt_2d.x(), pt_2d.y(), z))
}

fn interpolate_z<T: GeoFloat + CoordNumT>(px: T, py: T, ring: &LineString<T, T>) -> T {
    let mut total_weight = T::zero();
    let mut weighted_z = T::zero();
    let epsilon = T::from(1e-12).unwrap_or_else(T::epsilon);

    for coord in &ring.0 {
        let dx = px - coord.x;
        let dy = py - coord.y;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq < epsilon {
            return coord.z;
        }
        let weight = T::one() / dist_sq;
        weighted_z = weighted_z + weight * coord.z;
        total_weight = total_weight + weight;
    }

    if total_weight > T::zero() {
        weighted_z / total_weight
    } else {
        T::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{coordinate::Coordinate, line_string::LineString, polygon::Polygon};

    fn make_polygon_2d(coords: &[(f64, f64)]) -> Polygon2D<f64> {
        let ls: LineString<f64, NoValue> = LineString::new(
            coords
                .iter()
                .map(|&(x, y)| Coordinate::new_(x, y))
                .collect(),
        );
        Polygon::new(ls, vec![])
    }

    fn make_polygon_3d(coords: &[(f64, f64, f64)]) -> Polygon3D<f64> {
        let ls: LineString<f64, f64> = LineString::new(
            coords
                .iter()
                .map(|&(x, y, z)| Coordinate::new__(x, y, z))
                .collect(),
        );
        Polygon::new(ls, vec![])
    }

    #[test]
    fn test_square_interior_point() {
        let poly = make_polygon_2d(&[(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0), (0.0, 0.0)]);
        let pt = poly.interior_point().unwrap();
        assert!((pt.x() - 2.0).abs() < 0.1);
        assert!((pt.y() - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_l_shaped_polygon_interior_point_is_inside() {
        // L-shaped polygon where centroid falls outside
        let poly = make_polygon_2d(&[
            (0.0, 0.0),
            (2.0, 0.0),
            (2.0, 1.0),
            (1.0, 1.0),
            (1.0, 2.0),
            (0.0, 2.0),
            (0.0, 0.0),
        ]);
        let pt = poly.interior_point().unwrap();
        // Verify the point is inside the polygon
        assert!(point_inside_polygon(pt.x(), pt.y(), &poly));
    }

    #[test]
    fn test_concave_c_shape() {
        // C-shaped polygon
        let poly = make_polygon_2d(&[
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 1.0),
            (1.0, 1.0),
            (1.0, 9.0),
            (10.0, 9.0),
            (10.0, 10.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ]);
        let pt = poly.interior_point().unwrap();
        assert!(point_inside_polygon(pt.x(), pt.y(), &poly));
    }

    #[test]
    fn test_polygon_with_hole() {
        let exterior = [
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ];
        let hole = [(3.0, 3.0), (7.0, 3.0), (7.0, 7.0), (3.0, 7.0), (3.0, 3.0)];
        let ext_ls: LineString<f64, NoValue> = LineString::new(
            exterior
                .iter()
                .map(|&(x, y)| Coordinate::new_(x, y))
                .collect(),
        );
        let hole_ls: LineString<f64, NoValue> =
            LineString::new(hole.iter().map(|&(x, y)| Coordinate::new_(x, y)).collect());
        let poly = Polygon::new(ext_ls, vec![hole_ls]);
        let pt = poly.interior_point().unwrap();
        assert!(point_inside_polygon(pt.x(), pt.y(), &poly));
    }

    #[test]
    fn test_3d_polygon_interior_point() {
        let poly = make_polygon_3d(&[
            (0.0, 0.0, 10.0),
            (4.0, 0.0, 10.0),
            (4.0, 4.0, 10.0),
            (0.0, 4.0, 10.0),
            (0.0, 0.0, 10.0),
        ]);
        let pt = poly.interior_point().unwrap();
        assert!((pt.x() - 2.0).abs() < 0.1);
        assert!((pt.y() - 2.0).abs() < 0.1);
        assert!((pt.z() - 10.0).abs() < 0.1);
    }
}
