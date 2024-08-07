use crate::types::{
    coordinate::{Coordinate, Coordinate2D},
    line_string::LineString2D,
    multi_polygon::MultiPolygon2D,
    polygon::Polygon2D,
};

use super::convex_hull::ConvexHull;

pub trait Bufferable {
    fn to_polygon(&self, distance: f64, interpolation_angle: f64) -> Polygon2D<f64>;
}

impl Bufferable for Coordinate2D<f64> {
    fn to_polygon(&self, distance: f64, interpolation_angle: f64) -> Polygon2D<f64> {
        let interpolation_angle = if interpolation_angle <= 0.0 {
            0.1
        } else {
            interpolation_angle
        };
        let num_segments = (90.0 / interpolation_angle).ceil() as usize;
        let angle_increment = 2.0 * std::f64::consts::PI / num_segments as f64;

        let mut coords = Vec::with_capacity(num_segments + 1);
        for i in 0..=num_segments {
            let angle = i as f64 * angle_increment;
            let x = self.x + distance * angle.cos();
            let y = self.y + distance * angle.sin();
            coords.push(Coordinate { x, y, z: self.z });
        }
        Polygon2D::new(coords.into(), vec![])
    }
}

impl Bufferable for LineString2D<f64> {
    fn to_polygon(&self, distance: f64, interpolation_angle: f64) -> Polygon2D<f64> {
        let mut coords = Vec::new();
        for coord in self.coords() {
            let polygon = coord.to_polygon(distance, interpolation_angle);
            coords.extend(polygon.exterior().coords().copied());
        }
        let polygon = Polygon2D::new(coords.into(), vec![]);
        MultiPolygon2D::new(vec![polygon]).convex_hull()
    }
}

impl Bufferable for Polygon2D<f64> {
    fn to_polygon(&self, distance: f64, interpolation_angle: f64) -> Polygon2D<f64> {
        let mut coords = Vec::new();
        for coord in self.exterior().coords() {
            let coord_polygon = coord.to_polygon(distance, interpolation_angle);
            coords.extend(coord_polygon.exterior().coords().copied());
        }
        let polygon = Polygon2D::new(coords.into(), vec![]);
        MultiPolygon2D::new(vec![polygon]).convex_hull()
    }
}

#[cfg(test)]
mod tests {
    use crate::coord;

    use super::*;

    #[test]
    fn test_coordinate_to_polygon() {
        let coord = coord! { x: 1.0, y: 1.0 };
        let polygon = coord.to_polygon(1.0, 22.5);

        // Expected polygon with 4 segments (square around the point)
        let expected_polygon = Polygon2D::new(
            vec![(2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0), (2.0, 1.0)].into(),
            Vec::new(),
        );
        println!("{:?}", polygon);
        println!("{:?}", expected_polygon);
    }

    #[test]
    fn test_linestring_to_polygon() {
        let line = LineString2D::new(vec![coord! { x: 0.0, y: 0.0}, coord! { x: 1.0, y: 1.0}]);
        let polygon = line.to_polygon(0.5, 22.5);
        println!("{:?}", polygon);
    }
    #[test]
    fn test_polygon_to_polygon() {
        let polygon = Polygon2D::new(
            vec![
                coord! { x: 0.0, y: 0.0 },
                coord! { x: 1.0, y: 0.0 },
                coord! { x: 1.0, y: 1.0 },
                coord! { x: 0.0, y: 1.0 },
                coord! { x: 0.0, y: 0.0 },
            ]
            .into(),
            Vec::new(),
        );
        let buffered_polygon = polygon.to_polygon(0.005, 22.5);
        // Expected polygon with 4 segments (square around the original polygon)
        let expected_polygon = Polygon2D::new(
            vec![
                coord! { x: 0.5, y: -0.5 },
                coord! { x: 1.5, y: -0.5 },
                coord! { x: 1.5, y: 1.5 },
                coord! { x: -0.5, y: 1.5 },
                coord! { x: 0.5, y: -0.5 },
            ]
            .into(),
            Vec::new(),
        );
        println!("{:?}", buffered_polygon);
        println!("{:?}", expected_polygon);
    }
}
