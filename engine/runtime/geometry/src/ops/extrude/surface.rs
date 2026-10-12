//! The areal sweep. Filled in by Task 4.
use super::{ExtrudeError, ExtrudeSpec};
use crate::appearance::Appearance;
use crate::coordinate::CoordinateFrame;
use crate::Euclidean3DGeometry;

/// One face to sweep: its rings (exterior first) exactly as stored, and its
/// appearance, which only survives on a cap that keeps this ring layout.
pub(crate) struct CapFace {
    pub(crate) rings: Vec<Vec<[f64; 3]>>,
    pub(crate) appearance: Option<Appearance>,
}

pub(crate) fn sweep_surface(
    _frame: &CoordinateFrame,
    _faces: Vec<CapFace>,
    _spec: &ExtrudeSpec,
) -> Result<Euclidean3DGeometry, ExtrudeError> {
    Err(ExtrudeError::UnsupportedType(
        "not yet implemented".to_string(),
    ))
}
