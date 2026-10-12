//! Point and line sweeps. Filled in by Task 3.
use super::{ExtrudeError, ExtrudeSpec};
use crate::coordinate::CoordinateFrame;
use crate::Euclidean3DGeometry;

pub(crate) fn sweep_point(
    _frame: &CoordinateFrame,
    _p: [f64; 3],
    _spec: &ExtrudeSpec,
) -> Result<Euclidean3DGeometry, ExtrudeError> {
    Err(ExtrudeError::UnsupportedType(
        "not yet implemented".to_string(),
    ))
}

pub(crate) fn sweep_line(
    _frame: &CoordinateFrame,
    _coords: &[[f64; 3]],
    _spec: &ExtrudeSpec,
) -> Result<Euclidean3DGeometry, ExtrudeError> {
    Err(ExtrudeError::UnsupportedType(
        "not yet implemented".to_string(),
    ))
}
