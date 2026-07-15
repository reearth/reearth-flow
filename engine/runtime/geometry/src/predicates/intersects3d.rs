//! The `intersects` predicate over 3D × 3D pairs.
//!
//! The 3D continuation of [`intersects`](super::intersects()): whether two 3D
//! geometries share at least one point, with boundary contact counting.
//! Surface-bearing leaves (`Polygon3D`, the meshes, `Solid` shells) are
//! re-expressed as triangle sets (see [`view3d`](super::view3d)) and tested
//! with the **exact** [`kernel3d`](super::kernel3d) primitives; candidate
//! triangle pairs come from an rstar tree over per-triangle bounding boxes, so
//! mesh × mesh work scales with the contact region rather than the pair
//! product.
//!
//! A `Solid` is volumetric: a geometry with no shell contact still intersects
//! it when it lies inside (decided by the exact ray parity of
//! [`position3d`](super::position3d), one representative vertex per connected
//! component — with no boundary contact a component is entirely on one side).
//! Every other pair is a surface/curve point-set test. `Csg` and `PointCloud`
//! leaves are [`Unsupported`](super::PredicateError::Unsupported).

use rstar::{RTree, RTreeObject, AABB};

use crate::ops::triangulation::Cache;
use crate::ops::{Aabb, BoundingBox};
use crate::solid::Solid;
use crate::Euclidean3DGeometry;

use super::kernel::CoordPos;
use super::kernel3d::{
    point_in_triangle_3d, segment_intersects_triangle_3d, segments_intersect_3d,
    triangles_intersect_3d,
};
use super::position3d::{on_chain_3d, solid_position_sets};
use super::view3d::{flatten_3d, require_common_frame_3d, Leaf3D, TriangleSet};
use super::{PredicateError, Result};

/// [`intersects`](super::intersects()) over two 3D geometries.
pub fn intersects_3d(a: &Euclidean3DGeometry, b: &Euclidean3DGeometry) -> Result<bool> {
    let (a_leaves, b_leaves) = flatten_3d_pair(a, b)?;
    require_common_frame_3d(&a_leaves, &b_leaves)?;

    let mut cache = Cache::new();
    let a_prepared: Vec<Prepared3D<'_>> = a_leaves
        .iter()
        .map(|l| Prepared3D::new(*l, &mut cache))
        .collect();
    let b_prepared: Vec<Prepared3D<'_>> = b_leaves
        .iter()
        .map(|l| Prepared3D::new(*l, &mut cache))
        .collect();

    for pa in &a_prepared {
        for pb in &b_prepared {
            if leaf_intersects(pa, pb) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Flatten both operands of a 3D binary operation into their leaves, erring on
/// a leaf kind the 3D operations do not support.
pub(crate) fn flatten_3d_pair<'a>(
    a: &'a Euclidean3DGeometry,
    b: &'a Euclidean3DGeometry,
) -> Result<(Vec<Leaf3D<'a>>, Vec<Leaf3D<'a>>)> {
    let mut unsupported = None;
    let mut a_leaves = Vec::new();
    flatten_3d(a, &mut a_leaves, &mut unsupported);
    let mut b_leaves = Vec::new();
    flatten_3d(b, &mut b_leaves, &mut unsupported);
    match unsupported {
        Some(name) => Err(PredicateError::Unsupported { geometry: name }),
        None => Ok((a_leaves, b_leaves)),
    }
}

// --- prepared operands --------------------------------------------------------

/// One triangle's index and precomputed rstar envelope.
struct TriObj {
    idx: u32,
    envelope: AABB<[f64; 3]>,
}

impl RTreeObject for TriObj {
    type Envelope = AABB<[f64; 3]>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

/// A triangle set with an rstar tree over its triangles' bounding boxes.
struct Surface<'a> {
    set: TriangleSet<'a>,
    tree: RTree<TriObj>,
}

impl<'a> Surface<'a> {
    fn new(set: TriangleSet<'a>) -> Self {
        let objs = (0..set.len())
            .map(|i| {
                let t = set.triangle(i);
                let mut min = t[0];
                let mut max = t[0];
                for p in &t[1..] {
                    for k in 0..3 {
                        min[k] = min[k].min(p[k]);
                        max[k] = max[k].max(p[k]);
                    }
                }
                TriObj {
                    idx: i as u32,
                    envelope: AABB::from_corners(min, max),
                }
            })
            .collect();
        Surface {
            set,
            tree: RTree::bulk_load(objs),
        }
    }

    /// Whether the coordinate lies on any triangle.
    fn contains_point(&self, p: [f64; 3]) -> bool {
        self.tree
            .locate_in_envelope_intersecting(&AABB::from_point(p))
            .any(|obj| point_in_triangle_3d(p, self.set.triangle(obj.idx as usize)))
    }

    /// Whether the closed segment shares a point with any triangle.
    fn meets_segment(&self, a: [f64; 3], b: [f64; 3]) -> bool {
        let mut min = a;
        let mut max = a;
        for k in 0..3 {
            min[k] = min[k].min(b[k]);
            max[k] = max[k].max(b[k]);
        }
        self.tree
            .locate_in_envelope_intersecting(&AABB::from_corners(min, max))
            .any(|obj| segment_intersects_triangle_3d(a, b, self.set.triangle(obj.idx as usize)))
    }

    /// Whether any triangle of `self` shares a point with any of `other`,
    /// through a tree × tree candidate traversal.
    fn meets_surface(&self, other: &Surface<'_>) -> bool {
        self.tree
            .intersection_candidates_with_other_tree(&other.tree)
            .any(|(oa, ob)| {
                triangles_intersect_3d(
                    self.set.triangle(oa.idx as usize),
                    other.set.triangle(ob.idx as usize),
                )
            })
    }

    /// One vertex per connected component (triangles connected through shared
    /// pool indices). With no boundary contact against another operand, each
    /// component lies entirely on one side of it, so these decide containment.
    fn component_reps(&self) -> Vec<[f64; 3]> {
        let n = self.set.pool().len();
        let mut parent: Vec<u32> = (0..n as u32).collect();
        fn find(parent: &mut [u32], mut x: u32) -> u32 {
            while parent[x as usize] != x {
                parent[x as usize] = parent[parent[x as usize] as usize];
                x = parent[x as usize];
            }
            x
        }
        for i in 0..self.set.len() {
            let [a, b, c] = self.set.indices(i);
            let ra = find(&mut parent, a);
            let rb = find(&mut parent, b);
            parent[rb as usize] = ra;
            let rc = find(&mut parent, c);
            parent[rc as usize] = ra;
        }
        let mut seen: Vec<u32> = Vec::new();
        let mut reps: Vec<[f64; 3]> = Vec::new();
        for i in 0..self.set.len() {
            let root = find(&mut parent, self.set.indices(i)[0]);
            if !seen.contains(&root) {
                seen.push(root);
                reps.push(self.set.triangle(i)[0]);
            }
        }
        reps
    }
}

/// A leaf's point set in dispatch form.
enum Body<'a> {
    Point([f64; 3]),
    Line(&'a [[f64; 3]]),
    Surface(Surface<'a>),
    /// The shells, exterior first.
    Solid(Vec<Surface<'a>>),
}

/// One flattened leaf prepared for the pair tests: its body and bounding box
/// (`None` for an empty point set, which intersects nothing).
struct Prepared3D<'a> {
    body: Body<'a>,
    bbox: Option<Aabb>,
}

impl<'a> Prepared3D<'a> {
    fn new(leaf: Leaf3D<'a>, cache: &mut Cache) -> Self {
        let bbox = match leaf {
            Leaf3D::Point(p) => p.bounding_box(),
            Leaf3D::Line(l) => l.bounding_box(),
            Leaf3D::Polygon(p) => p.bounding_box(),
            Leaf3D::PolygonMesh(m) => m.bounding_box(),
            Leaf3D::TriangularMesh(m) => m.bounding_box(),
            Leaf3D::Solid(s) => s.bounding_box(),
        }
        .ok();
        let body = match leaf {
            Leaf3D::Point(p) => Body::Point(p.position()),
            Leaf3D::Line(l) => Body::Line(l.coords()),
            Leaf3D::Polygon(p) => Body::Surface(Surface::new(TriangleSet::from_polygon(p, cache))),
            Leaf3D::PolygonMesh(m) => Body::Surface(Surface::new(
                TriangleSet::from_polygon_mesh_data(m.data(), cache),
            )),
            Leaf3D::TriangularMesh(m) => {
                Body::Surface(Surface::new(TriangleSet::from_triangular_data(m.data())))
            }
            Leaf3D::Solid(s) => Body::Solid(shells(s, cache)),
        };
        // A surface whose faces are all degenerate re-represents as no
        // triangles: an empty point set, like an empty line.
        let empty = match &body {
            Body::Point(_) => false,
            Body::Line(l) => l.is_empty(),
            Body::Surface(s) => s.set.is_empty(),
            Body::Solid(shells) => shells[0].set.is_empty(),
        };
        Prepared3D {
            body,
            bbox: if empty { None } else { bbox },
        }
    }
}

/// The shells of a solid as surfaces, exterior first.
fn shells<'a>(solid: &'a Solid, cache: &mut Cache) -> Vec<Surface<'a>> {
    core::iter::once(solid.exterior())
        .chain(solid.interiors().iter())
        .map(|shell| Surface::new(TriangleSet::from_shell(shell, cache)))
        .collect()
}

/// Whether a coordinate lies in a solid's closed point set.
fn solid_has_point(shells: &[Surface<'_>], p: [f64; 3]) -> bool {
    solid_position_sets(p, &shells[0].set, shells[1..].iter().map(|s| &s.set)) != CoordPos::Outside
}

// --- pair dispatch -------------------------------------------------------------

/// Whether two prepared leaves intersect, after a bounding-box quick reject.
fn leaf_intersects(a: &Prepared3D<'_>, b: &Prepared3D<'_>) -> bool {
    let (Some(box_a), Some(box_b)) = (&a.bbox, &b.bbox) else {
        return false;
    };
    if !box_a.intersects(box_b) {
        return false;
    }
    match (&a.body, &b.body) {
        (Body::Point(p), _) => body_has_point(&b.body, *p),
        (_, Body::Point(p)) => body_has_point(&a.body, *p),
        (Body::Line(la), Body::Line(lb)) => line_vs_line(la, lb),
        (Body::Line(l), _) => line_vs_body(l, &b.body),
        (_, Body::Line(l)) => line_vs_body(l, &a.body),
        (Body::Surface(sa), Body::Surface(sb)) => sa.meets_surface(sb),
        (Body::Surface(s), Body::Solid(shells)) | (Body::Solid(shells), Body::Surface(s)) => {
            shells.iter().any(|shell| shell.meets_surface(s))
                || s.component_reps()
                    .into_iter()
                    .any(|p| solid_has_point(shells, p))
        }
        (Body::Solid(sa), Body::Solid(sb)) => {
            sa.iter().any(|x| sb.iter().any(|y| x.meets_surface(y)))
                || solid_has_point(sb, sa[0].set.triangle(0)[0])
                || solid_has_point(sa, sb[0].set.triangle(0)[0])
        }
    }
}

/// Whether the coordinate lies in the body's closed point set.
fn body_has_point(body: &Body<'_>, p: [f64; 3]) -> bool {
    match body {
        Body::Point(q) => p == *q,
        Body::Line(l) => on_chain_3d(p, l),
        Body::Surface(s) => s.contains_point(p),
        Body::Solid(shells) => solid_has_point(shells, p),
    }
}

fn line_vs_line(a: &[[f64; 3]], b: &[[f64; 3]]) -> bool {
    // A single-vertex chain is point-like.
    if a.len() == 1 {
        return on_chain_3d(a[0], b);
    }
    if b.len() == 1 {
        return on_chain_3d(b[0], a);
    }
    a.windows(2).any(|sa| {
        b.windows(2)
            .any(|sb| segments_intersect_3d(sa[0], sa[1], sb[0], sb[1]))
    })
}

/// Line versus a surface or solid body.
fn line_vs_body(coords: &[[f64; 3]], body: &Body<'_>) -> bool {
    if coords.len() == 1 {
        return body_has_point(body, coords[0]);
    }
    match body {
        Body::Surface(s) => coords.windows(2).any(|w| s.meets_segment(w[0], w[1])),
        Body::Solid(shells) => {
            // Any shell contact intersects; otherwise the chain never crosses
            // the boundary, so it lies entirely in one region: one vertex
            // decides.
            coords
                .windows(2)
                .any(|w| shells.iter().any(|s| s.meets_segment(w[0], w[1])))
                || coords.first().is_some_and(|&p| solid_has_point(shells, p))
        }
        Body::Point(_) | Body::Line(_) => unreachable!("handled by the dispatch above"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection3D;
    use crate::coordinate::{CoordinateFrame, EpsgCode};
    use crate::line_string::LineString3D;
    use crate::point::Point3D;
    use crate::polygon::Polygon3D;
    use crate::predicates::intersects;
    use crate::predicates::test3d::{box_solid, box_solid_with_void, e, g3, solid_geometry, Rng};
    use crate::triangular_mesh::TriangularMesh3D;
    use pretty_assertions::assert_eq;

    fn point(p: [f64; 3]) -> Euclidean3DGeometry {
        Euclidean3DGeometry::Point(Point3D::new(e(), p))
    }

    fn line(coords: impl IntoIterator<Item = [f64; 3]>) -> Euclidean3DGeometry {
        Euclidean3DGeometry::LineString(LineString3D::from_coords(e(), coords))
    }

    fn triangle(t: [[f64; 3]; 3]) -> Euclidean3DGeometry {
        Euclidean3DGeometry::TriangularMesh(Box::new(
            TriangularMesh3D::from_parts(e(), t.to_vec(), [0u32, 1, 2]).unwrap(),
        ))
    }

    #[test]
    fn point_line_and_surface_pairs() {
        let tri = triangle([[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 4.0, 0.0]]);
        assert_eq!(intersects_3d(&point([1.0, 1.0, 0.0]), &tri), Ok(true));
        assert_eq!(intersects_3d(&point([1.0, 1.0, 0.5]), &tri), Ok(false));
        assert_eq!(
            intersects_3d(&point([1.0, 1.0, 1.0]), &point([1.0, 1.0, 1.0])),
            Ok(true)
        );
        assert_eq!(
            intersects_3d(
                &point([2.0, 2.0, 2.0]),
                &line([[0.0, 0.0, 0.0], [4.0, 4.0, 4.0]])
            ),
            Ok(true)
        );
        // Skew lines miss; coplanar crossing lines meet.
        assert_eq!(
            intersects_3d(
                &line([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]),
                &line([[0.0, 1.0, 1.0], [0.0, -1.0, 1.0]])
            ),
            Ok(false)
        );
        assert_eq!(
            intersects_3d(
                &line([[0.0, 0.0, 0.0], [2.0, 2.0, 2.0]]),
                &line([[2.0, 0.0, 0.0], [0.0, 2.0, 2.0]])
            ),
            Ok(true)
        );
        // A line piercing a triangle, resting on it, and missing it.
        assert_eq!(
            intersects_3d(&line([[1.0, 1.0, -1.0], [1.0, 1.0, 1.0]]), &tri),
            Ok(true)
        );
        assert_eq!(
            intersects_3d(&line([[0.5, 0.5, 0.0], [1.5, 1.5, 0.0]]), &tri),
            Ok(true)
        );
        assert_eq!(
            intersects_3d(&line([[1.0, 1.0, 1.0], [1.0, 1.0, 2.0]]), &tri),
            Ok(false)
        );
        // Surface × surface: shared edge counts.
        let folded = triangle([[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, -4.0, 4.0]]);
        assert_eq!(intersects_3d(&tri, &folded), Ok(true));
    }

    #[test]
    fn solids_are_volumetric() {
        let solid = solid_geometry(box_solid([0.0; 3], [6.0, 6.0, 6.0]));
        // Strictly inside, without any shell contact.
        assert_eq!(intersects_3d(&point([3.0, 3.0, 3.0]), &solid), Ok(true));
        assert_eq!(
            intersects_3d(&line([[2.0, 2.0, 2.0], [4.0, 4.0, 4.0]]), &solid),
            Ok(true)
        );
        let inner_tri = triangle([[2.0, 2.0, 2.0], [4.0, 2.0, 2.0], [2.0, 4.0, 2.0]]);
        assert_eq!(intersects_3d(&inner_tri, &solid), Ok(true));
        let inner_solid = solid_geometry(box_solid([2.0; 3], [2.0, 2.0, 2.0]));
        assert_eq!(intersects_3d(&inner_solid, &solid), Ok(true));
        assert_eq!(intersects_3d(&solid, &inner_solid), Ok(true));
        // Crossing the shell.
        assert_eq!(
            intersects_3d(&line([[-1.0, 3.0, 3.0], [1.0, 3.0, 3.0]]), &solid),
            Ok(true)
        );
        // Entirely outside.
        assert_eq!(intersects_3d(&point([7.0, 3.0, 3.0]), &solid), Ok(false));
        assert_eq!(
            intersects_3d(
                &triangle([[7.0, 0.0, 0.0], [8.0, 0.0, 0.0], [7.0, 1.0, 0.0]]),
                &solid
            ),
            Ok(false)
        );
    }

    #[test]
    fn voids_are_outside_the_solid() {
        let hollow = solid_geometry(box_solid_with_void(
            [0.0; 3],
            [6.0, 6.0, 6.0],
            [2.0; 3],
            [2.0, 2.0, 2.0],
        ));
        // Geometry fully inside the hollow does not intersect.
        assert_eq!(intersects_3d(&point([3.0, 3.0, 3.0]), &hollow), Ok(false));
        let in_void = solid_geometry(box_solid([2.5; 3], [1.0, 1.0, 1.0]));
        assert_eq!(intersects_3d(&in_void, &hollow), Ok(false));
        assert_eq!(intersects_3d(&hollow, &in_void), Ok(false));
        // Touching the void shell intersects (it is the solid's boundary).
        assert_eq!(intersects_3d(&point([2.0, 3.0, 3.0]), &hollow), Ok(true));
        // In the wall.
        assert_eq!(intersects_3d(&point([1.0, 1.0, 1.0]), &hollow), Ok(true));
    }

    #[test]
    fn containment_is_per_connected_component() {
        // Two disjoint triangles in one mesh: one deep inside the solid, one
        // far outside; neither touches the shell.
        let mesh = Euclidean3DGeometry::TriangularMesh(Box::new(
            TriangularMesh3D::from_parts(
                e(),
                vec![
                    [20.0, 0.0, 0.0],
                    [21.0, 0.0, 0.0],
                    [20.0, 1.0, 0.0],
                    [2.0, 2.0, 2.0],
                    [3.0, 2.0, 2.0],
                    [2.0, 3.0, 2.0],
                ],
                [0u32, 1, 2, 3, 4, 5],
            )
            .unwrap(),
        ));
        let solid = solid_geometry(box_solid([0.0; 3], [6.0, 6.0, 6.0]));
        assert_eq!(intersects_3d(&mesh, &solid), Ok(true));
        assert_eq!(intersects_3d(&solid, &mesh), Ok(true));

        // With only the far component, no intersection.
        let far_only = triangle([[20.0, 0.0, 0.0], [21.0, 0.0, 0.0], [20.0, 1.0, 0.0]]);
        assert_eq!(intersects_3d(&far_only, &solid), Ok(false));
    }

    #[test]
    fn axis_aligned_boxes_match_the_interval_oracle() {
        let mut rng = Rng(20260716);
        for case in 0..200 {
            let (a_min, a_size) = random_box(&mut rng);
            let (b_min, b_size) = random_box(&mut rng);
            let expected = (0..3)
                .all(|k| a_min[k] <= b_min[k] + b_size[k] && b_min[k] <= a_min[k] + a_size[k]);
            let a = solid_geometry(box_solid(a_min, a_size));
            let b = solid_geometry(box_solid(b_min, b_size));
            assert_eq!(
                intersects_3d(&a, &b),
                Ok(expected),
                "case {case}: {a_min:?}+{a_size:?} vs {b_min:?}+{b_size:?}"
            );
            assert_eq!(intersects_3d(&b, &a), Ok(expected), "case {case} swapped");

            // A grid point against a closed box has the same exact oracle.
            let p = rng.grid_point(-6, 6);
            let p_expected = (0..3).all(|k| p[k] >= a_min[k] && p[k] <= a_min[k] + a_size[k]);
            assert_eq!(
                intersects_3d(&point(p), &a),
                Ok(p_expected),
                "case {case}: point {p:?} vs {a_min:?}+{a_size:?}"
            );
        }
    }

    fn random_box(rng: &mut Rng) -> ([f64; 3], [f64; 3]) {
        let min = [
            rng.int(-5, 4) as f64,
            rng.int(-5, 4) as f64,
            rng.int(-5, 4) as f64,
        ];
        let size = [
            rng.int(1, 4) as f64,
            rng.int(1, 4) as f64,
            rng.int(1, 4) as f64,
        ];
        (min, size)
    }

    #[test]
    fn empty_and_degenerate_operands() {
        let solid = solid_geometry(box_solid([0.0; 3], [4.0; 3]));
        let empty = Euclidean3DGeometry::Collection(Collection3D::new([]));
        assert_eq!(intersects_3d(&empty, &solid), Ok(false));
        // A degenerate polygon (collinear ring) re-represents as no
        // triangles: an empty point set.
        let degenerate = Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
            e(),
            [
                [1.0, 1.0, 1.0],
                [2.0, 2.0, 2.0],
                [3.0, 3.0, 3.0],
                [1.0, 1.0, 1.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        )));
        assert_eq!(intersects_3d(&degenerate, &solid), Ok(false));
        // Through the public entry: None and cross-dimension policy.
        assert_eq!(intersects(&Geometry::None, &g3(solid.clone())), Ok(false));
    }

    #[test]
    fn unsupported_and_mixed_frame_errors() {
        use crate::csg::{Csg, ThreeDimensional};
        let solid = || Box::new(box_solid([0.0; 3], [1.0; 3]));
        let csg = Euclidean3DGeometry::Csg(Csg::Union(
            Box::new(ThreeDimensional::Solid(solid())),
            Box::new(ThreeDimensional::Solid(solid())),
        ));
        assert_eq!(
            intersects_3d(&csg, &point([0.0; 3])),
            Err(PredicateError::Unsupported { geometry: "Csg" })
        );
        let far_point = Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::Crs(EpsgCode::new(4979)),
            [0.0; 3],
        ));
        assert_eq!(
            intersects_3d(&far_point, &point([0.0; 3])),
            Err(PredicateError::MixedFrames)
        );
    }

    use crate::Geometry;
}
