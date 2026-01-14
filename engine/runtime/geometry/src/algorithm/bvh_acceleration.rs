//! BVH-accelerated ray intersection for geometry collections.
//!
//! This module provides efficient ray intersection testing against large collections
//! of geometries using Bounding Volume Hierarchy (BVH) acceleration.

use bvh::aabb::{Aabb, Bounded};
use bvh::bounding_hierarchy::{BHShape, BoundingHierarchy};
use bvh::bvh::Bvh;
use bvh::ray::Ray as BvhRay;
use nalgebra::{Point3, Vector3};
use rayon::prelude::*;

use crate::algorithm::bounding_rect::BoundingRect;
use crate::algorithm::ray_intersection::{IncludeOrigin, Ray3D, RayHit, RayIntersection3D};
use crate::types::geometry::Geometry3D;

/// Wrapper that stores geometry index and bounding box for BVH construction.
/// Does not clone the actual geometry data.
#[derive(Debug, Clone)]
pub struct BvhGeometryRef {
    /// Index into the original geometry slice
    pub index: usize,
    /// BVH node index (set during BVH construction)
    node_index: usize,
    /// Axis-aligned bounding box
    aabb: Aabb<f64, 3>,
}

impl BvhGeometryRef {
    /// Create a new BVH geometry reference from a geometry's bounding box.
    pub fn new(index: usize, geometry: &Geometry3D<f64>) -> Option<Self> {
        let rect = geometry.bounding_rect()?;
        let min = rect.min();
        let max = rect.max();

        // Expand flat AABBs slightly to ensure ray intersection works.
        // This handles cases like planar triangles where min.z == max.z.
        const EPSILON: f64 = 1e-10;
        let min_pt = Point3::new(min.x - EPSILON, min.y - EPSILON, min.z - EPSILON);
        let max_pt = Point3::new(max.x + EPSILON, max.y + EPSILON, max.z + EPSILON);

        Some(Self {
            index,
            node_index: 0,
            aabb: Aabb::with_bounds(min_pt, max_pt),
        })
    }
}

impl Bounded<f64, 3> for BvhGeometryRef {
    fn aabb(&self) -> Aabb<f64, 3> {
        self.aabb
    }
}

impl BHShape<f64, 3> for BvhGeometryRef {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

/// Accelerated geometry collection for ray intersection.
/// Builds BVH from geometry bounding boxes and references the original geometry slice.
pub struct AcceleratedGeometrySet<'a> {
    /// Original geometries (borrowed, not cloned)
    geometries: &'a [Geometry3D<f64>],
    /// BVH references with bounding boxes
    bvh_refs: Vec<BvhGeometryRef>,
    /// The BVH structure
    bvh: Bvh<f64, 3>,
}

impl<'a> AcceleratedGeometrySet<'a> {
    /// Build BVH from geometry bounding boxes using parallel construction.
    /// This is O(n log n) where n = number of geometries.
    pub fn build(geometries: &'a [Geometry3D<f64>]) -> Self {
        let mut bvh_refs: Vec<BvhGeometryRef> = geometries
            .par_iter()
            .enumerate()
            .filter_map(|(i, geom)| BvhGeometryRef::new(i, geom))
            .collect();

        let bvh = Bvh::build_par(&mut bvh_refs);

        Self {
            geometries,
            bvh_refs,
            bvh,
        }
    }

    /// Convert internal Ray3D to bvh crate's Ray type.
    fn to_bvh_ray(ray: &Ray3D) -> BvhRay<f64, 3> {
        let (origin, direction) = ray.origin_and_direction();
        BvhRay::new(
            Point3::new(origin.x, origin.y, origin.z),
            Vector3::new(direction.x, direction.y, direction.z),
        )
    }

    /// Traverse BVH and perform precise intersection on candidates.
    /// Returns tuples of (geometry_index, ray_hit).
    /// The `include_origin` parameter controls whether intersections at the ray origin are included:
    /// - `IncludeOrigin::Yes`: Include all intersections with t >= 0
    /// - `IncludeOrigin::No { tolerance }`: Exclude intersections where t < tolerance
    pub fn ray_intersections(
        &self,
        ray: &Ray3D,
        tolerance: f64,
        include_origin: IncludeOrigin,
    ) -> Vec<(usize, RayHit)> {
        let bvh_ray = Self::to_bvh_ray(ray);
        let min_t = match include_origin {
            IncludeOrigin::Yes => 0.0,
            IncludeOrigin::No {
                tolerance: origin_tolerance,
            } => origin_tolerance,
        };

        let candidates = self.bvh.traverse_iterator(&bvh_ray, &self.bvh_refs);

        let mut results = Vec::new();
        for bvh_ref in candidates {
            let geom = &self.geometries[bvh_ref.index];
            let hits = geom.ray_intersections(ray, tolerance);
            for hit in hits {
                if hit.t >= min_t {
                    results.push((bvh_ref.index, hit));
                }
            }
        }
        results
    }

    /// Get the closest ray intersection to the geometry (if any).
    /// The `include_origin` parameter controls whether intersections at the ray origin are included:
    /// - `IncludeOrigin::Yes`: Include all intersections with t >= 0
    /// - `IncludeOrigin::No { tolerance }`: Exclude intersections where t < tolerance
    pub fn closest_ray_intersection(
        &self,
        ray: &Ray3D,
        tolerance: f64,
        include_origin: IncludeOrigin,
    ) -> Option<(usize, RayHit)> {
        // We use nearest_traverse_iterator which returns candidates ordered by AABB distance.
        // Once we find an actual hit, we continue checking only while AABB entry distance
        // is less than our current best hit distance.
        let bvh_ray = Self::to_bvh_ray(ray);
        let min_t = match include_origin {
            IncludeOrigin::Yes => 0.0,
            IncludeOrigin::No {
                tolerance: origin_tolerance,
            } => origin_tolerance,
        };

        let candidates = self.bvh.nearest_traverse_iterator(&bvh_ray, &self.bvh_refs);

        let mut best_hit: Option<(usize, RayHit)> = None;
        let mut best_t = f64::INFINITY;

        for bvh_ref in candidates {
            let aabb_entry_t = Self::ray_aabb_entry_distance(&bvh_ray, &bvh_ref.aabb);

            if aabb_entry_t > best_t {
                break;
            }

            let geom = &self.geometries[bvh_ref.index];
            if let Some(hit) = geom.closest_ray_intersection(ray, tolerance) {
                if hit.t >= min_t && hit.t < best_t {
                    best_t = hit.t;
                    best_hit = Some((bvh_ref.index, hit));
                }
            }
        }

        best_hit
    }

    /// Compute the entry distance of a ray into an AABB using the slab method.
    /// Returns the t value where the ray enters the AABB.
    fn ray_aabb_entry_distance(ray: &BvhRay<f64, 3>, aabb: &Aabb<f64, 3>) -> f64 {
        let inv_dir = Vector3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );

        let t1 = (aabb.min.x - ray.origin.x) * inv_dir.x;
        let t2 = (aabb.max.x - ray.origin.x) * inv_dir.x;
        let t3 = (aabb.min.y - ray.origin.y) * inv_dir.y;
        let t4 = (aabb.max.y - ray.origin.y) * inv_dir.y;
        let t5 = (aabb.min.z - ray.origin.z) * inv_dir.z;
        let t6 = (aabb.max.z - ray.origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        // If tmax < 0, the AABB is behind the ray
        // If tmin > tmax, there's no intersection
        if tmax < 0.0 || tmin > tmax {
            // We return infinity to indicate no intersection, but this should not happen
            // as we only call this for candidates that intersect the AABB.
            f64::INFINITY
        } else {
            tmin.max(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::coordinate::Coordinate3D;
    use crate::types::triangle::Triangle3D;

    #[test]
    fn test_accelerated_ray_intersection() {
        let geometries = vec![
            // Triangle 1 at z=1
            Geometry3D::Triangle(Triangle3D::new(
                Coordinate3D::new__(0.0, 0.0, 1.0),
                Coordinate3D::new__(2.0, 0.0, 1.0),
                Coordinate3D::new__(1.0, 2.0, 1.0),
            )),
            // Triangle 2 at z=2
            Geometry3D::Triangle(Triangle3D::new(
                Coordinate3D::new__(0.0, 0.0, 2.0),
                Coordinate3D::new__(2.0, 0.0, 2.0),
                Coordinate3D::new__(1.0, 2.0, 2.0),
            )),
            // Triangle 3 far away
            Geometry3D::Triangle(Triangle3D::new(
                Coordinate3D::new__(10.0, 0.0, 0.0),
                Coordinate3D::new__(11.0, 0.0, 0.0),
                Coordinate3D::new__(10.0, 1.0, 0.0),
            )),
        ];

        let accel_set = AcceleratedGeometrySet::build(&geometries);

        // Ray shooting up from origin
        let ray = Ray3D::new(
            Coordinate3D::new__(1.0, 0.5, 0.0),
            Coordinate3D::new__(0.0, 0.0, 1.0),
        );

        let hits = accel_set.ray_intersections(&ray, 1e-10, IncludeOrigin::Yes);
        // Should hit triangles at z=1 and z=2, but not the far one
        assert_eq!(hits.len(), 2);

        let closest = accel_set.closest_ray_intersection(&ray, 1e-10, IncludeOrigin::Yes);
        assert!(closest.is_some());
        let (idx, hit) = closest.unwrap();
        assert_eq!(idx, 0); // First triangle is closest
        assert!((hit.t - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_accelerated_ray_miss() {
        let geometries = vec![Geometry3D::Triangle(Triangle3D::new(
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
        ))];

        let accel_set = AcceleratedGeometrySet::build(&geometries);

        // Ray that misses the triangle
        let ray = Ray3D::new(
            Coordinate3D::new__(10.0, 10.0, 10.0),
            Coordinate3D::new__(0.0, 0.0, 1.0),
        );

        let hits = accel_set.ray_intersections(&ray, 1e-10, IncludeOrigin::Yes);
        assert!(hits.is_empty());
    }

    #[test]
    fn test_accelerated_ray_origin_exclusion() {
        let geometries = vec![Geometry3D::Triangle(Triangle3D::new(
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
        ))];

        let accel_set = AcceleratedGeometrySet::build(&geometries);

        // Ray originating on the triangle, pointing up
        let ray = Ray3D::new(
            Coordinate3D::new__(0.5, 0.5, 0.0),
            Coordinate3D::new__(0.0, 0.0, 1.0),
        );

        // With IncludeOrigin::No, the intersection at t≈0 should be excluded
        let hits = accel_set.ray_intersections(&ray, 1e-10, IncludeOrigin::No { tolerance: 1e-6 });
        assert!(hits.is_empty());

        // With IncludeOrigin::Yes, the intersection at t≈0 should be included
        let hits_with_origin = accel_set.ray_intersections(&ray, 1e-10, IncludeOrigin::Yes);
        assert_eq!(hits_with_origin.len(), 1);
        assert!(hits_with_origin[0].1.t.abs() < 1e-6);
    }

    #[test]
    fn test_closest_ray_intersection_early_termination() {
        // Create many triangles at increasing z distances
        let geometries: Vec<Geometry3D<f64>> = (0..100)
            .map(|i| {
                let z = i as f64;
                Geometry3D::Triangle(Triangle3D::new(
                    Coordinate3D::new__(0.0, 0.0, z),
                    Coordinate3D::new__(2.0, 0.0, z),
                    Coordinate3D::new__(1.0, 2.0, z),
                ))
            })
            .collect();

        let accel_set = AcceleratedGeometrySet::build(&geometries);

        let ray = Ray3D::new(
            Coordinate3D::new__(1.0, 0.5, -1.0),
            Coordinate3D::new__(0.0, 0.0, 1.0),
        );

        // Should find the first triangle (z=0) as closest
        let closest = accel_set.closest_ray_intersection(&ray, 1e-10, IncludeOrigin::Yes);
        assert!(closest.is_some());
        let (idx, hit) = closest.unwrap();
        assert_eq!(idx, 0);
        assert!((hit.t - 1.0).abs() < 1e-10); // Distance from z=-1 to z=0 is 1
    }
}
