//! Boundary extraction: a volume is bounded by surfaces, an area by curves, a
//! curve by its two endpoints, and a set of points by nothing at all.

use std::collections::HashMap;

use crate::coordinate::CoordinateFrame;
use crate::line_string::{LineString2D, LineString3D};
use crate::ops::coerce::{wrap_2d, wrap_3d};
use crate::ops::UnsupportedOperation;
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// The boundary of a geometry: the shells bounding a volume, the rings bounding
/// a surface, the endpoints bounding a curve.
///
/// * `Ok(geometry)`: the boundary.
/// * `Ok(`[`Geometry::None`]`)`: the boundary is empty.
/// * `Err(`[`UnsupportedOperation`]`)`: the geometry has no boundary to report
///   at all.
///
/// A face's rings come out verbatim, neither re-wound nor re-closed. Appearance
/// is dropped from bounding curves and kept on the shells a volume bounds. A
/// 2.5D chain's endpoints lose its elevation, having no slot for one.
///
/// A container bounds each member into a container of the same kind, keeping the
/// attributes of the members that contributed, and reports `Err` only when every
/// member did. Endpoints shared by two members are not cancelled against each
/// other: cancellation happens only at shared vertex indices within one surface,
/// never by matching coordinates, which float noise makes unreliable.
#[enum_dispatch::enum_dispatch]
pub trait ExtractBoundary {
    /// This geometry's boundary.
    fn extract_boundary(&self) -> Result<Geometry, UnsupportedOperation> {
        Err(unsupported::<Self>())
    }
}

// The boxed enum variants (`Box<Polygon3D>`, `Box<Solid>`, …) need the trait on
// the `Box` itself: `enum_dispatch` forwards by UFCS, not auto-deref.
impl<T: ExtractBoundary + ?Sized> ExtractBoundary for Box<T> {
    fn extract_boundary(&self) -> Result<Geometry, UnsupportedOperation> {
        (**self).extract_boundary()
    }
}

pub(crate) fn unsupported<T: ?Sized>() -> UnsupportedOperation {
    UnsupportedOperation {
        geometry: core::any::type_name::<T>(),
        operation: "extract_boundary",
    }
}

/// The two ends of an open chain, or `None` if it closes on itself or spans
/// nothing.
pub(crate) fn endpoints<const N: usize>(coords: &[[f64; N]]) -> Option<([f64; N], [f64; N])> {
    let (first, last) = (coords.first()?, coords.last()?);
    if coords.len() < 2 || first == last {
        return None;
    }
    Some((*first, *last))
}

/// Counts how many face corners walk each edge of a surface, so the edges only
/// one of them walks, its boundary, can be chained into rings.
///
/// Edges are keyed by vertex index, so corners the mesh keeps distinct stay
/// distinct: a pool that repeats a position reports both copies as boundary,
/// which is the truth for faces it does not join.
#[derive(Default)]
pub(crate) struct BoundaryEdges {
    /// Undirected edge `[min, max]` to how many corners walked it and the
    /// direction the first of them took.
    seen: HashMap<[u32; 2], (u32, [u32; 2])>,
}

impl BoundaryEdges {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Walk one face ring, stored open or closed: it closes either way.
    pub(crate) fn add_ring(&mut self, ring: &[u32]) {
        let n = match ring.split_last() {
            Some((last, rest)) if !rest.is_empty() && *last == ring[0] => rest.len(),
            _ => ring.len(),
        };
        if n < 2 {
            return;
        }
        for i in 0..n {
            self.add_edge(ring[i], ring[(i + 1) % n]);
        }
    }

    pub(crate) fn add_triangle(&mut self, [i, j, k]: [u32; 3]) {
        self.add_edge(i, j);
        self.add_edge(j, k);
        self.add_edge(k, i);
    }

    fn add_edge(&mut self, from: u32, to: u32) {
        // A corner repeated back to back spans nothing.
        if from == to {
            return;
        }
        let key = if from < to { [from, to] } else { [to, from] };
        self.seen.entry(key).or_insert((0, [from, to])).0 += 1;
    }

    /// The boundary edges chained into paths of vertex indices, a path that
    /// closes repeating its first index last. Empty when the surface is closed.
    pub(crate) fn into_paths(self) -> Vec<Vec<u32>> {
        let mut edges: Vec<[u32; 2]> = self
            .seen
            .into_values()
            .filter_map(|(walks, edge)| (walks == 1).then_some(edge))
            .collect();
        if edges.is_empty() {
            return Vec::new();
        }

        edges.sort_unstable();

        let mut incident: HashMap<u32, Vec<usize>> = HashMap::new();
        for (i, &[from, to]) in edges.iter().enumerate() {
            incident.entry(from).or_default().push(i);
            incident.entry(to).or_default().push(i);
        }

        let mut walked = vec![false; edges.len()];
        let mut paths = Vec::new();
        let odd = |v: u32| incident.get(&v).is_some_and(|at| at.len() % 2 == 1);
        for start in 0..edges.len() {
            if walked[start] {
                continue;
            }
            let [from, to] = edges[start];
            let end = if odd(from) {
                from
            } else if odd(to) {
                to
            } else {
                continue;
            };
            paths.push(walk(end, &edges, &incident, &mut walked));
        }
        for start in 0..edges.len() {
            if !walked[start] {
                paths.push(walk(edges[start][0], &edges, &incident, &mut walked));
            }
        }
        paths
    }
}

/// Follow boundary edges from `start` until the path closes or runs out.
fn walk(
    start: u32,
    edges: &[[u32; 2]],
    incident: &HashMap<u32, Vec<usize>>,
    walked: &mut [bool],
) -> Vec<u32> {
    let mut path = vec![start];
    let mut current = start;
    while let Some(next) = incident
        .get(&current)
        .and_then(|at| at.iter().copied().find(|&i| !walked[i]))
    {
        walked[next] = true;
        let [from, to] = edges[next];
        current = if from == current { to } else { from };
        path.push(current);
        if current == path[0] {
            break;
        }
    }
    path
}

/// The boundary a 2D surface's ledger describes, at its elevation.
/// [`Geometry::None`] when the surface is closed.
pub(crate) fn surface_boundary_2d(
    frame: &CoordinateFrame,
    vertices: &[[f64; 2]],
    elevation: Option<f64>,
    edges: BoundaryEdges,
) -> Geometry {
    let lines = edges
        .into_paths()
        .into_iter()
        .map(|path| {
            let coords = path.into_iter().map(|i| vertices[i as usize]);
            Euclidean2DGeometry::LineString(match elevation {
                None => LineString2D::from_coords(frame.clone(), coords),
                Some(elevation) => {
                    LineString2D::from_coords_at_elevation(frame.clone(), coords, elevation)
                }
            })
        })
        .collect();
    wrap_2d(lines).unwrap_or(Geometry::None)
}

/// The 3D counterpart of [`surface_boundary_2d`].
pub(crate) fn surface_boundary_3d(
    frame: &CoordinateFrame,
    vertices: &[[f64; 3]],
    edges: BoundaryEdges,
) -> Geometry {
    let lines = edges
        .into_paths()
        .into_iter()
        .map(|path| {
            let coords = path.into_iter().map(|i| vertices[i as usize]);
            Euclidean3DGeometry::LineString(LineString3D::from_coords(frame.clone(), coords))
        })
        .collect();
    wrap_3d(lines).unwrap_or(Geometry::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::point::Point3D;
    use crate::polygon::Polygon3D;
    use crate::polygon_mesh::PolygonMesh3D;
    use crate::triangular_mesh::TriangularMesh3D;
    use crate::GeometryCollection;
    use pretty_assertions::assert_eq;
    use reearth_flow_common::attribute::{Attribute, AttributeValue, Attributes};

    /// Only the chaining is worth a test here: every other leaf hands its
    /// boundary straight back, which reading the leaf's `ops.rs` shows.
    const SQUARE: [[f64; 3]; 5] = [
        [0.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [4.0, 4.0, 0.0],
        [0.0, 4.0, 0.0],
        [0.0, 0.0, 0.0],
    ];

    fn hole_ring() -> Vec<[f64; 3]> {
        vec![
            [1.0, 1.0, 0.0],
            [1.0, 3.0, 0.0],
            [3.0, 3.0, 0.0],
            [3.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]
    }

    fn face(holes: usize) -> Polygon3D {
        Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            SQUARE,
            (0..holes).map(|_| hole_ring()),
        )
    }

    fn boundary(geometry: &impl ExtractBoundary) -> Geometry {
        geometry.extract_boundary().expect("bounded")
    }

    /// The chain of every polyline a boundary is made of, in emission order.
    fn chains(geometry: &Geometry) -> Vec<Vec<[f64; 3]>> {
        fn parts(g: &Euclidean3DGeometry) -> Vec<Vec<[f64; 3]>> {
            match g {
                Euclidean3DGeometry::LineString(l) => vec![l.coords().to_vec()],
                Euclidean3DGeometry::Collection(c) => c.members().iter().flat_map(parts).collect(),
                other => panic!("unexpected boundary part {other:?}"),
            }
        }
        match geometry {
            Geometry::Euclidean3D(g) => parts(g),
            Geometry::None => Vec::new(),
            other => panic!("expected a 3D boundary, got {other:?}"),
        }
    }

    /// The undirected edges a set of chains covers, panicking on a repeat.
    fn edges_of(chains: &[Vec<[f64; 3]>]) -> Vec<([u64; 3], [u64; 3])> {
        let key = |p: [f64; 3]| p.map(f64::to_bits);
        let mut seen = Vec::new();
        for chain in chains {
            for pair in chain.windows(2) {
                let (a, b) = (key(pair[0]), key(pair[1]));
                let edge = if a <= b { (a, b) } else { (b, a) };
                assert!(!seen.contains(&edge), "edge handed back twice");
                seen.push(edge);
            }
        }
        seen
    }

    fn ring_lengths(geometry: &Geometry) -> Vec<usize> {
        let rings = chains(geometry);
        for ring in &rings {
            assert_eq!(ring.first(), ring.last(), "a ring has to close");
        }
        rings.iter().map(Vec::len).collect()
    }

    #[test]
    fn edges_two_faces_share_cancel_and_the_rest_close_into_rings() {
        // Two triangles sharing a diagonal: the diagonal drops out and the four
        // outer edges make one ring.
        let pair = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            [0u32, 1, 2, 0, 2, 3],
        )
        .unwrap();
        assert_eq!(ring_lengths(&boundary(&pair)), [5]);

        // A closed tetrahedron: every edge is walked twice, so nothing is left.
        let closed = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            [0u32, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3],
        )
        .unwrap();
        assert_eq!(boundary(&closed), Geometry::None);

        // A hole no neighbouring face fills is walked once, like any outer edge,
        // so it bounds the surface too.
        let holed = PolygonMesh3D::from_polygons(CoordinateFrame::Euclidean, [&face(1)]).unwrap();
        assert_eq!(ring_lengths(&boundary(&holed)), [5, 5]);
    }

    // Chaining follows the edges, not the direction they were walked in, so faces
    // wound against each other still bound one closed ring.
    #[test]
    fn a_surface_whose_faces_disagree_on_winding_still_closes() {
        let mesh = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            // The second triangle walks the shared edge 0-2 the same way as the
            // first, so the pair is wound inconsistently.
            [0u32, 1, 2, 0, 3, 2],
        )
        .unwrap();
        assert_eq!(ring_lengths(&boundary(&mesh)), [5]);
    }

    // Three faces on one edge: that edge is walked three times, so it is no more
    // a boundary than a shared edge is. How the six that remain split at the
    // junction is arbitrary, but each must come back exactly once and none as a
    // stranded two-vertex fragment.
    #[test]
    fn a_branching_boundary_comes_back_whole() {
        let mesh = PolygonMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![vec![4u32, 5, 0], vec![4, 5, 1], vec![4, 5, 2]],
        )
        .unwrap();
        let chains = chains(&boundary(&mesh));
        assert_eq!(edges_of(&chains).len(), 6);
        assert!(chains.iter().all(|c| c.len() > 2));
    }

    // Hash iteration order is not stable, so the same surface has to chain the
    // same way every time or downstream splits would shuffle between runs.
    #[test]
    fn chaining_the_same_surface_twice_gives_the_same_answer() {
        let offset = |d: f64| {
            Polygon3D::from_rings(
                CoordinateFrame::Euclidean,
                SQUARE.map(|[x, y, z]| [x + d, y, z]),
                Vec::<Vec<[f64; 3]>>::new(),
            )
        };
        let mesh = PolygonMesh3D::from_polygons(
            CoordinateFrame::Euclidean,
            [&face(1), &offset(10.0), &offset(20.0)],
        )
        .unwrap();
        let first = boundary(&mesh);
        assert_eq!(chains(&first).len(), 4);
        for _ in 0..8 {
            assert_eq!(boundary(&mesh), first);
        }
    }

    // A member that drops out must not shift the attributes of the ones that
    // stay: they are indexed against the source, and the output is compacted.
    #[test]
    fn a_container_keeps_the_attributes_lined_up_when_a_member_drops_out() {
        let attrs = |n: i64| {
            Attributes::from([(
                Attribute::new("lod"),
                AttributeValue::Number(serde_json::Number::from(n)),
            )])
        };
        // The point is bounded by nothing, so only the face contributes.
        let members = vec![
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                CoordinateFrame::Euclidean,
                [0.0; 3],
            ))),
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(face(0)))),
        ];
        let g = Geometry::GeometryCollection(
            GeometryCollection::with_attributes(members, vec![attrs(1), attrs(2)]).unwrap(),
        );
        let Geometry::GeometryCollection(out) = boundary(&g) else {
            panic!("expected a geometry collection");
        };
        assert_eq!(out.members().len(), 1);
        assert_eq!(out.member_attributes(), [attrs(2)]);
    }
}
