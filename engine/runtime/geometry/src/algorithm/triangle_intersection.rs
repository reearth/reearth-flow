use crate::algorithm::segment_triangle_intersection::segment_triangle_intersection;
use crate::types::{coordinate::Coordinate3D, line::Line3D};
use crate::utils::circumcenter;

pub fn triangles_intersect(t: &[Coordinate3D<f64>; 3], s: &[Coordinate3D<f64>; 3]) -> bool {
    let epsilon = 1e-10;

    // filter out the obvious non-intersecting cases first
    {
        let Some((ct, rt)) = circumcenter(t[0], t[1], t[2]) else {
            return false;
        };

        let Some((cs, rs)) = circumcenter(s[0], s[1], s[2]) else {
            return false;
        };

        let d = (ct - cs).norm();
        if d > rt + rs {
            return false;
        }
    }

    let (mut t, mut s) = (*t, *s);

    normalize_triangle_pair(&mut t, &mut s);

    // Check if any edge of triangle t intersects triangle s
    for i in 0..3 {
        let j = (i + 1) % 3;
        let line = Line3D::new_(t[i], t[j]);
        if segment_triangle_intersection(&line, &s, epsilon).is_some() {
            return true;
        }
    }

    // Check if any edge of triangle s intersects triangle t
    for i in 0..3 {
        let j = (i + 1) % 3;
        let line = Line3D::new_(s[i], s[j]);
        if segment_triangle_intersection(&line, &t, epsilon).is_some() {
            return true;
        }
    }

    false
}

/// returns the intersection geometry of two triangles if they intersect, otherwise returns None.
/// Coplanar triangles are not considered intersecting in this implementation.
/// Furthermore, if the intersection occurs at a single point, it is also considered as non-intersecting.
pub fn triangles_intersection(mut t: [Coordinate3D<f64>; 3], mut s: [Coordinate3D<f64>; 3]) -> Result<Option<[Coordinate3D<f64>; 2]>, ()> {
    let epsilon = 1e-10;
    {
        let Some((ct, rt)) = circumcenter(t[0], t[1], t[2]) else {
            return Err(());
        };

        let Some((cs, rs)) = circumcenter(s[0], s[1], s[2]) else {
            return Err(());
        };

        let d = (ct - cs).norm();
        if d > rt + rs {
            return Ok(None);
        }
    }

    
    let (avg, norm_avg) = normalize_triangle_pair(&mut t, &mut s);
    // Check for coplanar case
    if (t[1] - t[0]).cross(&(t[2] - t[0])).dot(&(s[0] - t[0])).abs() < epsilon
        && (s[1] - s[0]).cross(&(s[2] - s[0])).dot(&(t[0] - s[0])).abs() < epsilon
    {
        // Coplanar triangles are not considered intersecting in this implementation.
        return Ok(None);
    }

    let mut intersection_points = Vec::new();
    // Check if any edge of triangle t intersects any edges of triangle s
    for [i,j] in [[0,1],[1,2],[0,2]] {
        let l1 = Line3D::new_(t[i], t[j]);
        for [k,l] in [[0,1],[1,2],[0,2]] {
            let l2 = Line3D::new_(s[k], s[l]);
            if let Some(p) = &l1.intersection(&l2, Some(epsilon)) {
                let intersection = (*p * norm_avg) + avg;
                intersection_points.push(intersection);
            }
        }
    }

    // Check if any edge of triangle s intersects triangle t
    for [i,j] in [[0,1],[1,2],[0,2]] {
        let l = Line3D::new_(t[i], t[j]);
        if let Some(p) = segment_triangle_intersection(&l, &s, epsilon) {
            let intersection = (p * norm_avg) + avg;
            intersection_points.push(intersection);
        }
    }
    for [i,j] in [[0,1],[1,2],[0,2]] {
        let l = Line3D::new_(s[i], s[j]);
        if let Some(p) = segment_triangle_intersection(&l, &t, epsilon) {
            let intersection = (p * norm_avg) + avg;
            intersection_points.push(intersection);
        }
    }

    // Remove duplicate points
    for i in (0..intersection_points.len()).rev() {
        for j in (0..i).rev() {
            if (intersection_points[i] - intersection_points[j]).norm() < epsilon {
                intersection_points.remove(i);
                break;
            }
        }
    }

    if intersection_points.len() < 2 {
        Ok(None)
    } else if intersection_points.len() == 2 {
        Ok(Some([intersection_points[0], intersection_points[1]]))
    } else {
        Err(())
    }
}


/// Normalizes two triangles by translating them to the origin and scaling them to fit within a unit sphere.
/// Returns the translation and the scaling factor used for normalization.
/// To recover the original coordinates, first multiply by the scaling factor, then add the translation.
fn normalize_triangle_pair(
    t1: &mut [Coordinate3D<f64>; 3],
    t2: &mut [Coordinate3D<f64>; 3],
) -> (Coordinate3D<f64>, f64) {
    let avg = (t1[0] + t1[1] + t1[2] + t2[0] + t2[1] + t2[2]) / 6.0;

    *t1 = [t1[0] - avg, t1[1] - avg, t1[2] - avg];
    *t2 = [t2[0] - avg, t2[1] - avg, t2[2] - avg];

    let norm_avg =
        (t1[0].norm() + t1[1].norm() + t1[2].norm() + t2[0].norm() + t2[1].norm() + t2[2].norm())
            / 6.0;
    *t1 = [t1[0] / norm_avg, t1[1] / norm_avg, t1[2] / norm_avg];
    *t2 = [t2[0] / norm_avg, t2[1] / norm_avg, t2[2] / norm_avg];

    (avg, norm_avg)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_triangles_intersect_coplanar_separate() {
        // Two triangles in the same plane but not intersecting
        let t1 = [
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
        ];

        let t2 = [
            Coordinate3D::new__(2.0, 0.0, 0.0),
            Coordinate3D::new__(3.0, 0.0, 0.0),
            Coordinate3D::new__(2.0, 1.0, 0.0),
        ];

        assert!(!triangles_intersect(&t1, &t2));
        assert!(triangles_intersection(t1, t2).unwrap().is_none());
    }

    #[test]
    fn test_triangles_intersect_coplanar_overlapping() {
        // Two triangles in the same plane with overlapping edges
        let t1 = [
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(2.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 2.0, 0.0),
        ];

        let t2 = [
            Coordinate3D::new__(1.0, -1.0, 0.0),
            Coordinate3D::new__(3.0, -1.0, 0.0),
            Coordinate3D::new__(2.0, 1.0, 0.0),
        ];

        assert!(!triangles_intersect(&t1, &t2));
        assert!(triangles_intersection(t1, t2).unwrap().is_none());
    }

    #[test]
    fn test_triangles_intersect_perpendicular() {
        // Triangle in XY plane
        let t1 = [
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(2.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 2.0, 0.0),
        ];

        // Triangle in XZ plane intersecting the first
        let t2 = [
            Coordinate3D::new__(1.0, 1.0, -1.0),
            Coordinate3D::new__(1.0, 1.0, 1.0),
            Coordinate3D::new__(1.0, -1.0, 0.0),
        ];

        assert!(triangles_intersect(&t1, &t2));
        let intersection = triangles_intersection(t1, t2).unwrap().unwrap();
        assert_eq!(intersection[0], Coordinate3D::new__(1.0, 0.0, 0.0));
        assert_eq!(intersection[1], Coordinate3D::new__(1.0, 1.0, 0.0));
    }

    #[test]
    fn test_triangles_intersect_parallel_planes() {
        // Triangle in z=0 plane
        let t1 = [
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
        ];

        // Triangle in z=1 plane (parallel, no intersection)
        let t2 = [
            Coordinate3D::new__(0.0, 0.0, 1.0),
            Coordinate3D::new__(1.0, 0.0, 1.0),
            Coordinate3D::new__(0.0, 1.0, 1.0),
        ];

        assert!(!triangles_intersect(&t1, &t2));
        assert!(triangles_intersection(t1, t2).unwrap().is_none());
    }

    #[test]
    fn test_triangles_intersect_touching_vertex() {
        // First triangle
        let t1 = [
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
        ];

        // Second triangle touching at a vertex (should not count as intersection)
        let t2 = [
            Coordinate3D::new__(1.0, 1.0, 0.0),
            Coordinate3D::new__(2.0, 1.0, 0.0),
            Coordinate3D::new__(1.0, 2.0, 0.0),
        ];

        assert!(!triangles_intersect(&t1, &t2));
    }

    #[test]
    fn test_triangles_intersect_at_edge() {
        let t1 = [
            Coordinate3D::new__(-1.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, 2.0, 0.0),
        ];

        let t2 = [
            Coordinate3D::new__(-0.5, 0.0, 0.0),
            Coordinate3D::new__(0.5, 0.0, 0.0),
            Coordinate3D::new__(0.0, -1.5, 0.0),
        ];

        assert!(!triangles_intersect(&t1, &t2));
        assert!(triangles_intersection(t1, t2).unwrap().is_none());
    }
}
