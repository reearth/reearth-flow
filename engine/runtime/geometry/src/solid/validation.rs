use super::{Shell, Solid};
use crate::validation_next::{
    check_duplicate_points_3d, check_finite_3d, Validate, ValidationReport, ValidationType,
};

impl Validate for Solid {
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationReport> {
        // A solid validates by its shells' vertex pools. The indices refer into
        // these vertices, so scanning each shell's pool covers the whole boundary.
        let mut report = ValidationReport::default();
        for shell in std::iter::once(self.exterior()).chain(self.interiors()) {
            check_finite_3d(&self.frame, shell.vertices().iter().copied(), &mut report);
        }
        match &valid_type {
            ValidationType::TooFewPoints => {
                for shell in std::iter::once(self.exterior()).chain(self.interiors()) {
                    // A triangle shell's faces always have three corners; only a
                    // polygon shell can carry a degenerate ring.
                    if let Shell::PolygonMesh(data) = shell {
                        data.check_too_few_points(&self.frame, &mut report);
                    }
                }
            }
            ValidationType::UnclosedRing => {
                for shell in std::iter::once(self.exterior()).chain(self.interiors()) {
                    // A triangle shell's faces are implicitly closed; only a
                    // polygon shell stores an explicit closing vertex.
                    if let Shell::PolygonMesh(data) = shell {
                        data.check_unclosed_rings(&self.frame, &mut report);
                    }
                }
            }
            ValidationType::DuplicatePoints { tolerance } => {
                for shell in std::iter::once(self.exterior()).chain(self.interiors()) {
                    check_duplicate_points_3d(
                        &self.frame,
                        shell.vertices().iter().copied(),
                        *tolerance,
                        &mut report,
                    );
                }
            }
            _ => {}
        }
        // TODO(new-geometry validation): implement the remaining `_valid_type`
        // checks — `ShellManifold` (watertight boundary), `Connected` (each shell
        // a single component), `Orientation` (consistent winding),
        // `NormalDirection` (exterior normals outward, void normals inward), and
        // `Planarity` per face. A solid's interiors are void *shells*, not
        // interior rings, so `InteriorRingContainment` does not apply.
        report.into_option()
    }
}
