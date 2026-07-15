//! The `intersects` predicate: whether two geometries share at least one point.
//!
//! Boundary contact counts: a shared vertex, a T-junction, or a segment lying
//! along a ring all intersect. Collections are point-set unions (any member
//! pair intersecting suffices). Every leaf pair goes through a bounding-box
//! quick reject first.
//!
//! Scope: 2D × 2D pairs. A 2D × 3D pair is a
//! [`CrossDimension`](PredicateError::CrossDimension) error, a 3D × 3D pair an
//! [`UnsupportedPair`](PredicateError::UnsupportedPair). `Geometry::None`
//! intersects nothing.

use super::kernel::segment_intersection;
use super::kernel::CoordPos;
use super::position::{face_position, line_position};
use super::view::{require_common_frame, AreaView, Leaf2D, Operand2D, PreparedLeaf};
use super::{PredicateError, Result};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// Whether `a` and `b` share at least one point.
pub fn intersects(a: &Geometry, b: &Geometry) -> Result<bool> {
    match (a, b) {
        (Geometry::None, _) | (_, Geometry::None) => Ok(false),
        (Geometry::GeometryCollection(c), other) => {
            for member in c.members() {
                if intersects(member, other)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        (other, Geometry::GeometryCollection(c)) => {
            for member in c.members() {
                if intersects(other, member)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        (Geometry::Euclidean2D(a), Geometry::Euclidean2D(b)) => intersects_2d(a, b),
        (Geometry::Euclidean2D(_), Geometry::Euclidean3D(_))
        | (Geometry::Euclidean3D(_), Geometry::Euclidean2D(_)) => {
            Err(PredicateError::CrossDimension)
        }
        (Geometry::Euclidean3D(a), Geometry::Euclidean3D(b)) => {
            Err(PredicateError::UnsupportedPair {
                left: type_name_3d(a),
                right: type_name_3d(b),
            })
        }
    }
}

/// `intersects` over two 2D geometries.
pub fn intersects_2d(a: &Euclidean2DGeometry, b: &Euclidean2DGeometry) -> Result<bool> {
    let a = Operand2D::new(a);
    let b = Operand2D::new(b);
    require_common_frame(&a, &b)?;
    for la in &a.leaves {
        for lb in &b.leaves {
            if leaf_intersects(la, lb) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// The concrete 3D leaf name, for `UnsupportedPair` diagnostics.
pub(crate) fn type_name_3d(g: &Euclidean3DGeometry) -> &'static str {
    match g {
        Euclidean3DGeometry::Point(_) => "Point3D",
        Euclidean3DGeometry::PointCloud(_) => "PointCloud",
        Euclidean3DGeometry::LineString(_) => "LineString3D",
        Euclidean3DGeometry::Polygon(_) => "Polygon3D",
        Euclidean3DGeometry::PolygonMesh(_) => "PolygonMesh3D",
        Euclidean3DGeometry::TriangularMesh(_) => "TriangularMesh3D",
        Euclidean3DGeometry::Solid(_) => "Solid",
        Euclidean3DGeometry::Csg(_) => "Csg",
        Euclidean3DGeometry::Collection(_) => "Collection3D",
    }
}

/// Whether two prepared leaves intersect, after a bounding-box quick reject.
/// An empty leaf (no bounding box) intersects nothing.
pub(crate) fn leaf_intersects(a: &PreparedLeaf<'_>, b: &PreparedLeaf<'_>) -> bool {
    let (Some(box_a), Some(box_b)) = (&a.bbox, &b.bbox) else {
        return false;
    };
    if !box_a.intersects(box_b) {
        return false;
    }
    match (&a.leaf, &b.leaf) {
        (Leaf2D::Point(pa), Leaf2D::Point(pb)) => pa.position() == pb.position(),
        (Leaf2D::Point(p), Leaf2D::Line(l)) | (Leaf2D::Line(l), Leaf2D::Point(p)) => {
            line_position(p.position(), l.coords()) != CoordPos::Outside
        }
        (Leaf2D::Point(p), _) => point_vs_area(p.position(), area(b)),
        (_, Leaf2D::Point(p)) => point_vs_area(p.position(), area(a)),
        (Leaf2D::Line(la), Leaf2D::Line(lb)) => line_vs_line(la.coords(), lb.coords()),
        (Leaf2D::Line(l), _) => line_vs_area(l.coords(), area(b)),
        (_, Leaf2D::Line(l)) => line_vs_area(l.coords(), area(a)),
        (_, _) => area_vs_area(area(a), area(b)),
    }
}

/// The areal view of a leaf known to be areal.
fn area<'a, 'b>(leaf: &'b PreparedLeaf<'a>) -> &'b AreaView<'a> {
    leaf.area.as_ref().expect("leaf is areal")
}

fn point_vs_area(coord: [f64; 2], area: &AreaView<'_>) -> bool {
    area.faces()
        .any(|f| face_position(coord, f) != CoordPos::Outside)
}

fn line_vs_line(a: &[[f64; 2]], b: &[[f64; 2]]) -> bool {
    // A single-vertex chain is point-like.
    if a.len() == 1 {
        return line_position(a[0], b) != CoordPos::Outside;
    }
    if b.len() == 1 {
        return line_position(b[0], a) != CoordPos::Outside;
    }
    a.windows(2).any(|sa| {
        b.windows(2)
            .any(|sb| segment_intersection(sa[0], sa[1], sb[0], sb[1]).is_some())
    })
}

fn line_vs_area(coords: &[[f64; 2]], area: &AreaView<'_>) -> bool {
    if coords.len() == 1 {
        return point_vs_area(coords[0], area);
    }
    // Any chain segment meeting the boundary intersects. Otherwise the chain
    // never touches the boundary, so it lies entirely in one region: one
    // vertex decides.
    if coords.windows(2).any(|s| {
        area.edges()
            .any(|(u, v)| segment_intersection(s[0], s[1], u, v).is_some())
    }) {
        return true;
    }
    coords.first().is_some_and(|&c| point_vs_area(c, area))
}

fn area_vs_area(a: &AreaView<'_>, b: &AreaView<'_>) -> bool {
    // Any boundary contact intersects.
    if a.edges().any(|(u, v)| {
        b.edges()
            .any(|(s, t)| segment_intersection(u, v, s, t).is_some())
    }) {
        return true;
    }
    // No contact: every ring lies entirely in one region of the other operand,
    // so one vertex per ring decides full containment either way (a ring
    // inside the other's hole classifies as outside).
    let ring_inside = |x: &AreaView<'_>, y: &AreaView<'_>| {
        x.faces().any(|f| {
            f.rings().any(|r| {
                !r.is_empty()
                    && y.faces()
                        .any(|g| face_position(r.coord(0), g) != CoordPos::Outside)
            })
        })
    };
    ring_inside(a, b) || ring_inside(b, a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection2D;
    use crate::coordinate::{CoordinateFrame, EpsgCode};
    use crate::line_string::{LineString2D, LineString3D};
    use crate::point::Point2D;
    use crate::polygon::Polygon2D;
    use crate::{Euclidean2DGeometry, Euclidean3DGeometry, GeometryCollection};
    use pretty_assertions::assert_eq;

    fn e() -> CoordinateFrame {
        CoordinateFrame::Euclidean
    }

    fn poly(ring: [[f64; 2]; 5]) -> Euclidean2DGeometry {
        Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            e(),
            ring,
            Vec::<Vec<[f64; 2]>>::new(),
        )))
    }

    fn unit_square_at(x: f64, y: f64, s: f64) -> Geometry {
        g2(poly([
            [x, y],
            [x + s, y],
            [x + s, y + s],
            [x, y + s],
            [x, y],
        ]))
    }

    fn g2(g: Euclidean2DGeometry) -> Geometry {
        Geometry::Euclidean2D(g)
    }

    #[test]
    fn point_pairs() {
        let p = |x, y| g2(Euclidean2DGeometry::Point(Point2D::new(e(), [x, y])));
        assert_eq!(intersects(&p(1.0, 1.0), &p(1.0, 1.0)), Ok(true));
        assert_eq!(intersects(&p(1.0, 1.0), &p(1.0, 2.0)), Ok(false));
    }

    #[test]
    fn line_line_crossing_touching_disjoint() {
        let l = |coords: Vec<[f64; 2]>| {
            g2(Euclidean2DGeometry::LineString(LineString2D::from_coords(
                e(),
                coords,
            )))
        };
        let cross_a = l(vec![[0.0, 0.0], [2.0, 2.0]]);
        let cross_b = l(vec![[0.0, 2.0], [2.0, 0.0]]);
        assert_eq!(intersects(&cross_a, &cross_b), Ok(true));
        // T-junction (improper) also intersects.
        let touch = l(vec![[1.0, 1.0], [1.0, 5.0]]);
        assert_eq!(intersects(&cross_a, &touch), Ok(true));
        let disjoint = l(vec![[5.0, 5.0], [6.0, 5.0]]);
        assert_eq!(intersects(&cross_a, &disjoint), Ok(false));
    }

    #[test]
    fn polygon_pairs_cover_touch_contain_hole() {
        let big = unit_square_at(0.0, 0.0, 8.0);
        let inside = unit_square_at(1.0, 1.0, 2.0);
        let overlapping = unit_square_at(6.0, 6.0, 4.0);
        let touching = unit_square_at(8.0, 0.0, 2.0);
        let disjoint = unit_square_at(20.0, 0.0, 1.0);
        assert_eq!(intersects(&big, &inside), Ok(true)); // containment, no crossing
        assert_eq!(intersects(&inside, &big), Ok(true));
        assert_eq!(intersects(&big, &overlapping), Ok(true));
        assert_eq!(intersects(&big, &touching), Ok(true)); // shared edge only
        assert_eq!(intersects(&big, &disjoint), Ok(false));

        // A polygon sitting entirely in the other's hole does not intersect.
        let outer = [[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0], [0.0, 0.0]];
        let hole = vec![[2.0, 2.0], [2.0, 6.0], [6.0, 6.0], [6.0, 2.0], [2.0, 2.0]];
        let with_hole = g2(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(e(), outer, vec![hole]),
        )));
        let in_hole = unit_square_at(3.0, 3.0, 2.0);
        assert_eq!(intersects(&with_hole, &in_hole), Ok(false));
        // But one crossing the hole ring does.
        let across = unit_square_at(1.0, 3.0, 2.0);
        assert_eq!(intersects(&with_hole, &across), Ok(true));
    }

    #[test]
    fn line_polygon_inside_crossing_disjoint() {
        let square = unit_square_at(0.0, 0.0, 4.0);
        let l = |coords: Vec<[f64; 2]>| {
            g2(Euclidean2DGeometry::LineString(LineString2D::from_coords(
                e(),
                coords,
            )))
        };
        assert_eq!(
            intersects(&square, &l(vec![[1.0, 1.0], [2.0, 2.0]])),
            Ok(true)
        );
        assert_eq!(
            intersects(&square, &l(vec![[-1.0, 2.0], [5.0, 2.0]])),
            Ok(true)
        );
        assert_eq!(
            intersects(&square, &l(vec![[5.0, 5.0], [6.0, 6.0]])),
            Ok(false)
        );
    }

    #[test]
    fn collections_intersect_through_any_member() {
        let c = g2(Euclidean2DGeometry::Collection(Collection2D::new([
            Euclidean2DGeometry::Point(Point2D::new(e(), [10.0, 10.0])),
            Euclidean2DGeometry::Point(Point2D::new(e(), [1.0, 1.0])),
        ])));
        let square = unit_square_at(0.0, 0.0, 2.0);
        assert_eq!(intersects(&c, &square), Ok(true));

        let gc = Geometry::GeometryCollection(GeometryCollection::new([square.clone()]));
        assert_eq!(intersects(&gc, &c), Ok(true));
    }

    #[test]
    fn none_intersects_nothing() {
        let square = unit_square_at(0.0, 0.0, 2.0);
        assert_eq!(intersects(&Geometry::None, &square), Ok(false));
        assert_eq!(intersects(&Geometry::None, &Geometry::None), Ok(false));
    }

    #[test]
    fn mixed_frames_error() {
        let a = g2(Euclidean2DGeometry::Point(Point2D::new(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            [0.0, 0.0],
        )));
        let b = g2(Euclidean2DGeometry::Point(Point2D::new(e(), [0.0, 0.0])));
        assert_eq!(intersects(&a, &b), Err(PredicateError::MixedFrames));
    }

    #[test]
    fn cross_dimension_and_3d_errors() {
        let a = unit_square_at(0.0, 0.0, 1.0);
        let b3 = Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
            e(),
            [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
        )));
        assert_eq!(intersects(&a, &b3), Err(PredicateError::CrossDimension));
        assert!(matches!(
            intersects(&b3, &b3),
            Err(PredicateError::UnsupportedPair { .. })
        ));
    }

    #[test]
    fn bbox_reject_short_circuits() {
        // Far-apart geometries with expensive shapes still answer quickly and
        // correctly through the box reject.
        let a = unit_square_at(0.0, 0.0, 1.0);
        let b = unit_square_at(1000.0, 1000.0, 1.0);
        assert_eq!(intersects(&a, &b), Ok(false));
    }
}
