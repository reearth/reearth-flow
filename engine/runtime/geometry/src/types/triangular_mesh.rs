use crate::algorithm::contains::Contains;
use crate::algorithm::triangle_intersection::{triangles_intersect, triangles_intersection};
use crate::algorithm::utils::{denormalize_vertices, normalize_vertices};
use crate::algorithm::{GeoFloat, GeoNum};
use crate::types::coordinate::{are_coplanar, Coordinate};
use crate::types::line_string::LineString3D;
use crate::types::polygon::Polygon3D;
use crate::types::triangle::Triangle;
use crate::types::{coordinate::Coordinate3D, coordnum::CoordNum, line::Line3D};
use crate::validation::{
    ValidationProblem, ValidationProblemAtPosition, ValidationProblemPosition,
    ValidationProblemReport, ValidationType,
};
use num_traits::{Float, FromPrimitive, NumCast};
use nusamai_projection::vshift::Jgd2011ToWgs84;
use serde::{Deserialize, Serialize};
use std::vec;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct TriangularMesh<T: CoordNum, Z: CoordNum = T> {
    // Vertices of the solid boundary. No duplicate vertices.
    vertices: Vec<Coordinate<T, Z>>,
    // Edges of the solid boundary with their multiplicity (i.e. how many faces share the edge).
    // they are defined as pairs of vertex indices, and the vertex indices in each pair are sorted.
    // The edges themselves are also sorted.
    edges_with_multiplicity: Vec<([usize; 2], usize)>,
    // Triangles of the solid boundary. Each triangle is represented by the indices of its vertices.
    // They are defined as triplets of vertex indices, and the vertex indices in each triangle are sorted.
    // The triangles themselves are also sorted.
    triangles: Vec<[usize; 3]>,
}

impl<T: CoordNum, Z: CoordNum> TriangularMesh<T, Z> {
    pub fn get_vertices(&self) -> &[Coordinate<T, Z>] {
        &self.vertices
    }
    pub fn get_vertices_mut(&mut self) -> &mut [Coordinate<T, Z>] {
        &mut self.vertices
    }

    pub fn get_triangles(&self) -> &[[usize; 3]] {
        &self.triangles
    }

    pub fn from_triangles(trinangles: Vec<[Coordinate<T, Z>; 3]>) -> Self {
        let mut mesh = Self::default();
        for triangle in trinangles {
            let mut tri_indices = [0usize; 3];
            for (i, &vertex) in triangle.iter().enumerate() {
                // Get or insert vertex index
                let vertex_index = match mesh.vertices.iter().position(|&v| v == vertex) {
                    Some(idx) => idx,
                    None => {
                        let idx = mesh.vertices.len();
                        mesh.vertices.push(vertex);
                        idx
                    }
                };

                tri_indices[i] = vertex_index;
            }

            // Add triangle
            tri_indices.sort_unstable();
            mesh.triangles.push(tri_indices);
        }
        mesh.edges_with_multiplicity = Self::compute_edges_with_multiplicity(&mesh.triangles);

        // Sort triangles for consistent representation
        mesh.triangles.sort_unstable();
        mesh.triangles.dedup();
        mesh
    }

    fn compute_edges_with_multiplicity(triangles: &[[usize; 3]]) -> Vec<([usize; 2], usize)> {
        let mut edges: Vec<[usize; 2]> = Vec::new();
        for triangle in triangles {
            for [i, j] in [[0, 1], [1, 2], [0, 2]] {
                let edge = if triangle[i] < triangle[j] {
                    [triangle[i], triangle[j]]
                } else {
                    [triangle[j], triangle[i]]
                };
                edges.push(edge);
            }
        }
        edges.sort_unstable();
        if edges.is_empty() {
            return Vec::new();
        }
        let mut edges_with_multiplicity = vec![(edges[0], 1)];
        edges_with_multiplicity.reserve(edges.len());

        for edge in edges.into_iter().skip(1) {
            let (last_edge, count) = edges_with_multiplicity.last_mut().unwrap();
            if *last_edge == edge {
                *count += 1;
            } else {
                edges_with_multiplicity.push((edge, 1));
            }
        }
        edges_with_multiplicity
    }
}

impl TriangularMesh<f64> {
    pub fn elevation(&self) -> f64 {
        self.vertices.first().map(|c| c.z).unwrap_or(0.0)
    }

    pub fn is_elevation_zero(&self) -> bool {
        self.vertices.iter().all(|c| c.z == 0.0)
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64> + FromPrimitive + GeoFloat + From<Z>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64> + FromPrimitive + GeoFloat,
    > TriangularMesh<T, Z>
{
    pub fn validate(&self, valid_type: ValidationType) -> Option<ValidationProblemReport> {
        match valid_type {
            ValidationType::CorruptGeometry => {
                // Check for degenerate triangles
                let mut problem_reports = Vec::new();
                for triangle_indices in self.triangles.iter() {
                    let a = self.vertices[triangle_indices[0]];
                    let b = self.vertices[triangle_indices[1]];
                    let c = self.vertices[triangle_indices[2]];

                    // TODO: use normalize_vertices function here. Currently it is not possible to use this function for 'Coordinate<T, Z>'.
                    // Need to wait refactoring of geometry types.
                    let (a, b, c) = {
                        let mean = Coordinate {
                            x: (a.x + b.x + c.x) / <T as NumCast>::from(3.0).unwrap(),
                            y: (a.y + b.y + c.y) / <T as NumCast>::from(3.0).unwrap(),
                            z: (a.z + b.z + c.z) / <Z as NumCast>::from(3.0).unwrap(),
                        };
                        let a = a - mean;
                        let b = b - mean;
                        let c = c - mean;
                        let a = Coordinate {
                            x: a.x / a.norm(),
                            y: a.y / a.norm(),
                            z: a.z / Z::from(a.norm()).unwrap(),
                        };
                        let b = Coordinate {
                            x: b.x / b.norm(),
                            y: b.y / b.norm(),
                            z: b.z / Z::from(b.norm()).unwrap(),
                        };
                        let c = Coordinate {
                            x: c.x / c.norm(),
                            y: c.y / c.norm(),
                            z: c.z / Z::from(c.norm()).unwrap(),
                        };
                        (a, b, c)
                    };

                    let ab = (b - a).norm();
                    let ac = (c - a).norm();
                    let bc = (c - b).norm();
                    let epsilon: f64 = <f64 as NumCast>::from(ab + bc + ac).unwrap() / (3.0 * 1e5);
                    let is_degenerate = (ab + ac).abs_diff_eq(&bc, epsilon)
                        || (ab + bc).abs_diff_eq(&ac, epsilon)
                        || (ac + bc).abs_diff_eq(&ab, epsilon);
                    if is_degenerate {
                        let report = ValidationProblemAtPosition(
                            ValidationProblem::DegenerateGeometry,
                            ValidationProblemPosition::Point,
                        );
                        problem_reports.push(report);
                    }
                }

                if problem_reports.is_empty() {
                    None
                } else {
                    Some(ValidationProblemReport(problem_reports))
                }
            }
            _ => unimplemented!(),
        }
    }
}

impl<T: Float + CoordNum> TriangularMesh<T> {
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
            && self.triangles.is_empty()
            && self.edges_with_multiplicity.is_empty()
    }

    /// Create a triangular mesh from a list of faces by triangulating each face.
    pub fn from_faces(faces: &[LineString3D<T>]) -> Result<Self, String> {
        let epsilon = T::from(1e-10).unwrap();
        let mut out = Self::default();
        let mut vertices = Vec::new();
        for v in faces.iter().flat_map(|f| f.0.iter()) {
            if vertices
                .iter()
                .all(|&existing_v: &Coordinate3D<T>| (existing_v - *v).norm() >= epsilon)
            {
                vertices.push(*v);
            }
        }

        for face in faces {
            // Triangulate the face
            let face_triangles = match Self::triangulate_face(face.clone()) {
                Ok(triangles) => triangles,
                Err(err) => {
                    // If triangulation fails, then return empty mesh
                    return Err(format!(
                        "Failed to triangulate face: {face:?}\n, error: {err}",
                    ));
                }
            };

            for triangle in face_triangles {
                let mut tri_indices = [0usize; 3];
                for (i, &vertex) in triangle.iter().enumerate() {
                    // Get or insert vertex index
                    let vertex_index =
                        match vertices.iter().position(|&v| (v - vertex).norm() < epsilon) {
                            Some(idx) => idx,
                            None => {
                                let idx = out.vertices.len();
                                out.vertices.push(vertex);
                                idx
                            }
                        };

                    tri_indices[i] = vertex_index;
                }

                // Add triangle
                tri_indices.sort_unstable();
                out.triangles.push(tri_indices);
            }
        }
        out.edges_with_multiplicity = Self::compute_edges_with_multiplicity(&out.triangles);

        // Sort triangles for consistent representation
        out.triangles.sort_unstable();
        out.triangles.dedup();
        out.vertices = vertices;
        Ok(out)
    }

    /// triangulates the input face.
    /// We assume the following conditions about the face:
    /// - The face is planar
    /// - The face contour is closed (i.e. the first point is the same as the last point)
    /// - The face does not intersect itself
    fn triangulate_face(face: LineString3D<T>) -> Result<Vec<[Coordinate3D<T>; 3]>, String> {
        let mut face = face.0;
        let norm = normalize_vertices(&mut face);
        // face at least must be triangle
        if face.len() < 4 {
            return Err("Face must have at least 3 vertices")?;
        }

        let tau = T::from(std::f64::consts::TAU).unwrap();
        let pi = T::from(std::f64::consts::PI).unwrap();
        let epsilon = T::from(1e-10).unwrap();

        // remove the last point
        face.pop();

        // Compute the face normal. We assume the face is planar as this is not the validation process for the solid face.
        let normal = (0..face.len())
            .map(|i| {
                let p0 = face[i];
                let p1 = face[(i + 1) % face.len()];
                let p2 = face[(i + 2) % face.len()];
                let v1 = p1 - p0;
                let v2 = p2 - p0;
                let n = v1.cross(&v2);
                (n, n.norm())
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap()
            .0
            .normalize();

        // Compute the angle at each vertex
        let mut angles = Vec::new();
        let mut hair_count = 0;
        for i in 0..face.len() {
            let p0 = face[(i + face.len() - 1) % face.len()];
            let p1 = face[i];
            let p2 = face[(i + 1) % face.len()];
            let v1 = p0 - p1;
            let v2 = p2 - p1;
            let mut angle = v1.angle(&v2);
            let cross_norm = v1.cross(&v2).norm();

            // Take the sign of the orientation into account
            angle = if cross_norm < epsilon {
                if v1.dot(&v2) < T::zero() {
                    // Vectors are nearly collinear but pointing in opposite directions
                    // Inner angle is approximately π, so outer angle should be computed normally
                    // We need to determine the sign based on which side the slight deviation is
                    if !normal.dot(&v1.cross(&v2)).is_sign_positive() {
                        pi - angle
                    } else {
                        angle - pi
                    }
                } else {
                    // Vectors point in same direction (hairpin turn)
                    hair_count += 1;
                    -pi
                }
            } else if !normal.dot(&v1.cross(&v2)).is_sign_positive() {
                // General positive case
                pi - angle
            } else {
                // General negative case
                angle - pi
            };
            angles.push(angle);
        }

        // The sum of the angles is either TAU or -TAU.
        // If the sum is -TAU, flip the angles.
        // otherwise something about the assumption is wrong
        let angle_sum: T = angles.iter().copied().fold(T::zero(), |acc, x| acc + x);
        let sum_epsilon = T::from(1e-4).unwrap();
        if (angle_sum + tau + T::from(hair_count).unwrap() * tau).abs() < sum_epsilon {
            // if the sum is -TAU, flip the angles
            angles = angles
                .iter()
                .map(|&a| if a == -pi { a } else { -a })
                .collect();
        } else if (angle_sum.abs() - tau).abs() >= sum_epsilon {
            // something is wrong with the assumption
            return Err(format!(
                "Face is likely not closed as the outer angle is: {angle_sum:?}. Face: {face:?}"
            ))?;
        } // otherwise the sum is TAU and we are good to go

        let angle_sum: T = angles.iter().copied().fold(T::zero(), |acc, x| acc + x);
        if (angle_sum + tau).abs() < sum_epsilon {
            // Case of a degenerate face
            return Err(format!("Face is likely degenerate: {face:?}"))?;
        }

        // Triangulate the face by the following process:
        // 1. Find the vertex with the positive angle.
        // 2. Create a triangle with the two adjacent vertices.
        // 3. Remove the vertex from the face.
        // 4. Update the angles of the adjacent vertices.
        // 5. Repeat until the face boundary is empty.
        let mut triangles = Vec::new();
        while !face.is_empty() {
            // Find the vertex with the positive outer angle
            // A polygon in an Euclidean space must have at least one convex vertex
            let removed_vtx_idx = angles
                .iter()
                .enumerate()
                .filter(|&(_i, &a)| a > epsilon)
                .map(|(i, _a)| i)
                .find(|&i| {
                    let t = Triangle::new(
                        face[(i + face.len() - 1) % face.len()],
                        face[i],
                        face[(i + 1) % face.len()],
                    );
                    (i + 2..).take(face.len() - 3).all(|j: usize| {
                        let j = face[j % face.len()];
                        (t.0 - j).norm() < epsilon
                            || (t.1 - j).norm() < epsilon
                            || (t.2 - j).norm() < epsilon
                            || !t.contains(&j) && !t.boundary_contains(&j)
                    })
                })
                .ok_or("No convex vertex found")?;

            // Create a triangle with the two adjacent vertices
            let prev_idx = (removed_vtx_idx + face.len() - 1) % face.len();
            let next_idx = (removed_vtx_idx + 1) % face.len();
            triangles.push([face[prev_idx], face[removed_vtx_idx], face[next_idx]]);

            // compute the angles of the triangle vertices at the previous and next vertices.
            // Since `removed_vtx_idx` is a convex vertex, the angles at the previous and next vertices must be positive.
            let vp = face[prev_idx] - face[removed_vtx_idx];
            let vn = face[next_idx] - face[removed_vtx_idx];
            let vpn = face[next_idx] - face[prev_idx];
            let prev_angle = (-vp).angle(&vpn);
            let next_angle = vn.angle(&vpn);

            // Remove the vertex from the face
            face.remove(removed_vtx_idx);
            angles.remove(removed_vtx_idx);

            // Update the angles of the adjacent vertices
            if face.len() < 3 {
                break;
            }
            let new_prev_idx = (removed_vtx_idx + face.len() - 1) % face.len();
            let new_next_idx = removed_vtx_idx % face.len();
            angles[new_prev_idx] = angles[new_prev_idx] + prev_angle;
            angles[new_next_idx] = angles[new_next_idx] + next_angle;
        }

        for t in triangles.iter_mut() {
            denormalize_vertices(t, norm);
        }

        Ok(triangles)
    }

    pub fn edges_violating_manifold_condition(&self) -> Vec<Line3D<T>> {
        let mut non_manifold_edges = Vec::new();

        for &(edge, count) in &self.edges_with_multiplicity {
            if count != 2 {
                non_manifold_edges.push(edge);
            }
        }

        non_manifold_edges
            .into_iter()
            .map(|e| Line3D::new_(self.vertices[e[0]], self.vertices[e[1]]))
            .collect::<Vec<_>>()
    }

    /// Check if the solid boundary is orientable
    /// This function assumes the following conditions:
    /// - The solid boundary is a 2-manifold (i.e. each edge is shared by exactly two triangles)
    /// - The solid boundary is connected
    /// - 'edges_with_multiplicity' is sorted by the edge's vertex indices, and each of its edges is sorted by the vertex indices.
    pub fn is_orientable(&self) -> bool {
        if self.triangles.is_empty() {
            return true;
        }
        // Build adjacency information
        let mut edge_to_triangles: Vec<[usize; 2]> =
            vec![[usize::MAX; 2]; self.edges_with_multiplicity.len()];
        let mut triangle_to_edges: Vec<[(usize, bool); 3]> =
            vec![[(usize::MAX, false); 3]; self.triangles.len()]; // Initialize with (MAX, false)
        let mut orientation: Vec<Option<bool>> = vec![None; self.triangles.len()];

        // Compute the edge-triangle mappings
        for (t_idx, triangle) in self.triangles.iter().enumerate() {
            for [i, j] in [[0, 1], [1, 2], [0, 2]] {
                let edge = if triangle[i] < triangle[j] {
                    [triangle[i], triangle[j]]
                } else {
                    [triangle[j], triangle[i]]
                };
                if let Ok(e_idx) = self
                    .edges_with_multiplicity
                    .binary_search_by_key(&edge, |(e, _)| *e)
                {
                    if edge_to_triangles[e_idx][0] == usize::MAX {
                        edge_to_triangles[e_idx][0] = t_idx;
                    } else if edge_to_triangles[e_idx][1] == usize::MAX {
                        edge_to_triangles[e_idx][1] = t_idx;
                    }
                    let orientation = {
                        let edge = self.edges_with_multiplicity[e_idx].0;
                        (edge[0] == triangle[0] && edge[1] == triangle[1])
                            || (edge[0] == triangle[1] && edge[1] == triangle[2])
                    };
                    if triangle_to_edges[t_idx][0].0 == usize::MAX {
                        triangle_to_edges[t_idx][0] = (e_idx, orientation);
                    } else if triangle_to_edges[t_idx][1].0 == usize::MAX {
                        triangle_to_edges[t_idx][1] = (e_idx, orientation);
                    } else if triangle_to_edges[t_idx][2].0 == usize::MAX {
                        triangle_to_edges[t_idx][2] = (e_idx, orientation);
                        triangle_to_edges[t_idx].sort_unstable();
                    }
                }
            }
        }

        // DFS to assign orientations
        let mut stack = vec![0];
        orientation[0] = Some(false);
        while let Some(curr_face_idx) = stack.pop() {
            let curr_orientation = orientation[curr_face_idx].unwrap();

            // Visit adjacent triangles
            for &(e_idx, direction) in &triangle_to_edges[curr_face_idx] {
                let adj_faces = edge_to_triangles[e_idx];
                let adj_face_idx = if adj_faces[0] == curr_face_idx {
                    adj_faces[1]
                } else {
                    adj_faces[0]
                };

                // Skip if the adjacent face is not valid (boundary edge)
                if adj_face_idx == usize::MAX || adj_face_idx >= self.triangles.len() {
                    continue;
                }

                let o_adj_face_idx = {
                    let other_direction = triangle_to_edges[adj_face_idx]
                        .iter()
                        .find(|&&(edge_idx, _)| edge_idx == e_idx)
                        .map(|&(_, dir)| dir)
                        .unwrap();
                    if direction == other_direction {
                        !curr_orientation
                    } else {
                        curr_orientation
                    }
                };

                if let Some(o_adj_face_idx_prev_computed) = orientation[adj_face_idx] {
                    if o_adj_face_idx != o_adj_face_idx_prev_computed {
                        // Conflict in orientation
                        return false;
                    } // else, orientations match, no action needed
                } else {
                    // Assign the orientation to the adjacent face
                    orientation[adj_face_idx] = Some(o_adj_face_idx);
                    stack.push(adj_face_idx);
                }
            }
        }

        true
    }

    pub fn is_connected(&self) -> bool {
        let num_vertices = self.vertices.len();
        if num_vertices == 0 {
            return true;
        }
        // Check if the solid boundary is connected
        let mut visited = vec![false; num_vertices];
        let mut stack = vec![0];
        visited[0] = true;
        let mut edges = self
            .edges_with_multiplicity
            .iter()
            .map(|(e, _)| *e)
            .collect::<Vec<_>>();
        while let Some(v) = stack.pop() {
            let mut edges_ = Vec::with_capacity(edges.len());
            for edge in edges {
                if edge[0] == v && !visited[edge[1]] {
                    visited[edge[1]] = true;
                    stack.push(edge[1]);
                } else if edge[1] == v && !visited[edge[0]] {
                    visited[edge[0]] = true;
                    stack.push(edge[0]);
                } else {
                    edges_.push(edge);
                }
            }
            edges = edges_;
        }
        visited.iter().all(|&v| v)
    }

    /// returns true if the solid bounded by the triangular mesh contains the point.
    /// This function assumes the following conditions:
    /// - The solid boundary is an orientable 2-manifold.
    /// - The solid boundary is connected.
    pub fn bounding_solid_contains(&self, point: &Coordinate3D<T>) -> bool {
        if self.triangles.is_empty() {
            return false;
        }
        if self.contains(point) {
            return true;
        }
        // (face index, orientation difference)
        let face_adjacency: Vec<Vec<(usize, bool)>> = {
            let mut adjacency = vec![Vec::new(); self.triangles.len()];
            let mut boundary_2 = self
                .triangles
                .iter()
                .enumerate()
                .flat_map(|(i, t)| [[t[0], t[1], i], [t[1], t[2], i], [t[0], t[2], i]])
                .collect::<Vec<_>>();
            boundary_2.sort_unstable();
            for edge in boundary_2.windows(2) {
                let e1 = edge[0];
                let e2 = edge[1];
                let o_e1 = {
                    let t = self.triangles[e1[2]];
                    e1[0] == t[0] && e1[1] == t[1] || e1[0] == t[1] && e1[1] == t[2]
                };
                let o_e2 = {
                    let t = self.triangles[e2[2]];
                    e2[0] == t[0] && e2[1] == t[1] || e2[0] == t[1] && e2[1] == t[2]
                };
                if e1[0..2] == e2[0..2] {
                    adjacency[e1[2]].push((e2[2], o_e1 == o_e2));
                    adjacency[e2[2]].push((e1[2], o_e1 == o_e2));
                }
            }
            adjacency
        };

        let mut stack = vec![(0, false)]; // (face index, orientation)
        let mut visited = vec![false; self.triangles.len()];

        let mut solid_angle = T::zero();

        while let Some((curr_face_idx, orientation)) = stack.pop() {
            if visited[curr_face_idx] {
                continue;
            }
            visited[curr_face_idx] = true;

            // compute the partial solid angle
            let face = self.triangles[curr_face_idx];
            let a = self.vertices[face[0]];
            let b = self.vertices[face[1]];
            let c = self.vertices[face[2]];
            if orientation {
                solid_angle = solid_angle + Self::solid_angle_of_triangle(a, b, c, *point);
            } else {
                solid_angle = solid_angle + Self::solid_angle_of_triangle(a, c, b, *point);
            }

            // Visit adjacent triangles
            for &(adj_face_idx, o_diff) in &face_adjacency[curr_face_idx] {
                stack.push((adj_face_idx, o_diff ^ orientation));
            }
        }
        solid_angle.abs() > T::from(std::f64::consts::TAU).unwrap()
    }

    /// returns true if the point is on the mesh surface or not.
    pub fn contains(&self, point: &Coordinate3D<T>) -> bool {
        let epsilon = T::from(1e-5).unwrap();
        let triangles = self.triangles.clone();
        // quick check with bounding box
        let triangles = triangles
            .into_iter()
            .filter_map(|triangle| {
                let v0 = self.vertices[triangle[0]];
                let v1 = self.vertices[triangle[1]];
                let v2 = self.vertices[triangle[2]];
                let bbox_min = Coordinate3D {
                    x: v0.x.min(v1.x).min(v2.x) - epsilon,
                    y: v0.y.min(v1.y).min(v2.y) - epsilon,
                    z: v0.z.min(v1.z).min(v2.z) - epsilon,
                };
                let bbox_max = Coordinate3D {
                    x: v0.x.max(v1.x).max(v2.x) + epsilon,
                    y: v0.y.max(v1.y).max(v2.y) + epsilon,
                    z: v0.z.max(v1.z).max(v2.z) + epsilon,
                };
                if point.x >= bbox_min.x
                    && point.x <= bbox_max.x
                    && point.y >= bbox_min.y
                    && point.y <= bbox_max.y
                    && point.z >= bbox_min.z
                    && point.z <= bbox_max.z
                {
                    Some([v0, v1, v2])
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for triangle in triangles {
            let t = Triangle::new(triangle[0], triangle[1], triangle[2]);
            if t.contains(point) {
                return true;
            }
        }

        false
    }

    fn solid_angle_of_triangle(
        a: Coordinate3D<T>,
        b: Coordinate3D<T>,
        c: Coordinate3D<T>,
        p: Coordinate3D<T>,
    ) -> T {
        let pa = a - p;
        let pb = b - p;
        let pc = c - p;

        let la = pa.norm();
        let lb = pb.norm();
        let lc = pc.norm();

        let numerator = pa.dot(&pb.cross(&pc));
        let denominator = la * lb * lc + pa.dot(&pb) * lc + pa.dot(&pc) * lb + pb.dot(&pc) * la;

        let two = T::from(2.0).unwrap();
        two * numerator.atan2(denominator)
    }

    /// decomposes the triangular mesh into a collection of 2-manifolds with boundaries.
    pub fn into_2_manifolds_with_boundaries(&self) -> Vec<Vec<[usize; 3]>> {
        let face_adjacency: Vec<Vec<usize>> = {
            let mut adjacency = vec![Vec::new(); self.triangles.len()];
            let mut boundary_2 = self
                .triangles
                .iter()
                .enumerate()
                .flat_map(|(i, t)| [[t[0], t[1], i], [t[1], t[2], i], [t[0], t[2], i]])
                .collect::<Vec<_>>();
            boundary_2.sort_unstable();
            let boundary_2 = boundary_2.into_iter();
            // group faces by edges
            let mut edge_to_faces: Vec<Vec<usize>> = Vec::new();
            let mut prev_edge = [usize::MAX; 2];
            for edge in boundary_2 {
                if prev_edge == edge[0..2] {
                    edge_to_faces.last_mut().unwrap().push(edge[2]);
                } else {
                    edge_to_faces.push(vec![edge[2]]);
                    prev_edge = [edge[0], edge[1]];
                }
            }
            for faces in edge_to_faces {
                // we only care about edges shared by exactly two faces.
                // If an edge is shared by one face, it is a boundary edge, so there is no adjacent.
                // If an edge is shared by more than two faces, it is a non-manifold edge, so we skip it.
                if faces.len() == 2 {
                    adjacency[faces[0]].push(faces[1]);
                    adjacency[faces[1]].push(faces[0]);
                }
            }
            adjacency
        };

        let mut visited = vec![false; self.triangles.len()];
        let mut components = Vec::new();
        while let Some(curr_face_idx) = visited.iter().position(|&v| !v) {
            let mut stack = vec![curr_face_idx];
            let mut component = Vec::new();
            while let Some(curr_face_idx) = stack.pop() {
                if visited[curr_face_idx] {
                    continue;
                }
                visited[curr_face_idx] = true;
                component.push(self.triangles[curr_face_idx]);
                for &adj_face_idx in &face_adjacency[curr_face_idx] {
                    stack.push(adj_face_idx);
                }
            }
            component.sort_unstable();
            components.push(component);
        }
        components
    }

    /// Retains only the faces specified in `faces` and removes all other faces from the mesh.
    /// Also removes any vertices and edges that are no longer used by any face.
    pub fn retain_faces(&mut self, faces: &[[usize; 3]]) {
        let used_vertices: Vec<bool> = {
            let mut used = vec![false; self.vertices.len()];
            for face in faces {
                for &v in face {
                    used[v] = true;
                }
            }
            used
        };
        let vertex_mapping: Vec<usize> = {
            let mut mapping = vec![usize::MAX; self.vertices.len()];
            let mut new_idx = 0;
            for (i, &used) in used_vertices.iter().enumerate() {
                if used {
                    mapping[i] = new_idx;
                    new_idx += 1;
                }
            }
            mapping
        };

        self.vertices = self
            .vertices
            .iter()
            .enumerate()
            .filter_map(|(i, v)| if used_vertices[i] { Some(*v) } else { None })
            .collect();
        self.triangles = faces
            .iter()
            .map(|f| {
                [
                    vertex_mapping[f[0]],
                    vertex_mapping[f[1]],
                    vertex_mapping[f[2]],
                ]
            })
            .collect();
        self.triangles.sort_unstable();
        self.edges_with_multiplicity = Self::compute_edges_with_multiplicity(&self.triangles);
    }
}

impl TryFrom<Vec<Polygon3D<f64>>> for TriangularMesh<f64> {
    type Error = String;
    fn try_from(faces: Vec<Polygon3D<f64>>) -> Result<Self, String> {
        let mut new_faces: Vec<super::line_string::LineString> = Vec::new();
        for f in faces {
            new_faces.push(f.into_merged_contour()?);
        }
        Self::from_faces(&new_faces)
    }
}

impl TriangularMesh<f64> {
    pub fn create_simple_obj(&self, output_path: Option<&str>) {
        use std::io::Write;
        let filename = output_path.unwrap_or("union_output.obj");
        let mut file = std::fs::File::create(filename).unwrap();
        for v in &self.vertices {
            writeln!(file, "v {:.10?} {:.10?} {:.10?}", v.x, v.y, v.z).unwrap();
        }
        for t in &self.triangles {
            writeln!(
                file,
                "f {:.10?} {:.10?} {:.10?}",
                t[0] + 1,
                t[1] + 1,
                t[2] + 1
            )
            .unwrap();
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn self_intersection(&self) -> Vec<([Coordinate3D<f64>; 3], [Coordinate3D<f64>; 3])> {
        let mut intersection = Vec::new();
        for i in 0..self.triangles.len() {
            let tri1 = &self.triangles[i];
            let v0 = self.vertices[tri1[0]];
            let v1 = self.vertices[tri1[1]];
            let v2 = self.vertices[tri1[2]];

            for (j, tri2) in self.triangles.iter().enumerate().skip(i + 1) {
                // Skip if they share a vertex
                if tri1.iter().any(|&v| tri2.contains(&v)) {
                    continue;
                }

                let w0 = self.vertices[tri2[0]];
                let w1 = self.vertices[tri2[1]];
                let w2 = self.vertices[tri2[2]];

                let t: [Coordinate3D<f64>; 3] = [v0, v1, v2];
                let s: [Coordinate3D<f64>; 3] = [w0, w1, w2];

                if triangles_intersect(&t, &s) {
                    intersection.push((i, j));
                }
            }
        }
        intersection
            .into_iter()
            .map(|(i, j)| {
                let t = self.triangles[i];
                let s = self.triangles[j];
                let t = [
                    self.vertices[t[0]],
                    self.vertices[t[1]],
                    self.vertices[t[2]],
                ];
                let s = [
                    self.vertices[s[0]],
                    self.vertices[s[1]],
                    self.vertices[s[2]],
                ];
                (t, s)
            })
            .collect()
    }

    pub(crate) fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        self.vertices
            .iter_mut()
            .for_each(|c| c.transform_inplace(jgd2wgs));
    }

    pub(crate) fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        self.vertices
            .iter_mut()
            .for_each(|c| c.transform_offset(x, y, z));
    }

    /// takes in another triangular mesh and returns a new triangular mesh representing the union of the two meshes.
    /// When two meshes intersect, there will be new vertices and edges created at the intersection.
    /// When the two meshes intersect at a face (i.e. they have the identical face), then the face will be merged.
    pub fn union(self, other: Self) -> Result<Self, String> {
        let Self {
            vertices: vertices1,
            triangles: triangles1,
            edges_with_multiplicity: edges1,
        } = self;
        let Self {
            vertices: vertices2,
            triangles: mut triangles2,
            edges_with_multiplicity: edges2,
        } = other;

        if !triangles1.is_sorted() || !triangles2.is_sorted() {
            return Err("Triangles are not sorted")?;
        }
        if !edges1.iter().all(|(e, _)| e.is_sorted()) || !edges2.iter().all(|(e, _)| e.is_sorted())
        {
            return Err("Edges are not sorted")?;
        }
        if !edges1.is_sorted() || !edges2.is_sorted() {
            return Err("Edges are not sorted")?;
        }

        triangles2.iter_mut().for_each(|tri| {
            for v in tri.iter_mut() {
                *v += vertices1.len();
            }
        });

        let num_orig_triangles = triangles1.len() + triangles2.len();
        let edges = {
            let mut edges = edges1
                .into_iter()
                .map(|(e, _)| e)
                .chain(
                    edges2
                        .into_iter()
                        .map(|(e, _)| [e[0] + vertices1.len(), e[1] + vertices1.len()]),
                )
                .collect::<Vec<_>>();
            edges.sort_unstable();
            edges.dedup();
            edges
        };
        let num_orig_edges = edges.len();
        let mut edges_cuts = vec![Vec::new(); num_orig_edges];
        let mut vertices = vertices1
            .clone() // TODO: remove clone
            .into_iter()
            .chain(vertices2.clone()) // TODO: remove clone
            .collect::<Vec<_>>();
        let norm = normalize_vertices(&mut vertices);
        let triangles = triangles1
            .iter()
            .copied()
            .chain(triangles2.iter().copied())
            .collect::<Vec<_>>();

        let vertex_map = {
            let mut map = std::collections::HashMap::new();
            for (i, &v) in vertices.iter().enumerate().skip(vertices1.len()) {
                if let Some(pos) = vertices
                    .iter()
                    .take(vertices1.len())
                    .position(|&w| (w - v).norm() < 1e-10)
                {
                    map.insert(i, pos);
                }
            }
            map
        };

        let mut vertex_to_remove = Vec::new();
        for (&k, &v) in vertex_map.iter() {
            if k != v {
                vertex_to_remove.push(k);
            }
        }
        vertex_to_remove.sort_unstable();
        let num_original_vertices = vertices.len();
        for &v in vertex_to_remove.iter().rev() {
            vertices.remove(v);
        }

        // Now we reflect the vertex removal in the vertex map
        let vertex_map = {
            let mut map = (0..num_original_vertices).collect::<Vec<_>>();
            let mut count = 0;
            vertex_to_remove.push(num_original_vertices);
            for wdw in vertex_to_remove.windows(2) {
                count += 1;
                let start = wdw[0];
                let end = wdw[1];
                for (i, v) in map.iter_mut().enumerate().take(end).skip(start + 1) {
                    *v = i - count;
                }
            }
            for (k, v) in vertex_map {
                map[k] = v;
            }
            map
        };

        let mut triangles = triangles
            .into_iter()
            .map(|t| [vertex_map[t[0]], vertex_map[t[1]], vertex_map[t[2]]])
            .collect::<Vec<_>>();
        triangles.sort_unstable();
        let mut triangles2 = triangles2
            .into_iter()
            .map(|t| [vertex_map[t[0]], vertex_map[t[1]], vertex_map[t[2]]])
            .collect::<Vec<_>>();
        triangles2.sort_unstable();
        let mut edges = edges
            .into_iter()
            .map(|e| [vertex_map[e[0]], vertex_map[e[1]]])
            .collect::<Vec<_>>();
        edges.sort_unstable();

        let mut intersections = vec![Vec::new(); num_orig_triangles];
        let mut additional_edges = Vec::new();
        for (ti, t) in triangles1.iter().copied().enumerate() {
            let tt = [vertices[t[0]], vertices[t[1]], vertices[t[2]]];
            for (si, s) in triangles2.iter().copied().enumerate() {
                let ss: [Coordinate; 3] = [vertices[s[0]], vertices[s[1]], vertices[s[2]]];
                let Some(intersection) = triangles_intersection(tt, ss)
                    .map_err(|e| format!("Error in triangle-triangle intersection: {e}"))?
                else {
                    continue;
                };
                intersections[ti].push(intersection);
                intersections[si + triangles1.len()].push(intersection);
                let v1 = if let Some(idx) = vertices
                    .iter()
                    .position(|&x| (x - intersection[0]).norm() < 1e-10)
                {
                    idx
                } else {
                    vertices.push(intersection[0]);
                    // need to check if the intersection point is on the edge of the triangle.
                    // if it is, then we need to remove the edge and add two new edges.
                    let mut divide_edge = |vv, ww, v, w| -> Result<(), String> {
                        if Line3D::new_(vv, ww).contains(intersection[0]) {
                            let e_idx = edges.binary_search(&[v, w]).map_err(|_| {
                                format!("Edge not found: edges: {edges:?}, v: {v}, w: {w}")
                            })?;
                            edges_cuts[e_idx].push(vertices.len() - 1);
                        };
                        Ok(())
                    };
                    divide_edge(tt[0], tt[1], t[0], t[1])?;
                    divide_edge(tt[1], tt[2], t[1], t[2])?;
                    divide_edge(tt[0], tt[2], t[0], t[2])?;
                    divide_edge(ss[0], ss[1], s[0], s[1])?;
                    divide_edge(ss[1], ss[2], s[1], s[2])?;
                    divide_edge(ss[0], ss[2], s[0], s[2])?;
                    vertices.len() - 1
                };
                let v2 = if let Some(idx) = vertices
                    .iter()
                    .position(|&x| (x - intersection[1]).norm() < 1e-10)
                {
                    idx
                } else {
                    vertices.push(intersection[1]);
                    let mut divide_edge = |vv, ww, v, w| -> Result<(), String> {
                        if Line3D::new_(vv, ww).contains(intersection[1]) {
                            let e_idx = edges.binary_search(&[v, w]).map_err(|_| {
                                format!("Edge not found: edges: {edges:?}, v: {v}, w: {w}")
                            })?;
                            edges_cuts[e_idx].push(vertices.len() - 1);
                        };
                        Ok(())
                    };
                    divide_edge(tt[0], tt[1], t[0], t[1])?;
                    divide_edge(tt[1], tt[2], t[1], t[2])?;
                    divide_edge(tt[0], tt[2], t[0], t[2])?;
                    divide_edge(ss[0], ss[1], s[0], s[1])?;
                    divide_edge(ss[1], ss[2], s[1], s[2])?;
                    divide_edge(ss[0], ss[2], s[0], s[2])?;
                    vertices.len() - 1
                };
                if v1 < v2 {
                    additional_edges.push([v1, v2]);
                } else {
                    additional_edges.push([v2, v1]);
                }
            }
        }
        // quick return when there is no intersection.
        if intersections
            .iter()
            .all(|intersection| intersection.is_empty())
        {
            let edges_with_multiplicity = Self::compute_edges_with_multiplicity(&triangles);
            let result = Self {
                vertices,
                triangles,
                edges_with_multiplicity,
            };
            return Ok(result);
        };
        edges_cuts.resize(edges.len() + additional_edges.len(), Vec::new());
        edges.append(&mut additional_edges);
        let mut edges_with_cuts = edges.into_iter().zip(edges_cuts).collect::<Vec<_>>();
        edges_with_cuts.sort_unstable_by_key(|(e, _)| *e);

        // Reflect the edge cuts in the edges list.
        let mut edges = {
            let mut edges = edges_with_cuts
                .into_iter()
                .flat_map(|(edge, mut cuts)| {
                    // sort the cuts in order that cuts closer to edge[0] come first.
                    cuts.sort_by(|&v, &w| {
                        (vertices[v] - vertices[edge[0]])
                            .norm()
                            .partial_cmp(&(vertices[w] - vertices[edge[0]]).norm())
                            .unwrap()
                    });
                    cuts.dedup();
                    let mut result = Vec::new();
                    let mut last_v = edge[0];
                    for &cut in &cuts {
                        result.push([last_v, cut]);
                        last_v = cut;
                    }
                    result.push([last_v, edge[1]]);
                    result.last_mut().unwrap().sort(); // Only the last one may be unsorted, so we sort it.
                    result
                })
                .collect::<Vec<_>>();
            edges.sort_unstable();
            edges.dedup();
            edges
        };

        // Cut the edges further if there are vertices on the edge that are not from intersection.
        // This can happen when a vertex of one mesh is on the edge of another mesh.
        let mut updated = true;
        while updated {
            updated = false;
            let mut new_edges = Vec::new();
            for e in &mut edges {
                for i in 0..vertices.len() {
                    let v = &vertices[i];
                    let line = Line3D::new_(vertices[e[0]], vertices[e[1]]);
                    if (vertices[e[0]] - *v).norm() < 1e-10 || (vertices[e[1]] - *v).norm() < 1e-10
                    {
                        continue;
                    }
                    if line.contains(*v) {
                        updated = true;
                        let new_edge = if e[1] < i { [e[1], i] } else { [i, e[1]] };
                        new_edges.push(new_edge);
                        e[1] = i;
                        continue; // the same edge should not be split more than once.
                    }
                }
            }
            edges.append(&mut new_edges);
        }
        edges.sort_unstable();
        edges.dedup();

        // Also create the vertex-adjacency list for easy traversal.
        let vertex_adj: Vec<Vec<usize>> = {
            let mut out = vec![Vec::new(); vertices.len()];
            for edge in &edges {
                out[edge[0]].push(edge[1]);
                out[edge[1]].push(edge[0]);
            }
            out
        };

        let intersections = intersections
            .into_iter()
            .map(|intersection| {
                intersection
                    .into_iter()
                    .map(|edge| {
                        // TODO: Optimize these linear-time lookups. (No hash map, because they are floats.)
                        let v1 = (0..vertices.len())
                            .find(|&v| (vertices[v] - edge[0]).norm() < 1e-10)
                            .unwrap();
                        let v2 = (0..vertices.len())
                            .find(|&v| (vertices[v] - edge[1]).norm() < 1e-10)
                            .unwrap();
                        if v1 < v2 {
                            [v1, v2]
                        } else {
                            [v2, v1]
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // For each triangle, create polygons by dividing the triangle along the intersection edges.
        let mut line_strings = Vec::new();
        for (t, mut intersection) in triangles1.into_iter().chain(triangles2).zip(intersections) {
            let mut line_strings_t = Vec::new();
            while let Some(last) = intersection.pop() {
                let mut line_str_coords = vec![last[1], last[0]];
                // ToDo: This can be optimized.
                let mut next_vertex = |line_str_coords: &Vec<usize>| {
                    let last_vertex = *line_str_coords.last().unwrap();
                    let t = Triangle::new(vertices[t[0]], vertices[t[1]], vertices[t[2]]);
                    if t.boundary_contains(&vertices[last_vertex]) {
                        return None;
                    }
                    let pos = intersection.iter().position(|e| e.contains(&last_vertex));
                    if let Some(pos) = pos {
                        let e = intersection.remove(pos);
                        if e[0] == *line_str_coords.last().unwrap() {
                            Some(e[1])
                        } else if e[1] == *line_str_coords.last().unwrap() {
                            Some(e[0])
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };
                while let Some(vertex) = next_vertex(&line_str_coords) {
                    line_str_coords.push(vertex);
                    if line_str_coords.len() > 1000 {
                        return Err(
                            "Infinite loop detected in line string construction".to_string()
                        );
                    }
                }
                line_str_coords.reverse();
                while let Some(vertex) = next_vertex(&line_str_coords) {
                    line_str_coords.push(vertex);
                    if line_str_coords.len() > 1000 {
                        return Err(
                            "Infinite loop detected in line string construction".to_string()
                        );
                    }
                }

                line_strings_t.push(line_str_coords);
            }
            line_strings.push((line_strings_t, t));
            if line_strings.last().unwrap().0.is_empty() {
                // No intersection
                let ls = vec![t[0], t[1], t[2], t[0]];
                line_strings.last_mut().unwrap().0 = vec![ls];
            }
        }

        let mut polygons = Vec::new();
        let mut interior_boundary = Vec::new();
        let mut polygon_added = vec![false; num_orig_triangles];
        for (lss, t) in line_strings {
            // This means that the line string is open at the edge of the triangle.
            // We need to close the string to create a polygon.

            let v1 = vertices[t[0]];
            let v2 = vertices[t[1]];
            let v3 = vertices[t[2]];
            let n = (v2 - v1).cross(&(v3 - v1)).normalize();
            // closure to compute the angle between two vectors.
            // These two vectors are assumed to be in the plane defined by `n` and return `None` if not.
            let angle = |a: Coordinate3D<f64>, b: Coordinate3D<f64>, o: Coordinate3D<f64>| {
                let epsilon = 1e-10;
                let oa = (a - o).normalize();
                let ob = (b - o).normalize();
                if n.dot(&oa).abs() > epsilon || n.dot(&ob).abs() > epsilon {
                    return None;
                }
                let mut angle = oa.angle(&ob);
                if !n.dot(&oa.cross(&ob)).is_sign_positive() {
                    angle = std::f64::consts::TAU - angle;
                }
                Some(angle - std::f64::consts::PI)
            };
            let mut triangle_vertex_set = t
                .iter()
                .copied()
                .chain(lss.iter().flatten().copied())
                .collect::<Vec<_>>();
            triangle_vertex_set.sort();
            triangle_vertex_set.dedup();
            for ls in lss {
                if ls.first().unwrap() == ls.last().unwrap() {
                    // This means that the line string is closed i.e. the intersection geometry is closed inside the triangle.
                    let ls_coords = ls
                        .clone()
                        .into_iter()
                        .map(|i| vertices[i])
                        .collect::<Vec<_>>();
                    if !ls.iter().all(|&i| t.contains(&i)) {
                        interior_boundary.push(ls);
                    } else {
                        let polygon = Polygon3D::new(LineString3D::new(ls_coords), Vec::new());
                        polygon_added[triangles.binary_search(&t).unwrap()] = true;
                        polygons.push(polygon);
                    }
                    continue;
                }
                let mut right_polygon = ls.clone();
                let mut left_polygon = ls.clone();

                // The outer angle sum is used to determine when the polygon is closed.
                // This should be initialized to the sum of angles of the current line string.
                let mut angle_sum = right_polygon
                    .iter()
                    .zip(right_polygon.iter().skip(1))
                    .zip(right_polygon.iter().skip(2))
                    .filter_map(|((a, b), c)| angle(vertices[*c], vertices[*a], vertices[*b]))
                    .sum::<f64>();
                while (angle_sum.abs() - std::f64::consts::TAU).abs() > 1e-5 {
                    let last_v = vertices[*right_polygon.last().unwrap()];
                    let second_last_v_idx = right_polygon[right_polygon.len() - 2];
                    let second_last_v = vertices[second_last_v_idx];
                    let (next, angle) = vertex_adj[*right_polygon.last().unwrap()]
                        .iter()
                        .filter(|v| triangle_vertex_set.binary_search(v).is_ok())
                        .filter_map(|v| {
                            let angle = angle(vertices[*v], second_last_v, last_v);
                            angle.map(|angle| (*v, angle))
                        })
                        .map(|(v, angle)| {
                            if v == second_last_v_idx {
                                (v, std::f64::consts::PI)
                            } else {
                                (v, angle)
                            }
                        })
                        .min_by(|&a, &b| a.1.partial_cmp(&b.1).unwrap())
                        .unwrap();
                    angle_sum += angle;
                    right_polygon.push(next);
                    if right_polygon.len() > 1000 {
                        return Err("Infinite loop detected in polygon construction".to_string());
                    }
                }
                right_polygon.pop(); // remove the last vertex which is redundant.

                // TODO: the left polygon computation can be skipped if the right polygon already covers the entire triangle.
                // Implement this optimization later.

                let mut angle_sum = left_polygon
                    .iter()
                    .zip(left_polygon.iter().skip(1))
                    .zip(left_polygon.iter().skip(2))
                    .filter_map(|((a, b), c)| angle(vertices[*c], vertices[*a], vertices[*b]))
                    .sum::<f64>();
                while (angle_sum.abs() - std::f64::consts::TAU).abs() > 1e-5 {
                    let last_v = vertices[*left_polygon.last().unwrap()];
                    let second_last_v_idx = left_polygon[left_polygon.len() - 2];
                    let second_last_v = vertices[second_last_v_idx];
                    let (next, angle) = vertex_adj[*left_polygon.last().unwrap()]
                        .iter()
                        .filter(|v| triangle_vertex_set.binary_search(v).is_ok())
                        .filter_map(|v| {
                            let angle = angle(vertices[*v], second_last_v, last_v);
                            angle.map(|angle| (*v, angle))
                        })
                        .map(|(v, angle)| {
                            if v == second_last_v_idx {
                                (v, -std::f64::consts::PI)
                            } else {
                                (v, angle)
                            }
                        })
                        .max_by(|&a, &b| a.1.partial_cmp(&b.1).unwrap())
                        .unwrap();
                    angle_sum += angle;
                    left_polygon.push(next);
                    if left_polygon.len() > 1000 {
                        return Err("Infinite loop detected in polygon construction".to_string());
                    }
                }
                left_polygon.pop(); // remove the last vertex which is redundant.

                // add the polygons.
                let is_interior_boundary = {
                    let mut r = right_polygon.clone();
                    r.sort();
                    let mut l = left_polygon.clone();
                    l.sort();
                    if r == l {
                        t.iter().any(|v| !r.contains(v))
                    } else {
                        false
                    }
                };
                if is_interior_boundary {
                    // This means that the intersection geometry is not connected to triangle boundary.
                    // In this case, we create a polygon with interior boundary later.
                    interior_boundary.push(right_polygon.clone());
                } else {
                    let right_polygon = right_polygon
                        .into_iter()
                        .map(|i| vertices[i])
                        .collect::<Vec<_>>();
                    let right_polygon =
                        Polygon3D::new(LineString3D::new(right_polygon), Vec::new());
                    if !polygons.contains(&right_polygon) {
                        polygons.push(right_polygon);
                    }
                    let left_polygon = left_polygon
                        .into_iter()
                        .map(|i| vertices[i])
                        .collect::<Vec<_>>();
                    let left_polygon = Polygon3D::new(LineString3D::new(left_polygon), Vec::new());
                    if !polygons.contains(&left_polygon) {
                        polygons.push(left_polygon);
                    }
                    polygon_added[triangles.binary_search(&t).unwrap()] = true;
                }
            }
        }

        // There are triangles where no polygon is created for them.
        // We need to add them as is.
        for (added, t) in polygon_added.into_iter().zip(triangles) {
            if added {
                continue;
            }
            let tri = t
                .iter()
                .map(|&i| vertices[i])
                .chain(std::iter::once(vertices[t[0]])) // close the loop
                .collect::<Vec<_>>();
            let polygon = Polygon3D::new(LineString3D::new(tri), Vec::new());
            polygons.push(polygon);
        }

        // Now we need to add the interior boundaries to the polygons.
        for boundary in interior_boundary {
            let boudary = boundary
                .into_iter()
                .map(|i| vertices[i])
                .collect::<Vec<_>>();
            let boundary = LineString3D::new(boudary);
            let poly = polygons
                .iter_mut()
                .filter(|p| {
                    are_coplanar(
                        &p.exterior()
                            .iter()
                            .copied()
                            .chain(boundary.iter().copied())
                            .collect::<Vec<_>>(),
                    )
                    .is_some()
                })
                .find(|p| boundary.iter().all(|v| p.contains(v)))
                .unwrap();
            poly.interiors_push(boundary);
        }

        let mut out: TriangularMesh<f64> = polygons.try_into()?;
        denormalize_vertices(&mut out.vertices, norm);
        Ok(out)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_triangulate_face() {
        // Simple square face
        let face = vec![
            Coordinate3D::new__(0_f64, 0_f64, 0_f64),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 1.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
            Coordinate3D::new__(0.0, 0.0, 0.0),
        ];

        let face = LineString3D::new(face);

        let triangles = TriangularMesh::triangulate_face(face).unwrap();
        assert_eq!(triangles.len(), 2);
        assert_eq!(
            triangles[0],
            [
                Coordinate3D::new__(0_f64, 1.0, 0_f64),
                Coordinate3D::new__(0.0, 0.0, 0.0),
                Coordinate3D::new__(1.0, 0.0, 0.0),
            ]
        );
        assert_eq!(
            triangles[1],
            [
                Coordinate3D::new__(0.0, 1.0, 0.0),
                Coordinate3D::new__(1.0, 0.0, 0.0),
                Coordinate3D::new__(1.0, 1.0, 0.0),
            ]
        );

        // face whose boundary contains edges with multiplicity > 1.
        let face = vec![
            Coordinate3D::new__(2.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(2.0, 0.0, 0.0),
            Coordinate3D::new__(2.0, -1.0, 0.0),
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(2.0, 1.0, 0.0),
            Coordinate3D::new__(2.0, 0.0, 0.0),
        ];
        let face = LineString3D::new(face);
        let triangles = TriangularMesh::triangulate_face(face).unwrap();
        assert_eq!(triangles.len(), 4);
        for i in 1..4 {
            for j in 0..i - 1 {
                assert!(
                    !(triangles[j].contains(&triangles[i][0])
                        && triangles[j].contains(&triangles[i][1])
                        && triangles[j].contains(&triangles[i][2]))
                );
            }
        }

        // face whose boundary is merged
        let face = vec![
            Coordinate3D::new__(0_f64, 0.0, 0.0),
            Coordinate3D::new__(3.0, 0.0, 0.0),
            Coordinate3D::new__(3.0, 3.0, 0.0),
            Coordinate3D::new__(0.0, 3.0, 0.0),
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 1.0, 0.0),
            Coordinate3D::new__(1.0, 2.0, 0.0),
            Coordinate3D::new__(2.0, 2.0, 0.0),
            Coordinate3D::new__(2.0, 1.0, 0.0),
            Coordinate3D::new__(1.0, 1.0, 0.0),
            Coordinate3D::new__(0.0, 0.0, 0.0),
        ];
        let face = LineString3D::new(face);
        let triangles = TriangularMesh::triangulate_face(face).unwrap();
        assert_eq!(triangles.len(), 8);
    }

    #[test]
    fn test_triangulate_face_collinear() {
        // Collinear points
        let face = vec![
            Coordinate3D::new__(
                -0.3291783360681644,
                -0.32917833606816443,
                0.32917833606816443,
            ),
            Coordinate3D::new__(
                -0.10972611202272133,
                -0.3291783360681645,
                0.32917833606816443,
            ),
            Coordinate3D::new__(
                0.10972611202272146,
                -0.32917833606816443,
                0.32917833606816443,
            ),
            Coordinate3D::new__(
                -0.3291783360681644,
                -0.32917833606816443,
                0.7680827841590503,
            ),
            Coordinate3D::new__(
                -0.3291783360681644,
                -0.32917833606816443,
                0.32917833606816443,
            ),
        ];
        let face = LineString3D::new(face);
        let result = TriangularMesh::triangulate_face(face);
        assert!(result.is_ok(), "Triangulation failed: {:?}", result.err());
    }

    #[test]
    fn test_check_self_intersection_no_intersection() {
        // Simple tetrahedron vertices
        let vertices = vec![
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.5, 1.0, 0.0),
            Coordinate3D::new__(0.5, 0.5, 1.0),
        ];

        // Tetrahedron faces (triangles)
        let triangles = vec![[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]];

        let m = TriangularMesh {
            vertices: vertices.clone(),
            triangles: triangles.clone(),
            edges_with_multiplicity: vec![],
        };
        let result = m.self_intersection();
        assert!(result.is_empty()); // No self-intersections
    }

    #[test]
    fn test_check_self_intersection_with_intersection() {
        // Create two intersecting triangles that don't share vertices
        let vertices = vec![
            // First triangle
            Coordinate3D::new__(-1.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, 2.0, 0.0),
            // Second triangle (perpendicular, intersecting)
            Coordinate3D::new__(0.0, 1.0, -1.0),
            Coordinate3D::new__(0.0, 1.0, 1.0),
            Coordinate3D::new__(0.0, -1.0, 0.0),
        ];

        let triangles = vec![
            [0, 1, 2], // First triangle
            [3, 4, 5], // Second triangle
        ];

        let m = TriangularMesh {
            vertices: vertices.clone(),
            triangles: triangles.clone(),
            edges_with_multiplicity: vec![],
        };

        let result = m.self_intersection();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_check_self_intersection_shared_vertex() {
        // Two triangles sharing a vertex (should not count as intersection)
        let vertices = vec![
            Coordinate3D::new__(0.0, 0.0, 0.0), // Shared vertex
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
            Coordinate3D::new__(-1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, -1.0, 0.0),
        ];

        let triangles = vec![
            [0, 1, 2], // First triangle
            [0, 3, 4], // Second triangle sharing vertex 0
        ];

        let m = TriangularMesh {
            vertices: vertices.clone(),
            triangles: triangles.clone(),
            edges_with_multiplicity: vec![],
        };
        let result = m.self_intersection();
        assert!(result.is_empty()); // Sharing a vertex is allowed
    }

    pub fn get_cube() -> TriangularMesh<f64> {
        TriangularMesh::try_from(vec![
            // Bottom face
            Polygon3D::new(
                LineString3D::new(vec![
                    Coordinate3D::new__(0.0, 0.0, 0.0),
                    Coordinate3D::new__(1.0, 0.0, 0.0),
                    Coordinate3D::new__(1.0, 1.0, 0.0),
                    Coordinate3D::new__(0.0, 1.0, 0.0),
                    Coordinate3D::new__(0.0, 0.0, 0.0),
                ]),
                vec![],
            ),
            // Top face
            Polygon3D::new(
                LineString3D::new(vec![
                    Coordinate3D::new__(0.0, 0.0, 1.0),
                    Coordinate3D::new__(1.0, 0.0, 1.0),
                    Coordinate3D::new__(1.0, 1.0, 1.0),
                    Coordinate3D::new__(0.0, 1.0, 1.0),
                    Coordinate3D::new__(0.0, 0.0, 1.0),
                ]),
                vec![],
            ),
            // Side faces
            Polygon3D::new(
                LineString3D::new(vec![
                    Coordinate3D::new__(0.0, 0.0, 0.0),
                    Coordinate3D::new__(1.0, 0.0, 0.0),
                    Coordinate3D::new__(1.0, 0.0, 1.0),
                    Coordinate3D::new__(0.0, 0.0, 1.0),
                    Coordinate3D::new__(0.0, 0.0, 0.0),
                ]),
                vec![],
            ),
            Polygon3D::new(
                LineString3D::new(vec![
                    Coordinate3D::new__(1.0, 0.0, 0.0),
                    Coordinate3D::new__(1.0, 1.0, 0.0),
                    Coordinate3D::new__(1.0, 1.0, 1.0),
                    Coordinate3D::new__(1.0, 0.0, 1.0),
                    Coordinate3D::new__(1.0, 0.0, 0.0),
                ]),
                vec![],
            ),
            Polygon3D::new(
                LineString3D::new(vec![
                    Coordinate3D::new__(1.0, 1.0, 0.0),
                    Coordinate3D::new__(0.0, 1.0, 0.0),
                    Coordinate3D::new__(0.0, 1.0, 1.0),
                    Coordinate3D::new__(1.0, 1.0, 1.0),
                    Coordinate3D::new__(1.0, 1.0, 0.0),
                ]),
                vec![],
            ),
            Polygon3D::new(
                LineString3D::new(vec![
                    Coordinate3D::new__(0.0, 1.0, 0.0),
                    Coordinate3D::new__(0.0, 0.0, 0.0),
                    Coordinate3D::new__(0.0, 0.0, 1.0),
                    Coordinate3D::new__(0.0, 1.0, 1.0),
                    Coordinate3D::new__(0.0, 1.0, 0.0),
                ]),
                vec![],
            ),
        ])
        .unwrap()
    }

    #[test]
    fn test_bounding_solid_contains() {
        let cube = get_cube();

        assert!(cube.bounding_solid_contains(&Coordinate3D::new__(0.5, 0.5, 0.5)));
        assert!(cube.bounding_solid_contains(&Coordinate3D::new__(0.0, 0.0, 0.0)));
        assert!(cube.bounding_solid_contains(&Coordinate3D::new__(1.0, 0.2, 0.2)));
        assert!(!cube.bounding_solid_contains(&Coordinate3D::new__(1.5, 0.5, 0.5)));
        assert!(!cube.bounding_solid_contains(&Coordinate3D::new__(0.5, 1.5, 0.5)));
        assert!(!cube.bounding_solid_contains(&Coordinate3D::new__(0.5, 0.5, 1.5)));
        assert!(!cube.bounding_solid_contains(&Coordinate3D::new__(-0.5, 0.5, 0.5)));
        assert!(!cube.bounding_solid_contains(&Coordinate3D::new__(0.5, -0.5, 0.5)));
        assert!(!cube.bounding_solid_contains(&Coordinate3D::new__(0.5, 0.5, -0.5)));
    }

    #[test]
    fn test_contains() {
        let cube = get_cube();
        let epsilon = 1e-12;

        assert!(!cube.contains(&Coordinate3D::new__(0.5, 0.5, 0.5)));
        assert!(cube.contains(&Coordinate3D::new__(0.0, 0.0, 0.0)));
        assert!(cube.contains(&Coordinate3D::new__(-epsilon, -epsilon, -epsilon)));
        assert!(cube.contains(&Coordinate3D::new__(1.0, 0.2, 0.2)));
        assert!(!cube.contains(&Coordinate3D::new__(1.5, 0.5, 0.5)));
        assert!(!cube.contains(&Coordinate3D::new__(0.5, 1.5, 0.5)));
        assert!(!cube.contains(&Coordinate3D::new__(0.5, 0.5, 1.5)));
        assert!(!cube.contains(&Coordinate3D::new__(-0.5, 0.5, 0.5)));
        assert!(!cube.contains(&Coordinate3D::new__(0.5, -0.5, 0.5)));
        assert!(!cube.contains(&Coordinate3D::new__(0.5, 0.5, -0.5)));
    }

    #[test]
    fn test_into_two_manifolds_with_boundaries() {
        // Ok case. A cube is a single manifold with boundary.
        let cube = get_cube();
        let manifolds = cube.into_2_manifolds_with_boundaries();
        assert_eq!(manifolds.len(), 1);
        assert_eq!(manifolds[0].len(), 12); // A cube has 12 triangles
        assert_eq!(manifolds[0], cube.triangles);

        // Failure case. Three faces sharing a single edge.
        let triangles = vec![[0, 1, 2], [0, 1, 3], [0, 1, 4]];
        let mesh: TriangularMesh<f64> = TriangularMesh {
            triangles,
            vertices: Vec::new(),
            edges_with_multiplicity: Vec::new(),
        };
        let manifolds = mesh.into_2_manifolds_with_boundaries();
        assert_eq!(manifolds.len(), 3);
        for manifold in manifolds {
            assert_eq!(manifold.len(), 1);
        }

        // Another failure case. Two separate triangles.
        let triangles = vec![
            [0, 1, 2],
            [0, 1, 3],
            [0, 2, 3],
            [1, 2, 3],
            [0, 1, 4],
            [0, 1, 5],
            [0, 4, 5],
            [1, 4, 5],
        ];

        let mesh: TriangularMesh<f64> = TriangularMesh {
            triangles,
            vertices: Vec::new(),
            edges_with_multiplicity: Vec::new(),
        };
        let manifolds = mesh.into_2_manifolds_with_boundaries();
        assert_eq!(manifolds.len(), 2);
        for manifold in manifolds {
            assert_eq!(manifold.len(), 4);
        }
    }

    #[test]
    fn test_retain_faces() {
        let cube = get_cube();
        let faces_to_retain = vec![[0, 1, 3], [0, 1, 4]];
        let mut retained_cube = cube.clone();
        retained_cube.retain_faces(&faces_to_retain);
        assert_eq!(retained_cube.triangles.len(), 2);
        // Note: 4 -> 3 and 3 -> 2 after removing unused vertices
        assert_eq!(retained_cube.triangles, &[[0, 1, 2], [0, 1, 3]]);
        assert_eq!(retained_cube.vertices.len(), 4);
    }

    #[test]
    fn test_union_disjoint_case() {
        let cube1 = get_cube();
        let cube2 = {
            let mut cube2 = get_cube();
            cube2.transform_offset(2.0, 0.0, 0.0);
            cube2
        };

        let union = cube1.union(cube2).unwrap();
        assert_eq!(union.triangles.len(), 24);
        assert_eq!(union.vertices.len(), 16);
    }

    #[test]
    fn test_union_joint_case_simplest1() {
        // test with two triangles intersecting
        let vertices1 = vec![
            Coordinate::new__(0.0, 0.0, 0.0),
            Coordinate::new__(2.0, 1.0, 0.0),
            Coordinate::new__(2.0, -1.0, 0.0),
        ];
        let t1 = TriangularMesh {
            vertices: vertices1,
            triangles: vec![[0, 1, 2]],
            edges_with_multiplicity: vec![([0, 1], 1), ([0, 2], 1), ([1, 2], 1)],
        };

        let vertices2 = vec![
            Coordinate::new__(1.0, 0.0, 1.0),
            Coordinate::new__(1.0, 0.0, -1.0),
            Coordinate::new__(3.0, 0.0, 0.0),
        ];

        let t2 = TriangularMesh {
            vertices: vertices2,
            triangles: vec![[0, 1, 2]],
            edges_with_multiplicity: vec![([0, 1], 1), ([0, 2], 1), ([1, 2], 1)],
        };

        let union = t1.union(t2).unwrap();
        assert_eq!(union.vertices.len(), 8);
        assert!(union
            .vertices
            .iter()
            .any(|&v| (v - Coordinate::new__(1.0, 0.0, 0.0)).norm() < 1e-10));
        assert!(union
            .vertices
            .iter()
            .any(|&v| (v - Coordinate::new__(2.0, 0.0, 0.0)).norm() < 1e-10));
        assert_eq!(union.triangles.len(), 8);
    }

    #[test]
    fn test_union_joint_case_simplest2() {
        // test with two triangles intersecting
        let vertices1 = vec![
            Coordinate::new__(-2.0, 0.0, 0.0),
            Coordinate::new__(2.0, 1.0, 0.0),
            Coordinate::new__(2.0, -1.0, 0.0),
        ];
        let t1 = TriangularMesh {
            vertices: vertices1,
            triangles: vec![[0, 1, 2]],
            edges_with_multiplicity: vec![([0, 1], 1), ([0, 2], 1), ([1, 2], 1)],
        };

        let vertices2 = vec![
            Coordinate::new__(2.0, 0.0, -1.0),
            Coordinate::new__(-2.0, 0.0, -1.0),
            Coordinate::new__(0.0, 0.0, 1.0),
        ];

        let t2 = TriangularMesh {
            vertices: vertices2,
            triangles: vec![[0, 1, 2]],
            edges_with_multiplicity: vec![([0, 1], 1), ([0, 2], 1), ([1, 2], 1)],
        };

        let union = t1.union(t2).unwrap();
        assert_eq!(union.vertices.len(), 8);
        assert!(union
            .vertices
            .iter()
            .any(|&v| (v - Coordinate::new__(1.0, 0.0, 0.0)).norm() < 1e-10));
        assert!(union
            .vertices
            .iter()
            .any(|&v| (v - Coordinate::new__(-1.0, 0.0, 0.0)).norm() < 1e-10));
        assert_eq!(union.triangles.len(), 8);
    }

    #[test]
    fn test_union_joint_case_simplest3() {
        // test with two triangles intersecting
        let vertices1 = vec![
            Coordinate::new__(-1.0, 2.0, 0.0),
            Coordinate::new__(-1.0, -2.0, 0.0),
            Coordinate::new__(1.0, 0.0, 0.0),
        ];
        let t1 = TriangularMesh {
            vertices: vertices1,
            triangles: vec![[0, 1, 2]],
            edges_with_multiplicity: vec![([0, 1], 1), ([0, 2], 1), ([1, 2], 1)],
        };

        let vertices2 = vec![
            Coordinate::new__(0.0, 2.0, -1.0),
            Coordinate::new__(0.0, -2.0, -1.0),
            Coordinate::new__(0.0, 0.0, 1.0),
        ];

        let t2 = TriangularMesh {
            vertices: vertices2,
            triangles: vec![[0, 1, 2]],
            edges_with_multiplicity: vec![([0, 1], 1), ([0, 2], 1), ([1, 2], 1)],
        };

        let union = t1.union(t2).unwrap();
        assert_eq!(union.vertices.len(), 8);
        assert!(union
            .vertices
            .iter()
            .any(|&v| (v - Coordinate::new__(0.0, 1.0, 0.0)).norm() < 1e-10));
        assert!(union
            .vertices
            .iter()
            .any(|&v| (v - Coordinate::new__(0.0, -1.0, 0.0)).norm() < 1e-10));
        assert_eq!(union.triangles.len(), 6);
    }

    #[test]
    fn test_union_joint_case1() {
        let cube1 = get_cube();
        let cube2 = {
            let mut cube2 = get_cube();
            cube2.transform_offset(-0.8, -0.7, -0.6);
            cube2
        };

        let union = cube1.union(cube2).unwrap();
        assert_eq!(union.vertices.len(), 24);
        // No triangles should be degenerate.
        for t in union.triangles {
            let a = union.vertices[t[0]];
            let b = union.vertices[t[1]];
            let c = union.vertices[t[2]];
            assert!(
                (b - a).cross(&(c - a)).norm() > 1e-10,
                "Degenerate triangle found: {t:?}, vertices: {a:.2?}, {b:.2?}, {c:.2?}"
            );
        }
    }

    #[test]
    fn test_union_joint_case2() {
        let cube1 = get_cube();
        let cube2 = {
            let mut cube2 = get_cube();
            cube2.transform_offset(-0.4, -0.4, -0.4);
            cube2
        };

        let union = cube1.union(cube2).unwrap();
        assert_eq!(union.vertices.len(), 30);
        // No triangles should be degenerate.
        for t in union.triangles {
            let a = union.vertices[t[0]];
            let b = union.vertices[t[1]];
            let c = union.vertices[t[2]];
            assert!(
                (b - a).cross(&(c - a)).norm() > 1e-10,
                "Degenerate triangle found: {t:?}, vertices: {a:.2?}, {b:.2?}, {c:.2?}"
            );
        }
    }

    #[test]
    fn test_union_joint_case3() {
        let cube1 = get_cube();
        let cube2 = {
            let mut cube2 = get_cube();
            cube2.transform_offset(-0.5, -0.5, -0.5);
            cube2
        };

        let union = cube1.union(cube2).unwrap();
        assert_eq!(union.vertices.len(), 22);
        // No triangles should be degenerate.
        for t in union.triangles {
            let a = union.vertices[t[0]];
            let b = union.vertices[t[1]];
            let c = union.vertices[t[2]];
            assert!(
                (b - a).cross(&(c - a)).norm() > 1e-10,
                "Degenerate triangle found: {t:?}, vertices: {a:.2?}, {b:.2?}, {c:.2?}"
            );
        }
    }

    #[test]
    fn test_union_joint_case4() {
        let tetrahedron1 = TriangularMesh::try_from(vec![
            Polygon3D::new(
                LineString3D::new(vec![
                    Coordinate3D::new__(0.0, 0.0, 0.0),
                    Coordinate3D::new__(2.0, 0.0, 0.0),
                    Coordinate3D::new__(1.0, 2.0, 0.0),
                    Coordinate3D::new__(0.0, 0.0, 0.0),
                ]),
                vec![],
            ),
            Polygon3D::new(
                LineString3D::new(vec![
                    Coordinate3D::new__(0.0, 0.0, 0.0),
                    Coordinate3D::new__(2.0, 0.0, 0.0),
                    Coordinate3D::new__(1.0, 1.0, 2.0),
                    Coordinate3D::new__(0.0, 0.0, 0.0),
                ]),
                vec![],
            ),
            Polygon3D::new(
                LineString3D::new(vec![
                    Coordinate3D::new__(2.0, 0.0, 0.0),
                    Coordinate3D::new__(1.0, 2.0, 0.0),
                    Coordinate3D::new__(1.0, 1.0, 2.0),
                    Coordinate3D::new__(2.0, 0.0, 0.0),
                ]),
                vec![],
            ),
            Polygon3D::new(
                LineString3D::new(vec![
                    Coordinate3D::new__(1.0, 2.0, 0.0),
                    Coordinate3D::new__(0.0, 0.0, 0.0),
                    Coordinate3D::new__(1.0, 1.0, 2.0),
                    Coordinate3D::new__(1.0, 2.0, 0.0),
                ]),
                vec![],
            ),
        ])
        .unwrap();

        let mut tetrahedron2 = tetrahedron1.clone();
        tetrahedron2.transform_offset(0.0, 0.0, -1.0);

        let union = tetrahedron1.union(tetrahedron2).unwrap();
        assert_eq!(union.vertices.len(), 11);
    }
}
