//! Ray casting against 3D geometry.
//!
//! [`ray_cast`] intersects a [`Ray3D`] with every surface a 3D geometry
//! carries (a `TriangularMesh` verbatim, `Polygon` / `PolygonMesh` faces and
//! `Solid` shells through the borrowed triangulation of
//! [`view3d`](super::view3d)) and returns the [`RayHit`]s sorted by distance.
//! `Point` and `LineString` leaves yield no hits: a ray almost never meets a
//! measure-zero target.
//!
//! Unlike the exact boolean tests in [`kernel3d`](super::kernel3d), ray
//! casting *constructs* hit coordinates, so it uses the Möller–Trumbore
//! formulation with an explicit `tolerance` (see [`DEFAULT_RAY_TOLERANCE`]):
//! near-parallel rays are rejected by the determinant test and hits are
//! accepted from `t >= -tolerance`. A ray
//! passing exactly through an edge shared by two triangles reports one hit per
//! triangle; callers that need one crossing per surface point can collapse
//! equal `(t, point)` pairs.
//!
//! The ray is taken to be in the geometry's own coordinate frame; the
//! geometry's leaves must agree on one
//! ([`MixedFrames`](super::PredicateError::MixedFrames) otherwise).
//!
//! The one-shot [`ray_cast`] scans every triangle: for a single ray that beats
//! any index, because building one costs more than the scan it would save. To
//! cast **many** rays at one geometry, build a [`RayCaster`] once; its
//! large surfaces carry a bounding-volume hierarchy (an rstar tree over
//! per-triangle boxes) that descends only the nodes the ray's slab test
//! pierces, gated on triangle count ([`BVH_MIN_TRIANGLES`]). Both strategies
//! run the same exact [`ray_triangle_intersection`] on every triangle they
//! reach, so acceleration never changes which hits are reported.

use rstar::RTree;

use crate::ops::triangulation::Cache;
use crate::{Euclidean3DGeometry, Geometry};

use super::kernel::{cross3, dot3, sub3};
use super::view3d::{
    flatten_3d, flatten_geometry_3d, require_common_frame_3d, Leaf3D, SlabSelection, TriBox,
    TriangleSet,
};
use super::{PredicateError, Result};

/// The default `tolerance` for ray casting.
pub const DEFAULT_RAY_TOLERANCE: f64 = 1e-10;

/// A ray: an origin and a unit direction, extending infinitely in the
/// positive direction. `t` parameters are therefore Euclidean distances.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray3D {
    origin: [f64; 3],
    /// Normalized at construction.
    direction: [f64; 3],
}

impl Ray3D {
    /// A ray from `origin` along `direction` (normalized here). `None` when
    /// the direction is zero or not finite, which defines no ray.
    pub fn new(origin: [f64; 3], direction: [f64; 3]) -> Option<Self> {
        let norm = dot3(direction, direction).sqrt();
        if !norm.is_finite() || norm == 0.0 || origin.iter().any(|c| !c.is_finite()) {
            return None;
        }
        Some(Ray3D {
            origin,
            direction: direction.map(|c| c / norm),
        })
    }

    /// The ray's origin.
    #[inline]
    pub fn origin(&self) -> [f64; 3] {
        self.origin
    }

    /// The ray's unit direction.
    #[inline]
    pub fn direction(&self) -> [f64; 3] {
        self.direction
    }

    /// The point at distance `t` along the ray.
    #[inline]
    pub fn point_at(&self, t: f64) -> [f64; 3] {
        [
            self.origin[0] + self.direction[0] * t,
            self.origin[1] + self.direction[1] * t,
            self.origin[2] + self.direction[2] * t,
        ]
    }
}

/// One ray × surface intersection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    /// Distance along the ray (`point = origin + t * direction`).
    pub t: f64,
    /// The intersection point.
    pub point: [f64; 3],
}

/// All intersections of the ray with the geometry's surfaces, sorted by `t`
/// (then point, for a total order). `Geometry::None` yields no hits;
/// collections are the union of their members' hits; a 2D geometry is
/// [`CrossDimension`](PredicateError::CrossDimension).
///
/// This is the one-shot entry: it scans every triangle (no acceleration
/// structure), which is the right choice for a single ray. To cast **many**
/// rays at one geometry, build a [`RayCaster`] once and reuse it; it amortizes
/// a bounding-volume hierarchy over the queries.
pub fn ray_cast(geometry: &Geometry, ray: &Ray3D, tolerance: f64) -> Result<Vec<RayHit>> {
    let mut hits = collect_hits(geometry, ray, tolerance)?;
    sort_by_distance(&mut hits);
    Ok(hits)
}

/// Sort hits by `t`, then by point for a total order (the order every entry
/// returns them in).
fn sort_by_distance(hits: &mut [RayHit]) {
    hits.sort_by(|a, b| {
        a.t.total_cmp(&b.t).then_with(|| {
            a.point
                .iter()
                .zip(&b.point)
                .map(|(x, y)| x.total_cmp(y))
                .find(|o| o.is_ne())
                .unwrap_or(core::cmp::Ordering::Equal)
        })
    });
}

/// The closest forward hit (`t >= 0`), if any.
pub fn closest_ray_hit(geometry: &Geometry, ray: &Ray3D, tolerance: f64) -> Result<Option<RayHit>> {
    Ok(ray_cast(geometry, ray, tolerance)?
        .into_iter()
        .find(|hit| hit.t >= 0.0))
}

/// [`ray_cast`] over a 3D geometry, unsorted.
fn collect_hits(geometry: &Geometry, ray: &Ray3D, tolerance: f64) -> Result<Vec<RayHit>> {
    match geometry {
        Geometry::None => Ok(Vec::new()),
        Geometry::GeometryCollection(c) => {
            let mut hits = Vec::new();
            for member in c.members() {
                hits.extend(collect_hits(member, ray, tolerance)?);
            }
            Ok(hits)
        }
        Geometry::Euclidean2D(_) => Err(PredicateError::CrossDimension),
        Geometry::Euclidean3D(g) => ray_cast_3d(g, ray, tolerance),
    }
}

/// [`ray_cast`] over a 3D geometry.
pub fn ray_cast_3d(
    geometry: &Euclidean3DGeometry,
    ray: &Ray3D,
    tolerance: f64,
) -> Result<Vec<RayHit>> {
    let mut leaves = Vec::new();
    let mut unsupported = None;
    flatten_3d(geometry, &mut leaves, &mut unsupported);
    if let Some(name) = unsupported {
        return Err(PredicateError::Unsupported { geometry: name });
    }
    require_common_frame_3d(&leaves, &[])?;

    let mut cache = Cache::new();
    let mut hits = Vec::new();
    for leaf in &leaves {
        match leaf {
            Leaf3D::Point(_) | Leaf3D::Line(_) => {}
            Leaf3D::Polygon(p) => cast_linear(
                &TriangleSet::from_polygon(p, &mut cache),
                ray,
                tolerance,
                &mut hits,
            ),
            Leaf3D::PolygonMesh(m) => cast_linear(
                &TriangleSet::from_polygon_mesh_data(m.data(), &mut cache),
                ray,
                tolerance,
                &mut hits,
            ),
            Leaf3D::TriangularMesh(m) => cast_linear(
                &TriangleSet::from_triangular_data(m.data()),
                ray,
                tolerance,
                &mut hits,
            ),
            Leaf3D::Solid(s) => {
                for shell in core::iter::once(s.exterior()).chain(s.interiors().iter()) {
                    cast_linear(
                        &TriangleSet::from_shell(shell, &mut cache),
                        ray,
                        tolerance,
                        &mut hits,
                    );
                }
            }
        }
    }
    Ok(hits)
}

/// Triangle count at or above which a [`RayCaster`] surface builds a
/// bounding-volume hierarchy for its ray queries instead of scanning every
/// triangle. Below it the scan wins even with the tree already built: the
/// traversal overhead exceeds the handful of Möller-Trumbore tests it saves.
/// Calibrated by the `bvh_vs_linear_crossover` benchmark
/// (`cargo test -p reearth-flow-geometry --features new-geometry
/// predicates::ray::tests::bvh_vs_linear_crossover -- --ignored --nocapture`):
/// the prebuilt-tree query overtakes the linear scan between 8 and 18
/// triangles.
pub(crate) const BVH_MIN_TRIANGLES: usize = 16;

/// The linear strategy: the ray against every triangle. The one-shot
/// [`ray_cast`] always uses this; a single ray cannot amortize a tree build.
fn cast_linear(set: &TriangleSet<'_>, ray: &Ray3D, tolerance: f64, hits: &mut Vec<RayHit>) {
    hits.extend(
        set.triangles()
            .filter_map(|t| ray_triangle_intersection(ray, t, tolerance)),
    );
}

/// A geometry prepared for repeated ray casting: its surfaces triangulated once
/// and, for large ones, indexed by a bounding-volume hierarchy. Build it once
/// and call [`cast`](Self::cast) / [`closest`](Self::closest) many times; that
/// is where the hierarchy pays off (a per-ray rebuild is strictly slower than a
/// linear scan, so the one-shot [`ray_cast`] never builds one).
///
/// Same contract as [`ray_cast`]: `Point` / `LineString` leaves contribute no
/// hits, a 2D leaf is [`CrossDimension`](PredicateError::CrossDimension), and
/// `Csg` / `PointCloud` are [`Unsupported`](PredicateError::Unsupported). The
/// geometry's leaves must share one coordinate frame. Borrows the geometry for
/// the caster's lifetime.
pub struct RayCaster<'a> {
    surfaces: Vec<PreparedSurface<'a>>,
}

/// One triangulated surface, with an optional BVH over its triangles (present
/// only when the triangle count clears [`BVH_MIN_TRIANGLES`]).
struct PreparedSurface<'a> {
    set: TriangleSet<'a>,
    accel: Option<RTree<TriBox>>,
}

impl<'a> RayCaster<'a> {
    /// Prepare a geometry for repeated casting: flatten its leaves, triangulate
    /// each surface once, and index the large ones. The `tolerance` is supplied
    /// per cast, not here.
    pub fn new(geometry: &'a Geometry) -> Result<Self> {
        let (leaves, saw_2d, unsupported) = flatten_geometry_3d(geometry);
        if let Some(name) = unsupported {
            return Err(PredicateError::Unsupported { geometry: name });
        }
        if saw_2d {
            return Err(PredicateError::CrossDimension);
        }
        require_common_frame_3d(&leaves, &[])?;

        let mut cache = Cache::new();
        let mut surfaces = Vec::new();
        for leaf in &leaves {
            match leaf {
                Leaf3D::Point(_) | Leaf3D::Line(_) => {}
                Leaf3D::Polygon(p) => {
                    push_surface(TriangleSet::from_polygon(p, &mut cache), &mut surfaces)
                }
                Leaf3D::PolygonMesh(m) => push_surface(
                    TriangleSet::from_polygon_mesh_data(m.data(), &mut cache),
                    &mut surfaces,
                ),
                Leaf3D::TriangularMesh(m) => {
                    push_surface(TriangleSet::from_triangular_data(m.data()), &mut surfaces)
                }
                Leaf3D::Solid(s) => {
                    for shell in core::iter::once(s.exterior()).chain(s.interiors().iter()) {
                        push_surface(TriangleSet::from_shell(shell, &mut cache), &mut surfaces);
                    }
                }
            }
        }
        Ok(RayCaster { surfaces })
    }

    /// All intersections of the ray with the prepared surfaces, sorted by `t`
    /// then point, exactly as [`ray_cast`] returns them.
    pub fn cast(&self, ray: &Ray3D, tolerance: f64) -> Vec<RayHit> {
        let mut hits = Vec::new();
        for surface in &self.surfaces {
            match &surface.accel {
                Some(tree) => query_tree(tree, &surface.set, ray, tolerance, &mut hits),
                None => cast_linear(&surface.set, ray, tolerance, &mut hits),
            }
        }
        sort_by_distance(&mut hits);
        hits
    }

    /// The closest forward hit (`t >= 0`), if any.
    pub fn closest(&self, ray: &Ray3D, tolerance: f64) -> Option<RayHit> {
        self.cast(ray, tolerance)
            .into_iter()
            .find(|hit| hit.t >= 0.0)
    }
}

/// Push a non-empty surface, building its BVH when it clears the size gate.
fn push_surface<'a>(set: TriangleSet<'a>, out: &mut Vec<PreparedSurface<'a>>) {
    if set.is_empty() {
        return;
    }
    let accel = (set.len() >= BVH_MIN_TRIANGLES).then(|| set.rtree());
    out.push(PreparedSurface { set, accel });
}

/// Query a prebuilt tree: the exact triangle test only on the triangles whose
/// box the ray's slab test pierces (the shared [`SlabSelection`]).
fn query_tree(
    tree: &RTree<TriBox>,
    set: &TriangleSet<'_>,
    ray: &Ray3D,
    tolerance: f64,
    hits: &mut Vec<RayHit>,
) {
    let selection = SlabSelection::ray(ray.origin(), ray.direction(), tolerance);
    for candidate in tree.locate_with_selection_function(selection) {
        if let Some(hit) =
            ray_triangle_intersection(ray, set.triangle(candidate.idx as usize), tolerance)
        {
            hits.push(hit);
        }
    }
}

/// Möller–Trumbore ray × triangle intersection: rays parallel to the triangle
/// within the (raw-determinant) `tolerance` miss, boundary hits (`u`/`v` at
/// their bounds) count, and hits from `t >= -tolerance` are accepted so a ray
/// starting on the surface still reports it.
pub fn ray_triangle_intersection(
    ray: &Ray3D,
    triangle: [[f64; 3]; 3],
    tolerance: f64,
) -> Option<RayHit> {
    let [v0, v1, v2] = triangle;
    let edge1 = sub3(v1, v0);
    let edge2 = sub3(v2, v0);

    let h = cross3(ray.direction, edge2);
    let a = dot3(edge1, h);
    // Ray parallel to the triangle's plane.
    if a.abs() < tolerance {
        return None;
    }

    let f = 1.0 / a;
    let s = sub3(ray.origin, v0);
    let u = f * dot3(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = cross3(s, edge1);
    let v = f * dot3(ray.direction, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * dot3(edge2, q);
    if t >= -tolerance {
        Some(RayHit {
            t,
            point: ray.point_at(t),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::point::Point3D;
    use crate::predicates::test3d::{box_solid, e, g3, solid_geometry};
    use pretty_assertions::assert_eq;

    #[test]
    fn ray_through_a_box_hits_entry_and_exit() {
        // (y, z) = (1, 2) avoids the faces' triangulation diagonals.
        let solid = g3(solid_geometry(box_solid([0.0; 3], [4.0; 3])));
        let ray = Ray3D::new([-1.0, 1.0, 2.0], [1.0, 0.0, 0.0]).unwrap();
        let hits = ray_cast(&solid, &ray, DEFAULT_RAY_TOLERANCE).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].t, 1.0);
        assert_eq!(hits[0].point, [0.0, 1.0, 2.0]);
        assert_eq!(hits[1].t, 5.0);
        assert_eq!(hits[1].point, [4.0, 1.0, 2.0]);
        assert_eq!(
            closest_ray_hit(&solid, &ray, DEFAULT_RAY_TOLERANCE).unwrap(),
            Some(hits[0])
        );
        // Pointing away: no hits.
        let away = Ray3D::new([-1.0, 1.0, 2.0], [-1.0, 0.0, 0.0]).unwrap();
        assert_eq!(
            ray_cast(&solid, &away, DEFAULT_RAY_TOLERANCE).unwrap(),
            vec![]
        );
    }

    #[test]
    fn direction_is_normalized_so_t_is_distance() {
        let solid = g3(solid_geometry(box_solid([0.0; 3], [4.0; 3])));
        let fast = Ray3D::new([-2.0, 1.0, 2.0], [10.0, 0.0, 0.0]).unwrap();
        let hits = ray_cast(&solid, &fast, DEFAULT_RAY_TOLERANCE).unwrap();
        assert_eq!(hits[0].t, 2.0);
        assert!(Ray3D::new([0.0; 3], [0.0; 3]).is_none());
        assert!(Ray3D::new([0.0; 3], [f64::NAN, 0.0, 0.0]).is_none());
    }

    #[test]
    fn ray_starting_on_the_surface_reports_t_zero() {
        let solid = g3(solid_geometry(box_solid([0.0; 3], [4.0; 3])));
        let inward = Ray3D::new([0.0, 1.0, 2.0], [1.0, 0.0, 0.0]).unwrap();
        let hits = ray_cast(&solid, &inward, DEFAULT_RAY_TOLERANCE).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].t, 0.0);
        assert_eq!(hits[1].t, 4.0);
    }

    #[test]
    fn point_and_line_leaves_yield_no_hits_and_errors_propagate() {
        let p = g3(crate::Euclidean3DGeometry::Point(Point3D::new(
            e(),
            [0.0; 3],
        )));
        let ray = Ray3D::new([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]).unwrap();
        assert_eq!(ray_cast(&p, &ray, DEFAULT_RAY_TOLERANCE).unwrap(), vec![]);
        assert_eq!(
            ray_cast(&Geometry::None, &ray, DEFAULT_RAY_TOLERANCE).unwrap(),
            vec![]
        );

        let square_2d = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Point(
            crate::point::Point2D::new(e(), [0.0, 0.0]),
        ));
        assert_eq!(
            ray_cast(&square_2d, &ray, DEFAULT_RAY_TOLERANCE),
            Err(PredicateError::CrossDimension)
        );
    }

    /// A `w x w` grid of quads in `z = 0`, each split into two triangles:
    /// `(w + 1)^2` vertices, `2 * w^2` triangles.
    fn grid_mesh(w: usize) -> crate::triangular_mesh::TriangularMesh3D {
        let mut verts = Vec::with_capacity((w + 1) * (w + 1));
        for j in 0..=w {
            for i in 0..=w {
                verts.push([i as f64, j as f64, 0.0]);
            }
        }
        let idx = |i: usize, j: usize| (j * (w + 1) + i) as u32;
        let mut tris = Vec::with_capacity(w * w * 6);
        for j in 0..w {
            for i in 0..w {
                tris.extend_from_slice(&[
                    idx(i, j),
                    idx(i + 1, j),
                    idx(i + 1, j + 1),
                    idx(i, j),
                    idx(i + 1, j + 1),
                    idx(i, j + 1),
                ]);
            }
        }
        crate::triangular_mesh::TriangularMesh3D::from_parts(e(), verts, tris).unwrap()
    }

    /// Sort hits into the total order [`ray_cast`] uses, so two strategies'
    /// outputs compare directly.
    fn sort_hits(hits: &mut [RayHit]) {
        hits.sort_by(|a, b| {
            a.t.total_cmp(&b.t).then_with(|| {
                a.point
                    .iter()
                    .zip(&b.point)
                    .map(|(x, y)| x.total_cmp(y))
                    .find(|o| o.is_ne())
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
        });
    }

    /// The BVH strategy with the tree built inline: only the benchmark's
    /// rebuild-per-ray column and the low-level equivalence test use it, since
    /// production never rebuilds per ray.
    fn cast_bvh(set: &TriangleSet<'_>, ray: &Ray3D, tolerance: f64, hits: &mut Vec<RayHit>) {
        let tree = set.rtree();
        query_tree(&tree, set, ray, tolerance, hits);
    }

    #[test]
    fn bvh_and_linear_report_identical_hits() {
        // 288 triangles, above BVH_MIN_TRIANGLES, so both strategies are live.
        let mesh = grid_mesh(12);
        let set = TriangleSet::from_triangular_data(mesh.data());

        let mut rays = Vec::new();
        for j in 0..12 {
            for i in 0..12 {
                let (x, y) = (i as f64, j as f64);
                // Cell interior (one face), the split diagonal (a shared edge,
                // two faces), and a grid vertex (several faces): the boundary
                // cases where a too-tight box test would drop a hit.
                rays.push(Ray3D::new([x + 0.7, y + 0.3, 5.0], [0.0, 0.0, -1.0]).unwrap());
                rays.push(Ray3D::new([x + 0.5, y + 0.5, 5.0], [0.0, 0.0, -1.0]).unwrap());
                rays.push(Ray3D::new([x, y, 5.0], [0.0, 0.0, -1.0]).unwrap());
            }
        }
        // Oblique, in-plane (grazes every triangle it crosses), reversed, and a
        // clean miss.
        rays.push(Ray3D::new([-1.0, -1.0, 5.0], [1.0, 1.0, -1.0]).unwrap());
        rays.push(Ray3D::new([-1.0, 3.0, 0.0], [1.0, 0.0, 0.0]).unwrap());
        rays.push(Ray3D::new([5.5, 5.5, -5.0], [0.0, 0.0, 1.0]).unwrap());
        rays.push(Ray3D::new([100.0, 100.0, 5.0], [0.0, 0.0, -1.0]).unwrap());

        for ray in &rays {
            let mut linear = Vec::new();
            cast_linear(&set, ray, DEFAULT_RAY_TOLERANCE, &mut linear);
            let mut bvh = Vec::new();
            cast_bvh(&set, ray, DEFAULT_RAY_TOLERANCE, &mut bvh);
            sort_hits(&mut linear);
            sort_hits(&mut bvh);
            assert_eq!(linear, bvh, "ray {ray:?}");
        }
    }

    #[test]
    fn raycaster_matches_the_one_shot_entry() {
        // A mesh over the BVH gate (288 triangles) and a solid: the caster's
        // indexed path must agree with the linear one-shot for every ray.
        let mesh = g3(Euclidean3DGeometry::TriangularMesh(Box::new(grid_mesh(12))));
        let solid = g3(solid_geometry(box_solid([0.0; 3], [4.0; 3])));
        for geometry in [&mesh, &solid] {
            let caster = RayCaster::new(geometry).unwrap();
            for (ox, oy) in [(1.7, 2.3), (2.0, 2.0), (0.0, 0.0), (3.5, 1.5)] {
                let ray = Ray3D::new([ox, oy, 9.0], [0.0, 0.0, -1.0]).unwrap();
                let one_shot = ray_cast(geometry, &ray, DEFAULT_RAY_TOLERANCE).unwrap();
                assert_eq!(caster.cast(&ray, DEFAULT_RAY_TOLERANCE), one_shot);
                assert_eq!(
                    caster.closest(&ray, DEFAULT_RAY_TOLERANCE),
                    closest_ray_hit(geometry, &ray, DEFAULT_RAY_TOLERANCE).unwrap()
                );
            }
        }
    }

    #[test]
    fn raycaster_propagates_the_same_errors() {
        use crate::csg::{Csg, ThreeDimensional};
        let two_d = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Point(
            crate::point::Point2D::new(e(), [0.0, 0.0]),
        ));
        assert_eq!(
            RayCaster::new(&two_d).err(),
            Some(PredicateError::CrossDimension)
        );
        let csg = g3(Euclidean3DGeometry::Csg(Csg::Union(
            Box::new(ThreeDimensional::Solid(Box::new(box_solid(
                [0.0; 3], [1.0; 3],
            )))),
            Box::new(ThreeDimensional::Solid(Box::new(box_solid(
                [0.0; 3], [1.0; 3],
            )))),
        )));
        assert_eq!(
            RayCaster::new(&csg).err(),
            Some(PredicateError::Unsupported { geometry: "Csg" })
        );
    }

    /// Deterministic fractions in `[0.05, 0.95)^2` (xorshift64), reused across
    /// grid sizes so every mesh is probed at the same relative positions.
    fn probe_fractions(n: usize) -> Vec<(f64, f64)> {
        let mut state = 0x2545_F491_4F6C_DD1Du64;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1u64 << 53) as f64
        };
        (0..n)
            .map(|_| (0.05 + 0.9 * next(), 0.05 + 0.9 * next()))
            .collect()
    }

    /// Mean nanoseconds per ray for one strategy (tree build included in the
    /// BVH path, since each ray builds its own).
    fn time_strategy(set: &TriangleSet<'_>, rays: &[Ray3D], bvh: bool) -> f64 {
        let mut sink = Vec::new();
        let mut guard = 0u64;
        let start = std::time::Instant::now();
        for ray in rays {
            sink.clear();
            if bvh {
                cast_bvh(set, ray, DEFAULT_RAY_TOLERANCE, &mut sink);
            } else {
                cast_linear(set, ray, DEFAULT_RAY_TOLERANCE, &mut sink);
            }
            guard = guard.wrapping_add(sink.len() as u64);
        }
        let elapsed = start.elapsed().as_nanos() as f64;
        std::hint::black_box(guard);
        elapsed / rays.len() as f64
    }

    /// Mean nanoseconds per ray querying a tree built once and reused: the
    /// amortized BVH cost when the same mesh serves many rays.
    fn time_query_only(set: &TriangleSet<'_>, rays: &[Ray3D]) -> f64 {
        let tree = set.rtree();
        let mut sink = Vec::new();
        let mut guard = 0u64;
        let start = std::time::Instant::now();
        for ray in rays {
            sink.clear();
            query_tree(&tree, set, ray, DEFAULT_RAY_TOLERANCE, &mut sink);
            guard = guard.wrapping_add(sink.len() as u64);
        }
        let elapsed = start.elapsed().as_nanos() as f64;
        std::hint::black_box(guard);
        elapsed / rays.len() as f64
    }

    /// Calibration benchmark for [`BVH_MIN_TRIANGLES`]: prints per-ray cost of
    /// each strategy across triangle counts. Run manually:
    /// `cargo test -p reearth-flow-geometry --features new-geometry \
    ///  predicates::ray::tests::bvh_vs_linear_crossover -- --ignored --nocapture`.
    #[test]
    #[ignore = "timing benchmark; run manually to calibrate BVH_MIN_TRIANGLES"]
    fn bvh_vs_linear_crossover() {
        let widths = [2usize, 3, 4, 5, 6, 8, 11, 16, 23, 32, 45];
        let fracs = probe_fractions(3000);
        println!("\n  tris | linear ns/ray | bvh rebuild ns/ray | bvh query ns/ray | query winner");
        for w in widths {
            let mesh = grid_mesh(w);
            let set = TriangleSet::from_triangular_data(mesh.data());
            let rays: Vec<Ray3D> = fracs
                .iter()
                .map(|&(fx, fy)| {
                    Ray3D::new(
                        [fx * w as f64, fy * w as f64, w as f64 + 5.0],
                        [0.0, 0.0, -1.0],
                    )
                    .unwrap()
                })
                .collect();
            // Warm up before timing.
            let _ = time_strategy(&set, &rays, false);
            let _ = time_strategy(&set, &rays, true);
            let linear = time_strategy(&set, &rays, false);
            let bvh_rebuild = time_strategy(&set, &rays, true);
            let bvh_query = time_query_only(&set, &rays);
            println!(
                "{:6} | {:13.1} | {:15.1} | {:13.1} | {}",
                set.len(),
                linear,
                bvh_rebuild,
                bvh_query,
                if bvh_query < linear {
                    "bvh(query)"
                } else {
                    "linear"
                }
            );
        }
    }
}
