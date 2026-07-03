use super::Solid;
use crate::ops::validation::{check_finite_3d, Validate, ValidationReport};
use crate::ops::ValidationType;

impl Validate for Solid {
    fn validate(&self, _valid_type: ValidationType) -> Option<ValidationReport> {
        // A solid validates by its shells' vertex pools. Finiteness is the only
        // check for now; the indices refer into these vertices, so scanning each
        // shell's pool covers the whole boundary.
        let mut report = ValidationReport::default();
        check_finite_3d("Solid", self.exterior.vertices(), &mut report);
        for shell in &self.interiors {
            check_finite_3d("Solid", shell.vertices(), &mut report);
        }
        // TODO(new-geometry validation): implement the `_valid_type` checks —
        // `ShellManifold` (watertight boundary), `Connected` (each shell a
        // single component), `Orientation` (consistent winding), `NormalDirection`
        // (exterior normals outward, void normals inward), and `Planarity` /
        // `Degenerate` per face. A solid's interiors are void *shells*, not
        // interior rings, so `InteriorRingContainment` does not apply.
        report.into_option()
    }
}
