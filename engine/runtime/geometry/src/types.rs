pub mod conversion;
pub mod coordinate;
pub mod coordnum;
pub mod csg;
pub mod face;
pub mod geometry;
pub mod geometry_collection;
pub mod line;
pub mod line_string;
pub mod multi_line_string;
pub mod multi_point;
pub mod multi_polygon;
pub mod no_value;
pub mod point;
pub mod polygon;
pub mod rect;
pub mod solid;
pub mod traits;
pub mod triangle;
pub mod triangular_mesh;
pub mod validation;

pub enum ConversionResult<T, Info, Err> {
    Ok((T, Info)),
    Err(Err),
}

/// A trait for converting geometry types with possible errors.
pub trait GeometryConvertFrom<T: Sized> {
    type Error;
    type Info;

    fn convert_from(value: T) -> ConversionResult<Self, Self::Info, Self::Error>
    where
        Self: Sized;
}
