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

use crate::ops::triangulation::Cache;
use crate::{Euclidean3DGeometry, Geometry};

use super::kernel::{cross3, dot3, sub3};
use super::view3d::{flatten_3d, require_common_frame_3d, Leaf3D, TriangleSet};
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
pub fn ray_cast(geometry: &Geometry, ray: &Ray3D, tolerance: f64) -> Result<Vec<RayHit>> {
    let mut hits = collect_hits(geometry, ray, tolerance)?;
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
    Ok(hits)
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
            Leaf3D::Polygon(p) => cast_into(
                &TriangleSet::from_polygon(p, &mut cache),
                ray,
                tolerance,
                &mut hits,
            ),
            Leaf3D::PolygonMesh(m) => cast_into(
                &TriangleSet::from_polygon_mesh_data(m.data(), &mut cache),
                ray,
                tolerance,
                &mut hits,
            ),
            Leaf3D::TriangularMesh(m) => cast_into(
                &TriangleSet::from_triangular_data(m.data()),
                ray,
                tolerance,
                &mut hits,
            ),
            Leaf3D::Solid(s) => {
                for shell in core::iter::once(s.exterior()).chain(s.interiors().iter()) {
                    cast_into(
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

/// Append the ray's hits on every triangle of the set.
fn cast_into(set: &TriangleSet<'_>, ray: &Ray3D, tolerance: f64, hits: &mut Vec<RayHit>) {
    hits.extend(
        set.triangles()
            .filter_map(|t| ray_triangle_intersection(ray, t, tolerance)),
    );
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
}
