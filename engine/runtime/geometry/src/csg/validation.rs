use super::{Csg, ThreeDimensional};
use crate::validation_next::ValidationType;
use crate::validation_next::{Validate, ValidationReport};

impl Validate for Csg {
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationReport> {
        // A `Csg` holds no coordinates of its own; it validates by recursing into
        // its two operands, exactly as `BoundingBox` / `Reproject` would.
        let (left, right) = match self {
            Csg::Union(a, b) | Csg::Intersection(a, b) | Csg::Difference(a, b) => (a, b),
        };
        let mut report = ValidationReport::default();
        if let Some(r) = validate_operand(left, valid_type.clone()) {
            report.extend(r);
        }
        if let Some(r) = validate_operand(right, valid_type) {
            report.extend(r);
        }
        report.into_option()
    }
}

/// Recurse into a CSG operand, dispatching to the concrete `Solid` or nested
/// `Csg`.
fn validate_operand(
    operand: &ThreeDimensional,
    valid_type: ValidationType,
) -> Option<ValidationReport> {
    match operand {
        ThreeDimensional::Solid(solid) => solid.validate(valid_type),
        ThreeDimensional::Csg(csg) => csg.validate(valid_type),
    }
}
