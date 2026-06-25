use super::{Csg, ThreeDimensional};
use crate::ops::{union_results, Aabb, BoundingBox, UnsupportedOperation};

impl BoundingBox for Csg {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        let (left, right) = match self {
            Csg::Union(a, b) | Csg::Intersection(a, b) | Csg::Difference(a, b) => (a, b),
        };
        // A boolean result is always contained in the union of its operands'
        // extents, so the union of the two operand boxes is a valid bound for
        // every operator — loose for intersection/difference, exact for union.
        // Evaluating the tree for a tighter box is out of scope here.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::Coordinate;
    use crate::solid::Solid;
    use crate::triangular_mesh::TriangularMesh3DData;

    fn solid_at(origin: [f64; 3]) -> Solid {
        let [x, y, z] = origin;
        let shell = TriangularMesh3DData::from_parts(
            vec![[x, y, z], [x + 1.0, y, z], [x, y + 1.0, z + 1.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        Solid::from_exterior(Coordinate::Euclidean, shell)
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
}
