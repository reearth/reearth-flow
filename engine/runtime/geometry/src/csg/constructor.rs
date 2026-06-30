//! Csg constructors.
//!
//! The operands — `Solid`s or nested `Csg`s — are already constructed; these
//! constructors just box them into the boolean tree. The `From` impls let a
//! `Solid` or `Csg` be passed where a [`ThreeDimensional`] operand is expected.

use crate::solid::Solid;

use super::{Csg, ThreeDimensional};

impl From<Solid> for ThreeDimensional {
    fn from(solid: Solid) -> Self {
        ThreeDimensional::Solid(Box::new(solid))
    }
}

impl From<Csg> for ThreeDimensional {
    fn from(csg: Csg) -> Self {
        ThreeDimensional::Csg(Box::new(csg))
    }
}

impl Csg {
    /// The union of two volumetric operands.
    pub fn union(left: impl Into<ThreeDimensional>, right: impl Into<ThreeDimensional>) -> Self {
        Csg::Union(Box::new(left.into()), Box::new(right.into()))
    }

    /// The intersection of two volumetric operands.
    pub fn intersection(
        left: impl Into<ThreeDimensional>,
        right: impl Into<ThreeDimensional>,
    ) -> Self {
        Csg::Intersection(Box::new(left.into()), Box::new(right.into()))
    }

    /// The difference `left - right` of two volumetric operands.
    pub fn difference(
        left: impl Into<ThreeDimensional>,
        right: impl Into<ThreeDimensional>,
    ) -> Self {
        Csg::Difference(Box::new(left.into()), Box::new(right.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::Coordinate;
    use crate::triangular_mesh::TriangularMesh3DData;

    fn solid() -> Solid {
        let shell = TriangularMesh3DData::from_parts(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        Solid::from_exterior(Coordinate::Euclidean, shell)
    }

    #[test]
    fn union_of_two_solids() {
        let csg = Csg::union(solid(), solid());
        assert!(matches!(
            csg,
            Csg::Union(l, r)
                if matches!(*l, ThreeDimensional::Solid(_))
                && matches!(*r, ThreeDimensional::Solid(_))
        ));
    }

    #[test]
    fn nested_csg_is_an_operand() {
        let inner = Csg::union(solid(), solid());
        let outer = Csg::difference(solid(), inner);
        assert!(matches!(
            outer,
            Csg::Difference(_, r) if matches!(*r, ThreeDimensional::Csg(_))
        ));
    }
}
