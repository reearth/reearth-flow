use approx::{AbsDiffEq, RelativeEq};
use nalgebra::{Point2 as NaPoint2, Point3 as NaPoint3};
use num_traits::Zero;
use serde::{Deserialize, Serialize};

use crate::polygon;

use super::{
    conversion::geojson::create_from_rect_type,
    coordinate::{Coordinate, Coordinate2D, Coordinate3D},
    coordnum::{CoordFloat, CoordNum, CoordNumT},
    no_value::NoValue,
    polygon::Polygon,
    traits::Elevation,
};

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub struct Rect<T: CoordNum = f64, Z: CoordNum = f64> {
    pub(crate) min: Coordinate<T, Z>,
    pub(crate) max: Coordinate<T, Z>,
}

pub type Rect2D<T> = Rect<T, NoValue>;
pub type Rect3D<T> = Rect<T, T>;

impl<T: CoordNum, Z: CoordNum> Rect<T, Z> {
    pub fn new<C>(c1: C, c2: C) -> Self
    where
        C: Into<Coordinate<T, Z>>,
    {
        let c1 = c1.into();
        let c2 = c2.into();
        let (min_x, max_x) = if c1.x < c2.x {
            (c1.x, c2.x)
        } else {
            (c2.x, c1.x)
        };
        let (min_y, max_y) = if c1.y < c2.y {
            (c1.y, c2.y)
        } else {
            (c2.y, c1.y)
        };
        let (min_z, max_z) = if c1.z < c2.z {
            (c1.z, c2.z)
        } else {
            (c2.z, c1.z)
        };
        Self {
            min: Coordinate::new__(min_x, min_y, min_z),
            max: Coordinate::new__(max_x, max_y, max_z),
        }
    }

    pub fn min(self) -> Coordinate<T, Z> {
        self.min
    }

    pub fn set_min<C>(&mut self, min: C)
    where
        C: Into<Coordinate<T, Z>>,
    {
        self.min = min.into();
    }

    pub fn max(self) -> Coordinate<T, Z> {
        self.max
    }

    pub fn set_max<C>(&mut self, max: C)
    where
        C: Into<Coordinate<T, Z>>,
    {
        self.max = max.into();
    }

    pub fn width(self) -> T {
        self.max().x - self.min().x
    }

    pub fn height(self) -> T {
        self.max().y - self.min().y
    }

    pub fn depth(self) -> Z {
        self.max().z - self.min().z
    }

    pub fn to_polygon(self) -> Polygon<T, Z> {
        polygon![
            (x: self.min.x, y: self.min.y, z: self.min.z),
            (x: self.min.x, y: self.max.y, z: self.min.z),
            (x: self.max.x, y: self.max.y, z: self.min.z),
            (x: self.max.x, y: self.max.y, z: self.max.z),
            (x: self.max.x, y: self.min.y, z: self.max.z),
            (x: self.min.x, y: self.max.y, z: self.max.z),
            (x: self.min.x, y: self.min.y, z: self.max.z),
        ]
    }

    pub fn has_valid_bounds(&self) -> bool {
        self.min.x <= self.max.x && self.min.y <= self.max.y && self.min.z <= self.max.z
    }
}

impl From<Rect2D<f64>> for Vec<NaPoint2<f64>> {
    #[inline]
    fn from(p: Rect2D<f64>) -> Vec<NaPoint2<f64>> {
        let result = p
            .to_polygon()
            .rings()
            .into_iter()
            .map(|c| c.into())
            .collect::<Vec<Vec<NaPoint2<f64>>>>();
        result.into_iter().flatten().collect()
    }
}

impl From<Rect3D<f64>> for Vec<NaPoint3<f64>> {
    #[inline]
    fn from(p: Rect3D<f64>) -> Vec<NaPoint3<f64>> {
        let result = p
            .to_polygon()
            .rings()
            .into_iter()
            .map(|c| c.into())
            .collect::<Vec<Vec<NaPoint3<f64>>>>();
        result.into_iter().flatten().collect()
    }
}

impl From<Rect3D<f64>> for Rect2D<f64> {
    #[inline]
    fn from(p: Rect3D<f64>) -> Rect2D<f64> {
        Rect2D::new(p.min.x_y(), p.max.x_y())
    }
}

impl<T: CoordFloat> Rect2D<T> {
    pub fn center(self) -> Coordinate2D<T> {
        let two = T::one() + T::one();
        Coordinate::new_(
            (self.max.x + self.min.x) / two,
            (self.max.y + self.min.y) / two,
        )
    }
}

impl<T: CoordFloat + CoordNumT> Rect3D<T> {
    pub fn center(self) -> Coordinate3D<T> {
        let two = T::one() + T::one();
        Coordinate::new__(
            (self.max.x + self.min.x) / two,
            (self.max.y + self.min.y) / two,
            (self.max.z + self.min.z) / two,
        )
    }
}

impl<T: CoordFloat, Z: CoordFloat> From<Rect<T, Z>> for geojson::Value {
    fn from(rect: Rect<T, Z>) -> Self {
        let coords = create_from_rect_type(&rect);
        geojson::Value::Polygon(coords)
    }
}

impl<T, Z> RelativeEq for Rect<T, Z>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum + RelativeEq,
    Z: AbsDiffEq<Epsilon = Z> + CoordNum + RelativeEq,
{
    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    #[inline]
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        if !self.min.relative_eq(&other.min, epsilon, max_relative) {
            return false;
        }

        if !self.max.relative_eq(&other.max, epsilon, max_relative) {
            return false;
        }

        true
    }
}

impl<T, Z> AbsDiffEq for Rect<T, Z>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum,
    Z: AbsDiffEq<Epsilon = Z> + CoordNum,
    T::Epsilon: Copy,
    Z::Epsilon: Copy,
{
    type Epsilon = T;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }
    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        if !self.min.abs_diff_eq(&other.min, epsilon) {
            return false;
        }

        if !self.max.abs_diff_eq(&other.max, epsilon) {
            return false;
        }

        true
    }
}

impl<T, Z> Elevation for Rect<T, Z>
where
    T: CoordNum + Zero,
    Z: CoordNum + Zero,
{
    #[inline]
    fn is_elevation_zero(&self) -> bool {
        self.min.is_elevation_zero() && self.max.is_elevation_zero()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::coord;

    #[test]
    fn rect() {
        let rect = Rect::new((10, 10), (20, 20));
        assert_eq!(rect.min, coord! { x: 10, y: 10 });
        assert_eq!(rect.max, coord! { x: 20, y: 20 });

        let rect = Rect::new((20, 20), (10, 10));
        assert_eq!(rect.min, coord! { x: 10, y: 10 });
        assert_eq!(rect.max, coord! { x: 20, y: 20 });

        let rect = Rect::new((10, 20), (20, 10));
        assert_eq!(rect.min, coord! { x: 10, y: 10 });
        assert_eq!(rect.max, coord! { x: 20, y: 20 });
    }

    #[test]
    fn rect_width() {
        let rect = Rect::new((10, 10), (20, 20));
        assert_eq!(rect.width(), 10);
    }
}
