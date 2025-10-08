use crate::algorithm::triangle_intersection::{triangles_intersect, triangles_intersection};
use crate::types::coordinate::Coordinate;
use crate::types::line_string::LineString3D;
use crate::types::polygon::Polygon3D;
use crate::types::triangle::Triangle;
use crate::types::{coordinate::Coordinate3D, coordnum::CoordNum, line::Line3D};
use num_traits::Float;
use nusamai_projection::vshift::Jgd2011ToWgs84;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    pub fn get_triangles(&self) -> &[[usize; 3]] {
        &self.triangles
    }
}

impl<T: Float + CoordNum> TriangularMesh<T> {
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

        // Sort triangles for consistent representation
        out.triangles.sort_unstable();
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
            let mut boundary_2 = self.triangles.iter()
                .enumerate()
                .flat_map(|(i, t)| [[t[0], t[1], i], [t[1], t[2], i], [t[0], t[2], i]]).collect::<Vec<_>>();
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
        let epsilon = T::from(1e-1).unwrap();
        let triangles = self.triangles.clone();
        // quick check with bounding box
        let triangles = triangles.into_iter().filter_map(|triangle| {
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
            if point.x >= bbox_min.x && point.x <= bbox_max.x &&
               point.y >= bbox_min.y && point.y <= bbox_max.y &&
               point.z >= bbox_min.z && point.z <= bbox_max.z {
                Some([v0, v1, v2])
            } else {
                None
            }
        }).collect::<Vec<_>>();

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
            let mut boundary_2 = self.triangles.iter()
                .enumerate()
                .flat_map(|(i, t)| [[t[0], t[1], i], [t[1], t[2], i], [t[0], t[2], i]]).collect::<Vec<_>>();
            boundary_2.sort_unstable();
            let boundary_2 = boundary_2.into_iter();
            // group faces by edges
            let mut edge_to_faces: Vec<Vec<usize>> = Vec::new();
            let mut prev_edge = [usize::MAX; 2];
            for edge in boundary_2 {
                if &prev_edge == &edge[0..2] {
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
        assert!(self.triangles.is_sorted());
        assert!(faces.iter().all(|f| self.triangles.binary_search(f).is_ok()));

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
            .map(|f| [vertex_mapping[f[0]], vertex_mapping[f[1]], vertex_mapping[f[2]]])
            .collect();
        self.edges_with_multiplicity = {
            let mut edges: Vec<[usize; 2]> = Vec::new();
            for triangle in &self.triangles {
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
                return self.edges_with_multiplicity = Vec::new();
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
}

impl<T: Float + CoordNum> From<Vec<Polygon3D<T>>> for TriangularMesh<T> {
    fn from(faces: Vec<Polygon3D<T>>) -> Self {
        let faces = faces.into_iter().map(|p| p.into_merged_contour()).collect::<Vec<_>>();
        Self::from_faces(&faces)
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

    pub(crate) fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        self.vertices.iter_mut().for_each(|c| c.transform_inplace(jgd2wgs));
    }

    pub(crate) fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        self.vertices.iter_mut().for_each(|c| c.transform_offset(x, y, z));
    }


    /// takes in another triangular mesh and returns a new triangular mesh representing the union of the two meshes.
    /// When two meshes intersect, there will be new vertices and edges created at the intersection.
    /// When the two meshes intersect at a face (i.e. they have the identical face), then the face will be merged.
    pub fn union(self, other: Self) -> Result<Self, ()> {
        let Self{vertices: vertices1, triangles: triangles1, edges_with_multiplicity: edges1} = self;
        let Self{vertices: vertices2, triangles: mut triangles2, edges_with_multiplicity: edges2} = other;

        if !triangles1.is_sorted() || !triangles2.is_sorted() {
            return Err(());
        }
        if !edges1.iter().all(|(e, _)| e.is_sorted()) || !edges2.iter().all(|(e, _)| e.is_sorted()) {
            return Err(());
        }
        if !edges1.is_sorted() || !edges2.is_sorted() {
            return Err(());
        }

        triangles2.iter_mut().for_each(|tri| {
            for v in tri.iter_mut() {
                *v += vertices1.len();
            }
        });

        let num_orig_vertices = vertices1.len() + vertices2.len();
        let mut edges = {
            let mut edges = edges1.into_iter().map(|(e, _)| e)
                .chain(edges2.into_iter().map(|(e, _)| [e[0] + vertices1.len(), e[1] + vertices1.len()]))
                .collect::<Vec<_>>();
            edges.sort_unstable();
            edges.dedup();
            edges
        };
        let mut edges_cuts = vec![Vec::new(); edges.len()];
        let num_orig_edges = edges.len();
        let mut vertices = vertices1.into_iter().chain(vertices2.into_iter()).collect::<Vec<_>>();

        println!("edges: {:?}", edges);

        let mut intersections = vec![Vec::new(); triangles1.len() + triangles2.len()];
        for (ti, t) in triangles1.iter().copied().enumerate() {
            let tt = [
                vertices[t[0]],
                vertices[t[1]],
                vertices[t[2]],
            ];
            for (si, s) in triangles2.iter().copied().enumerate() {
                let ss = [
                    vertices[s[0]],
                    vertices[s[1]],
                    vertices[s[2]],
                ];
                let Some(intersection) = triangles_intersection(tt, ss).unwrap_or_else(|_| panic!("Error in triangle-triangle intersection"))  else {
                    continue;
                };
                intersections[ti].push(intersection);
                intersections[si + triangles1.len()].push(intersection);
                let v1 = if let Some(idx) = vertices.iter().skip(num_orig_vertices).position(|&x| x == intersection[0]) {
                    idx + num_orig_vertices
                } else {
                    vertices.push(intersection[0]);
                    // need to check if the intersection point is on the edge of the triangle.
                    // if it is, then we need to remove the edge and add two new edges.
                    let mut divide_edge = |vv, ww, v, w| {
                        if Line3D::new_(vv, ww).contains(intersection[0]) {
                            let e_idx = edges.binary_search(&[v, w]).map_err(|_| ()).unwrap_or_else(|_| panic!("Edge not found: edges: {:?}, v: {}, w: {}", edges, v, w));
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
                let v2 = if let Some(idx) = vertices.iter().skip(num_orig_vertices).position(|&x| x == intersection[1]) {
                    idx + num_orig_vertices
                } else {
                    vertices.push(intersection[1]);
                    let mut divide_edge = |vv, ww, v, w| {
                        if Line3D::new_(vv, ww).contains(intersection[1]) {
                            let e_idx = edges.binary_search(&[v, w]).map_err(|_| ()).unwrap_or_else(|_| panic!("Edge not found: edges: {:?}, v: {}, w: {}", edges, v, w));
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
                    edges.push([v1, v2]);
                } else {
                    edges.push([v2, v1]);
                }
            }
        }
        edges[num_orig_edges..].sort_unstable();
        assert_eq!(edges.iter().zip(edges.iter().skip(1)).filter(|(a, b)| a == b).count(), 0);
        assert!(
            edges_cuts.iter().all(|cuts1| edges_cuts.iter().all(|cuts2| {
                let len = cuts1.len() + cuts2.len();
                let mut cuts1 = cuts1.clone();
                cuts1.append(&mut cuts2.clone());
                cuts1.sort_unstable();
                cuts1.dedup();
                cuts1.len() == len
            }))
        );

        // Reflect the edge cuts in the edges list.
        edges_cuts.resize(edges.len(), Vec::new());
        let mut edges = edges.into_iter().zip(edges_cuts.into_iter())
            .flat_map(|(edge, mut cuts)| {
                cuts.sort_unstable();
                cuts.dedup();
                let mut result = Vec::new();
                let mut last_v = edge[0];
                for &cut in &cuts {
                    result.push([last_v, cut]);
                    last_v = cut;
                }
                result.push([last_v, edge[1]]);
                result.last_mut().unwrap().sort(); // Only the last one may be unsorted, so we sort it.
                {
                    result.sort_unstable();
                    assert!(result.iter().zip(result.iter().skip(1)).filter(|(a, b)| a == b).count() == 0);
                }
                if result.contains(&[17,19]) {
                    println!("[17,19] found in edge: result: {:?}", result);
                    println!("cuts: {:?}", cuts);
                }   
                result
            }).collect::<Vec<_>>();
        edges.sort_unstable();
        assert_eq!(edges.iter().zip(edges.iter().skip(1)).filter(|(a, b)| a == b).count(), 0, "edges.len(): {}, edges: {:?}", edges.len(), edges);
        assert!(edges.iter().all(|e| e[0] < e[1]));

        // Now all the new vertices and edges are created. we create a map from vertex to its index for easy lookup.
        let vertex_key = |v: &Coordinate3D<f64>| [v.x.to_bits(), v.y.to_bits(), v.z.to_bits()];
        let vertex_map = {
            let mut map = HashMap::new();
            for (i, v) in vertices.iter().enumerate() {
                let key = vertex_key(v);
                map.insert(key, i);
            }
            map
        };
        // Also create the vertex-adjacency list for easy traversal.
        let mut vertex_adj: Vec<Vec<usize>> = vec![Vec::new(); vertices.len()];
        for edge in &edges {
            vertex_adj[edge[0]].push(edge[1]);
            vertex_adj[edge[1]].push(edge[0]);
        }

        let intersections = intersections.into_iter().map(|intersection|
            intersection.into_iter().map(|edge| {
                let v1 = *vertex_map.get(&vertex_key(&edge[0])).unwrap();
                let v2 = *vertex_map.get(&vertex_key(&edge[1])).unwrap();
                if v1 < v2 {
                    [v1, v2]
                } else {
                    [v2, v1]
                }
            }).collect::<Vec<_>>()
        ).collect::<Vec<_>>();

        // For each triangle, create polygons by dividing the triangle along the intersection edges.
        let mut line_strings = Vec::new();
        for (t, mut intersection) in triangles1.into_iter().chain(triangles2).zip(intersections) {
            let Some(last) = intersection.pop() else { 
                // No intersection, just add the triangle as is.
                let mut tri = t.to_vec();
                tri.push(t[0]); // close the loop
                line_strings.push((tri, t));
                continue;
            };
            let mut line_str_coords = vec![last[1], last[0]];
            // ToDo: This can be optimized.
            let mut next_vertex = |line_str_coords: &Vec<usize>| {
                let pos = intersection.iter().position(|e| 
                    e.contains(line_str_coords.last().unwrap())
                );
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
            }
            line_str_coords.reverse();
            while let Some(vertex) = next_vertex(&line_str_coords) {
                line_str_coords.push(vertex);
            }

            if line_str_coords.first() != line_str_coords.last() {

            }
            line_strings.push((line_str_coords, t));
        }

        let mut polygons = Vec::new();
        for (ls, t) in line_strings {
            if ls.first().ok_or(())? == ls.last().ok_or(())? {
                // This means that the line string is closed i.e. the intersection is closed inside the triangle.
                let ls = ls.into_iter().map(|i| vertices[i]).collect::<Vec<_>>();
                let polygon = Polygon3D::new(LineString3D::new(ls), Vec::new());
                polygons.push(polygon);
            } else {
                // This means that the line string is open at the edge of the triangle.
                // We need to close the string to create a polygon.
                let mut right_polygon = ls.clone();
                let mut left_polygon = ls;

                let v1 = vertices[t[0]];
                let v2 = vertices[t[1]];
                let v3 = vertices[t[2]];
                let n = (v2 - v1).cross(&(v3 - v1)).normalize();
                let angle = |a: Coordinate3D<f64>, b: Coordinate3D<f64>, o: Coordinate3D<f64>| {
                    let epsilon = 1e-10;
                    let oa = (a - o).normalize();
                    let ob = (b - o).normalize();
                    let mut angle = oa.angle(&ob);
                    if n.dot(&oa) > epsilon || n.dot(&ob) > epsilon {
                        println!("Not on the plane: {}, {}", n.dot(&oa), n.dot(&ob));
                        return None;
                    }
                    if !n.dot(&oa.cross(&ob)).is_sign_positive() {
                        angle = std::f64::consts::TAU - angle;
                    }
                    println!("angle: {}", angle);
                    Some(angle)
                };
                println!("\ncomputing right polygon");
                while right_polygon.last().unwrap() != right_polygon.first().unwrap() {
                    println!("right_polygon: {:?}", right_polygon);
                    let last_v = vertices[*right_polygon.last().unwrap()];
                    let second_last_v_idx = right_polygon[right_polygon.len() - 2];
                    let second_last_v = vertices[second_last_v_idx];
                    println!("adj: {:?}", vertex_adj[*right_polygon.last().unwrap()]);
                    let next = *vertex_adj[*right_polygon.last().unwrap()]
                        .iter()
                        .filter(|&&v| v != second_last_v_idx)
                        .min_by(|&&a, &&b| {
                            let alpha = angle(vertices[a], second_last_v, last_v).unwrap_or(std::f64::INFINITY);
                            let beta = angle(vertices[b], second_last_v, last_v).unwrap_or(std::f64::INFINITY);
                            alpha.partial_cmp(&beta).unwrap()
                        })
                        .unwrap();
                    right_polygon.push(next);
                    assert!(right_polygon.len() < 10, "Infinite loop detected: right_polygon: {:?}", right_polygon);
                }
                println!("right_polygon: {:?}", right_polygon);
                let right_polygon = right_polygon.into_iter().map(|i| vertices[i]).collect::<Vec<_>>();
                let right_polygon = Polygon3D::new(LineString3D::new(right_polygon), Vec::new());
                if !polygons.contains(&right_polygon) {
                    polygons.push(right_polygon);
                }

                println!("\ncomputing left polygon");
                while left_polygon.last().unwrap() != left_polygon.first().unwrap() {
                    let last_v = vertices[*left_polygon.last().unwrap()];
                    let second_last_v_idx = left_polygon[left_polygon.len() - 2];
                    let second_last_v = vertices[second_last_v_idx];
                    let next = *vertex_adj[*left_polygon.last().unwrap()]
                        .iter()
                        .filter(|&&v| v != second_last_v_idx)
                        .max_by(|&&a, &&b| {
                            let alpha = angle(vertices[a], second_last_v, last_v).unwrap_or(std::f64::NEG_INFINITY);
                            let beta = angle(vertices[b], second_last_v, last_v).unwrap_or(std::f64::NEG_INFINITY);
                            alpha.partial_cmp(&beta).unwrap()
                        })
                        .unwrap();
                    left_polygon.push(next);
                    assert!(left_polygon.len() < 10, "Infinite loop detected: left_polygon: {:?}", left_polygon);
                }
                let left_polygon = left_polygon.into_iter().map(|i| vertices[i]).collect::<Vec<_>>();
                let left_polygon = Polygon3D::new(LineString3D::new(left_polygon), Vec::new());
                if !polygons.contains(&left_polygon) {
                    polygons.push(left_polygon);
                }

            }
        }

        Ok(polygons.into())
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

    fn get_cube() -> TriangularMesh<f64> {
        TriangularMesh::from( vec![
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
        let triangles = vec![
            [0, 1, 2],
            [0, 1, 3],
            [0, 1, 4],
        ];
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
        let faces_to_retain = vec![
            [0, 1, 2],
            [0, 1, 4]
        ];
        let mut retained_cube = cube.clone();
        retained_cube.retain_faces(&faces_to_retain);
        assert_eq!(retained_cube.triangles.len(), 2);
        assert_eq!(retained_cube.triangles, &[[0, 1, 2], [0, 1, 3]]); // Note: 4 becomes 3 after removing unused vertices
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
    fn test_union_joint_case() {
        let cube1 = get_cube();
        let cube2 = {
            let mut cube2 = get_cube();
            cube2.transform_offset(0.5, 0.5, 0.5);
            cube2
        };

        let union = cube1.union(cube2).unwrap();
        assert_eq!(union.vertices.len(), 22);
    }
}
