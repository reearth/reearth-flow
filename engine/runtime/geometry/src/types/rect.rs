use approx::{AbsDiffEq, RelativeEq};
use nalgebra::{Point2 as NaPoint2, Point3 as NaPoint3};
use num_traits::{Bounded, Zero};
use nusamai_projection::vshift::Jgd2011ToWgs84;
use rstar::{RTreeNum, RTreeObject, AABB};
use serde::{Deserialize, Serialize};

use crate::polygon;

use super::{
    coordinate::{Coordinate, Coordinate2D, Coordinate3D},
    coordnum::{CoordFloat, CoordNum, CoordNumT},
    multi_polygon::MultiPolygon3D,
    no_value::NoValue,
    polygon::Polygon,
    polygon::Polygon3D,
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

    pub fn merge(self, other: Self) -> Self {
        let min_x = if self.min.x < other.min.x {
            self.min.x
        } else {
            other.min.x
        };
        let min_y = if self.min.y < other.min.y {
            self.min.y
        } else {
            other.min.y
        };
        let min_z = if self.min.z < other.min.z {
            self.min.z
        } else {
            other.min.z
        };
        let max_x = if self.max.x > other.max.x {
            self.max.x
        } else {
            other.max.x
        };
        let max_y = if self.max.y > other.max.y {
            self.max.y
        } else {
            other.max.y
        };
        let max_z = if self.max.z > other.max.z {
            self.max.z
        } else {
            other.max.z
        };
        Self {
            min: Coordinate::new__(min_x, min_y, min_z),
            max: Coordinate::new__(max_x, max_y, max_z),
        }
    }

    pub fn overlap(&self, other: &Rect<T, Z>) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    pub fn has_valid_bounds(&self) -> bool {
        self.min.x <= self.max.x && self.min.y <= self.max.y && self.min.z <= self.max.z
    }
}

impl<T: CoordNum> Rect2D<T> {
    pub fn to_polygon(&self) -> Polygon<T, NoValue> {
        polygon![
            (x: self.min.x, y: self.min.y),
            (x: self.max.x, y: self.min.y),
            (x: self.max.x, y: self.max.y),
            (x: self.min.x, y: self.max.y),
            (x: self.min.x, y: self.min.y),
        ]
    }
}

impl<T: CoordNum + Bounded + RTreeNum> RTreeObject for Rect2D<T> {
    type Envelope = AABB<[T; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners([self.min.x, self.min.y], [self.max.x, self.max.y])
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
        let points = [
            Coordinate3D {
                x: p.min.x,
                y: p.min.y,
                z: p.min.z,
            },
            Coordinate3D {
                x: p.min.x,
                y: p.max.y,
                z: p.min.z,
            },
            Coordinate3D {
                x: p.max.x,
                y: p.max.y,
                z: p.min.z,
            },
            Coordinate3D {
                x: p.max.x,
                y: p.max.y,
                z: p.max.z,
            },
            Coordinate3D {
                x: p.max.x,
                y: p.min.y,
                z: p.max.z,
            },
            Coordinate3D {
                x: p.min.x,
                y: p.max.y,
                z: p.max.z,
            },
            Coordinate3D {
                x: p.min.x,
                y: p.min.y,
                z: p.max.z,
            },
        ];
        points
            .into_iter()
            .map(|c| c.into())
            .collect::<Vec<NaPoint3<f64>>>()
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

    /// Converts the 3D rectangle (axis-aligned bounding box) to a MultiPolygon
    /// representing its 6 faces.
    ///
    /// The faces are ordered as:
    /// 1. Bottom face (z = min_z)
    /// 2. Top face (z = max_z)
    /// 3. Front face (y = min_y)
    /// 4. Back face (y = max_y)
    /// 5. Left face (x = min_x)
    /// 6. Right face (x = max_x)
    pub fn to_multi_polygon(&self) -> MultiPolygon3D<T> {
        // Define all 8 vertices of the box
        // Bottom face vertices (z = min_z)
        let v0 = Coordinate::new__(self.min.x, self.min.y, self.min.z); // front-left-bottom
        let v1 = Coordinate::new__(self.max.x, self.min.y, self.min.z); // front-right-bottom
        let v2 = Coordinate::new__(self.max.x, self.max.y, self.min.z); // back-right-bottom
        let v3 = Coordinate::new__(self.min.x, self.max.y, self.min.z); // back-left-bottom

        // Top face vertices (z = max_z)
        let v4 = Coordinate::new__(self.min.x, self.min.y, self.max.z); // front-left-top
        let v5 = Coordinate::new__(self.max.x, self.min.y, self.max.z); // front-right-top
        let v6 = Coordinate::new__(self.max.x, self.max.y, self.max.z); // back-right-top
        let v7 = Coordinate::new__(self.min.x, self.max.y, self.max.z); // back-left-top

        // Bottom face (z = min_z) - outward normal -Z
        let bottom = Polygon3D::new(vec![v0, v1, v2, v3, v0].into(), Vec::new());

        // Top face (z = max_z) - outward normal +Z
        let top = Polygon3D::new(vec![v4, v7, v6, v5, v4].into(), Vec::new());

        // Front face (y = min_y) - outward normal -Y
        let front = Polygon3D::new(vec![v0, v4, v5, v1, v0].into(), Vec::new());

        // Back face (y = max_y) - outward normal +Y
        let back = Polygon3D::new(vec![v3, v2, v6, v7, v3].into(), Vec::new());

        // Left face (x = min_x) - outward normal -X
        let left = Polygon3D::new(vec![v0, v3, v7, v4, v0].into(), Vec::new());

        // Right face (x = max_x) - outward normal +X
        let right = Polygon3D::new(vec![v1, v5, v6, v2, v1].into(), Vec::new());

        MultiPolygon3D::new(vec![bottom, top, front, back, left, right])
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

impl Rect3D<f64> {
    pub fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        self.min.transform_inplace(jgd2wgs);
        self.max.transform_inplace(jgd2wgs);
    }

    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        self.min.transform_offset(x, y, z);
        self.max.transform_offset(x, y, z);
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

    #[test]
    fn rect_overlap() {
        let rect1 = Rect::new((10, 10), (20, 20));
        let rect2 = Rect::new((15, 15), (25, 25));
        assert!(rect1.overlap(&rect2));

        let rect3 = Rect::new((20, 20), (30, 30));
        assert!(rect1.overlap(&rect3));

        let rect4 = Rect::new((5, 5), (15, 15));
        assert!(rect1.overlap(&rect4));

        let rect5 = Rect::new((0, 0), (5, 5));
        assert!(!rect1.overlap(&rect5));
    }
}
