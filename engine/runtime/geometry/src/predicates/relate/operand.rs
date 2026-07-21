//! The operand facade the relate graphs are built from.
//!
//! [`RelateOperand`] is a flattened 2D operand (the [`Operand2D`]) plus
//! everything `RelateOperation` asks of a whole geometry (dimensions, union
//! point position, bounding box) and the per-mesh union-boundary rings the
//! graph input layer consumes instead of raw mesh faces (see
//! [`boundary`](super::boundary)).

use crate::ops::Aabb;
use crate::predicates::kernel::CoordPos;
use crate::predicates::position::union_position;
use crate::predicates::view::{Leaf2D, Operand2D, PreparedLeaf};

use super::boundary::union_boundary_rings;
use super::intersection_matrix::Dimensions;

pub(crate) struct RelateOperand<'a> {
    operand: Operand2D<'a>,
    /// Union-boundary rings per leaf: `Some` for mesh leaves, `None` otherwise.
    boundaries: Vec<Option<Vec<Vec<[f64; 2]>>>>,
}

impl<'a> RelateOperand<'a> {
    pub fn new(leaves: Vec<Leaf2D<'a>>) -> Self {
        let operand = Operand2D::from_leaves(leaves);
        let boundaries = operand
            .leaves
            .iter()
            .map(|prepared| match prepared.leaf {
                Leaf2D::PolygonMesh(_) | Leaf2D::TriangularMesh(_) => {
                    let area = prepared.area.as_ref().expect("meshes are areal");
                    Some(union_boundary_rings(area))
                }
                _ => None,
            })
            .collect();
        Self {
            operand,
            boundaries,
        }
    }

    pub fn leaves(&self) -> &[PreparedLeaf<'a>] {
        &self.operand.leaves
    }

    /// The union-boundary rings of the `i`-th leaf; `None` for non-mesh leaves.
    pub fn boundary_rings(&self, i: usize) -> Option<&[Vec<[f64; 2]>]> {
        self.boundaries[i].as_deref()
    }

    /// Whether the operand contains a mesh leaf. Meshes are the MultiPolygon
    /// analog, which in JTS switches off the mod-2 boundary determination rule
    /// for the whole graph.
    pub fn has_mesh(&self) -> bool {
        self.boundaries.iter().any(Option::is_some)
    }

    /// Whether every component is a ring (areal, or a closed chain): the JTS
    /// cue that self-intersection noding may skip the self-intersecting-edge
    /// check on valid input.
    pub fn is_rings(&self) -> bool {
        self.leaves().iter().all(|prepared| match prepared.leaf {
            Leaf2D::Point(_) => false,
            Leaf2D::Line(l) => is_closed(l.coords()),
            Leaf2D::Polygon(_) | Leaf2D::PolygonMesh(_) | Leaf2D::TriangularMesh(_) => true,
        })
    }

    /// The union bounding box, `None` when the operand is empty.
    pub fn bounding_box(&self) -> Option<Aabb> {
        self.leaves()
            .iter()
            .filter_map(|prepared| prepared.bbox)
            .reduce(Aabb::union)
    }

    /// The position of a coordinate relative to the operand's point-set union.
    pub fn coordinate_position(&self, coord: [f64; 2]) -> CoordPos {
        union_position(coord, &self.operand)
    }

    /// The point-set dimension: the maximum over the leaves.
    pub fn dimensions(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for prepared in self.leaves() {
            let dimensions = leaf_dimensions(&prepared.leaf);
            if dimensions == Dimensions::TwoDimensional {
                // short-circuit since we know none can be larger
                return Dimensions::TwoDimensional;
            }
            max = max.max(dimensions);
        }
        max
    }

    /// The dimension of the point-set boundary: the maximum over the leaves.
    /// Points have no boundary; a closed chain has none either (SFS mod-2);
    /// an open chain's endpoints are 0-dimensional; areal boundaries are rings.
    pub fn boundary_dimensions(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for prepared in self.leaves() {
            let dimensions = match prepared.leaf {
                Leaf2D::Point(_) => Dimensions::Empty,
                Leaf2D::Line(l) => {
                    if leaf_dimensions(&prepared.leaf) < Dimensions::OneDimensional
                        || is_closed(l.coords())
                    {
                        Dimensions::Empty
                    } else {
                        Dimensions::ZeroDimensional
                    }
                }
                Leaf2D::Polygon(_) | Leaf2D::PolygonMesh(_) | Leaf2D::TriangularMesh(_) => {
                    if leaf_dimensions(&prepared.leaf) == Dimensions::Empty {
                        Dimensions::Empty
                    } else {
                        Dimensions::OneDimensional
                    }
                }
            };
            max = max.max(dimensions);
        }
        max
    }
}

fn is_closed(coords: &[[f64; 2]]) -> bool {
    coords.len() >= 2 && coords.first() == coords.last()
}

/// The point-set dimension of one leaf. Degenerate leaves collapse: a chain of
/// coincident vertices is a point, an areal leaf with no rings is empty.
fn leaf_dimensions(leaf: &Leaf2D<'_>) -> Dimensions {
    match leaf {
        Leaf2D::Point(_) => Dimensions::ZeroDimensional,
        Leaf2D::Line(l) => {
            let coords = l.coords();
            match coords {
                [] => Dimensions::Empty,
                [first, rest @ ..] => {
                    if rest.iter().all(|c| c == first) {
                        Dimensions::ZeroDimensional
                    } else {
                        Dimensions::OneDimensional
                    }
                }
            }
        }
        Leaf2D::Polygon(p) => {
            if p.exterior().is_empty() {
                Dimensions::Empty
            } else {
                Dimensions::TwoDimensional
            }
        }
        Leaf2D::PolygonMesh(_) | Leaf2D::TriangularMesh(_) => {
            let area = leaf.area_view().expect("meshes are areal");
            if area.num_faces() == 0 {
                Dimensions::Empty
            } else {
                Dimensions::TwoDimensional
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::line_string::LineString2D;
    use crate::point::Point2D;
    use crate::polygon::Polygon2D;
    use crate::polygon_mesh::PolygonMesh2D;
    use crate::predicates::view::flatten_2d;
    use crate::Euclidean2DGeometry;
    use pretty_assertions::assert_eq;

    fn e() -> CoordinateFrame {
        CoordinateFrame::Euclidean
    }

    fn operand(geometry: &Euclidean2DGeometry) -> RelateOperand<'_> {
        let mut leaves = Vec::new();
        flatten_2d(geometry, &mut leaves);
        RelateOperand::new(leaves)
    }

    #[test]
    fn dimensions_per_leaf_kind() {
        let point = Euclidean2DGeometry::Point(Point2D::new(e(), [1.0, 1.0]));
        let p = operand(&point);
        assert_eq!(p.dimensions(), Dimensions::ZeroDimensional);
        assert_eq!(p.boundary_dimensions(), Dimensions::Empty);

        let open = Euclidean2DGeometry::LineString(LineString2D::from_coords(
            e(),
            [[0.0, 0.0], [2.0, 0.0]],
        ));
        let l = operand(&open);
        assert_eq!(l.dimensions(), Dimensions::OneDimensional);
        assert_eq!(l.boundary_dimensions(), Dimensions::ZeroDimensional);

        let closed = Euclidean2DGeometry::LineString(LineString2D::from_coords(
            e(),
            [[0.0, 0.0], [2.0, 0.0], [1.0, 1.0], [0.0, 0.0]],
        ));
        let c = operand(&closed);
        assert_eq!(c.dimensions(), Dimensions::OneDimensional);
        assert_eq!(c.boundary_dimensions(), Dimensions::Empty);

        let degenerate = Euclidean2DGeometry::LineString(LineString2D::from_coords(
            e(),
            [[1.0, 1.0], [1.0, 1.0]],
        ));
        assert_eq!(
            operand(&degenerate).dimensions(),
            Dimensions::ZeroDimensional
        );

        let square = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let polygon = Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            e(),
            square,
            Vec::<Vec<[f64; 2]>>::new(),
        )));
        let a = operand(&polygon);
        assert_eq!(a.dimensions(), Dimensions::TwoDimensional);
        assert_eq!(a.boundary_dimensions(), Dimensions::OneDimensional);
        assert!(a.is_rings());
        assert!(!a.has_mesh());
    }

    #[test]
    fn mesh_boundaries_are_extracted_once() {
        let mesh = PolygonMesh2D::from_parts(
            e(),
            vec![
                [0.0, 0.0],
                [2.0, 0.0],
                [2.0, 2.0],
                [0.0, 2.0],
                [4.0, 0.0],
                [4.0, 2.0],
            ],
            vec![vec![0u32, 1, 2, 3], vec![1, 4, 5, 2]],
        )
        .unwrap();
        let geometry = Euclidean2DGeometry::PolygonMesh(Box::new(mesh));
        let m = operand(&geometry);
        assert!(m.has_mesh());
        assert!(m.is_rings());
        let rings = m.boundary_rings(0).expect("mesh leaf has boundary rings");
        assert_eq!(rings.len(), 1);
        assert_eq!(m.dimensions(), Dimensions::TwoDimensional);
        // The union position sees the shared edge as interior.
        assert_eq!(m.coordinate_position([2.0, 1.0]), CoordPos::Inside);
    }

    #[test]
    fn empty_operand() {
        let o = RelateOperand::new(Vec::new());
        assert_eq!(o.dimensions(), Dimensions::Empty);
        assert_eq!(o.boundary_dimensions(), Dimensions::Empty);
        assert!(o.bounding_box().is_none());
    }
}
