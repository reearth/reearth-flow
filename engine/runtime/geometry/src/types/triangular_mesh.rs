use crate::algorithm::triangle_intersection::triangles_intersect;
use crate::types::line_string::LineString3D;
use crate::types::{coordinate::Coordinate3D, coordnum::CoordNum, line::Line3D};
use num_traits::Float;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct TriangularMesh<T: Float + CoordNum = f64> {
    // Vertices of the solid boundary. No duplicate vertices.
    vertices: Vec<Coordinate3D<T>>,
    // Edges of the solid boundary with their multiplicity (i.e. how many faces share the edge).
    // they are defined as pairs of vertex indices, and the vertex indices in each pair are sorted.
    // The edges themselves are also sorted.
    edges_with_multiplicity: Vec<([usize; 2], usize)>,
    // Triangles of the solid boundary. Each triangle is represented by the indices of its vertices.
    // They are defined as triplets of vertex indices, and the vertex indices in each triangle are sorted.
    // The triangles themselves are also sorted.
    triangles: Vec<[usize; 3]>,
}

impl<T: Float + CoordNum> Default for TriangularMesh<T> {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            edges_with_multiplicity: Vec::new(),
            triangles: Vec::new(),
        }
    }
}

impl<T: Float + CoordNum> TriangularMesh<T> {
    pub fn get_vertices(&self) -> &[Coordinate3D<T>] {
        &self.vertices
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
            && self.triangles.is_empty()
            && self.edges_with_multiplicity.is_empty()
    }

    /// Create a triangular mesh from a list of faces by triangulating each face.
    pub fn from_faces(faces: &[LineString3D<T>]) -> Self {
        let mut out = Self::default();
        let mut vertex_map: HashMap<String, usize> = HashMap::new();
        let mut edges: Vec<[usize; 2]> = Vec::new();

        for face in faces {
            // Triangulate the face
            let face_triangles = if let Ok(triangles) = Self::triangulate_face(face.clone()) {
                triangles
            } else {
                // If triangulation fails, then return empty mesh
                return Self::default();
            };

            for triangle in face_triangles {
                let mut tri_indices = [0usize; 3];

                for (i, vertex) in triangle.iter().enumerate() {
                    // Create a key for the vertex
                    let key = format!("{:.10?},{:.10?},{:.10?}", vertex.x, vertex.y, vertex.z);

                    // Get or insert vertex index
                    let vertex_index = match vertex_map.get(&key) {
                        Some(&idx) => idx,
                        None => {
                            let idx = out.vertices.len();
                            out.vertices.push(*vertex);
                            vertex_map.insert(key, idx);
                            idx
                        }
                    };

                    tri_indices[i] = vertex_index;
                }

                // Add triangle
                tri_indices.sort_unstable();
                out.triangles.push(tri_indices);

                // Add edges. Count multiplicity for manifold check
                for [i, j] in [[0, 1], [1, 2], [0, 2]] {
                    let edge = [tri_indices[i], tri_indices[j]];
                    edges.push(edge);
                }
            }
        }
        edges.sort_unstable();
        if edges.is_empty() {
            return out;
        }
        out.edges_with_multiplicity = vec![(edges[0], 1)];
        out.edges_with_multiplicity.reserve(edges.len());

        for edge in edges.into_iter().skip(1) {
            let (last_edge, count) = out.edges_with_multiplicity.last_mut().unwrap();
            if *last_edge == edge {
                *count += 1;
            } else {
                out.edges_with_multiplicity.push((edge, 1));
            }
        }
        out
    }

    /// triangulates the input face.
    /// We assume the following conditions about the face:
    /// - The face is planar
    /// - The face contour is closed (i.e. the first point is the same as the last point)
    /// - The face does not intersect itself
    fn triangulate_face(face: LineString3D<T>) -> Result<Vec<[Coordinate3D<T>; 3]>, ()> {
        let mut face = face.0;
        // face at least must be triangle
        if face.len() < 4 {
            return Err(());
        }

        let tau = T::from(std::f64::consts::TAU).unwrap();
        let pi = T::from(std::f64::consts::PI).unwrap();
        let epsilon = T::from(1e-5).unwrap();

        // remove the last point
        face.pop();

        // Compute the face normal. We assume the face is planar as this is not the validation process for the solid face.
        let normal = {
            let p0 = face[0];
            let p1 = face[1];
            let p2 = face[2];
            let v1 = p1 - p0;
            let v2 = p2 - p0;
            v1.cross(&v2).normalize()
        };

        // Compute the angle at each vertex
        // We assume that the first point is the same as the last point
        let mut angles = Vec::new();
        for i in 0..face.len() {
            let p0 = face[(i + face.len() - 1) % face.len()];
            let p1 = face[i];
            let p2 = face[(i + 1) % face.len()];
            let v1 = p0 - p1;
            let v2 = p2 - p1;
            let mut angle = v1.angle(&v2);
            // Take the sign of the orientation into account
            if !normal.dot(&v1.cross(&v2)).is_sign_positive() {
                angle = tau - angle;
            }

            // Adjust angle to be in [-PI, PI] i.e. angle=0 when vectors are collinear and pointing in the opposite direction
            angle = angle - pi;
            angles.push(angle);
        }

        // The sum of the angles is either TAU or -TAU.
        // If the sum is -TAU, flip the angles.
        // otherwise something about the assumption is wrong
        let angle_sum: T = angles.iter().copied().fold(T::zero(), |acc, x| acc + x);
        if (angle_sum.abs() + tau).abs() < epsilon {
            // if the sum is -TAU, flip the angles
            angles = angles.iter().map(|&a| -a).collect();
        } else if (angle_sum.abs() - tau).abs() >= epsilon {
            // something is wrong with the assumption
            return Err(());
        } // otherwise the sum is TAU and we are good to go

        // Triangulate the face by the following process:
        // 1. Find the vertex with the positive angle.
        // 2. Create a triangle with the two adjacent vertices.
        // 3. Remove the vertex from the face.
        // 4. Update the angles of the adjacent vertices.
        // 5. Repeat until the face boundary is empty.
        let mut triangles = Vec::new();
        while !face.is_empty() {
            // Find the vertex with the positive angle
            let positive_angle_idx = angles
                .iter()
                .enumerate()
                .find_map(|(idx, &angle)| if angle > epsilon { Some(idx) } else { None })
                .unwrap_or(0);

            // Create a triangle with the two adjacent vertices
            let prev_idx = (positive_angle_idx + face.len() - 1) % face.len();
            let next_idx = (positive_angle_idx + 1) % face.len();
            triangles.push([face[prev_idx], face[positive_angle_idx], face[next_idx]]);

            // Remove the vertex from the face
            face.remove(positive_angle_idx);
            angles.remove(positive_angle_idx);

            // Update the angles of the adjacent vertices
            if face.len() < 3 {
                break;
            }
            let new_prev_idx = (positive_angle_idx + face.len() - 1) % face.len();
            let new_next_idx = positive_angle_idx % face.len();

            let p0 = face[(new_prev_idx + face.len() - 1) % face.len()];
            let p1 = face[new_prev_idx];
            let p2 = face[new_next_idx];
            let v1 = p0 - p1;
            let v2 = p2 - p1;
            let mut angle = v1.angle(&v2);
            if !normal.dot(&v1.cross(&v2)).is_sign_positive() {
                angle = tau - angle;
            }
            angle = angle - pi;
            angles[new_prev_idx] = angle;
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
}

impl TriangularMesh<f64> {
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
}

#[cfg(test)]
mod tests {
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
}
