//! Boundary extraction.
//!
//! What bounds a geometry lies one dimension below it: a volume is bounded by
//! surfaces, an area by curves, a curve by its two endpoints, and a set of
//! points by nothing at all.

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
/// * `Ok(`[`Geometry::None`]`)`: the boundary is **empty**. The geometry closes
///   on itself, or carries no extent to bound. An answer, not a failure.
/// * `Err(`[`UnsupportedOperation`]`)`: the type has no boundary to report at
///   all. Today only an unevaluated [`Csg`](crate::csg::Csg) tree and an absent
///   geometry.
///
/// A face's rings come out verbatim, neither re-wound nor re-closed. Appearance
/// is dropped from the curves a surface or face bounds, whose per-corner UV no
/// longer applies, and kept on the shells a volume bounds. A 2.5D chain's
/// endpoints come back as bare points, which have no elevation slot, so its
/// elevation is lost.
///
/// A container takes the boundary of each member and sets the results side by
/// side in a container of the same kind, keeping the attributes of the members
/// that contributed; it reports `Err` only when it has members and *every* one
/// of them did. Endpoints shared
/// by two members are **not** cancelled against each other: cancellation happens
/// only at shared vertex indices within one surface, never by matching
/// coordinates, which float noise makes unreliable.
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

/// This type has no boundary to report.
pub(crate) fn unsupported<T: ?Sized>() -> UnsupportedOperation {
    UnsupportedOperation {
        geometry: core::any::type_name::<T>(),
        operation: "extract_boundary",
    }
}

/// The two ends of an open chain. A chain whose ends meet closes on itself, and
/// one with fewer than two coordinates has no span, so neither is bounded.
pub(crate) fn endpoints<const N: usize>(coords: &[[f64; N]]) -> Option<([f64; N], [f64; N])> {
    let (first, last) = (coords.first()?, coords.last()?);
    if coords.len() < 2 || first == last {
        return None;
    }
    Some((*first, *last))
}

/// Counts how many face corners walk each edge of a surface, so the edges only
/// one of them walks, the surface's boundary, can be chained into rings.
///
/// Edges are keyed by vertex index, so corners the mesh keeps distinct stay
/// distinct. Meshes built from polygons or from a triangle soup deduplicate
/// their corners by exact bits, so their faces already share the vertices they
/// meet at; a caller-supplied pool that repeats a position instead reports both
/// copies as boundary, which is the truth for faces that pool does not join.
#[derive(Default)]
pub(crate) struct BoundaryEdges {
    /// Undirected edge `[min, max]` to how many corners walked it and the
    /// direction the first of them took, so a boundary ring inherits the winding
    /// of the face it came off.
    seen: HashMap<[u32; 2], (u32, [u32; 2])>,
}

impl BoundaryEdges {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Walk one face ring, stored open or closed. A face ring always closes, so
    /// its last vertex is joined back to its first either way.
    pub(crate) fn add_ring(&mut self, ring: &[u32]) {
        // A ring stored closed repeats its first vertex last; that repeat is not
        // a corner of its own.
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

    /// Walk one triangle, whose corners close implicitly.
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
        // Only an edge a single corner walked bounds the surface: two means two
        // faces meet along it, more means they branch there. Two coincident
        // copies of one face therefore cancel each other out, and the doubled
        // surface reports itself closed.
        let mut edges: Vec<[u32; 2]> = self
            .seen
            .into_values()
            .filter_map(|(walks, edge)| (walks == 1).then_some(edge))
            .collect();
        if edges.is_empty() {
            return Vec::new();
        }
        // Hash order is not stable across runs, so the same surface has to be
        // put back in order to chain the same way twice.
        edges.sort_unstable();

        // Chaining ignores which way each edge was walked. Neighbouring faces
        // may be wound against each other, and a ring has to close regardless;
        // following the walked directions would leave it open.
        let mut incident: HashMap<u32, Vec<usize>> = HashMap::new();
        for (i, &[from, to]) in edges.iter().enumerate() {
            incident.entry(from).or_default().push(i);
            incident.entry(to).or_default().push(i);
        }

        let mut walked = vec![false; edges.len()];
        let mut paths = Vec::new();
        // Open chains first, from the vertices an odd number of boundary edges
        // meet at, which can only be ends. Starting mid-chain would cut it in two.
        let odd = |v: u32| incident.get(&v).is_some_and(|at| at.len() % 2 == 1);
        for start in 0..edges.len() {
            if walked[start] {
                continue;
            }
            let [from, to] = edges[start];
            // `from` first, so a chain leaves the way its face walked it.
            let end = if odd(from) {
                from
            } else if odd(to) {
                to
            } else {
                continue;
            };
            paths.push(walk(end, &edges, &incident, &mut walked));
        }
        // Every edge not on one of those chains lies on a ring that closes.
        for start in 0..edges.len() {
            if !walked[start] {
                paths.push(walk(edges[start][0], &edges, &incident, &mut walked));
            }
        }
        paths
    }
}

/// Follow boundary edges from `start` until the path closes on its own first
/// vertex or leaves no unwalked edge to take, marking each edge walked.
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

/// The boundary a 2D surface's ledger describes, at the surface's elevation.
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
    use crate::collection::{Collection2D, Collection3D};
    use crate::csg::Csg;
    use crate::point::{Point2D, Point3D};
    use crate::point_cloud::PointCloud;
    use crate::polygon::{Polygon2D, Polygon3D};
    use crate::polygon_mesh::{PolygonMesh2D, PolygonMesh3D, PolygonMesh3DData};
    use crate::solid::{Shell, Solid};
    use crate::triangular_mesh::{TriangularMesh2D, TriangularMesh3D};
    use crate::GeometryCollection;
    use pretty_assertions::assert_eq;
    use reearth_flow_common::attribute::{Attribute, AttributeValue, Attributes};

    /// A closed 4x4 square, as an exterior ring.
    const SQUARE: [[f64; 3]; 5] = [
        [0.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [4.0, 4.0, 0.0],
        [0.0, 4.0, 0.0],
        [0.0, 0.0, 0.0],
    ];

    /// [`SQUARE`] without its z.
    const SQUARE_2D: [[f64; 2]; 5] = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];

    /// A closed square hole ring of side 2, centred in [`SQUARE`].
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

    fn g3(g: Euclidean3DGeometry) -> Geometry {
        Geometry::Euclidean3D(g)
    }

    fn point() -> Euclidean3DGeometry {
        Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Euclidean, [0.0; 3]))
    }

    fn csg() -> Euclidean3DGeometry {
        Euclidean3DGeometry::Csg(Csg::union(
            Solid::from_exterior(
                CoordinateFrame::Euclidean,
                PolygonMesh3DData::from_polygons([&face(0)]),
            ),
            Solid::from_exterior(
                CoordinateFrame::Euclidean,
                PolygonMesh3DData::from_polygons([&face(0)]),
            ),
        ))
    }

    /// A closed tetrahedron as a triangle mesh: every edge is walked twice.
    fn tetrahedron() -> TriangularMesh3D {
        TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            [0u32, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3],
        )
        .unwrap()
    }

    /// The parts of a boundary: the chain of every polyline, or the position of
    /// every point, in emission order.
    fn chains_3d(geometry: &Geometry) -> Vec<Vec<[f64; 3]>> {
        fn parts(g: &Euclidean3DGeometry) -> Vec<Vec<[f64; 3]>> {
            match g {
                Euclidean3DGeometry::LineString(l) => vec![l.coords().to_vec()],
                Euclidean3DGeometry::Point(p) => vec![vec![p.position()]],
                Euclidean3DGeometry::Collection(c) => c.members().iter().flat_map(parts).collect(),
                other => panic!("unexpected boundary part {other:?}"),
            }
        }
        match geometry {
            Geometry::Euclidean3D(g) => parts(g),
            other => panic!("expected a 3D boundary, got {other:?}"),
        }
    }

    /// The undirected edges a set of chains covers, panicking on a repeat: every
    /// boundary edge must be handed back exactly once.
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

    fn boundary(geometry: &impl ExtractBoundary) -> Geometry {
        geometry.extract_boundary().expect("bounded")
    }

    #[test]
    fn a_face_is_bounded_by_its_rings_exterior_first() {
        assert_eq!(chains_3d(&boundary(&face(0))), vec![SQUARE.to_vec()]);
        assert_eq!(
            chains_3d(&boundary(&face(1))),
            vec![SQUARE.to_vec(), hole_ring()]
        );
    }

    #[test]
    fn a_face_with_no_exterior_ring_has_no_boundary_to_give() {
        let empty = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            Vec::<[f64; 3]>::new(),
            Vec::<Vec<[f64; 3]>>::new(),
        );
        assert!(empty.extract_boundary().is_err());
    }

    #[test]
    fn a_2d_face_carries_its_elevation_onto_its_rings() {
        let face = Polygon2D::from_rings_at_elevation(
            CoordinateFrame::Euclidean,
            SQUARE_2D,
            Vec::<Vec<[f64; 2]>>::new(),
            7.5,
        );
        let Geometry::Euclidean2D(Euclidean2DGeometry::LineString(line)) = boundary(&face) else {
            panic!("expected one polyline");
        };
        assert_eq!(line.elevation(), Some(7.5));
    }

    #[test]
    fn an_open_chain_is_bounded_by_its_two_ends() {
        let chain = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2.0, 0.0, 0.0]],
        );
        assert_eq!(
            chains_3d(&boundary(&chain)),
            vec![vec![[0.0, 0.0, 0.0]], vec![[2.0, 0.0, 0.0]]]
        );
    }

    #[test]
    fn a_chain_that_closes_or_spans_nothing_is_bounded_by_nothing() {
        for coords in [
            SQUARE.to_vec(),
            vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
            vec![[0.0, 0.0, 0.0]],
            Vec::new(),
        ] {
            let chain = LineString3D::from_coords(CoordinateFrame::Euclidean, coords);
            assert_eq!(boundary(&chain), Geometry::None);
        }
    }

    #[test]
    fn a_position_is_bounded_by_nothing() {
        let cloud = PointCloud::from_positions(CoordinateFrame::Euclidean, [[0.0, 0.0, 0.0]]);
        assert_eq!(boundary(&point()), Geometry::None);
        assert_eq!(
            boundary(&Point2D::new(CoordinateFrame::Euclidean, [0.0, 0.0])),
            Geometry::None
        );
        assert_eq!(boundary(&cloud), Geometry::None);
    }

    #[test]
    fn an_unevaluated_boolean_tree_and_an_absent_geometry_have_no_boundary_to_give() {
        assert!(g3(csg()).extract_boundary().is_err());
        assert!(Geometry::None.extract_boundary().is_err());
    }

    // Two triangles sharing an edge: that edge is walked twice and drops out, so
    // the four outer edges chain into the one ring around the pair.
    #[test]
    fn a_surface_is_bounded_by_the_edges_only_one_face_walks() {
        let mesh = TriangularMesh3D::from_parts(
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
        let rings = chains_3d(&boundary(&mesh));
        assert_eq!(rings.len(), 1);
        let ring = &rings[0];
        // A closed ring around all four corners, repeating where it began.
        assert_eq!(ring.len(), 5);
        assert_eq!(ring.first(), ring.last());
        // Every corner once, so the ring goes round the pair rather than
        // doubling back over one triangle.
        for corner in [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ] {
            assert_eq!(ring[..4].iter().filter(|&&c| c == corner).count(), 1);
        }
        // The shared diagonal is not part of it.
        assert!(!ring.windows(2).any(|e| {
            (e[0] == [0.0, 0.0, 0.0] && e[1] == [1.0, 1.0, 0.0])
                || (e[0] == [1.0, 1.0, 0.0] && e[1] == [0.0, 0.0, 0.0])
        }));
    }

    // Neighbouring faces wound against each other still bound one closed ring:
    // chaining follows the edges, not the direction they were walked in.
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
        let rings = chains_3d(&boundary(&mesh));
        assert_eq!(rings.len(), 1);
        assert_eq!(rings[0].len(), 5);
        assert_eq!(rings[0].first(), rings[0].last());
    }

    #[test]
    fn a_closed_shell_is_bounded_by_nothing() {
        assert_eq!(boundary(&tetrahedron()), Geometry::None);
    }

    // A hole no neighbouring face fills is walked once, like any outer edge, so
    // the surface is bounded by both rings.
    #[test]
    fn a_surface_is_bounded_by_the_hole_rings_nothing_fills() {
        let mesh = PolygonMesh3D::from_polygons(CoordinateFrame::Euclidean, [&face(1)]).unwrap();
        let rings = chains_3d(&boundary(&mesh));
        assert_eq!(rings.len(), 2);
        for ring in &rings {
            assert_eq!(ring.len(), 5);
            assert_eq!(ring.first(), ring.last());
        }
    }

    #[test]
    fn a_2d_surface_carries_its_elevation_onto_its_boundary() {
        let corners = vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0]];
        let faces = PolygonMesh2D::from_parts_at_elevation(
            CoordinateFrame::Euclidean,
            corners.clone(),
            vec![vec![0u32, 1, 2]],
            3.0,
        )
        .unwrap();
        let triangles = TriangularMesh2D::from_parts_at_elevation(
            CoordinateFrame::Euclidean,
            corners,
            [0u32, 1, 2],
            3.0,
        )
        .unwrap();
        for boundary in [boundary(&faces), boundary(&triangles)] {
            let Geometry::Euclidean2D(Euclidean2DGeometry::LineString(line)) = boundary else {
                panic!("expected one ring");
            };
            assert_eq!(line.elevation(), Some(3.0));
            assert_eq!(line.coords().len(), 4);
        }
    }

    #[test]
    fn an_empty_surface_is_bounded_by_nothing() {
        let mesh =
            PolygonMesh3D::from_parts(CoordinateFrame::Euclidean, vec![], Vec::<Vec<u32>>::new())
                .unwrap();
        assert_eq!(boundary(&mesh), Geometry::None);
    }

    // Vertices the pool keeps distinct are distinct corners, so faces it does not
    // join are not joined here either.
    #[test]
    fn faces_the_vertex_pool_does_not_join_stay_apart() {
        let mesh = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                // The same three positions again, under their own indices.
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            [0u32, 1, 2, 3, 4, 5],
        )
        .unwrap();
        assert_eq!(chains_3d(&boundary(&mesh)).len(), 2);
    }

    // Three faces sharing one edge: that edge is walked three times, so it is no
    // more a boundary than a shared edge is. How the six that remain split at the
    // junction is arbitrary, but every one of them must come back exactly once,
    // and none may come back as a stranded two-vertex fragment.
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
        let chains = chains_3d(&boundary(&mesh));
        assert_eq!(edges_of(&chains).len(), 6);
        assert!(chains.iter().all(|c| c.len() > 2));
    }

    #[test]
    fn a_volume_is_bounded_by_its_shells_exterior_first() {
        let solid = Solid::new(
            CoordinateFrame::Euclidean,
            PolygonMesh3DData::from_polygons([&face(0)]),
            vec![Shell::from(PolygonMesh3DData::from_polygons([&face(1)]))],
        );
        let Geometry::Euclidean3D(Euclidean3DGeometry::Collection(shells)) = boundary(&solid)
        else {
            panic!("expected the two shells");
        };
        assert_eq!(shells.members().len(), 2);
        for member in shells.members() {
            assert!(matches!(member, Euclidean3DGeometry::PolygonMesh(_)));
        }
    }

    // Twice over: the shells of a watertight volume are bounded by nothing, which
    // is what makes them watertight.
    #[test]
    fn the_boundary_of_a_closed_volumes_boundary_is_empty() {
        let solid = Solid::from_exterior(CoordinateFrame::Euclidean, tetrahedron().into_data());
        let shell = boundary(&solid);
        assert_eq!(boundary(&shell), Geometry::None);
    }

    #[test]
    fn a_container_sets_its_members_boundaries_side_by_side() {
        let c = Collection3D::new([
            Euclidean3DGeometry::Polygon(Box::new(face(0))),
            Euclidean3DGeometry::Polygon(Box::new(face(1))),
        ]);
        let rings = chains_3d(&boundary(&g3(Euclidean3DGeometry::Collection(c))));
        assert_eq!(rings.len(), 3);
    }

    // One curve among the surfaces must not discard the surfaces, and a member
    // bounded by nothing contributes nothing rather than an empty slot.
    #[test]
    fn a_container_skips_the_members_that_give_nothing() {
        let c = Collection3D::new([
            Euclidean3DGeometry::Polygon(Box::new(face(0))),
            point(),
            csg(),
        ]);
        let rings = chains_3d(&boundary(&g3(Euclidean3DGeometry::Collection(c))));
        assert_eq!(rings, vec![SQUARE.to_vec()]);
    }

    #[test]
    fn a_container_no_member_of_which_has_a_boundary_has_none_to_give() {
        let c = Collection3D::new([csg()]);
        assert!(g3(Euclidean3DGeometry::Collection(c))
            .extract_boundary()
            .is_err());
    }

    #[test]
    fn a_container_whose_members_are_all_bounded_by_nothing_is_bounded_by_nothing() {
        let c = Collection3D::new([point(), point()]);
        assert_eq!(
            boundary(&g3(Euclidean3DGeometry::Collection(c))),
            Geometry::None
        );
        assert_eq!(
            boundary(&g3(Euclidean3DGeometry::Collection(Collection3D::new(
                Vec::new()
            )))),
            Geometry::None
        );
    }

    // A container's boundary is a container of the same kind, whatever its
    // members bound, so a downstream match does not turn on how many of them
    // contributed or on whether the source carried child attributes.
    #[test]
    fn a_2d_container_stays_a_two_dimensional_container() {
        let c = Collection2D::new([Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                CoordinateFrame::Euclidean,
                SQUARE_2D,
                Vec::<Vec<[f64; 2]>>::new(),
            ),
        ))]);
        let Geometry::Euclidean2D(Euclidean2DGeometry::Collection(out)) =
            boundary(&Geometry::Euclidean2D(Euclidean2DGeometry::Collection(c)))
        else {
            panic!("expected a 2D collection");
        };
        assert!(matches!(
            out.members(),
            [Euclidean2DGeometry::LineString(_)]
        ));
    }

    #[test]
    fn boundaries_reach_through_nested_collections() {
        let inner = Geometry::GeometryCollection(GeometryCollection::new([g3(
            Euclidean3DGeometry::Polygon(Box::new(face(0))),
        )]));
        let outer = Geometry::GeometryCollection(GeometryCollection::new([
            inner,
            g3(Euclidean3DGeometry::Polygon(Box::new(face(1)))),
            Geometry::None,
        ]));
        let Geometry::GeometryCollection(out) = boundary(&outer) else {
            panic!("expected a geometry collection");
        };
        // The absent member has no boundary to give and drops out.
        assert_eq!(out.members().len(), 2);
    }

    #[test]
    fn a_container_keeps_the_attributes_lined_up_when_a_member_drops_out() {
        let attrs = |n: i64| {
            Attributes::from([(
                Attribute::new("lod"),
                AttributeValue::Number(serde_json::Number::from(n)),
            )])
        };
        let members = vec![
            g3(point()),
            g3(Euclidean3DGeometry::Polygon(Box::new(face(0)))),
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

    // Hash iteration order is not stable, so the same surface has to chain the
    // same way every time or downstream splits would shuffle between runs. The
    // faces are disjoint, so several rings come back and there is an order to get
    // wrong.
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
        // Two rings from the holed face, one from each disjoint square.
        assert_eq!(chains_3d(&boundary(&mesh)).len(), 4);
        let first = boundary(&mesh);
        for _ in 0..8 {
            assert_eq!(boundary(&mesh), first);
        }
    }
}
