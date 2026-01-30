use geo_buffer::buffer_multi_polygon as geo_buffer_multi_polygon;

use crate::{
    algorithm::convex_hull::quick_hull_2d,
    types::{
        coordinate::{Coordinate, Coordinate2D},
        line_string::LineString2D,
        multi_polygon::MultiPolygon2D,
        polygon::Polygon2D,
    },
};

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
        Polygon2D::new(quick_hull_2d(&mut coords), vec![])
    }
}

impl Bufferable for Polygon2D<f64> {
    fn to_polygon(&self, distance: f64, interpolation_angle: f64) -> Polygon2D<f64> {
        let mut coords = Vec::new();
        for coord in self.exterior().coords() {
            let coord_polygon = coord.to_polygon(distance, interpolation_angle);
            coords.extend(coord_polygon.exterior().coords().copied());
        }
        Polygon2D::new(quick_hull_2d(&mut coords), vec![])
    }
}

pub fn buffer_polygon(input_polygon: &Polygon2D<f64>, distance: f64) -> Option<Polygon2D<f64>> {
    use crate::algorithm::area2d::Area2D;

    // geo_buffer requires CCW (counter-clockwise) polygons (positive area).
    // If the polygon is CW (clockwise, negative area), reverse the coordinates.
    let is_cw = input_polygon.signed_area2d() < 0.0;
    let polygon_to_buffer = if is_cw {
        // Reverse the exterior ring to make it CCW
        let mut exterior_coords: Vec<_> = input_polygon.exterior().coords().cloned().collect();
        exterior_coords.reverse();
        let reversed_exterior = LineString2D::new(exterior_coords);

        // Also reverse interior rings
        let reversed_interiors: Vec<_> = input_polygon
            .interiors()
            .iter()
            .map(|interior| {
                let mut coords: Vec<_> = interior.coords().cloned().collect();
                coords.reverse();
                LineString2D::new(coords)
            })
            .collect();

        Polygon2D::new(reversed_exterior, reversed_interiors)
    } else {
        input_polygon.clone()
    };

    let result = buffer_multi_polygon(&MultiPolygon2D::new(vec![polygon_to_buffer]), distance);
    let buffered = result.0.first().cloned();

    // If original was CW, reverse the result back to CW
    if is_cw {
        buffered.map(|polygon| {
            let mut exterior_coords: Vec<_> = polygon.exterior().coords().cloned().collect();
            exterior_coords.reverse();
            let reversed_exterior = LineString2D::new(exterior_coords);

            let reversed_interiors: Vec<_> = polygon
                .interiors()
                .iter()
                .map(|interior| {
                    let mut coords: Vec<_> = interior.coords().cloned().collect();
                    coords.reverse();
                    LineString2D::new(coords)
                })
                .collect();

            Polygon2D::new(reversed_exterior, reversed_interiors)
        })
    } else {
        buffered
    }
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

    #[test]
    fn test_buffer_polygon_ccw() {
        use crate::algorithm::area2d::Area2D;

        // Counter-clockwise (CCW) polygon - positive signed area
        let ccw_polygon = Polygon2D::new(
            vec![
                coord! { x: 0.1, y: 0.1 },
                coord! { x: 0.5, y: 0.1 },
                coord! { x: 0.5, y: 0.5 },
                coord! { x: 0.1, y: 0.5 },
                coord! { x: 0.1, y: 0.1 },
            ]
            .into(),
            Vec::new(),
        );

        assert!(ccw_polygon.signed_area2d() > 0.0, "Input should be CCW");

        let result = buffer_polygon(&ccw_polygon, 0.005);
        assert!(
            result.is_some(),
            "buffer_polygon should succeed for CCW polygon"
        );

        // Result should also be CCW (positive area)
        let buffered = result.unwrap();
        assert!(
            buffered.signed_area2d() > 0.0,
            "Buffered CCW polygon should remain CCW"
        );
    }

    #[test]
    fn test_buffer_polygon_cw() {
        use crate::algorithm::area2d::Area2D;

        // Clockwise (CW) polygon - negative signed area
        let cw_polygon = Polygon2D::new(
            vec![
                coord! { x: 0.1, y: 0.1 },
                coord! { x: 0.1, y: 0.5 },
                coord! { x: 0.5, y: 0.5 },
                coord! { x: 0.5, y: 0.1 },
                coord! { x: 0.1, y: 0.1 },
            ]
            .into(),
            Vec::new(),
        );

        assert!(cw_polygon.signed_area2d() < 0.0, "Input should be CW");

        let result = buffer_polygon(&cw_polygon, 0.005);
        assert!(
            result.is_some(),
            "buffer_polygon should succeed for CW polygon"
        );

        // Result should also be CW (negative area) - preserving original winding order
        let buffered = result.unwrap();
        assert!(
            buffered.signed_area2d() < 0.0,
            "Buffered CW polygon should remain CW"
        );
    }
}
