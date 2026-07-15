//! DE-9IM relates for 3D geometry through planar projection.
//!
//! There is no volumetric DE-9IM (plan §7); what 3D offers beyond the exact
//! [`intersects`](super::intersects()) tests are two projections into the 2D
//! [`relate`](super::relate()) machinery:
//!
//! - [`relate_coplanar`] — for **mutually coplanar** 3D geometries (a shared
//!   wall, a floor and its furniture footprints): verifies coplanarity
//!   exactly, projects both operands onto the shared plane, and relates
//!   in-plane. Non-coplanar operands are a
//!   [`NotCoplanar`](super::PredicateError::NotCoplanar) error.
//! - [`relate_xy`] — the legacy-compatible SpatialFilter semantics: every
//!   leaf, 2D or 3D, is taken by its `(x, y)` footprint and related in 2D.
//!   This is the explicit XY-projection opt-in; nothing else in the
//!   predicates crosses dimensions.
//!
//! Both project by **dropping a coordinate axis**, not by rotating onto a
//! tangent plane: an axis drop copies coordinates verbatim, so the 2D relate's
//! robust predicates see exact inputs and the matrix is exact for the 3D
//! configuration. (`relate_coplanar` picks an axis along which the shared
//! plane does not collapse — one always exists — and DE-9IM is invariant
//! under a linear bijection of the plane, so the skew of an axis drop is
//! harmless.) The projected operands are materialized as owned 2D leaves per
//! call.
//!
//! Because a projection may mirror orientation, mesh faces are re-normalized
//! to the validated 2D winding (exterior counter-clockwise) before the relate
//! — the mesh dissolve depends on it. Caveats carried over from `relate`:
//! leaves whose *projections* overlap areally (e.g. the faces of a closed
//! shell under `relate_xy`) are the documented unsupported relate input, and
//! a face that projects to zero area degrades like any other degenerate ring.
//! `Solid`, `Csg`, and `PointCloud` leaves are
//! [`Unsupported`](super::PredicateError::Unsupported).

use crate::collection::Collection2D;
use crate::line_string::LineString2D;
use crate::point::Point2D;
use crate::polygon::Polygon2D;
use crate::polygon_mesh::{PolygonMesh2D, PolygonMesh3D};
use crate::triangular_mesh::TriangularMesh2D;
use crate::{Euclidean2DGeometry, Geometry};

use super::kernel::{orient2d, Orientation};
use super::kernel3d::{collinear_3d, coplanar, drop_axis};
use super::relate::{relate_2d, IntersectionMatrix};
use super::view::{flatten_2d, Leaf2D};
use super::view3d::{flatten_3d, flatten_geometry_3d, Leaf3D};
use super::{PredicateError, Result};

/// The DE-9IM matrix of two mutually coplanar 3D geometries, related in their
/// shared plane.
///
/// Coplanarity is verified exactly (every vertex against a reference plane
/// through the first three non-collinear vertices);
/// [`NotCoplanar`](PredicateError::NotCoplanar) otherwise. Operands must both
/// be 3D ([`CrossDimension`](PredicateError::CrossDimension) if a 2D leaf
/// appears) and share one frame; `Point`, `LineString`, `Polygon`, and the
/// meshes are supported, while `Solid` (never planar), `Csg`, and
/// `PointCloud` are [`Unsupported`](PredicateError::Unsupported).
/// `Geometry::None` and empty collections relate as the empty geometry.
pub fn relate_coplanar(a: &Geometry, b: &Geometry) -> Result<IntersectionMatrix> {
    let a_leaves = planar_leaves(a)?;
    let b_leaves = planar_leaves(b)?;
    require_common_frame(&a_leaves, &b_leaves)?;

    let axis = shared_plane_axis(a_leaves.iter().chain(&b_leaves))?;
    relate_2d(&project(&a_leaves, axis), &project(&b_leaves, axis))
}

/// The DE-9IM matrix of the `(x, y)` footprints of two geometries — the
/// explicit XY-projection opt-in matching legacy SpatialFilter 3D semantics.
///
/// Each operand may be 2D, 3D, or a mixed collection: 2D leaves are taken
/// as-is (their optional elevation ignored, as everywhere), 3D leaves drop
/// `z`. All leaves must share one frame. `Solid`, `Csg`, and `PointCloud`
/// leaves are [`Unsupported`](PredicateError::Unsupported); note that a 3D
/// mesh whose faces overlap in `(x, y)` (any closed surface) is the
/// documented unsupported input of [`relate`](super::relate()) itself.
pub fn relate_xy(a: &Geometry, b: &Geometry) -> Result<IntersectionMatrix> {
    let a_members = xy_members(a)?;
    let b_members = xy_members(b)?;
    relate_2d(
        &Euclidean2DGeometry::Collection(Collection2D::new(a_members)),
        &Euclidean2DGeometry::Collection(Collection2D::new(b_members)),
    )
}

// --- operand gathering ---------------------------------------------------------

/// The 3D leaves of a planar-relate operand, with the kinds the projection
/// cannot take rejected.
fn planar_leaves(geometry: &Geometry) -> Result<Vec<Leaf3D<'_>>> {
    let (leaves, saw_2d, unsupported) = flatten_geometry_3d(geometry);
    if let Some(name) = unsupported {
        return Err(PredicateError::Unsupported { geometry: name });
    }
    if saw_2d {
        return Err(PredicateError::CrossDimension);
    }
    if leaves.iter().any(|l| matches!(l, Leaf3D::Solid(_))) {
        return Err(PredicateError::Unsupported { geometry: "Solid" });
    }
    Ok(leaves)
}

/// The projected-or-cloned 2D members of an XY-relate operand.
fn xy_members(geometry: &Geometry) -> Result<Vec<Euclidean2DGeometry>> {
    fn walk<'a>(
        geometry: &'a Geometry,
        two_d: &mut Vec<Leaf2D<'a>>,
        three_d: &mut Vec<Leaf3D<'a>>,
        unsupported: &mut Option<&'static str>,
    ) {
        match geometry {
            Geometry::None => {}
            Geometry::Euclidean2D(g) => flatten_2d(g, two_d),
            Geometry::Euclidean3D(g) => flatten_3d(g, three_d, unsupported),
            Geometry::GeometryCollection(c) => {
                for member in c.members() {
                    walk(member, two_d, three_d, unsupported);
                }
            }
        }
    }
    let mut two_d = Vec::new();
    let mut three_d = Vec::new();
    let mut unsupported = None;
    walk(geometry, &mut two_d, &mut three_d, &mut unsupported);
    if let Some(name) = unsupported {
        return Err(PredicateError::Unsupported { geometry: name });
    }
    if three_d.iter().any(|l| matches!(l, Leaf3D::Solid(_))) {
        return Err(PredicateError::Unsupported { geometry: "Solid" });
    }
    Ok(two_d
        .iter()
        .map(owned_2d)
        .chain(three_d.iter().map(|l| project_leaf(l, 2)))
        .collect())
}

/// Clone a 2D leaf into an owned member.
fn owned_2d(leaf: &Leaf2D<'_>) -> Euclidean2DGeometry {
    match leaf {
        Leaf2D::Point(p) => Euclidean2DGeometry::Point((*p).clone()),
        Leaf2D::Line(l) => Euclidean2DGeometry::LineString((*l).clone()),
        Leaf2D::Polygon(p) => Euclidean2DGeometry::Polygon(Box::new((*p).clone())),
        Leaf2D::PolygonMesh(m) => Euclidean2DGeometry::PolygonMesh(Box::new((*m).clone())),
        Leaf2D::TriangularMesh(m) => Euclidean2DGeometry::TriangularMesh(Box::new((*m).clone())),
    }
}

/// Require every leaf across both operands to share one coordinate frame.
fn require_common_frame(a: &[Leaf3D<'_>], b: &[Leaf3D<'_>]) -> Result<()> {
    super::view3d::require_common_frame_3d(a, b)
}

// --- the shared plane ----------------------------------------------------------

/// Verify all leaves' vertices lie in one plane (exactly) and return the axis
/// to drop: one along which that plane — or line, or point — does not
/// collapse, so the projection is injective on it.
fn shared_plane_axis<'a, 'b>(leaves: impl Iterator<Item = &'b Leaf3D<'a>> + Clone) -> Result<usize>
where
    'a: 'b,
{
    // The first vertex, the first distinct vertex, and the first vertex not
    // collinear with them span the affine hull (up to a plane).
    let mut first = None;
    let mut second = None;
    let mut triple = None;
    for leaf in leaves.clone() {
        for v in leaf_coords(leaf) {
            match (first, second) {
                (None, _) => first = Some(v),
                (Some(a), None) if v != a => second = Some(v),
                (Some(a), Some(b)) => {
                    if !collinear_3d(a, b, v) {
                        triple = Some([a, b, v]);
                        break;
                    }
                }
                _ => {}
            }
        }
        if triple.is_some() {
            break;
        }
    }

    match (first, second, triple) {
        // A genuine plane: every vertex must lie in it, exactly.
        (_, _, Some([a, b, c])) => {
            for leaf in leaves {
                for v in leaf_coords(leaf) {
                    if !coplanar(a, b, c, v) {
                        return Err(PredicateError::NotCoplanar);
                    }
                }
            }
            for axis in [2, 1, 0] {
                if orient2d(drop_axis(a, axis), drop_axis(b, axis), drop_axis(c, axis))
                    != Orientation::Collinear
                {
                    return Ok(axis);
                }
            }
            unreachable!("a non-collinear triple projects non-degenerately along some axis")
        }
        // All vertices collinear: coplanar trivially; keep the line's
        // direction visible.
        (Some(a), Some(b), None) => {
            for axis in [2, 1, 0] {
                if drop_axis(a, axis) != drop_axis(b, axis) {
                    return Ok(axis);
                }
            }
            unreachable!("distinct points project distinctly along some axis")
        }
        // At most one distinct vertex: any axis will do.
        _ => Ok(2),
    }
}

/// All vertices of a leaf (mesh leaves iterate their pools).
fn leaf_coords<'a>(leaf: &Leaf3D<'a>) -> Box<dyn Iterator<Item = [f64; 3]> + 'a> {
    match leaf {
        Leaf3D::Point(p) => Box::new(core::iter::once(p.position())),
        Leaf3D::Line(l) => Box::new(l.coords().iter().copied()),
        Leaf3D::Polygon(p) => {
            Box::new(super::view::polygon3d_rings(p).flat_map(|ring| ring.iter().copied()))
        }
        Leaf3D::PolygonMesh(m) => Box::new(m.vertices().iter().copied()),
        Leaf3D::TriangularMesh(m) => Box::new(m.vertices().iter().copied()),
        Leaf3D::Solid(_) => unreachable!("solids are rejected before projection"),
    }
}

// --- projection ----------------------------------------------------------------

/// Project leaves along `axis` into one owned 2D collection.
fn project(leaves: &[Leaf3D<'_>], axis: usize) -> Euclidean2DGeometry {
    Euclidean2DGeometry::Collection(Collection2D::new(
        leaves.iter().map(|l| project_leaf(l, axis)),
    ))
}

/// Project one leaf along `axis`, re-normalizing mesh face winding (a
/// projection may mirror, and the relate's mesh dissolve assumes the
/// validated exterior-counter-clockwise winding).
fn project_leaf(leaf: &Leaf3D<'_>, axis: usize) -> Euclidean2DGeometry {
    let drop = |p: [f64; 3]| drop_axis(p, axis);
    match leaf {
        Leaf3D::Point(p) => {
            Euclidean2DGeometry::Point(Point2D::new(p.frame().clone(), drop(p.position())))
        }
        Leaf3D::Line(l) => Euclidean2DGeometry::LineString(LineString2D::from_coords(
            l.frame().clone(),
            l.coords().iter().copied().map(drop),
        )),
        Leaf3D::Polygon(p) => Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            p.frame().clone(),
            p.exterior().iter().copied().map(drop),
            p.interiors().map(|ring| ring.iter().copied().map(drop)),
        ))),
        Leaf3D::PolygonMesh(m) => {
            Euclidean2DGeometry::PolygonMesh(Box::new(project_polygon_mesh(m, axis)))
        }
        Leaf3D::TriangularMesh(m) => {
            let vertices: Vec<[f64; 2]> = m.vertices().iter().copied().map(drop).collect();
            let indices = m.triangles().flat_map(|[a, b, c]| {
                // Normalize each triangle to counter-clockwise.
                let tri = [
                    vertices[a as usize],
                    vertices[b as usize],
                    vertices[c as usize],
                ];
                if orient2d(tri[0], tri[1], tri[2]) == Orientation::Clockwise {
                    [a, c, b]
                } else {
                    [a, b, c]
                }
            });
            let mesh = TriangularMesh2D::from_parts(m.frame().clone(), vertices.clone(), indices)
                .expect("projected indices stay valid");
            Euclidean2DGeometry::TriangularMesh(Box::new(mesh))
        }
        Leaf3D::Solid(_) => unreachable!("solids are rejected before projection"),
    }
}

/// Project a polygon mesh, reversing the rings of any face whose projected
/// exterior comes out clockwise.
fn project_polygon_mesh(mesh: &PolygonMesh3D, axis: usize) -> PolygonMesh2D {
    let (face_indices, face_offsets, interior_offsets) = mesh.data().csr_buffers();
    let mut indices: Vec<u32> = face_indices.iter_u32().map(|[i]| i).collect();
    let offsets: Vec<u32> = face_offsets.iter_u32().map(|[o]| o).collect();
    let holes: Vec<u32> = interior_offsets.iter_u32().map(|[o]| o).collect();
    let vertices: Vec<[f64; 2]> = mesh
        .vertices()
        .iter()
        .map(|&p| drop_axis(p, axis))
        .collect();

    let n = indices.len();
    if n != 0 {
        let n_faces = offsets.len() + 1;
        let mut start = 0usize;
        for fi in 0..n_faces {
            let end = offsets.get(fi).map_or(n, |&o| o as usize);
            // This face's ring boundaries: start, its holes, end.
            let mut bounds: Vec<usize> = vec![start];
            bounds.extend(
                holes
                    .iter()
                    .map(|&h| h as usize)
                    .filter(|&h| h > start && h < end),
            );
            bounds.push(end);

            let exterior: Vec<[f64; 2]> = indices[bounds[0]..bounds[1]]
                .iter()
                .map(|&i| vertices[i as usize])
                .collect();
            if ring_orientation(&exterior) == Orientation::Clockwise {
                for w in bounds.windows(2) {
                    indices[w[0]..w[1]].reverse();
                }
            }
            start = end;
        }
    }

    PolygonMesh2D::from_raw_parts(mesh.frame().clone(), vertices, indices, offsets, holes)
        .expect("projected CSR stays valid")
}

/// The winding of a stored ring (closing duplicate tolerated): the robust
/// orientation at the lexicographically extreme vertex, falling back to the
/// shoelace sign when that corner is degenerate. `Collinear` for a
/// zero-area ring.
fn ring_orientation(ring: &[[f64; 2]]) -> Orientation {
    let open = if ring.len() >= 2 && ring.first() == ring.last() {
        &ring[..ring.len() - 1]
    } else {
        ring
    };
    let n = open.len();
    if n < 3 {
        return Orientation::Collinear;
    }
    let lex = |p: [f64; 2], q: [f64; 2]| (p[1], p[0]) < (q[1], q[0]);
    let mut min = 0;
    for i in 1..n {
        if lex(open[i], open[min]) {
            min = i;
        }
    }
    let distinct = |from: usize, step: usize| {
        (1..n)
            .map(|k| open[(from + k * step) % n])
            .find(|&p| p != open[min])
    };
    let (Some(next), Some(prev)) = (distinct(min, 1), distinct(min, n - 1)) else {
        return Orientation::Collinear;
    };
    match orient2d(prev, open[min], next) {
        Orientation::Collinear => shoelace_orientation(open),
        o => o,
    }
}

/// The shoelace-sign orientation (not robust; only the fallback for a
/// degenerate extreme corner).
fn shoelace_orientation(open: &[[f64; 2]]) -> Orientation {
    let mut sum = 0.0;
    for i in 0..open.len() {
        let p = open[i];
        let q = open[(i + 1) % open.len()];
        sum += (q[0] - p[0]) * (q[1] + p[1]);
    }
    if sum < 0.0 {
        Orientation::CounterClockwise
    } else if sum > 0.0 {
        Orientation::Clockwise
    } else {
        Orientation::Collinear
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::line_string::LineString3D;
    use crate::point::{Point2D, Point3D};
    use crate::polygon::Polygon3D;
    use crate::polygon_mesh::PolygonMesh3D;
    use crate::predicates::relate::relate;
    use crate::predicates::test3d::{box_solid, e, g3, solid_geometry, Rng};
    use crate::triangular_mesh::TriangularMesh3D;
    use pretty_assertions::assert_eq;

    fn g2(g: Euclidean2DGeometry) -> Geometry {
        Geometry::Euclidean2D(g)
    }

    /// An exactly-representable lift of the XY plane onto `z = px*x + py*y + pz`
    /// (small integer coefficients over integer coordinates stay exact).
    fn lift(plane: [f64; 3]) -> impl Fn([f64; 2]) -> [f64; 3] {
        move |[x, y]| [x, y, plane[0] * x + plane[1] * y + plane[2]]
    }

    fn poly_2d(ring: Vec<[f64; 2]>, holes: Vec<Vec<[f64; 2]>>) -> Euclidean2DGeometry {
        Euclidean2DGeometry::Polygon(Box::new(crate::polygon::Polygon2D::from_rings(
            e(),
            ring,
            holes,
        )))
    }

    fn poly_3d(
        ring: Vec<[f64; 2]>,
        holes: Vec<Vec<[f64; 2]>>,
        up: &impl Fn([f64; 2]) -> [f64; 3],
    ) -> Euclidean3DGeometry {
        Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
            e(),
            ring.into_iter().map(up),
            holes
                .into_iter()
                .map(|h| h.into_iter().map(up).collect::<Vec<_>>()),
        )))
    }

    fn square(x: f64, y: f64, s: f64) -> Vec<[f64; 2]> {
        vec![[x, y], [x + s, y], [x + s, y + s], [x, y + s], [x, y]]
    }

    use crate::Euclidean3DGeometry;

    #[test]
    fn coplanar_relate_matches_the_2d_relate_exactly() {
        let mut rng = Rng(20260717);
        // Planes with negative coefficients also exercise mirrored
        // projections.
        let planes = [
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 5.0],
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 3.0],
            [2.0, -3.0, 1.0],
            [-1.0, -1.0, -4.0],
        ];
        for case in 0..300 {
            let plane = planes[case % planes.len()];
            let up = lift(plane);
            let (a2, a3) = random_member(&mut rng, &up);
            let (b2, b3) = random_member(&mut rng, &up);
            let expected = relate(&g2(a2), &g2(b2)).unwrap();
            let actual = relate_coplanar(&g3(a3), &g3(b3)).unwrap();
            assert_eq!(actual, expected, "case {case} on plane {plane:?}");
        }
    }

    /// One random operand as matching 2D and lifted-3D geometry: a point, a
    /// line, or a polygon (with an occasional hole).
    fn random_member(
        rng: &mut Rng,
        up: &impl Fn([f64; 2]) -> [f64; 3],
    ) -> (Euclidean2DGeometry, Euclidean3DGeometry) {
        let p = |rng: &mut Rng| [rng.int(0, 8) as f64, rng.int(0, 8) as f64];
        match rng.int(0, 3) {
            0 => {
                let pos = p(rng);
                (
                    Euclidean2DGeometry::Point(Point2D::new(e(), pos)),
                    Euclidean3DGeometry::Point(Point3D::new(e(), up(pos))),
                )
            }
            1 => {
                let coords: Vec<[f64; 2]> = (0..rng.int(2, 4)).map(|_| p(rng)).collect();
                (
                    Euclidean2DGeometry::LineString(crate::line_string::LineString2D::from_coords(
                        e(),
                        coords.clone(),
                    )),
                    Euclidean3DGeometry::LineString(LineString3D::from_coords(
                        e(),
                        coords.into_iter().map(up),
                    )),
                )
            }
            _ => {
                let (x, y, s) = (
                    rng.int(0, 5) as f64,
                    rng.int(0, 5) as f64,
                    rng.int(2, 4) as f64,
                );
                let holes = if s >= 3.0 && rng.int(0, 1) == 1 {
                    // A hole wound opposite to the exterior.
                    vec![vec![
                        [x + 1.0, y + 1.0],
                        [x + 1.0, y + 2.0],
                        [x + 2.0, y + 2.0],
                        [x + 2.0, y + 1.0],
                        [x + 1.0, y + 1.0],
                    ]]
                } else {
                    Vec::new()
                };
                (
                    poly_2d(square(x, y, s), holes.clone()),
                    poly_3d(square(x, y, s), holes, up),
                )
            }
        }
    }

    #[test]
    fn meshes_relate_in_a_mirrored_plane() {
        // Two quads sharing an edge in the plane z = 5 - x - y, whose
        // axis-drop projection mirrors: the dissolve must still see the shared
        // edge as interior.
        let up = lift([-1.0, -1.0, 5.0]);
        // Faces wound CW in (x, y) so their 3D canonical normals face away
        // from +z; the projected mesh must be re-normalized per face.
        let quad = |x0: f64| {
            vec![
                [x0, 0.0],
                [x0, 2.0],
                [x0 + 2.0, 2.0],
                [x0 + 2.0, 0.0],
                [x0, 0.0],
            ]
        };
        let faces = [
            Polygon3D::from_rings(
                e(),
                quad(0.0).into_iter().map(&up),
                Vec::<Vec<[f64; 3]>>::new(),
            ),
            Polygon3D::from_rings(
                e(),
                quad(2.0).into_iter().map(&up),
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        ];
        let mesh = PolygonMesh3D::from_polygons(e(), faces.iter()).unwrap();
        let mesh = g3(Euclidean3DGeometry::PolygonMesh(Box::new(mesh)));

        // A segment crossing the shared edge x = 2 strictly inside the union:
        // it must relate as interior-interior with no boundary contact.
        let probe = g3(Euclidean3DGeometry::LineString(LineString3D::from_coords(
            e(),
            [up([1.0, 1.0]), up([3.0, 1.0])],
        )));
        let matrix = relate_coplanar(&probe, &mesh).unwrap();
        assert!(matrix.is_within());
        // And the equivalent triangular mesh agrees.
        let tri_mesh = TriangularMesh3D::from_parts(
            e(),
            vec![
                up([0.0, 0.0]),
                up([2.0, 0.0]),
                up([2.0, 2.0]),
                up([0.0, 2.0]),
                up([4.0, 0.0]),
                up([4.0, 2.0]),
            ],
            // Mixed windings on purpose.
            [0u32, 1, 2, 2, 3, 0, 1, 4, 5, 1, 5, 2],
        )
        .unwrap();
        let tri_mesh = g3(Euclidean3DGeometry::TriangularMesh(Box::new(tri_mesh)));
        let matrix = relate_coplanar(&probe, &tri_mesh).unwrap();
        assert!(matrix.is_within());
    }

    #[test]
    fn vertical_plane_picks_a_usable_axis() {
        // Two squares in the plane x = 5 sharing the edge y = 2.
        let sq = |y0: f64| {
            Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
                e(),
                [
                    [5.0, y0, 0.0],
                    [5.0, y0 + 2.0, 0.0],
                    [5.0, y0 + 2.0, 2.0],
                    [5.0, y0, 2.0],
                    [5.0, y0, 0.0],
                ],
                Vec::<Vec<[f64; 3]>>::new(),
            )))
        };
        let matrix = relate_coplanar(&g3(sq(0.0)), &g3(sq(2.0))).unwrap();
        assert!(matrix.is_touches());
    }

    #[test]
    fn collinear_and_degenerate_hulls_are_coplanar() {
        let line = |a: [f64; 3], b: [f64; 3]| {
            g3(Euclidean3DGeometry::LineString(LineString3D::from_coords(
                e(),
                [a, b],
            )))
        };
        // Two overlapping collinear segments along a skew 3D line.
        let m = relate_coplanar(
            &line([0.0, 0.0, 0.0], [4.0, 4.0, 4.0]),
            &line([2.0, 2.0, 2.0], [6.0, 6.0, 6.0]),
        )
        .unwrap();
        assert!(m.is_overlaps());
        // A vertical line (the z axis) must not collapse under projection.
        let m = relate_coplanar(
            &line([0.0, 0.0, 0.0], [0.0, 0.0, 4.0]),
            &line([0.0, 0.0, 2.0], [0.0, 0.0, 6.0]),
        )
        .unwrap();
        assert!(m.is_overlaps());
        // Coincident points.
        let p = g3(Euclidean3DGeometry::Point(Point3D::new(e(), [1.0; 3])));
        assert!(relate_coplanar(&p, &p).unwrap().is_equal_topo());
    }

    #[test]
    fn not_coplanar_and_unsupported_errors() {
        let up0 = lift([0.0, 0.0, 0.0]);
        let up1 = lift([0.0, 0.0, 1.0]);
        let a = g3(poly_3d(square(0.0, 0.0, 4.0), Vec::new(), &up0));
        let b = g3(poly_3d(square(1.0, 1.0, 4.0), Vec::new(), &up1));
        assert_eq!(relate_coplanar(&a, &b), Err(PredicateError::NotCoplanar));
        // A non-planar operand on its own is caught too.
        let bent = g3(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                e(),
                [
                    [0.0, 0.0, 0.0],
                    [4.0, 0.0, 0.0],
                    [4.0, 4.0, 1.0],
                    [0.0, 4.0, 0.0],
                    [0.0, 0.0, 0.0],
                ],
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        )));
        assert_eq!(relate_coplanar(&bent, &a), Err(PredicateError::NotCoplanar));

        let solid = g3(solid_geometry(box_solid([0.0; 3], [1.0; 3])));
        assert_eq!(
            relate_coplanar(&solid, &a),
            Err(PredicateError::Unsupported { geometry: "Solid" })
        );
        assert_eq!(
            relate_xy(&solid, &a),
            Err(PredicateError::Unsupported { geometry: "Solid" })
        );
        // 2D × 3D is cross-dimension for the in-plane relate...
        let flat = g2(poly_2d(square(0.0, 0.0, 4.0), Vec::new()));
        assert_eq!(
            relate_coplanar(&flat, &a),
            Err(PredicateError::CrossDimension)
        );
        // ...but exactly what relate_xy is for.
        assert!(relate_xy(&flat, &a).unwrap().is_intersects());
    }

    #[test]
    fn relate_xy_matches_the_footprint_relate() {
        let mut rng = Rng(20260718);
        for case in 0..150 {
            // Independent lifts per operand: XY relate ignores z entirely.
            let up_a = lift([
                rng.int(-2, 2) as f64,
                rng.int(-2, 2) as f64,
                rng.int(-3, 3) as f64,
            ]);
            let up_b = lift([
                rng.int(-2, 2) as f64,
                rng.int(-2, 2) as f64,
                rng.int(-3, 3) as f64,
            ]);
            let (a2, a3) = random_member(&mut rng, &up_a);
            let (b2, b3) = random_member(&mut rng, &up_b);
            let expected = relate(&g2(a2.clone()), &g2(b2)).unwrap();
            assert_eq!(
                relate_xy(&g3(a3.clone()), &g3(b3.clone())).unwrap(),
                expected,
                "case {case}"
            );
            // Mixed 2D × 3D operands agree as well.
            assert_eq!(
                relate_xy(&g2(a2), &g3(b3)).unwrap(),
                expected,
                "case {case} mixed"
            );
        }
    }

    #[test]
    fn mixed_frames_error() {
        let a = g3(Euclidean3DGeometry::Point(Point3D::new(e(), [0.0; 3])));
        let b = g3(Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::Crs(crate::coordinate::EpsgCode::new(4979)),
            [0.0; 3],
        )));
        assert_eq!(relate_coplanar(&a, &b), Err(PredicateError::MixedFrames));
        assert_eq!(relate_xy(&a, &b), Err(PredicateError::MixedFrames));
    }
}
