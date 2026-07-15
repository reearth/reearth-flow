/// The quadrant of a direction vector, for sorting edge ends around a node.
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Eq)]
pub enum Quadrant {
    NE,
    NW,
    SW,
    SE,
}

impl Quadrant {
    pub fn new(dx: f64, dy: f64) -> Option<Quadrant> {
        if dx == 0.0 && dy == 0.0 {
            return None;
        }

        match (dy >= 0.0, dx >= 0.0) {
            (true, true) => Quadrant::NE,
            (true, false) => Quadrant::NW,
            (false, false) => Quadrant::SW,
            (false, true) => Quadrant::SE,
        }
        .into()
    }
}
