//! Ray-geometry intersection algorithms.
//!
//! This module provides internal Ray types and intersection testing against various geometry types.
//! Rays are unbounded in the positive direction (t >= 0).

use crate::types::{
    coordinate::Coordinate3D, geometry::Geometry3D, multi_polygon::MultiPolygon3D,
    polygon::Polygon3D, solid::Solid, triangle::Triangle3D, triangular_mesh::TriangularMesh,
};

/// Controls whether intersections at the ray origin are included.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IncludeOrigin {
    /// Include intersections at the ray origin (t ≈ 0).
    Yes,
    /// Exclude intersections within the specified tolerance of the ray origin.
    No { tolerance: f64 },
}

/// A 3D ray defined by an origin point and a direction vector.
/// The ray extends infinitely in the positive direction from the origin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray3D {
    /// Origin point of the ray
    pub origin: Coordinate3D<f64>,
    /// Direction vector (should be normalized for consistent distance calculations)
    pub direction: Coordinate3D<f64>,
}

/// Result of a ray intersection test
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    /// Distance along the ray (t parameter where point = origin + t * direction)
    pub t: f64,
    /// Intersection point
    pub point: Coordinate3D<f64>,
}

impl Ray3D {
    /// Create a new ray from origin and direction.
    /// The direction should ideally be normalized for consistent t values.
    #[inline]
    pub fn new(origin: Coordinate3D<f64>, direction: Coordinate3D<f64>) -> Self {
        Self { origin, direction }
    }

    /// Create a ray with normalized direction vector.
    #[inline]
    pub fn normalized(origin: Coordinate3D<f64>, direction: Coordinate3D<f64>) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Get a point along the ray at parameter t.
    /// For t >= 0, the point is in the ray's positive direction.
    #[inline]
    pub fn point_at(&self, t: f64) -> Coordinate3D<f64> {
        self.origin + self.direction * t
    }
}

/// Trait for geometry types that can be tested for ray intersection.
pub trait RayIntersection3D {
    /// Find all intersections with a ray.
    /// Returns hits with t >= 0 (in ray's positive direction).
    fn ray_intersections(&self, ray: &Ray3D, tolerance: f64) -> Vec<RayHit>;

    /// Get only the closest intersection (optimization for common case).
    fn closest_ray_intersection(&self, ray: &Ray3D, tolerance: f64) -> Option<RayHit> {
        self.ray_intersections(ray, tolerance)
            .into_iter()
            .filter(|hit| hit.t >= 0.0)
            .min_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal))
    }
}

// =============================================================================
// Triangle3D Implementation (Möller-Trumbore algorithm)
// =============================================================================

impl RayIntersection3D for Triangle3D<f64> {
    fn ray_intersections(&self, ray: &Ray3D, tolerance: f64) -> Vec<RayHit> {
        // Möller-Trumbore ray-triangle intersection algorithm
        let v0 = self.0;
        let v1 = self.1;
        let v2 = self.2;

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;

        let h = ray.direction.cross(&edge2);
        let a = edge1.dot(&h);

        // Check if ray is parallel to triangle
        if a.abs() < tolerance {
            return vec![];
        }

        let f = 1.0 / a;
        let s = ray.origin - v0;
        let u = f * s.dot(&h);

        // Check barycentric coordinate u
        if !(0.0..=1.0).contains(&u) {
            return vec![];
        }

        let q = s.cross(&edge1);
        let v = f * ray.direction.dot(&q);

        // Check barycentric coordinate v
        if v < 0.0 || u + v > 1.0 {
            return vec![];
        }

        // Compute t to find intersection point
        let t = f * edge2.dot(&q);

        // For rays (unlike segments), we accept t >= 0 (unbounded in positive direction)
        // Use a small negative tolerance to handle numerical precision at t=0
        if t >= -tolerance {
            let point = ray.point_at(t);
            vec![RayHit { t, point }]
        } else {
            vec![]
        }
    }
}

// =============================================================================
// TriangularMesh Implementation
// =============================================================================

impl RayIntersection3D for TriangularMesh<f64, f64> {
    fn ray_intersections(&self, ray: &Ray3D, tolerance: f64) -> Vec<RayHit> {
        let vertices = self.get_vertices();
        let triangles = self.get_triangles();

        let mut results = Vec::new();
        for tri_indices in triangles {
            let triangle = Triangle3D::new(
                vertices[tri_indices[0]],
                vertices[tri_indices[1]],
                vertices[tri_indices[2]],
            );
            results.extend(triangle.ray_intersections(ray, tolerance));
        }
        results
    }
}

// =============================================================================
// Polygon3D Implementation (triangulate on-demand)
// =============================================================================

impl RayIntersection3D for Polygon3D<f64> {
    fn ray_intersections(&self, ray: &Ray3D, tolerance: f64) -> Vec<RayHit> {
        // Fan triangulation from first vertex (works for convex polygons)
        let exterior = self.exterior();
        let coords = &exterior.0;

        if coords.len() < 3 {
            return vec![];
        }

        let mut results = Vec::new();
        let v0 = coords[0];
        for i in 1..coords.len().saturating_sub(1) {
            let v1 = coords[i];
            let v2 = coords[i + 1];
            let triangle = Triangle3D::new(v0, v1, v2);
            results.extend(triangle.ray_intersections(ray, tolerance));
        }

        results
    }
}

// =============================================================================
// MultiPolygon3D Implementation
// =============================================================================

impl RayIntersection3D for MultiPolygon3D<f64> {
    fn ray_intersections(&self, ray: &Ray3D, tolerance: f64) -> Vec<RayHit> {
        self.0
            .iter()
            .flat_map(|polygon| polygon.ray_intersections(ray, tolerance))
            .collect()
    }
}

// =============================================================================
// Solid Implementation
// =============================================================================

impl RayIntersection3D for Solid<f64, f64> {
    fn ray_intersections(&self, ray: &Ray3D, tolerance: f64) -> Vec<RayHit> {
        // Test against all boundary faces
        let faces = self.all_faces();
        let mut results = Vec::new();

        for face in faces {
            // Face is a LineString representing a polygon boundary
            let coords = &face.0;
            if coords.len() < 3 {
                continue;
            }

            // Fan triangulation
            let v0 = coords[0];
            for i in 1..coords.len().saturating_sub(1) {
                let v1 = coords[i];
                let v2 = coords[i + 1];
                let triangle = Triangle3D::new(v0, v1, v2);
                results.extend(triangle.ray_intersections(ray, tolerance));
            }
        }

        results
    }
}

// =============================================================================
// Geometry3D enum dispatch
// =============================================================================

impl RayIntersection3D for Geometry3D<f64> {
    fn ray_intersections(&self, ray: &Ray3D, tolerance: f64) -> Vec<RayHit> {
        match self {
            Geometry3D::Triangle(t) => t.ray_intersections(ray, tolerance),
            Geometry3D::TriangularMesh(mesh) => mesh.ray_intersections(ray, tolerance),
            Geometry3D::Polygon(p) => p.ray_intersections(ray, tolerance),
            Geometry3D::MultiPolygon(mp) => mp.ray_intersections(ray, tolerance),
            Geometry3D::Solid(s) => s.ray_intersections(ray, tolerance),
            Geometry3D::GeometryCollection(gc) => gc
                .iter()
                .flat_map(|g| g.ray_intersections(ray, tolerance))
                .collect(),
            // Point, Line, LineString, etc. have no area - rays pass through them
            // without intersection (in the surface sense)
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray3d_point_at() {
        let ray = Ray3D::new(
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
        );
        let p = ray.point_at(5.0);
        assert!((p.x - 5.0).abs() < 1e-10);
        assert!((p.y - 0.0).abs() < 1e-10);
        assert!((p.z - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_ray3d_normalized() {
        let ray = Ray3D::normalized(
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(3.0, 0.0, 0.0),
        );
        assert!((ray.direction.x - 1.0).abs() < 1e-10);
        assert!((ray.direction.y - 0.0).abs() < 1e-10);
        assert!((ray.direction.z - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_ray_triangle_intersection_direct_hit() {
        let triangle = Triangle3D::new(
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(2.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 2.0, 0.0),
        );

        // Ray from above, pointing down at the triangle
        let ray = Ray3D::new(
            Coordinate3D::new__(1.0, 0.5, 1.0),
            Coordinate3D::new__(0.0, 0.0, -1.0),
        );

        let results = triangle.ray_intersections(&ray, 1e-10);
        assert_eq!(results.len(), 1);
        assert!((results[0].point.z).abs() < 1e-10);
        assert!((results[0].t - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ray_triangle_intersection_miss() {
        let triangle = Triangle3D::new(
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
        );

        // Ray parallel to triangle
        let ray = Ray3D::new(
            Coordinate3D::new__(0.0, 0.0, 1.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
        );

        let results = triangle.ray_intersections(&ray, 1e-10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_ray_triangle_intersection_behind_origin() {
        let triangle = Triangle3D::new(
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(2.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 2.0, 0.0),
        );

        // Ray pointing away from triangle
        let ray = Ray3D::new(
            Coordinate3D::new__(1.0, 0.5, 1.0),
            Coordinate3D::new__(0.0, 0.0, 1.0), // pointing up, away from triangle
        );

        let results = triangle.ray_intersections(&ray, 1e-10);
        // t would be negative, so no intersection in positive direction
        assert!(results.is_empty());
    }

    #[test]
    fn test_closest_ray_intersection() {
        // Two triangles at different distances
        let t1 = Triangle3D::new(
            Coordinate3D::new__(0.0, 0.0, 1.0),
            Coordinate3D::new__(2.0, 0.0, 1.0),
            Coordinate3D::new__(1.0, 2.0, 1.0),
        );
        let t2 = Triangle3D::new(
            Coordinate3D::new__(0.0, 0.0, 2.0),
            Coordinate3D::new__(2.0, 0.0, 2.0),
            Coordinate3D::new__(1.0, 2.0, 2.0),
        );

        let ray = Ray3D::new(
            Coordinate3D::new__(1.0, 0.5, 0.0),
            Coordinate3D::new__(0.0, 0.0, 1.0),
        );

        let hit1 = t1.closest_ray_intersection(&ray, 1e-10);
        let hit2 = t2.closest_ray_intersection(&ray, 1e-10);

        assert!(hit1.is_some());
        assert!(hit2.is_some());
        assert!(hit1.unwrap().t < hit2.unwrap().t);
        assert!((hit1.unwrap().t - 1.0).abs() < 1e-10);
        assert!((hit2.unwrap().t - 2.0).abs() < 1e-10);
    }
}
