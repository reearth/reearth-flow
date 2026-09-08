//! Geometric predicates over polyline leaves.

use super::{LineString2D, LineString3D};
use crate::predicates::{self, Equal};

impl Equal for LineString2D {
    fn equal(&self, rhs: &Self, tolerance: f64) -> predicates::Result<bool> {
        use crate::predicates::equal::chain_curves_2d;

        predicates::require_same_frame(self.frame(), rhs.frame())?;
        let ours = chain_curves_2d(self.coords(), self.elevation());
        let theirs = chain_curves_2d(rhs.coords(), rhs.elevation());
        Ok(ours.within(&theirs, tolerance))
    }
}

impl Equal for LineString3D {
    fn equal(&self, rhs: &Self, tolerance: f64) -> predicates::Result<bool> {
        use crate::predicates::equal::Curves;

        predicates::require_same_frame(self.frame(), rhs.frame())?;
        let chain = |line: &Self| {
            let mut curves = Curves::new();
            curves.push_chain(line.coords().iter().copied());
            curves.finish()
        };
        Ok(chain(self).within(&chain(rhs), tolerance))
    }
}
