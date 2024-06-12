pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

pub mod line;
pub mod line_string;
pub mod point;
pub mod polygon;
pub mod rect;
pub mod triangle;
