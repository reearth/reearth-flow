use super::{Csg, ThreeDimensional};
use crate::ops::{union_results, Aabb, BoundingBox, RemoveAppearance, UnsupportedOperation, Translate};

impl BoundingBox for Csg {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        let (left, right) = match self {
            Csg::Union(a, b) | Csg::Intersection(a, b) | Csg::Difference(a, b) => (a, b),
        };
        union_results([operand_box(left), operand_box(right)]).ok_or(UnsupportedOperation {
            geometry: "Csg",
            operation: "bounding_box",
        })
    }
}

/// The box of a CSG operand, recursing into nested trees.
fn operand_box(operand: &ThreeDimensional) -> Result<Aabb, UnsupportedOperation> {
    match operand {
        ThreeDimensional::Solid(s) => s.bounding_box(),
        ThreeDimensional::Csg(c) => c.bounding_box(),
    }
}

impl RemoveAppearance for Csg {
    fn remove_appearance(&mut self) {
        let (left, right) = match self {
            Csg::Union(a, b) | Csg::Intersection(a, b) | Csg::Difference(a, b) => (a, b),
        };
        remove_operand_appearance(left);
        remove_operand_appearance(right);
    }
}

/// Strip an operand's appearance, recursing into nested trees.
fn remove_operand_appearance(operand: &mut ThreeDimensional) {
    match operand {
        ThreeDimensional::Solid(s) => s.remove_appearance(),
        ThreeDimensional::Csg(c) => c.remove_appearance(),
    }
}

impl Translate for Csg {
    /// Shift both operands, leaving the boolean operator untouched.
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        let (left, right) = match self {
            Csg::Union(a, b) | Csg::Intersection(a, b) | Csg::Difference(a, b) => (a, b),
        };
        translate_operand(left, delta)?;
        translate_operand(right, delta)
    }
}

/// Shift a CSG operand, recursing into nested trees.
fn translate_operand(operand: &mut ThreeDimensional, delta: [f64; 3]) -> crate::error::Result<()> {
    match operand {
        ThreeDimensional::Solid(s) => s.translate(delta),
        ThreeDimensional::Csg(c) => c.translate(delta),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::solid::Solid;
    use crate::triangular_mesh::TriangularMesh3DData;

    fn solid_at(origin: [f64; 3]) -> Solid {
        let [x, y, z] = origin;
        let shell = TriangularMesh3DData::from_parts(
            vec![[x, y, z], [x + 1.0, y, z], [x, y + 1.0, z + 1.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        Solid::from_exterior(CoordinateFrame::Euclidean, shell)
    }

    #[test]
    fn csg_box_is_the_union_of_operands() {
        // Two disjoint solids; the box covers both, regardless of operator.
        let csg = Csg::union(solid_at([0.0, 0.0, 0.0]), solid_at([10.0, 10.0, 10.0]));
        assert_eq!(
            csg.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, 0.0, 0.0],
                max: [11.0, 11.0, 11.0]
            }
        );
    }

    #[test]
    fn csg_box_recurses_into_nested_trees() {
        let inner = Csg::difference(solid_at([0.0, 0.0, 0.0]), solid_at([2.0, 0.0, 0.0]));
        let outer = Csg::intersection(inner, solid_at([0.0, 5.0, 0.0]));
        assert_eq!(
            outer.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, 0.0, 0.0],
                max: [3.0, 6.0, 1.0]
            }
        );
    }

    #[test]
    fn csg_translate_shifts_both_operands() {
        let mut csg = Csg::union(solid_at([0.0, 0.0, 0.0]), solid_at([10.0, 10.0, 10.0]));
        csg.translate([1.0, 2.0, 3.0]).unwrap();
        assert_eq!(
            csg.bounding_box().unwrap(),
            Aabb::D3 {
                min: [1.0, 2.0, 3.0],
                max: [12.0, 13.0, 14.0]
            }
        );
    }

    #[test]
    fn csg_translate_recurses_into_nested_trees() {
        let inner = Csg::difference(solid_at([0.0, 0.0, 0.0]), solid_at([2.0, 0.0, 0.0]));
        let mut outer = Csg::intersection(inner, solid_at([0.0, 5.0, 0.0]));
        outer.translate([-1.0, 0.0, 0.5]).unwrap();
        assert_eq!(
            outer.bounding_box().unwrap(),
            Aabb::D3 {
                min: [-1.0, 0.0, 0.5],
                max: [2.0, 6.0, 1.5]
            }
        );
    }
}
