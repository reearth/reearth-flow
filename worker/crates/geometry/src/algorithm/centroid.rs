use std::cmp::Ordering;

use crate::{
    algorithm::{
        area2d::get_linestring_area2d, area3d::get_linestring_area3d, map_coords::MapCoords,
    },
    coord,
    types::{
        coordinate::{Coordinate, Coordinate2D, Coordinate3D},
        coordnum::CoordNumT,
        geometry::{Geometry2D, Geometry3D},
        geometry_collection::{GeometryCollection2D, GeometryCollection3D},
        line::{Line, Line2D, Line3D},
        line_string::{LineString2D, LineString3D},
        multi_line_string::{MultiLineString2D, MultiLineString3D},
        multi_point::{MultiPoint2D, MultiPoint3D},
        multi_polygon::{MultiPolygon2D, MultiPolygon3D},
        no_value::NoValue,
        point::{Point, Point2D, Point3D},
        polygon::{Polygon2D, Polygon3D},
        rect::{Rect2D, Rect3D},
        triangle::{Triangle2D, Triangle3D},
    },
};

use super::{
    area2d::Area2D,
    area3d::Area3D,
    dimensions::{Dimensions, HasDimensions},
    euclidean_length::EuclideanLength,
    GeoFloat,
};

pub trait Centroid {
    type Output;

    fn centroid(&self) -> Self::Output;
}

impl<T> Centroid for Line2D<T>
where
    T: GeoFloat,
{
    type Output = Point2D<T>;

    fn centroid(&self) -> Self::Output {
        let two = T::one() + T::one();
        let sum = self.start_point() + self.end_point();
        let x = sum.x() / two;
        let y = sum.y() / two;
        Point::from(coord! { x: x, y: y})
    }
}

impl<T> Centroid for Line3D<T>
where
    T: GeoFloat + CoordNumT,
{
    type Output = Point3D<T>;

    fn centroid(&self) -> Self::Output {
        let two = T::one() + T::one();
        let sum = self.start_point() + self.end_point();
        let x = sum.x() / two;
        let y = sum.y() / two;
        let z = sum.z() / two;
        Point::from(coord! { x: x, y: y, z: z})
    }
}

impl<T> Centroid for LineString2D<T>
where
    T: GeoFloat,
{
    type Output = Option<Point2D<T>>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation2D::new();
        operation.add_line_string(self);
        operation.centroid()
    }
}

impl<T> Centroid for LineString3D<T>
where
    T: GeoFloat + CoordNumT,
{
    type Output = Option<Point3D<T>>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation3D::new();
        operation.add_line_string(self);
        operation.centroid()
    }
}

impl<T> Centroid for MultiLineString2D<T>
where
    T: GeoFloat,
{
    type Output = Option<Point2D<T>>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation2D::new();
        operation.add_multi_line_string(self);
        operation.centroid()
    }
}

impl<T> Centroid for MultiLineString3D<T>
where
    T: GeoFloat + CoordNumT,
{
    type Output = Option<Point3D<T>>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation3D::new();
        operation.add_multi_line_string(self);
        operation.centroid()
    }
}

impl<T> Centroid for Polygon2D<T>
where
    T: GeoFloat,
{
    type Output = Option<Point2D<T>>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation2D::new();
        operation.add_polygon(self);
        operation.centroid()
    }
}

impl<T> Centroid for Polygon3D<T>
where
    T: GeoFloat + CoordNumT,
{
    type Output = Option<Point3D<T>>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation3D::new();
        operation.add_polygon(self);
        operation.centroid()
    }
}

impl<T> Centroid for MultiPolygon2D<T>
where
    T: GeoFloat,
{
    type Output = Option<Point2D<T>>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation2D::new();
        operation.add_multi_polygon(self);
        operation.centroid()
    }
}

impl<T> Centroid for MultiPolygon3D<T>
where
    T: GeoFloat + CoordNumT,
{
    type Output = Option<Point3D<T>>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation3D::new();
        operation.add_multi_polygon(self);
        operation.centroid()
    }
}

impl<T> Centroid for Rect2D<T>
where
    T: GeoFloat,
{
    type Output = Point2D<T>;

    fn centroid(&self) -> Self::Output {
        self.center().into()
    }
}

impl<T> Centroid for Rect3D<T>
where
    T: GeoFloat + CoordNumT,
{
    type Output = Point3D<T>;

    fn centroid(&self) -> Self::Output {
        self.center().into()
    }
}

impl<T> Centroid for Triangle2D<T>
where
    T: GeoFloat,
{
    type Output = Point2D<T>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation2D::new();
        operation.add_triangle(self);
        operation
            .centroid()
            .expect("triangle cannot have an empty centroid")
    }
}

impl<T> Centroid for Triangle3D<T>
where
    T: GeoFloat + CoordNumT,
{
    type Output = Point3D<T>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation3D::new();
        operation.add_triangle(self);
        operation
            .centroid()
            .expect("triangle cannot have an empty centroid")
    }
}

impl<T> Centroid for Point2D<T>
where
    T: GeoFloat,
{
    type Output = Point2D<T>;

    fn centroid(&self) -> Self::Output {
        *self
    }
}

impl<T> Centroid for Point3D<T>
where
    T: GeoFloat + CoordNumT,
{
    type Output = Point3D<T>;

    fn centroid(&self) -> Self::Output {
        *self
    }
}

impl<T> Centroid for MultiPoint2D<T>
where
    T: GeoFloat,
{
    type Output = Option<Point2D<T>>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation2D::new();
        operation.add_multi_point(self);
        operation.centroid()
    }
}

impl<T> Centroid for MultiPoint3D<T>
where
    T: GeoFloat + CoordNumT,
{
    type Output = Option<Point3D<T>>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation3D::new();
        operation.add_multi_point(self);
        operation.centroid()
    }
}

impl<T> Centroid for Geometry2D<T>
where
    T: GeoFloat,
{
    type Output = Option<Point2D<T>>;

    fn centroid(&self) -> Self::Output {
        match self {
            Geometry2D::Point(g) => Some(g.centroid()),
            Geometry2D::Line(g) => Some(g.centroid()),
            Geometry2D::LineString(g) => g.centroid(),
            Geometry2D::Polygon(g) => g.centroid(),
            Geometry2D::MultiPoint(g) => g.centroid(),
            Geometry2D::MultiLineString(g) => g.centroid(),
            Geometry2D::MultiPolygon(g) => g.centroid(),
            Geometry2D::Rect(g) => Some(g.centroid()),
            Geometry2D::Triangle(g) => Some(g.centroid()),
            Geometry2D::GeometryCollection(g) => {
                let mut operation = CentroidOperation2D::new();
                operation.add_geometry_collection(&GeometryCollection2D::new(g.clone()));
                operation.centroid()
            }
            _ => None,
        }
    }
}

impl<T> Centroid for Geometry3D<T>
where
    T: GeoFloat + CoordNumT,
{
    type Output = Option<Point3D<T>>;

    fn centroid(&self) -> Self::Output {
        match self {
            Geometry3D::Point(g) => Some(g.centroid()),
            Geometry3D::Line(g) => Some(g.centroid()),
            Geometry3D::LineString(g) => g.centroid(),
            Geometry3D::Polygon(g) => g.centroid(),
            Geometry3D::MultiPoint(g) => g.centroid(),
            Geometry3D::MultiLineString(g) => g.centroid(),
            Geometry3D::MultiPolygon(g) => g.centroid(),
            Geometry3D::Rect(g) => Some(g.centroid()),
            Geometry3D::Triangle(g) => Some(g.centroid()),
            Geometry3D::GeometryCollection(g) => {
                let mut operation = CentroidOperation3D::new();
                operation.add_geometry_collection(&GeometryCollection3D::new(g.clone()));
                operation.centroid()
            }
            _ => None,
        }
    }
}

impl<T> Centroid for GeometryCollection2D<T>
where
    T: GeoFloat,
{
    type Output = Option<Point2D<T>>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation2D::new();
        operation.add_geometry_collection(self);
        operation.centroid()
    }
}

impl<T> Centroid for GeometryCollection3D<T>
where
    T: GeoFloat + CoordNumT,
{
    type Output = Option<Point3D<T>>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation3D::new();
        operation.add_geometry_collection(self);
        operation.centroid()
    }
}

struct CentroidOperation2D<T: GeoFloat>(Option<WeightedCentroid<T, NoValue>>);

impl<T: GeoFloat> CentroidOperation2D<T> {
    fn new() -> Self {
        CentroidOperation2D(None)
    }

    fn centroid(&self) -> Option<Point2D<T>> {
        self.0.as_ref().map(|weighted_centroid| {
            Point::from(weighted_centroid.accumulated / weighted_centroid.weight)
        })
    }

    fn centroid_dimensions(&self) -> Dimensions {
        self.0
            .as_ref()
            .map(|weighted_centroid| weighted_centroid.dimensions)
            .unwrap_or(Dimensions::Empty)
    }

    fn add_coord(&mut self, coord: Coordinate2D<T>) {
        self.add_centroid(Dimensions::ZeroDimensional, coord, T::one());
    }

    fn add_line(&mut self, line: &Line2D<T>) {
        match line.dimensions() {
            Dimensions::ZeroDimensional => self.add_coord(line.start),
            Dimensions::OneDimensional => self.add_centroid(
                Dimensions::OneDimensional,
                line.centroid().0,
                line.euclidean_length(),
            ),
            _ => unreachable!("Line must be zero or one dimensional"),
        }
    }

    fn add_line_string(&mut self, line_string: &LineString2D<T>) {
        if self.centroid_dimensions() > Dimensions::OneDimensional {
            return;
        }

        if line_string.0.len() == 1 {
            self.add_coord(line_string.0[0]);
            return;
        }

        for line in line_string.lines() {
            self.add_line(&line);
        }
    }

    fn add_multi_line_string(&mut self, multi_line_string: &MultiLineString2D<T>) {
        if self.centroid_dimensions() > Dimensions::OneDimensional {
            return;
        }

        for element in &multi_line_string.0 {
            self.add_line_string(element);
        }
    }

    fn add_polygon(&mut self, polygon: &Polygon2D<T>) {
        let mut exterior_operation = CentroidOperation2D::new();
        exterior_operation.add_ring(polygon.exterior());

        let mut interior_operation = CentroidOperation2D::new();
        for interior in polygon.interiors() {
            interior_operation.add_ring(interior);
        }

        if let Some(exterior_weighted_centroid) = exterior_operation.0 {
            let mut poly_weighted_centroid = exterior_weighted_centroid;
            if let Some(interior_weighted_centroid) = interior_operation.0 {
                poly_weighted_centroid.sub_assign(interior_weighted_centroid);
                if poly_weighted_centroid.weight.is_zero() {
                    self.add_line_string(polygon.exterior());
                    return;
                }
            }
            self.add_weighted_centroid(poly_weighted_centroid);
        }
    }

    fn add_multi_point(&mut self, multi_point: &MultiPoint2D<T>) {
        if self.centroid_dimensions() > Dimensions::ZeroDimensional {
            return;
        }

        for element in &multi_point.0 {
            self.add_coord(element.0);
        }
    }

    fn add_multi_polygon(&mut self, multi_polygon: &MultiPolygon2D<T>) {
        for element in &multi_polygon.0 {
            self.add_polygon(element);
        }
    }

    fn add_geometry_collection(&mut self, geometry_collection: &GeometryCollection2D<T>) {
        for element in &geometry_collection.0 {
            self.add_geometry(element);
        }
    }

    fn add_rect(&mut self, rect: &Rect2D<T>) {
        match rect.dimensions() {
            Dimensions::ZeroDimensional => self.add_coord(rect.min()),
            Dimensions::OneDimensional => {
                // Degenerate rect is a line, treat it the same way we treat flat polygons
                self.add_line(&Line::new(rect.min(), rect.min()));
                self.add_line(&Line::new(rect.min(), rect.max()));
                self.add_line(&Line::new(rect.max(), rect.max()));
                self.add_line(&Line::new(rect.max(), rect.min()));
            }
            Dimensions::TwoDimensional => self.add_centroid(
                Dimensions::TwoDimensional,
                rect.centroid().0,
                rect.unsigned_area2d(),
            ),
            _ => unreachable!("Rect dimensions cannot be empty"),
        }
    }

    fn add_triangle(&mut self, triangle: &Triangle2D<T>) {
        match triangle.dimensions() {
            Dimensions::ZeroDimensional => self.add_coord(triangle.0),
            Dimensions::OneDimensional => {
                // Degenerate triangle is a line, treat it the same way we treat flat
                // polygons
                let l0_1 = Line::new(triangle.0, triangle.1);
                let l1_2 = Line::new(triangle.1, triangle.2);
                let l2_0 = Line::new(triangle.2, triangle.0);
                self.add_line(&l0_1);
                self.add_line(&l1_2);
                self.add_line(&l2_0);
            }
            Dimensions::TwoDimensional => {
                let centroid = (triangle.0 + triangle.1 + triangle.2) / T::from(3).unwrap();
                self.add_centroid(
                    Dimensions::TwoDimensional,
                    centroid,
                    triangle.unsigned_area2d(),
                );
            }
            _ => unreachable!("Rect dimensions cannot be empty"),
        }
    }

    fn add_geometry(&mut self, geometry: &Geometry2D<T>) {
        match geometry {
            Geometry2D::Point(g) => self.add_coord(g.0),
            Geometry2D::Line(g) => self.add_line(g),
            Geometry2D::LineString(g) => self.add_line_string(g),
            Geometry2D::Polygon(g) => self.add_polygon(g),
            Geometry2D::MultiPoint(g) => self.add_multi_point(g),
            Geometry2D::MultiLineString(g) => self.add_multi_line_string(g),
            Geometry2D::MultiPolygon(g) => self.add_multi_polygon(g),
            Geometry2D::Rect(g) => self.add_rect(g),
            Geometry2D::Triangle(g) => self.add_triangle(g),
            _ => {}
        }
    }

    fn add_ring(&mut self, ring: &LineString2D<T>) {
        debug_assert!(ring.is_closed());

        let area = get_linestring_area2d(ring);
        if area == T::zero() {
            match ring.dimensions() {
                // empty ring doesn't contribute to centroid
                Dimensions::Empty => {}
                // degenerate ring is a point
                Dimensions::ZeroDimensional => self.add_coord(ring[0]),
                // zero-area ring is a line string
                _ => self.add_line_string(ring),
            }
            return;
        }

        // Since area is non-zero, we know the ring has at least one point
        let shift = ring.0[0];

        let accumulated_coord = ring.lines().fold(Coordinate::zero(), |accum, line| {
            let line = line.map_coords(|c| c - shift);
            let tmp = line.determinant2d();
            accum + (line.end + line.start) * tmp
        });
        let six = T::from(6).unwrap();
        let centroid = accumulated_coord / (six * area) + shift;
        let weight = area.abs();
        self.add_centroid(Dimensions::TwoDimensional, centroid, weight);
    }

    fn add_centroid(&mut self, dimensions: Dimensions, centroid: Coordinate2D<T>, weight: T) {
        let weighted_centroid = WeightedCentroid {
            dimensions,
            weight,
            accumulated: centroid * weight,
        };
        self.add_weighted_centroid(weighted_centroid);
    }

    fn add_weighted_centroid(&mut self, other: WeightedCentroid<T, NoValue>) {
        match self.0.as_mut() {
            Some(centroid) => centroid.add_assign(other),
            None => self.0 = Some(other),
        }
    }
}

struct CentroidOperation3D<T: GeoFloat + CoordNumT>(Option<WeightedCentroid<T, T>>);

impl<T: GeoFloat + CoordNumT> CentroidOperation3D<T> {
    fn new() -> Self {
        CentroidOperation3D(None)
    }

    fn centroid(&self) -> Option<Point3D<T>> {
        self.0.as_ref().map(|weighted_centroid| {
            Point::from(weighted_centroid.accumulated / weighted_centroid.weight)
        })
    }

    fn centroid_dimensions(&self) -> Dimensions {
        self.0
            .as_ref()
            .map(|weighted_centroid| weighted_centroid.dimensions)
            .unwrap_or(Dimensions::Empty)
    }

    fn add_coord(&mut self, coord: Coordinate3D<T>) {
        self.add_centroid(Dimensions::ZeroDimensional, coord, T::one());
    }

    fn add_line(&mut self, line: &Line3D<T>) {
        match line.dimensions() {
            Dimensions::ZeroDimensional => self.add_coord(line.start),
            Dimensions::OneDimensional => self.add_centroid(
                Dimensions::OneDimensional,
                line.centroid().0,
                line.euclidean_length(),
            ),
            _ => unreachable!("Line must be zero or one dimensional"),
        }
    }

    fn add_line_string(&mut self, line_string: &LineString3D<T>) {
        if self.centroid_dimensions() > Dimensions::OneDimensional {
            return;
        }

        if line_string.0.len() == 1 {
            self.add_coord(line_string.0[0]);
            return;
        }

        for line in line_string.lines() {
            self.add_line(&line);
        }
    }

    fn add_multi_line_string(&mut self, multi_line_string: &MultiLineString3D<T>) {
        if self.centroid_dimensions() > Dimensions::OneDimensional {
            return;
        }

        for element in &multi_line_string.0 {
            self.add_line_string(element);
        }
    }

    fn add_polygon(&mut self, polygon: &Polygon3D<T>) {
        let mut exterior_operation = CentroidOperation3D::new();
        exterior_operation.add_ring(polygon.exterior());

        let mut interior_operation = CentroidOperation3D::new();
        for interior in polygon.interiors() {
            interior_operation.add_ring(interior);
        }

        if let Some(exterior_weighted_centroid) = exterior_operation.0 {
            let mut poly_weighted_centroid = exterior_weighted_centroid;
            if let Some(interior_weighted_centroid) = interior_operation.0 {
                poly_weighted_centroid.sub_assign(interior_weighted_centroid);
                if poly_weighted_centroid.weight.is_zero() {
                    self.add_line_string(polygon.exterior());
                    return;
                }
            }
            self.add_weighted_centroid(poly_weighted_centroid);
        }
    }

    fn add_multi_point(&mut self, multi_point: &MultiPoint3D<T>) {
        if self.centroid_dimensions() > Dimensions::ZeroDimensional {
            return;
        }

        for element in &multi_point.0 {
            self.add_coord(element.0);
        }
    }

    fn add_multi_polygon(&mut self, multi_polygon: &MultiPolygon3D<T>) {
        for element in &multi_polygon.0 {
            self.add_polygon(element);
        }
    }

    fn add_geometry_collection(&mut self, geometry_collection: &GeometryCollection3D<T>) {
        for element in &geometry_collection.0 {
            self.add_geometry(element);
        }
    }

    fn add_rect(&mut self, rect: &Rect3D<T>) {
        match rect.dimensions() {
            Dimensions::ZeroDimensional => self.add_coord(rect.min()),
            Dimensions::OneDimensional => {
                // Degenerate rect is a line, treat it the same way we treat flat polygons
                self.add_line(&Line::new_(rect.min(), rect.min()));
                self.add_line(&Line::new_(rect.min(), rect.max()));
                self.add_line(&Line::new_(rect.max(), rect.max()));
                self.add_line(&Line::new_(rect.max(), rect.min()));
            }
            Dimensions::TwoDimensional => self.add_centroid(
                Dimensions::TwoDimensional,
                rect.centroid().0,
                rect.unsigned_area3d(),
            ),
            _ => unreachable!("Rect dimensions cannot be empty"),
        }
    }

    fn add_triangle(&mut self, triangle: &Triangle3D<T>) {
        match triangle.dimensions() {
            Dimensions::ZeroDimensional => self.add_coord(triangle.0),
            Dimensions::OneDimensional => {
                // Degenerate triangle is a line, treat it the same way we treat flat
                // polygons
                let l0_1 = Line::new_(triangle.0, triangle.1);
                let l1_2 = Line::new_(triangle.1, triangle.2);
                let l2_0 = Line::new_(triangle.2, triangle.0);
                self.add_line(&l0_1);
                self.add_line(&l1_2);
                self.add_line(&l2_0);
            }
            Dimensions::TwoDimensional => {
                let centroid = (triangle.0 + triangle.1 + triangle.2) / T::from(3).unwrap();
                self.add_centroid(
                    Dimensions::TwoDimensional,
                    centroid,
                    triangle.unsigned_area3d(),
                );
            }
            Dimensions::ThreeDimensional => {
                let centroid = (triangle.0 + triangle.1 + triangle.2) / T::from(3).unwrap();
                self.add_centroid(
                    Dimensions::ThreeDimensional,
                    centroid,
                    triangle.unsigned_area3d(),
                );
            }
            _ => unreachable!("Rect dimensions cannot be empty"),
        }
    }

    fn add_geometry(&mut self, geometry: &Geometry3D<T>) {
        match geometry {
            Geometry3D::Point(g) => self.add_coord(g.0),
            Geometry3D::Line(g) => self.add_line(g),
            Geometry3D::LineString(g) => self.add_line_string(g),
            Geometry3D::Polygon(g) => self.add_polygon(g),
            Geometry3D::MultiPoint(g) => self.add_multi_point(g),
            Geometry3D::MultiLineString(g) => self.add_multi_line_string(g),
            Geometry3D::MultiPolygon(g) => self.add_multi_polygon(g),
            Geometry3D::Rect(g) => self.add_rect(g),
            Geometry3D::Triangle(g) => self.add_triangle(g),
            _ => {}
        }
    }

    fn add_ring(&mut self, ring: &LineString3D<T>) {
        debug_assert!(ring.is_closed());

        let area = get_linestring_area3d(ring);
        if area == T::zero() {
            match ring.dimensions() {
                // empty ring doesn't contribute to centroid
                Dimensions::Empty => {}
                // degenerate ring is a point
                Dimensions::ZeroDimensional => self.add_coord(ring[0]),
                // zero-area ring is a line string
                _ => self.add_line_string(ring),
            }
            return;
        }

        // Since area is non-zero, we know the ring has at least one point
        let shift = ring.0[0];

        let accumulated_coord = ring.lines().fold(Coordinate::zero(), |accum, line| {
            let line = line.map_coords(|c| c - shift);
            let tmp = line.determinant3d();
            accum + (line.end + line.start) * tmp
        });
        let six = T::from(6).unwrap();
        let centroid = accumulated_coord / (six * area) + shift;
        let weight = area.abs();
        self.add_centroid(Dimensions::TwoDimensional, centroid, weight);
    }

    fn add_centroid(&mut self, dimensions: Dimensions, centroid: Coordinate3D<T>, weight: T) {
        let weighted_centroid = WeightedCentroid {
            dimensions,
            weight,
            accumulated: centroid * weight,
        };
        self.add_weighted_centroid(weighted_centroid);
    }

    fn add_weighted_centroid(&mut self, other: WeightedCentroid<T, T>) {
        match self.0.as_mut() {
            Some(centroid) => centroid.add_assign(other),
            None => self.0 = Some(other),
        }
    }
}

// Aggregated state for accumulating the centroid of a geometry or collection of geometries.
struct WeightedCentroid<T: GeoFloat, Z: GeoFloat> {
    weight: T,
    accumulated: Coordinate<T, Z>,
    dimensions: Dimensions,
}

impl<T: GeoFloat, Z: GeoFloat> WeightedCentroid<T, Z> {
    fn add_assign(&mut self, b: WeightedCentroid<T, Z>) {
        match self.dimensions.cmp(&b.dimensions) {
            Ordering::Less => *self = b,
            Ordering::Greater => {}
            Ordering::Equal => {
                self.accumulated = self.accumulated + b.accumulated;
                self.weight = self.weight + b.weight;
            }
        }
    }

    fn sub_assign(&mut self, b: WeightedCentroid<T, Z>) {
        match self.dimensions.cmp(&b.dimensions) {
            Ordering::Less => *self = b,
            Ordering::Greater => {}
            Ordering::Equal => {
                self.accumulated = self.accumulated - b.accumulated;
                self.weight = self.weight - b.weight;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::line_string;

    use super::*;

    // Tests: Centroid of LineString
    #[test]
    fn empty_linestring_test() {
        let linestring: LineString2D<f32> = LineString2D::new(vec![]);
        let centroid = linestring.centroid();
        assert!(centroid.is_none());
    }
    #[test]
    fn linestring_one_point_test() {
        let coord = coord! {
            x: 40.02f64,
            y: 116.34,
        };
        let linestring = LineString2D::from(vec![coord]);
        let centroid = linestring.centroid();
        assert_eq!(centroid, Some(Point::from(coord)));
    }

    #[test]
    fn linestring_test() {
        let linestring = line_string![
            (x: 1., y: 1.),
            (x: 7., y: 1.),
            (x: 8., y: 1.),
            (x: 9., y: 1.),
            (x: 10., y: 1.),
            (x: 11., y: 1.)
        ];
        assert_eq!(linestring.centroid(), Some(Point::new(6., 1.)));
    }
}
