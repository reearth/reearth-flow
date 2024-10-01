use geo_buffer::buffer_multi_polygon as geo_buffer_multi_polygon;

use crate::types::{
    coordinate::{Coordinate, Coordinate2D},
    line_string::LineString2D,
    multi_polygon::MultiPolygon2D,
    polygon::Polygon2D,
};

use super::convex_hull::ConvexHull;

const DEFAULT_INTERPOLATION_ANGLE: f64 = 0.1;

pub trait Bufferable {
    fn to_polygon(&self, distance: f64, interpolation_angle: f64) -> Polygon2D<f64>;
}

impl Bufferable for Coordinate2D<f64> {
    fn to_polygon(&self, distance: f64, interpolation_angle: f64) -> Polygon2D<f64> {
        let interpolation_angle = if interpolation_angle <= 0.0 {
            DEFAULT_INTERPOLATION_ANGLE
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

pub fn buffer_polygon(input_polygon: &Polygon2D<f64>, distance: f64) -> Option<Polygon2D<f64>> {
    let result = buffer_multi_polygon(&MultiPolygon2D::new(vec![input_polygon.clone()]), distance);
    result.0.first().cloned()
}

pub fn buffer_multi_polygon(
    input_multi_polygon: &MultiPolygon2D<f64>,
    distance: f64,
) -> MultiPolygon2D<f64> {
    geo_buffer_multi_polygon(&input_multi_polygon.clone().into(), distance).into()
}

#[cfg(test)]
mod tests {
    use crate::{algorithm::coords_iter::CoordsIter, coord};

    use super::*;

    #[test]
    fn test_coordinate_to_polygon() {
        let coord = coord! { x: 1.0, y: 1.0 };
        let polygon = coord.to_polygon(1.0, 22.5);

        assert_eq!(
            polygon
                .exterior()
                .coords_iter()
                .collect::<Vec<Coordinate2D<f64>>>()
                .len(),
            6
        );
    }

    #[test]
    fn test_linestring_to_polygon() {
        let line = LineString2D::new(vec![coord! { x: 0.0, y: 0.0}, coord! { x: 1.0, y: 1.0}]);
        let polygon = line.to_polygon(0.5, 22.5);
        assert_eq!(
            polygon
                .exterior()
                .coords_iter()
                .collect::<Vec<Coordinate2D<f64>>>()
                .len(),
            7
        );
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
        let polygon = polygon.to_polygon(0.005, 22.5);
        assert_eq!(
            polygon
                .exterior()
                .coords_iter()
                .collect::<Vec<Coordinate2D<f64>>>()
                .len(),
            10
        );
    }
}
