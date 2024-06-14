use serde::{Deserialize, Serialize};

use crate::polygon;

use super::{
    coordinate::Coordinate,
    coordnum::{CoordFloat, CoordNum},
    no_value::NoValue,
    polygon::Polygon,
};

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub struct Rect<T: CoordNum = f64, Z: CoordNum = f64> {
    min: Coordinate<T, Z>,
    max: Coordinate<T, Z>,
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

impl<T: CoordFloat> Rect<T, NoValue> {
    pub fn center(self) -> Coordinate<T, NoValue> {
        let two = T::one() + T::one();
        Coordinate::new_(
            (self.max.x + self.min.x) / two,
            (self.max.y + self.min.y) / two,
        )
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
