use crate::{
    algorithm::{
        convex_hull::trivial_hull,
        kernels::{Orientation, RobustKernel},
        utils::{least_and_greatest_index, partition_slice},
        GeoNum,
    },
    types::{
        coordinate::{Coordinate, Coordinate2D, Coordinate3D},
        line_string::{LineString, LineString2D},
        no_value::NoValue,
        triangular_mesh::TriangularMesh,
    },
};

use num_traits::Float;

use super::swap_with_first_and_remove;

#[inline]
fn is_ccw<T, Z>(p_a: Coordinate<T, Z>, p_b: Coordinate<T, Z>, p_c: Coordinate<T, Z>) -> bool
where
    T: GeoNum,
    Z: GeoNum,
{
    RobustKernel::orient(p_a, p_b, p_c, None) == Orientation::CounterClockwise
}

pub fn quick_hull_2d<T>(mut points: &mut [Coordinate2D<T>]) -> LineString2D<T>
where
    T: GeoNum + From<NoValue>,
{
    // can't build a hull from fewer than four points
    if points.len() < 4 {
        return trivial_hull(points, false);
    }
    let mut hull = vec![];

    let (min, max) = {
        let (min_idx, mut max_idx) = least_and_greatest_index(points);
        let min = swap_with_first_and_remove(&mut points, min_idx);

        // Two special cases to consider:
        // (1) max_idx = 0, and got swapped
        if max_idx == 0 {
            max_idx = min_idx;
        }

        // (2) max_idx = min_idx: then any point could be
        // chosen as max. But from case (1), it could now be
        // 0, and we should not decrement it.
        max_idx = max_idx.saturating_sub(1);

        let max = swap_with_first_and_remove(&mut points, max_idx);
        (min, max)
    };

    {
        let (points, _) = partition_slice(points, |p| is_ccw(*max, *min, *p));
        hull_set_2d(*max, *min, points, &mut hull);
    }
    hull.push(*max);
    let (points, _) = partition_slice(points, |p| is_ccw(*min, *max, *p));
    hull_set_2d(*min, *max, points, &mut hull);
    hull.push(*min);
    // close the polygon
    let mut hull: LineString<_, _> = hull.into();
    hull.close();
    hull
}

/// Recursively calculate the convex hull of a subset of points
fn hull_set_2d<T>(
    p_a: Coordinate2D<T>,
    p_b: Coordinate2D<T>,
    mut set: &mut [Coordinate2D<T>],
    hull: &mut Vec<Coordinate2D<T>>,
) where
    T: GeoNum + From<NoValue>,
{
    if set.is_empty() {
        return;
    }
    if set.len() == 1 {
        hull.push(set[0]);
        return;
    }

    // Construct orthogonal vector to `p_b` - `p_a` We
    // compute inner product of this with `v` - `p_a` to
    // find the farthest point from the line segment a-b.
    let p_orth = Coordinate2D::new_(p_a.y - p_b.y, p_b.x - p_a.x);

    let furthest_idx = set
        .iter()
        .map(|pt| {
            let p_diff = *pt - p_a;
            p_orth.dot(&p_diff)
        })
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap()
        .0;

    // move Coord at furthest_point from set into hull
    let furthest_point = swap_with_first_and_remove(&mut set, furthest_idx);
    // points over PB
    {
        let (points, _) = partition_slice(set, |p| is_ccw(*furthest_point, p_b, *p));
        hull_set_2d(*furthest_point, p_b, points, hull);
    }
    hull.push(*furthest_point);
    // points over AP
    let (points, _) = partition_slice(set, |p| is_ccw(p_a, *furthest_point, *p));
    hull_set_2d(p_a, *furthest_point, points, hull);
}

/// A face in the 3D convex hull represented by 3 vertex indices.
/// The vertices are ordered counter-clockwise when viewed from outside the hull.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Face {
    vertices: [usize; 3],
}

impl Face {
    fn new(v0: usize, v1: usize, v2: usize) -> Self {
        Self {
            vertices: [v0, v1, v2],
        }
    }

    /// Returns the edges of this face as pairs of vertex indices
    fn edges(&self) -> [(usize, usize); 3] {
        [
            (self.vertices[0], self.vertices[1]),
            (self.vertices[1], self.vertices[2]),
            (self.vertices[2], self.vertices[0]),
        ]
    }
}

/// Computes the outward-facing normal of a face
fn face_normal<T: GeoNum + Float>(points: &[Coordinate3D<T>], face: &Face) -> Coordinate3D<T> {
    let a = points[face.vertices[0]];
    let b = points[face.vertices[1]];
    let c = points[face.vertices[2]];

    let ab = b - a;
    let ac = c - a;

    ab.cross(&ac)
}

/// Computes the signed distance from a point to a face plane
/// Positive if the point is on the "outside" (in the direction of the normal)
fn signed_distance_to_face<T: GeoNum + Float>(
    points: &[Coordinate3D<T>],
    face: &Face,
    point: Coordinate3D<T>,
) -> T {
    let normal = face_normal(points, face);
    let a = points[face.vertices[0]];
    let ap = point - a;
    normal.dot(&ap)
}

/// Finds 4 non-coplanar points to form the initial tetrahedron
/// If this function returns `threshold` returns `None`, then convex-hull of minimum-height at least `threshold` is not constructible.
/// Otherwise returns indices of the 4 points, or None if all points are coplanar
fn find_initial_tetrahedron<T: GeoNum + Float>(
    points: &[Coordinate3D<T>],
    threshold: T,
) -> Option<[usize; 4]> {
    if points.len() < 4 {
        return None;
    }

    let threshold = threshold / T::from(2.0).unwrap();

    // Find two points with maximum distance along x-axis
    let (mut min_x_idx, mut max_x_idx) = (0, 0);
    for (i, p) in points.iter().enumerate() {
        if p.x < points[min_x_idx].x {
            min_x_idx = i;
        }
        if p.x > points[max_x_idx].x {
            max_x_idx = i;
        }
    }

    if min_x_idx == max_x_idx || (points[max_x_idx].x - points[min_x_idx].x) <= threshold {
        // All points have the same x coordinate, which means
        // they are collinear or coplanar
        return None;
    }

    let p0 = min_x_idx;
    let p1 = max_x_idx;

    // Find point furthest from line p0-p1
    let line_dir = points[p1] - points[p0];
    let line_dir = line_dir / line_dir.norm(); // this is safe since points[p0] != points[p1]
    let mut max_dist = T::zero();
    let mut p2 = p0;

    for (i, &point) in points.iter().enumerate() {
        if i == p0 || i == p1 {
            continue;
        }
        let to_point = point - points[p0];
        // Distance from point to line = |cross product| / |line_dir|
        let cross = line_dir.cross(&to_point);
        let dist = cross.dot(&cross); // squared distance is fine for comparison
        if dist > max_dist {
            max_dist = dist;
            p2 = i;
        }
    }

    if p2 == p0 || max_dist <= threshold * threshold {
        return None; // All points are collinear
    }

    // Find point furthest from plane p0-p1-p2
    let normal = (points[p1] - points[p0]).cross(&(points[p2] - points[p0]));
    let normal = normal / normal.norm();
    let mut max_dist = T::zero();
    let mut p3 = p0;

    for (i, &point) in points.iter().enumerate() {
        if i == p0 || i == p1 || i == p2 {
            continue;
        }
        let to_point = point - points[p0];
        let dist = normal.dot(&to_point).abs();
        if dist > max_dist {
            max_dist = dist;
            p3 = i;
        }
    }

    if p3 == p0 || max_dist <= threshold {
        return None; // All points are coplanar
    }

    Some([p0, p1, p2, p3])
}

/// Creates the initial tetrahedron faces with correct winding
fn create_initial_faces<T: GeoNum + Float>(
    points: &[Coordinate3D<T>],
    tetra: [usize; 4],
) -> Vec<Face> {
    let [p0, p1, p2, p3] = tetra;

    // Create faces - we need to ensure outward-facing normals
    // A tetrahedron has 4 faces
    let mut faces = vec![
        Face::new(p0, p1, p2),
        Face::new(p0, p2, p3),
        Face::new(p0, p3, p1),
        Face::new(p1, p3, p2),
    ];

    // Compute centroid of tetrahedron
    let centroid = (points[p0] + points[p1] + points[p2] + points[p3]) / T::from(4.0).unwrap();

    // Ensure all normals point outward (away from centroid)
    for face in &mut faces {
        let face_center =
            (points[face.vertices[0]] + points[face.vertices[1]] + points[face.vertices[2]])
                / T::from(3.0).unwrap();
        let normal = face_normal(points, face);
        let to_centroid = centroid - face_center;

        // If normal points toward centroid, reverse the face winding
        if normal.dot(&to_centroid) > T::zero() {
            face.vertices.swap(1, 2);
        }
    }

    faces
}

/// Computes the 3D convex hull of a set of points using the QuickHull algorithm.
/// Returns a TriangularMesh representing the convex hull.
pub fn quick_hull_3d<T>(points: &[Coordinate3D<T>], threshold: T) -> Option<TriangularMesh<T, T>>
where
    T: GeoNum + Float,
{
    // Handle degenerate cases
    if points.len() < 4 {
        return None;
    }

    // Find initial tetrahedron (returns None if points are coplanar or degenerate)
    let tetra = find_initial_tetrahedron(points, threshold)?;

    // Create initial faces
    let mut faces = create_initial_faces(points, tetra);

    // Assign points to faces they're "outside" of
    let mut outside_sets: Vec<Vec<usize>> = vec![vec![]; faces.len()];
    let tetra_set: std::collections::HashSet<usize> = tetra.iter().copied().collect();

    for (i, &point) in points.iter().enumerate() {
        if tetra_set.contains(&i) {
            continue;
        }

        // Find the face this point is furthest outside of
        let mut max_dist = T::zero();
        let mut best_face = None;

        for (face_idx, face) in faces.iter().enumerate() {
            let dist = signed_distance_to_face(points, face, point);
            if dist > max_dist {
                max_dist = dist;
                best_face = Some(face_idx);
            }
        }

        if let Some(face_idx) = best_face {
            outside_sets[face_idx].push(i);
        }
    }

    // Process faces until no more outside points
    let mut face_idx = 0;
    while face_idx < faces.len() {
        if outside_sets[face_idx].is_empty() {
            face_idx += 1;
            continue;
        }

        // Find the furthest point from this face
        let (furthest_idx, _) = outside_sets[face_idx]
            .iter()
            .map(|&pt_idx| {
                let dist = signed_distance_to_face(points, &faces[face_idx], points[pt_idx]);
                (pt_idx, dist)
            })
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();

        let furthest_point = points[furthest_idx];

        // Find all faces visible from this point
        let mut visible_faces: Vec<usize> = Vec::new();
        for (i, face) in faces.iter().enumerate() {
            if signed_distance_to_face(points, face, furthest_point) > T::epsilon() {
                visible_faces.push(i);
            }
        }

        // If no faces are visible, the furthest point is within epsilon of all faces.
        // This means it's essentially on the hull surface (numerical noise).
        // Remove it from the outside set and continue.
        if visible_faces.is_empty() {
            outside_sets[face_idx].retain(|&p| p != furthest_idx);
            continue;
        }

        // Find horizon edges (edges shared by exactly one visible face)
        let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
            std::collections::HashMap::new();
        let mut edge_to_face: std::collections::HashMap<(usize, usize), usize> =
            std::collections::HashMap::new();

        for &face_idx in &visible_faces {
            for edge in faces[face_idx].edges() {
                let normalized_edge = if edge.0 < edge.1 {
                    edge
                } else {
                    (edge.1, edge.0)
                };
                *edge_count.entry(normalized_edge).or_insert(0) += 1;
                edge_to_face.insert(edge, face_idx);
            }
        }

        let horizon_edges: Vec<(usize, usize)> = edge_count
            .into_iter()
            .filter(|&(_, count)| count == 1)
            .map(|(edge, _)| edge)
            .collect();

        // Collect all outside points from visible faces
        let mut orphaned_points: Vec<usize> = Vec::new();
        for &vis_face_idx in &visible_faces {
            orphaned_points.append(&mut outside_sets[vis_face_idx]);
        }

        // Remove furthest point from orphaned points
        orphaned_points.retain(|&p| p != furthest_idx);

        // Remove visible faces (in reverse order to preserve indices)
        visible_faces.sort_unstable_by(|a, b| b.cmp(a));
        for &vis_face_idx in &visible_faces {
            faces.swap_remove(vis_face_idx);
            outside_sets.swap_remove(vis_face_idx);
        }

        // Create new faces from horizon edges to furthest point
        let mut new_faces: Vec<Face> = Vec::new();
        for edge in &horizon_edges {
            // Find the original edge direction from the visible face
            let original_edge = if edge_to_face.contains_key(edge) {
                *edge
            } else {
                (edge.1, edge.0)
            };

            // Create face with correct winding (reverse the edge direction)
            let new_face = Face::new(original_edge.1, original_edge.0, furthest_idx);
            new_faces.push(new_face);
        }

        // Verify and fix winding for new faces
        if !new_faces.is_empty() {
            // Compute hull centroid approximation using existing faces
            let mut centroid = Coordinate3D::zero();
            let mut count = T::zero();
            for face in &faces {
                centroid = centroid + points[face.vertices[0]];
                centroid = centroid + points[face.vertices[1]];
                centroid = centroid + points[face.vertices[2]];
                count = count + T::from(3.0).unwrap();
            }
            for face in &new_faces {
                centroid = centroid + points[face.vertices[0]];
                centroid = centroid + points[face.vertices[1]];
                centroid = centroid + points[face.vertices[2]];
                count = count + T::from(3.0).unwrap();
            }
            if count > T::zero() {
                centroid = centroid / count;
            }

            // Ensure normals point outward
            for face in &mut new_faces {
                let face_center = (points[face.vertices[0]]
                    + points[face.vertices[1]]
                    + points[face.vertices[2]])
                    / T::from(3.0).unwrap();
                let normal = face_normal(points, face);
                let to_centroid = centroid - face_center;

                if normal.dot(&to_centroid) > T::zero() {
                    face.vertices.swap(1, 2);
                }
            }
        }

        // Add new faces and their outside sets
        let new_face_start_idx = faces.len();
        faces.extend(new_faces);
        outside_sets.resize(faces.len(), vec![]);

        // Redistribute orphaned points to new faces
        for pt_idx in orphaned_points {
            let point = points[pt_idx];
            let mut max_dist = T::zero();
            let mut best_face = None;

            for (i, face) in faces.iter().enumerate().skip(new_face_start_idx) {
                let dist = signed_distance_to_face(points, face, point);
                if dist > max_dist {
                    max_dist = dist;
                    best_face = Some(i);
                }
            }

            if let Some(fi) = best_face {
                outside_sets[fi].push(pt_idx);
            }
        }

        // Reset to check all faces again
        face_idx = 0;
    }

    // Convert faces to triangles for TriangularMesh
    let triangles: Vec<[Coordinate3D<T>; 3]> = faces
        .iter()
        .map(|face| {
            [
                points[face.vertices[0]],
                points[face.vertices[1]],
                points[face.vertices[2]],
            ]
        })
        .collect();

    Some(TriangularMesh::from_triangles(triangles))
}

#[cfg(test)]
mod test {
    use crate::coord;

    use super::*;

    #[test]
    fn quick_hull_test2() {
        let mut v = vec![
            coord! { x: 0., y: 10. },
            coord! { x: 1., y: 1. },
            coord! { x: 10., y: 0. },
            coord! { x: 1., y: -1. },
            coord! { x: 0., y: -10. },
            coord! { x: -1., y: -1. },
            coord! { x: -10., y: 0. },
            coord! { x: -1., y: 1. },
            coord! { x: 0., y: 10. },
        ];
        let correct = vec![
            coord! { x: 0., y: -10. },
            coord! { x: 10., y: 0. },
            coord! { x: 0., y: 10. },
            coord! { x: -10., y: 0. },
            coord! { x: 0., y: -10. },
        ];
        let res = quick_hull_2d(&mut v);
        assert_eq!(res.0, correct);
    }

    // ========================================================================
    // 3D Convex Hull Tests
    // ========================================================================

    #[test]
    fn quick_hull_3d_tetrahedron() {
        // Simple tetrahedron - 4 points, should result in 4 faces
        let points = vec![
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.5, 1.0, 0.0),
            Coordinate3D::new__(0.5, 0.5, 1.0),
        ];

        let hull = quick_hull_3d(&points, 0.01).unwrap();

        // A tetrahedron has 4 faces
        assert_eq!(hull.get_triangles().len(), 4);
        // And 4 vertices
        assert_eq!(hull.get_vertices().len(), 4);
    }

    #[test]
    fn quick_hull_3d_cube() {
        // Cube vertices - should result in 12 triangular faces (6 faces * 2 triangles each)
        let points = vec![
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 1.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
            Coordinate3D::new__(0.0, 0.0, 1.0),
            Coordinate3D::new__(1.0, 0.0, 1.0),
            Coordinate3D::new__(1.0, 1.0, 1.0),
            Coordinate3D::new__(0.0, 1.0, 1.0),
        ];

        let hull = quick_hull_3d(&points, 0.01).unwrap();

        // A cube has 6 faces, each divided into 2 triangles = 12 triangles
        assert_eq!(hull.get_triangles().len(), 12);
        // And 8 vertices
        assert_eq!(hull.get_vertices().len(), 8);
    }

    #[test]
    fn quick_hull_3d_with_interior_points() {
        // Cube with interior point - interior point should be excluded
        let points = vec![
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 1.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
            Coordinate3D::new__(0.0, 0.0, 1.0),
            Coordinate3D::new__(1.0, 0.0, 1.0),
            Coordinate3D::new__(1.0, 1.0, 1.0),
            Coordinate3D::new__(0.0, 1.0, 1.0),
            // Interior point
            Coordinate3D::new__(0.5, 0.5, 0.5),
        ];

        let hull = quick_hull_3d(&points, 0.01).unwrap();

        // Should still be a cube (interior point excluded)
        assert_eq!(hull.get_triangles().len(), 12);
        assert_eq!(hull.get_vertices().len(), 8);
    }

    #[test]
    fn quick_hull_3d_octahedron() {
        // Octahedron - 6 vertices, 8 faces
        let points = vec![
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(-1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
            Coordinate3D::new__(0.0, -1.0, 0.0),
            Coordinate3D::new__(0.0, 0.0, 1.0),
            Coordinate3D::new__(0.0, 0.0, -1.0),
        ];

        let hull = quick_hull_3d(&points, 0.01).unwrap();

        // An octahedron has 8 triangular faces
        assert_eq!(hull.get_triangles().len(), 8);
        // And 6 vertices
        assert_eq!(hull.get_vertices().len(), 6);
    }

    #[test]
    fn quick_hull_3d_degenerate_coplanar() {
        // All points coplanar - cannot form a 3D hull
        let points = vec![
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 1.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
            Coordinate3D::new__(0.6, 0.5, 0.0),
        ];

        assert!(quick_hull_3d(&points, 0.01).is_none());
    }

    #[test]
    fn quick_hull_3d_random_sphere_points() {
        // Points on a sphere surface - hull should include most/all of them
        use std::f64::consts::PI;

        let mut points = Vec::new();
        let n = 10; // Number of latitude/longitude divisions

        for i in 0..=n {
            let phi = PI * i as f64 / n as f64;
            for j in 0..n {
                let theta = 2.0 * PI * j as f64 / n as f64;
                let x = phi.sin() * theta.cos();
                let y = phi.sin() * theta.sin();
                let z = phi.cos();
                points.push(Coordinate3D::new__(x, y, z));
            }
        }

        let hull = quick_hull_3d(&points, 0.01).unwrap();

        // Hull should have triangles (exact number depends on point distribution)
        assert!(!hull.get_triangles().is_empty());
        // All vertices should be approximately on the unit sphere
        for v in hull.get_vertices() {
            let dist = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
            assert!((dist - 1.0).abs() < 1e-10);
        }
    }
}
