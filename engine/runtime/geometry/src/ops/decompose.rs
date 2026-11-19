//! Decomposing a mesh into parts.
//!
//! [`DecomposeIntoEdgeConnectedParts`] splits a mesh into sets of faces joined
//! through shared edges. [`DecomposeIntoManifolds`] splits it further, so that
//! every edge of a part borders at most two of its faces. Each part keeps the
//! mesh's type and frame, has a vertex pool of its own, and carries the slice of
//! the appearance that covers its faces. Two faces share an edge when they use
//! the same two vertices of the pool. [`decompose_with_faces`] also tells which
//! faces of the mesh each part holds.

use std::collections::HashMap;
use std::ops::Range;

use crate::appearance::{Appearance, faces_appearance};
use crate::ops::UnsupportedOperation;
use crate::ops::merge::face_rings;
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// Split a mesh into its edge-connected parts.
#[enum_dispatch::enum_dispatch]
pub trait DecomposeIntoEdgeConnectedParts {
    /// One mesh per set of faces joined through shared edges, ordered by first
    /// face. Faces meeting at a vertex alone fall into different parts. A mesh
    /// that is one part comes back unchanged; one with no faces yields none.
    /// The default body reports the type as unsupported.
    fn decompose_into_edge_connected_parts(&self) -> Result<Vec<Geometry>, UnsupportedOperation> {
        Err(UnsupportedOperation {
            geometry: core::any::type_name::<Self>(),
            operation: "decompose_into_edge_connected_parts",
        })
    }
}

impl<T: DecomposeIntoEdgeConnectedParts + ?Sized> DecomposeIntoEdgeConnectedParts for Box<T> {
    fn decompose_into_edge_connected_parts(&self) -> Result<Vec<Geometry>, UnsupportedOperation> {
        (**self).decompose_into_edge_connected_parts()
    }
}

/// Split a mesh into manifold parts.
#[enum_dispatch::enum_dispatch]
pub trait DecomposeIntoManifolds {
    /// Meshes whose every edge borders one or two of their faces, ordered by
    /// first face. Faces join in face order, and a join is refused when an edge
    /// would border three or more faces of the result, so where more faces meet
    /// at one edge the first two stay together. Each part is edge-connected. A
    /// mesh that is one part comes back unchanged; one with no faces yields
    /// none. The default body reports the type as unsupported.
    fn decompose_into_manifolds(&self) -> Result<Vec<Geometry>, UnsupportedOperation> {
        Err(UnsupportedOperation {
            geometry: core::any::type_name::<Self>(),
            operation: "decompose_into_manifolds",
        })
    }
}

impl<T: DecomposeIntoManifolds + ?Sized> DecomposeIntoManifolds for Box<T> {
    fn decompose_into_manifolds(&self) -> Result<Vec<Geometry>, UnsupportedOperation> {
        (**self).decompose_into_manifolds()
    }
}

/// The meshes of parts paired with their faces.
pub(crate) fn geometries(parts: Vec<(Geometry, Vec<usize>)>) -> Vec<Geometry> {
    parts.into_iter().map(|(geometry, _)| geometry).collect()
}

/// How faces are grouped into parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Partition {
    /// Sets of faces joined through shared edges, as
    /// [`DecomposeIntoEdgeConnectedParts`] splits them.
    EdgeConnected,
    /// Parts whose every edge borders at most two of their faces, as
    /// [`DecomposeIntoManifolds`] splits them.
    Manifold,
}

impl Partition {
    fn operation(self) -> &'static str {
        match self {
            Partition::EdgeConnected => "decompose_into_edge_connected_parts",
            Partition::Manifold => "decompose_into_manifolds",
        }
    }
}

/// The parts of a polygon mesh or triangular mesh under `partition`, as the
/// matching decompose trait returns them, each paired with the indices of the
/// faces of `geometry` it holds, in increasing order. A face of a triangular
/// mesh is one triangle. Other geometries are unsupported.
pub fn decompose_with_faces(
    geometry: &Geometry,
    partition: Partition,
) -> Result<Vec<(Geometry, Vec<usize>)>, UnsupportedOperation> {
    match geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::PolygonMesh(m)) => {
            Ok(m.decompose_parts(partition))
        }
        Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(m)) => {
            Ok(m.decompose_parts(partition))
        }
        Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(m)) => {
            Ok(m.decompose_parts(partition))
        }
        Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => {
            Ok(m.decompose_parts(partition))
        }
        _ => Err(UnsupportedOperation {
            geometry: core::any::type_name::<Geometry>(),
            operation: partition.operation(),
        }),
    }
}

/// A mesh's faces as flat buffers: every corner's vertex, the end of every
/// face, and the start of every hole ring, all as corner positions.
pub(crate) struct MeshFaces<'a, const N: usize> {
    pub(crate) vertices: &'a [[f64; N]],
    pub(crate) indices: Vec<u32>,
    pub(crate) face_ends: Vec<usize>,
    pub(crate) hole_starts: Vec<usize>,
    pub(crate) appearance: &'a Option<Appearance>,
}

/// One part: its own vertex pool and CSR buffers, its appearance, and the
/// faces of the mesh it holds.
pub(crate) struct Part<const N: usize> {
    pub(crate) vertices: Vec<[f64; N]>,
    pub(crate) indices: Vec<u32>,
    /// The end of every face in `indices`.
    pub(crate) face_ends: Vec<u32>,
    pub(crate) interior_offsets: Vec<u32>,
    pub(crate) appearance: Option<Appearance>,
    /// The mesh's faces in this part, in increasing order.
    pub(crate) faces: Vec<usize>,
}

/// The parts of `mesh` under `partition`, or `None` when it is a single part
/// and can be kept as it is. No faces yield no parts.
pub(crate) fn decompose<const N: usize>(
    mesh: &MeshFaces<'_, N>,
    partition: Partition,
) -> Option<Vec<Part<N>>> {
    let rings = face_rings(&mesh.face_ends, &mesh.hole_starts);
    let edges = face_edges(&mesh.indices, &rings);
    let groups = match partition {
        Partition::EdgeConnected => edge_connected(&edges),
        Partition::Manifold => manifold(&edges),
    };
    if groups.len() == 1 {
        return None;
    }
    Some(
        groups
            .into_iter()
            .map(|faces| part(mesh, faces, &rings))
            .collect(),
    )
}

/// An undirected edge between two vertices of the pool, smaller index first.
type Edge = (u32, u32);

/// Each face's distinct edges. An open ring also has its closing edge.
fn face_edges(indices: &[u32], rings: &[Vec<Range<usize>>]) -> Vec<Vec<Edge>> {
    rings
        .iter()
        .map(|face| {
            let mut edges = Vec::new();
            for ring in face {
                let ids = &indices[ring.clone()];
                let closing = match (ids.first(), ids.last()) {
                    (Some(&first), Some(&last)) if first != last => Some((last, first)),
                    _ => None,
                };
                for (a, b) in ids.windows(2).map(|w| (w[0], w[1])).chain(closing) {
                    if a != b {
                        edges.push((a.min(b), a.max(b)));
                    }
                }
            }
            edges.sort_unstable();
            edges.dedup();
            edges
        })
        .collect()
}

/// The faces `faces` of `mesh` as a part of their own.
fn part<const N: usize>(
    mesh: &MeshFaces<'_, N>,
    faces: Vec<usize>,
    rings: &[Vec<Range<usize>>],
) -> Part<N> {
    let mut remap: HashMap<u32, u32> = HashMap::new();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut face_ends = Vec::with_capacity(faces.len());
    let mut interior_offsets = Vec::new();
    let mut corners = Vec::new();
    for &face in &faces {
        for (r, ring) in rings[face].iter().enumerate() {
            if r > 0 {
                interior_offsets.push(indices.len() as u32);
            }
            for corner in ring.clone() {
                let index = mesh.indices[corner];
                indices.push(*remap.entry(index).or_insert_with(|| {
                    vertices.push(mesh.vertices[index as usize]);
                    (vertices.len() - 1) as u32
                }));
                corners.push(corner);
            }
        }
        face_ends.push(indices.len() as u32);
    }
    Part {
        vertices,
        indices,
        face_ends,
        interior_offsets,
        appearance: faces_appearance(mesh.appearance, &faces, &corners),
        faces,
    }
}

/// Union-find root of `i`, compressing the path.
fn find(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}

/// The faces of each set, given each face's root, ordered by first face.
fn groups(parent: &mut [usize]) -> Vec<Vec<usize>> {
    let mut index_of_root: HashMap<usize, usize> = HashMap::new();
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for face in 0..parent.len() {
        let root = find(parent, face);
        let index = *index_of_root.entry(root).or_insert_with(|| {
            groups.push(Vec::new());
            groups.len() - 1
        });
        groups[index].push(face);
    }
    groups
}

/// Faces grouped by connection through shared edges.
fn edge_connected(face_edges: &[Vec<Edge>]) -> Vec<Vec<usize>> {
    let mut parent: Vec<usize> = (0..face_edges.len()).collect();
    let mut owner: HashMap<Edge, usize> = HashMap::new();
    for (face, edges) in face_edges.iter().enumerate() {
        for &edge in edges {
            match owner.get(&edge) {
                Some(&other) => {
                    let (a, b) = (find(&mut parent, face), find(&mut parent, other));
                    if a != b {
                        parent[a.max(b)] = a.min(b);
                    }
                }
                None => {
                    owner.insert(edge, face);
                }
            }
        }
    }
    groups(&mut parent)
}

/// Faces grouped by connection through shared edges, joining two groups only
/// while every edge borders at most two faces of the joined group. Edges are
/// visited in face order, so the result is deterministic.
fn manifold(face_edges: &[Vec<Edge>]) -> Vec<Vec<usize>> {
    let mut parent: Vec<usize> = (0..face_edges.len()).collect();
    // Per root: how many of its faces border each edge.
    let mut counts: Vec<HashMap<Edge, u8>> = face_edges
        .iter()
        .map(|edges| edges.iter().map(|&e| (e, 1)).collect())
        .collect();
    let mut first: HashMap<Edge, usize> = HashMap::new();
    for (face, edges) in face_edges.iter().enumerate() {
        for &edge in edges {
            let Some(&other) = first.get(&edge) else {
                first.insert(edge, face);
                continue;
            };
            let (a, b) = (find(&mut parent, face), find(&mut parent, other));
            if a == b {
                continue;
            }
            let (big, small) = if counts[a].len() >= counts[b].len() {
                (a, b)
            } else {
                (b, a)
            };
            let fits = counts[small]
                .iter()
                .all(|(e, &n)| counts[big].get(e).copied().unwrap_or(0) + n <= 2);
            if !fits {
                continue;
            }
            let moved = std::mem::take(&mut counts[small]);
            for (e, n) in moved {
                *counts[big].entry(e).or_insert(0) += n;
            }
            parent[small] = big;
        }
    }
    groups(&mut parent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::ops::merge::merge;
    use crate::polygon::Polygon3D;
    use crate::polygon_mesh::PolygonMesh3D;
    use crate::test_support::*;
    use crate::triangular_mesh::TriangularMesh3D;
    use crate::{Euclidean3DGeometry, Geometry};

    const F: CoordinateFrame = CoordinateFrame::Euclidean;

    fn tri(corners: [[f64; 3]; 3]) -> Polygon3D {
        Polygon3D::from_rings(F, corners, Vec::<Vec<[f64; 3]>>::new())
    }

    fn polygon(p: Polygon3D) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(p)))
    }

    fn merged(polygons: Vec<Polygon3D>) -> Geometry {
        merge(polygons.into_iter().map(polygon)).unwrap().unwrap()
    }

    fn as_polygon_mesh(g: &Geometry) -> &PolygonMesh3D {
        let Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(m)) = g else {
            panic!("expected a polygon mesh, got {g:?}");
        };
        m
    }

    fn face_counts(meshes: &[Geometry]) -> Vec<usize> {
        meshes
            .iter()
            .map(|m| as_polygon_mesh(m).num_faces())
            .collect()
    }

    /// Two triangles sharing an edge, a third touching them at one vertex.
    fn bow_tie() -> Vec<Polygon3D> {
        vec![
            tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]),
            tri([[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]),
            tri([[1., 1., 0.], [2., 1., 0.], [2., 2., 0.]]),
        ]
    }

    /// Three fins on the edge (0,0,0)-(1,0,0), plus a triangle continuing the
    /// third fin.
    fn fins() -> Vec<Polygon3D> {
        vec![
            tri([[0., 0., 0.], [1., 0., 0.], [0.5, 1., 0.]]),
            tri([[0., 0., 0.], [1., 0., 0.], [0.5, -1., 0.]]),
            tri([[0., 0., 0.], [1., 0., 0.], [0.5, 0., 1.]]),
            tri([[1., 0., 0.], [0.5, 0., 1.], [1., 0., 1.]]),
        ]
    }

    #[test]
    fn edge_connected_parts_split_at_a_vertex() {
        let parts = merged(bow_tie())
            .decompose_into_edge_connected_parts()
            .unwrap();
        assert_eq!(face_counts(&parts), vec![2, 1]);
        // Each part has a vertex pool of its own.
        assert_eq!(as_polygon_mesh(&parts[0]).vertices().len(), 4);
        assert_eq!(as_polygon_mesh(&parts[1]).vertices().len(), 3);
    }

    #[test]
    fn a_connected_mesh_comes_back_unchanged() {
        let mesh = merged(bow_tie()[..2].to_vec());
        let parts = mesh.decompose_into_edge_connected_parts().unwrap();
        assert_eq!(parts, vec![mesh.clone()]);
        assert_eq!(mesh.decompose_into_manifolds().unwrap(), vec![mesh]);
    }

    #[test]
    fn manifold_parts_split_at_an_edge_of_three_faces() {
        let mesh = merged(fins());
        let edge = mesh.decompose_into_edge_connected_parts().unwrap();
        assert_eq!(face_counts(&edge), vec![4]);
        let manifold = mesh.decompose_into_manifolds().unwrap();
        assert_eq!(face_counts(&manifold), vec![2, 2]);
    }

    #[test]
    fn manifold_parts_are_edge_connected() {
        let parts = merged(bow_tie()).decompose_into_manifolds().unwrap();
        assert_eq!(face_counts(&parts), vec![2, 1]);
    }

    #[test]
    fn a_part_keeps_its_faces_appearance() {
        let mut a = tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]);
        a.set_appearance(
            theme("rgb"),
            textured(),
            Some(explicit_uv(&[[0., 0.], [1., 0.], [0., 1.]])),
        )
        .unwrap();
        let b = tri([[5., 0., 0.], [6., 0., 0.], [5., 1., 0.]]);
        let parts = merged(vec![a, b])
            .decompose_into_edge_connected_parts()
            .unwrap();
        assert_eq!(parts.len(), 2);
        let app = as_polygon_mesh(&parts[0]).appearance().clone().unwrap();
        let crate::appearance::UvSource::Explicit(uv) = &app.themes()[0].uv_sets[0].uv else {
            panic!("expected explicit UV");
        };
        assert_eq!(uv.to_vec(), vec![[0., 0.], [1., 0.], [0., 1.], [0., 0.]]);
        // The bare triangle binds no material.
        assert!(as_polygon_mesh(&parts[1]).appearance().is_none());
    }

    #[test]
    fn triangular_parts_stay_triangular() {
        let mesh = Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(
            TriangularMesh3D::from_parts(
                F,
                vec![
                    [0., 0., 0.],
                    [1., 0., 0.],
                    [0., 1., 0.],
                    [5., 0., 0.],
                    [6., 0., 0.],
                    [5., 1., 0.],
                ],
                vec![0, 1, 2, 3, 4, 5],
            )
            .unwrap(),
        )));
        let parts = mesh.decompose_into_edge_connected_parts().unwrap();
        assert_eq!(parts.len(), 2);
        for part in &parts {
            assert!(matches!(
                part,
                Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(_))
            ));
        }
    }

    #[test]
    fn parts_tell_which_faces_they_hold() {
        let mut polygons = bow_tie();
        // Put the lone triangle between the two that share an edge.
        polygons.swap(1, 2);
        let parts = decompose_with_faces(&merged(polygons), Partition::EdgeConnected).unwrap();
        let faces: Vec<Vec<usize>> = parts.iter().map(|(_, f)| f.clone()).collect();
        assert_eq!(faces, vec![vec![0, 2], vec![1]]);
        assert_eq!(
            face_counts(&parts.into_iter().map(|(g, _)| g).collect::<Vec<_>>()),
            vec![2, 1]
        );
    }

    #[test]
    fn a_single_part_holds_every_face() {
        let mesh = merged(bow_tie()[..2].to_vec());
        let parts = decompose_with_faces(&mesh, Partition::Manifold).unwrap();
        assert_eq!(parts, vec![(mesh, vec![0, 1])]);
    }

    #[test]
    fn other_geometries_are_unsupported() {
        assert!(decompose_with_faces(&polygon(bow_tie().remove(0)), Partition::Manifold).is_err());
        assert!(
            polygon(bow_tie().remove(0))
                .decompose_into_edge_connected_parts()
                .is_err()
        );
        assert!(Geometry::None.decompose_into_manifolds().is_err());
    }
}
