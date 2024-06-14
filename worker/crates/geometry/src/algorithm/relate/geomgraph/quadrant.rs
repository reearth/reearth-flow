use crate::algorithm::GeoNum;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Eq)]
pub enum Quadrant {
    NE,
    NW,
    SW,
    SE,
}

impl Quadrant {
    pub fn new<T: GeoNum>(dx: T, dy: T) -> Option<Quadrant> {
        if dx.is_zero() && dy.is_zero() {
            return None;
        }

        match (dy >= T::zero(), dx >= T::zero()) {
            (true, true) => Quadrant::NE,
            (true, false) => Quadrant::NW,
            (false, false) => Quadrant::SW,
            (false, true) => Quadrant::SE,
        }
        .into()
    }
}
