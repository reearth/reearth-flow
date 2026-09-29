//! Coordinate rounding: every stored coordinate rounded to a fixed number of
//! decimal places per axis.

use super::UnsupportedOperation;

/// Decimal places per axis. An axis left as `None` is not rounded.
///
/// `x` and `y` are the first and second stored components, whatever the frame
/// calls them; `z` is the third, or the elevation of a 2D leaf.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CoordinatePrecision {
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub z: Option<u32>,
}

impl CoordinatePrecision {
    /// `coord` with each axis rounded to the places configured for it.
    pub fn apply(&self, coord: [f64; 3]) -> [f64; 3] {
        [
            round_to(coord[0], self.x),
            round_to(coord[1], self.y),
            round_to(coord[2], self.z),
        ]
    }

    pub(crate) fn apply_2d(&self, coord: [f64; 2]) -> [f64; 2] {
        [round_to(coord[0], self.x), round_to(coord[1], self.y)]
    }
}

/// `value` rounded half away from zero to `places` decimal places, or unchanged
/// when `places` is `None` or too many to scale `value` by.
pub fn round_to(value: f64, places: Option<u32>) -> f64 {
    match places {
        None => value,
        Some(places) => {
            let Ok(places) = i32::try_from(places) else {
                return value;
            };
            let scale = 10f64.powi(places);
            let scaled = value * scale;
            if scale.is_finite() && scaled.is_finite() {
                scaled.round() / scale
            } else {
                value
            }
        }
    }
}

/// Round every coordinate of a geometry in place.
///
/// Rounding never changes the structure: rings keep their vertex count, and a
/// ring that becomes degenerate is left for validation to report.
#[enum_dispatch::enum_dispatch]
pub trait RoundCoordinates {
    /// Round every coordinate, recursing into collections. The default body
    /// reports the type as unsupported.
    fn round_coordinates(
        &mut self,
        precision: &CoordinatePrecision,
    ) -> Result<(), UnsupportedOperation> {
        let _ = precision;
        Err(UnsupportedOperation {
            geometry: core::any::type_name::<Self>(),
            operation: "round_coordinates",
        })
    }
}

// The boxed enum variants (`Box<Polygon2D>`, `Box<Solid>`, …) need the trait on
// the `Box` itself: `enum_dispatch` forwards by UFCS, not auto-deref.
impl<T: RoundCoordinates + ?Sized> RoundCoordinates for Box<T> {
    fn round_coordinates(
        &mut self,
        precision: &CoordinatePrecision,
    ) -> Result<(), UnsupportedOperation> {
        (**self).round_coordinates(precision)
    }
}

/// Round a 2D coordinate buffer, and the leaf's elevation when present.
pub(crate) fn round_2d(
    coords: &mut [[f64; 2]],
    z: &mut Option<f64>,
    precision: &CoordinatePrecision,
) {
    for c in coords.iter_mut() {
        *c = precision.apply_2d(*c);
    }
    if let Some(elevation) = z {
        *elevation = round_to(*elevation, precision.z);
    }
}

/// Round a 3D coordinate buffer.
pub(crate) fn round_3d(coords: &mut [[f64; 3]], precision: &CoordinatePrecision) {
    for c in coords.iter_mut() {
        *c = precision.apply(*c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection3D;
    use crate::coordinate::CoordinateFrame;
    use crate::line_string::LineString2D;
    use crate::point::Point3D;
    use crate::polygon::Polygon3D;
    use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection};
    use pretty_assertions::assert_eq;

    const PRECISION: CoordinatePrecision = CoordinatePrecision {
        x: Some(12),
        y: Some(2),
        z: Some(4),
    };

    #[test]
    fn each_axis_is_rounded_to_its_own_places() {
        assert_eq!(
            PRECISION.apply([1.000_000_000_000_4, 1.234_9, 0.000_05]),
            [1.0, 1.23, 0.0001]
        );
    }

    #[test]
    fn places_too_many_to_scale_by_leave_the_value_unchanged() {
        assert_eq!(round_to(1.5, Some(u32::MAX)), 1.5);
        assert_eq!(round_to(1.5, Some(309)), 1.5);
        assert_eq!(round_to(0.0, Some(309)), 0.0);
        assert_eq!(round_to(1e10, Some(300)), 1e10);
    }

    #[test]
    fn an_axis_without_places_is_unchanged() {
        let precision = CoordinatePrecision {
            x: None,
            y: Some(0),
            z: None,
        };
        assert_eq!(
            precision.apply([1.234_567, 1.6, 0.000_05]),
            [1.234_567, 2.0, 0.000_05]
        );
    }

    #[test]
    fn a_polygon_rounds_its_holes_too() {
        let mut polygon = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [4.0, 4.0, 0.0],
                [0.0, 0.0, 0.000_04],
            ],
            vec![vec![
                [1.0, 1.0, 0.0],
                [1.0, 3.001, 0.0],
                [3.0, 3.0, 0.0],
                [1.0, 1.0, 0.000_04],
            ]],
        );
        polygon.round_coordinates(&PRECISION).unwrap();
        let expected = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [4.0, 4.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
            vec![vec![
                [1.0, 1.0, 0.0],
                [1.0, 3.0, 0.0],
                [3.0, 3.0, 0.0],
                [1.0, 1.0, 0.0],
            ]],
        );
        assert_eq!(polygon, expected);
    }

    #[test]
    fn a_2d_leaf_rounds_its_elevation() {
        let mut line = LineString2D::from_coords_at_elevation(
            CoordinateFrame::Euclidean,
            [[0.0, 1.234]],
            5.000_04,
        );
        line.round_coordinates(&PRECISION).unwrap();
        assert_eq!(line.coords(), &[[0.0, 1.23]]);
        assert_eq!(line.elevation(), Some(5.0));
    }

    #[test]
    fn every_member_of_a_collection_is_rounded() {
        let point =
            |z| Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Euclidean, [0.0, 0.0, z]));
        let mut geometry = Geometry::GeometryCollection(GeometryCollection::new(vec![
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new(vec![
                point(1.000_04),
                point(2.000_04),
            ]))),
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
                CoordinateFrame::Euclidean,
                [[0.0, 1.234]],
            ))),
            Geometry::None,
        ]));
        geometry.round_coordinates(&PRECISION).unwrap();
        let expected = Geometry::GeometryCollection(GeometryCollection::new(vec![
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new(vec![
                point(1.0),
                point(2.0),
            ]))),
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
                CoordinateFrame::Euclidean,
                [[0.0, 1.23]],
            ))),
            Geometry::None,
        ]));
        assert_eq!(geometry, expected);
    }
}
