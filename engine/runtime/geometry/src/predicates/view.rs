//! Zero-copy coordinate views feeding the [`kernel`].
//!
//! The predicates operate over lightweight borrows of the new leaves' flat
//! buffers rather than over the leaf types directly, so a `Polygon` ring, a
//! `PolygonMesh` face, and a `TriangularMesh` triangle can all reach the same
//! kernel code without copying coordinates.
//!
//! Three layers:
//!
//! - [`RingView`] — one boundary ring, contiguous (`Polygon`) or indexed into a
//!   vertex pool (meshes).
//! - [`FaceView`] / [`AreaView`] — a face as its rings (exterior first), and an
//!   areal leaf as its faces. A `Polygon` is one face; the meshes are face sets.
//!   Mesh views decode the packed CSR index buffers once (indices only, never
//!   coordinates).
//! - [`Leaf2D`] — a flattened, collection-free borrow of a 2D geometry: the
//!   normal form the binary predicates dispatch over.

use super::kernel::{self, Orientation};
use crate::coordinate::CoordinateFrame;
use crate::line_string::LineString2D;
use crate::ops::{Aabb, BoundingBox};
use crate::point::Point2D;
use crate::polygon::{Polygon2D, Polygon3D};
use crate::polygon_mesh::PolygonMesh2D;
use crate::triangular_mesh::TriangularMesh2D;
use crate::Euclidean2DGeometry;

/// The rings of a 2D polygon — exterior first, then each interior (hole) — as
/// borrowed slices over the polygon's own buffer.
pub fn polygon2d_rings(polygon: &Polygon2D) -> impl Iterator<Item = &[[f64; 2]]> {
    core::iter::once(polygon.exterior()).chain(polygon.interiors())
}

/// The rings of a 3D polygon — exterior first, then each interior — as borrowed
/// slices over the polygon's own buffer.
pub fn polygon3d_rings(polygon: &Polygon3D) -> impl Iterator<Item = &[[f64; 3]]> {
    core::iter::once(polygon.exterior()).chain(polygon.interiors())
}

/// Resolve an indexed triangle into its three 2D corners from a vertex pool.
#[inline]
pub fn resolve_triangle_2d(vertices: &[[f64; 2]], tri: [u32; 3]) -> [[f64; 2]; 3] {
    [
        vertices[tri[0] as usize],
        vertices[tri[1] as usize],
        vertices[tri[2] as usize],
    ]
}

/// Resolve an indexed triangle into its three 3D corners from a vertex pool.
#[inline]
pub fn resolve_triangle_3d(vertices: &[[f64; 3]], tri: [u32; 3]) -> [[f64; 3]; 3] {
    [
        vertices[tri[0] as usize],
        vertices[tri[1] as usize],
        vertices[tri[2] as usize],
    ]
}

/// Whether a coordinate lies inside or on a 2D triangle, by robust orientation.
///
/// A point is inside/on when it is on the same side of (or collinear with) all
/// three directed edges; this is orientation-independent, so a CW or CCW triangle
/// gives the same answer.
pub fn point_in_triangle_2d(p: [f64; 2], tri: [[f64; 2]; 3]) -> bool {
    let d0 = kernel::orient2d(tri[0], tri[1], p);
    let d1 = kernel::orient2d(tri[1], tri[2], p);
    let d2 = kernel::orient2d(tri[2], tri[0], p);
    let has_cw = d0 == Orientation::Clockwise
        || d1 == Orientation::Clockwise
        || d2 == Orientation::Clockwise;
    let has_ccw = d0 == Orientation::CounterClockwise
        || d1 == Orientation::CounterClockwise
        || d2 == Orientation::CounterClockwise;
    // Inside/on iff it never lies on both sides across the three edges.
    !(has_cw && has_ccw)
}

// --- ring / face / area views ------------------------------------------------

/// One boundary ring, over either contiguous or indexed storage. The stored
/// form is verbatim: a well-formed ring is closed (first == last) but an open
/// one (e.g. a mesh triangle) is closed implicitly by [`edges`](Self::edges).
#[derive(Clone, Copy, Debug)]
pub enum RingView<'a> {
    /// Ring vertices stored contiguously (`Polygon2D` rings).
    Slice(&'a [[f64; 2]]),
    /// Ring vertices indexed into a shared pool (mesh rings).
    Indexed {
        /// The shared vertex pool.
        pool: &'a [[f64; 2]],
        /// This ring's indices into `pool`, in order.
        indices: &'a [u32],
    },
}

impl<'a> RingView<'a> {
    /// The stored vertex count (including the closing duplicate, if stored).
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            RingView::Slice(s) => s.len(),
            RingView::Indexed { indices, .. } => indices.len(),
        }
    }

    /// Whether the ring stores no vertices.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The `i`-th stored vertex.
    #[inline]
    pub fn coord(&self, i: usize) -> [f64; 2] {
        match self {
            RingView::Slice(s) => s[i],
            RingView::Indexed { pool, indices } => pool[indices[i] as usize],
        }
    }

    /// The stored vertices, in order.
    pub fn coords(self) -> impl Iterator<Item = [f64; 2]> + 'a {
        (0..self.len()).map(move |i| self.coord(i))
    }

    /// The vertex count with the closing duplicate dropped.
    #[inline]
    pub fn open_len(&self) -> usize {
        let n = self.len();
        if n >= 2 && self.coord(0) == self.coord(n - 1) {
            n - 1
        } else {
            n
        }
    }

    /// The directed edges around the ring, closing it when stored open. A
    /// single-vertex ring yields one zero-length edge (so an on-vertex test
    /// still sees it); an empty ring yields nothing.
    pub fn edges(self) -> impl Iterator<Item = ([f64; 2], [f64; 2])> + 'a {
        let n = self.open_len();
        (0..n).map(move |i| {
            (
                self.coord(i),
                self.coord(if i + 1 == n { 0 } else { i + 1 }),
            )
        })
    }
}

/// A single face: its exterior ring followed by its hole rings.
#[derive(Clone, Copy, Debug)]
pub struct FaceView<'a> {
    repr: FaceRepr<'a>,
}

#[derive(Clone, Copy, Debug)]
enum FaceRepr<'a> {
    /// The one face of a polygon, rings borrowed as slices.
    Polygon(&'a Polygon2D),
    /// A mesh face: rings indexed into a pool. `bounds` holds `n_rings + 1`
    /// offsets into `indices`; ring `r` spans `indices[bounds[r]..bounds[r+1]]`.
    Indexed {
        pool: &'a [[f64; 2]],
        indices: &'a [u32],
        bounds: &'a [u32],
    },
}

impl<'a> FaceView<'a> {
    /// The number of rings (1 exterior + holes).
    pub fn num_rings(&self) -> usize {
        match self.repr {
            FaceRepr::Polygon(p) => 1 + p.interiors().count(),
            FaceRepr::Indexed { bounds, .. } => bounds.len() - 1,
        }
    }

    /// The `r`-th ring: 0 is the exterior, the rest are holes.
    pub fn ring(self, r: usize) -> RingView<'a> {
        match self.repr {
            FaceRepr::Polygon(p) => {
                if r == 0 {
                    RingView::Slice(p.exterior())
                } else {
                    RingView::Slice(p.interiors().nth(r - 1).expect("ring index in range"))
                }
            }
            FaceRepr::Indexed {
                pool,
                indices,
                bounds,
            } => RingView::Indexed {
                pool,
                indices: &indices[bounds[r] as usize..bounds[r + 1] as usize],
            },
        }
    }

    /// The rings, exterior first.
    pub fn rings(self) -> impl Iterator<Item = RingView<'a>> {
        (0..self.num_rings()).map(move |r| self.ring(r))
    }

    /// The exterior ring.
    #[inline]
    pub fn exterior(self) -> RingView<'a> {
        self.ring(0)
    }

    /// The hole rings.
    pub fn interiors(self) -> impl Iterator<Item = RingView<'a>> {
        (1..self.num_rings()).map(move |r| self.ring(r))
    }

    /// All directed boundary edges of the face, every ring closed.
    pub fn edges(self) -> impl Iterator<Item = ([f64; 2], [f64; 2])> + use<'a> {
        self.rings().flat_map(RingView::edges)
    }
}

/// Decoded CSR ring layout of a mesh: the coordinate pool stays borrowed, the
/// packed index buffers are widened into flat `u32` once at construction.
#[derive(Debug)]
pub struct MeshFaces<'a> {
    pool: &'a [[f64; 2]],
    /// All rings of all faces, concatenated.
    indices: Vec<u32>,
    /// `n_rings + 1` offsets into `indices`.
    ring_offsets: Vec<u32>,
    /// `n_faces + 1` offsets into `ring_offsets`.
    face_offsets: Vec<u32>,
}

/// An areal leaf as a set of faces: the single face of a `Polygon2D`, or the
/// decoded faces of a mesh.
#[derive(Debug)]
pub enum AreaView<'a> {
    /// A polygon: one face, rings borrowed as slices.
    Polygon(&'a Polygon2D),
    /// A mesh: faces over a decoded CSR layout.
    Mesh(MeshFaces<'a>),
}

impl<'a> AreaView<'a> {
    /// View a polygon as its single face.
    pub fn from_polygon(polygon: &'a Polygon2D) -> Self {
        AreaView::Polygon(polygon)
    }

    /// View a polygon mesh as its faces, decoding the CSR buffers.
    pub fn from_polygon_mesh(mesh: &'a PolygonMesh2D) -> Self {
        let (face_indices, face_bounds, interior_offsets) = mesh.csr_buffers();
        let indices: Vec<u32> = face_indices.iter_u32().map(|[i]| i).collect();
        let holes: Vec<u32> = interior_offsets.iter_u32().map(|[o]| o).collect();
        let n = indices.len() as u32;

        // Corner spans of each face: 0, internal boundaries, end.
        let corner_bounds: Vec<u32> = core::iter::once(0)
            .chain(face_bounds.iter_u32().map(|[o]| o))
            .chain(core::iter::once(n))
            .collect();
        let n_faces = if n == 0 { 0 } else { corner_bounds.len() - 1 };

        let mut ring_offsets: Vec<u32> = vec![0];
        let mut face_offsets: Vec<u32> = vec![0];
        let mut hi = 0;
        for f in 0..n_faces {
            let (start, end) = (corner_bounds[f], corner_bounds[f + 1]);
            // Hole starts strictly inside this face's span split its rings.
            while hi < holes.len() && holes[hi] <= start {
                hi += 1;
            }
            while hi < holes.len() && holes[hi] < end {
                ring_offsets.push(holes[hi]);
                hi += 1;
            }
            ring_offsets.push(end);
            face_offsets.push(ring_offsets.len() as u32 - 1);
        }

        AreaView::Mesh(MeshFaces {
            pool: mesh.vertices(),
            indices,
            ring_offsets,
            face_offsets,
        })
    }

    /// View a triangular mesh as its faces, one 3-vertex ring each.
    pub fn from_triangular_mesh(mesh: &'a TriangularMesh2D) -> Self {
        let n = mesh.num_triangles();
        let indices: Vec<u32> = mesh.triangles().flatten().collect();
        AreaView::Mesh(MeshFaces {
            pool: mesh.vertices(),
            indices,
            ring_offsets: (0..=n).map(|i| (3 * i) as u32).collect(),
            face_offsets: (0..=n).map(|i| i as u32).collect(),
        })
    }

    /// The number of faces.
    pub fn num_faces(&self) -> usize {
        match self {
            AreaView::Polygon(_) => 1,
            AreaView::Mesh(m) => m.face_offsets.len() - 1,
        }
    }

    /// The `f`-th face.
    pub fn face(&self, f: usize) -> FaceView<'_> {
        match self {
            AreaView::Polygon(p) => {
                assert_eq!(f, 0, "a polygon has a single face");
                FaceView {
                    repr: FaceRepr::Polygon(p),
                }
            }
            AreaView::Mesh(m) => FaceView {
                repr: FaceRepr::Indexed {
                    pool: m.pool,
                    indices: &m.indices,
                    bounds: &m.ring_offsets
                        [m.face_offsets[f] as usize..=m.face_offsets[f + 1] as usize],
                },
            },
        }
    }

    /// The faces, in order.
    pub fn faces(&self) -> impl Iterator<Item = FaceView<'_>> + '_ {
        (0..self.num_faces()).map(move |f| self.face(f))
    }

    /// All directed boundary edges across all faces, every ring closed.
    pub fn edges(&self) -> impl Iterator<Item = ([f64; 2], [f64; 2])> + '_ {
        self.faces().flat_map(FaceView::edges)
    }
}

// --- flattened leaf normal form ----------------------------------------------

/// A flattened, collection-free borrow of one 2D leaf: the normal form the
/// binary predicates dispatch over after [`flatten_2d`] unnests collections.
#[derive(Clone, Copy, Debug)]
pub enum Leaf2D<'a> {
    Point(&'a Point2D),
    Line(&'a LineString2D),
    Polygon(&'a Polygon2D),
    PolygonMesh(&'a PolygonMesh2D),
    TriangularMesh(&'a TriangularMesh2D),
}

impl<'a> Leaf2D<'a> {
    /// The leaf's coordinate frame.
    pub fn frame(&self) -> &'a CoordinateFrame {
        match self {
            Leaf2D::Point(p) => p.frame(),
            Leaf2D::Line(l) => l.frame(),
            Leaf2D::Polygon(p) => p.frame(),
            Leaf2D::PolygonMesh(m) => m.frame(),
            Leaf2D::TriangularMesh(m) => m.frame(),
        }
    }

    /// The areal view, for the areal leaves; `None` for points and lines.
    pub fn area_view(&self) -> Option<AreaView<'a>> {
        match self {
            Leaf2D::Point(_) | Leaf2D::Line(_) => None,
            Leaf2D::Polygon(p) => Some(AreaView::from_polygon(p)),
            Leaf2D::PolygonMesh(m) => Some(AreaView::from_polygon_mesh(m)),
            Leaf2D::TriangularMesh(m) => Some(AreaView::from_triangular_mesh(m)),
        }
    }
}

/// Flatten a 2D geometry into its leaves, recursing into (possibly nested)
/// collections.
pub fn flatten_2d<'a>(geometry: &'a Euclidean2DGeometry, out: &mut Vec<Leaf2D<'a>>) {
    match geometry {
        Euclidean2DGeometry::Point(p) => out.push(Leaf2D::Point(p)),
        Euclidean2DGeometry::LineString(l) => out.push(Leaf2D::Line(l)),
        Euclidean2DGeometry::Polygon(p) => out.push(Leaf2D::Polygon(p)),
        Euclidean2DGeometry::PolygonMesh(m) => out.push(Leaf2D::PolygonMesh(m)),
        Euclidean2DGeometry::TriangularMesh(m) => out.push(Leaf2D::TriangularMesh(m)),
        Euclidean2DGeometry::Collection(c) => {
            for member in c.members() {
                flatten_2d(member, out);
            }
        }
    }
}

/// One flattened leaf prepared for predicate evaluation: the borrow itself,
/// the decoded areal view (for areal leaves), and the bounding box (`None`
/// for an empty leaf, which no other geometry intersects).
pub(crate) struct PreparedLeaf<'a> {
    pub leaf: Leaf2D<'a>,
    pub area: Option<AreaView<'a>>,
    pub bbox: Option<Aabb>,
}

/// A predicate operand in normal form: a 2D geometry flattened into prepared
/// leaves (collections unnested, mesh CSR decoded once, boxes precomputed).
pub(crate) struct Operand2D<'a> {
    pub leaves: Vec<PreparedLeaf<'a>>,
}

impl<'a> Operand2D<'a> {
    pub fn new(geometry: &'a Euclidean2DGeometry) -> Self {
        let mut flat = Vec::new();
        flatten_2d(geometry, &mut flat);
        Self::from_leaves(flat)
    }

    pub fn from_leaves(flat: Vec<Leaf2D<'a>>) -> Self {
        let leaves = flat
            .into_iter()
            .map(|leaf| {
                let bbox = match leaf {
                    Leaf2D::Point(p) => p.bounding_box(),
                    Leaf2D::Line(l) => l.bounding_box(),
                    Leaf2D::Polygon(p) => p.bounding_box(),
                    Leaf2D::PolygonMesh(m) => m.bounding_box(),
                    Leaf2D::TriangularMesh(m) => m.bounding_box(),
                }
                .ok();
                PreparedLeaf {
                    leaf,
                    area: leaf.area_view(),
                    bbox,
                }
            })
            .collect();
        Self { leaves }
    }

    /// The areal views among the leaves, in order.
    pub fn areas(&self) -> impl Iterator<Item = &AreaView<'a>> + '_ {
        self.leaves.iter().filter_map(|l| l.area.as_ref())
    }
}

/// Require every leaf across both operands to share one coordinate frame.
pub(crate) fn require_common_frame(a: &Operand2D<'_>, b: &Operand2D<'_>) -> super::Result<()> {
    let mut frames = a
        .leaves
        .iter()
        .chain(b.leaves.iter())
        .map(|l| l.leaf.frame());
    let Some(first) = frames.next() else {
        return Ok(());
    };
    for frame in frames {
        super::require_same_frame(first, frame)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection2D;
    use crate::coordinate::CoordinateFrame;
    use crate::predicates::kernel::{
        coord_pos_relative_to_edges, coord_pos_relative_to_ring, CoordPos,
    };

    fn square_poly(e: CoordinateFrame) -> Polygon2D {
        let square = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let hole = vec![[1.0, 1.0], [1.0, 3.0], [3.0, 3.0], [3.0, 1.0], [1.0, 1.0]];
        Polygon2D::from_rings(e, square, vec![hole])
    }

    #[test]
    fn polygon_rings_yield_exterior_then_holes() {
        let e = CoordinateFrame::Euclidean;
        let square = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let hole = vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0], [1.0, 1.0]];
        let poly = Polygon2D::from_rings(e, square, vec![hole]);

        let rings: Vec<&[[f64; 2]]> = polygon2d_rings(&poly).collect();
        assert_eq!(rings.len(), 2);
        assert_eq!(rings[0], poly.exterior());
        assert_eq!(
            coord_pos_relative_to_ring([2.0, 2.0], rings[0]),
            CoordPos::Inside
        );
        // Second ring is the hole.
        assert_eq!(
            coord_pos_relative_to_ring([2.0, 2.0], rings[1]),
            CoordPos::Inside
        );
    }

    #[test]
    fn point_in_triangle_inside_edge_outside() {
        let tri = [[0.0, 0.0], [4.0, 0.0], [0.0, 4.0]];
        assert!(point_in_triangle_2d([1.0, 1.0], tri));
        assert!(point_in_triangle_2d([2.0, 0.0], tri)); // on an edge
        assert!(point_in_triangle_2d([0.0, 0.0], tri)); // on a vertex
        assert!(!point_in_triangle_2d([3.0, 3.0], tri));
        assert!(!point_in_triangle_2d([-1.0, 1.0], tri));
    }

    #[test]
    fn resolve_triangle_indexes_the_pool() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        assert_eq!(
            resolve_triangle_3d(&verts, [0, 1, 2]),
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
        );
    }

    #[test]
    fn ring_view_edges_close_open_rings() {
        // An open triangle (mesh convention): edges must include the closer.
        let pool = [[0.0, 0.0], [4.0, 0.0], [0.0, 4.0]];
        let ring = RingView::Indexed {
            pool: &pool,
            indices: &[0, 1, 2],
        };
        let edges: Vec<_> = ring.edges().collect();
        assert_eq!(edges.len(), 3);
        assert_eq!(edges[2], ([0.0, 4.0], [0.0, 0.0]));
        // A stored-closed ring adds no extra edge.
        let closed = [[0.0, 0.0], [4.0, 0.0], [0.0, 4.0], [0.0, 0.0]];
        let ring = RingView::Slice(&closed);
        assert_eq!(ring.open_len(), 3);
        assert_eq!(ring.edges().count(), 3);
        assert_eq!(
            coord_pos_relative_to_edges([1.0, 1.0], ring.edges()),
            CoordPos::Inside
        );
    }

    #[test]
    fn polygon_area_view_exposes_face_and_holes() {
        let poly = square_poly(CoordinateFrame::Euclidean);
        let view = AreaView::from_polygon(&poly);
        assert_eq!(view.num_faces(), 1);
        let face = view.face(0);
        assert_eq!(face.num_rings(), 2);
        assert_eq!(face.exterior().open_len(), 4);
        assert_eq!(face.interiors().count(), 1);
        // 4 exterior + 4 hole edges.
        assert_eq!(view.edges().count(), 8);
    }

    #[test]
    fn polygon_mesh_area_view_decodes_faces_and_holes() {
        // Two faces: a plain quad and a quad with a hole.
        let mesh = PolygonMesh2D::from_raw_parts(
            CoordinateFrame::Euclidean,
            vec![
                // Face 0: quad [0,0]..[2,2]
                [0.0, 0.0],
                [2.0, 0.0],
                [2.0, 2.0],
                [0.0, 2.0],
                // Face 1: quad [3,0]..[9,6] with hole [5,2]..[7,4]
                [3.0, 0.0],
                [9.0, 0.0],
                [9.0, 6.0],
                [3.0, 6.0],
                [5.0, 2.0],
                [5.0, 4.0],
                [7.0, 4.0],
                [7.0, 2.0],
            ],
            // Rings stored closed: face 0 = [0..5), face 1 = [5..15) with the
            // hole ring starting at 10.
            vec![0, 1, 2, 3, 0, 4, 5, 6, 7, 4, 8, 9, 10, 11, 8],
            vec![5],
            vec![10],
        )
        .unwrap();

        let view = AreaView::from_polygon_mesh(&mesh);
        assert_eq!(view.num_faces(), 2);
        assert_eq!(view.face(0).num_rings(), 1);
        assert_eq!(view.face(1).num_rings(), 2);
        // Face 1's hole ring resolves to the hole coordinates.
        let hole: Vec<[f64; 2]> = view.face(1).ring(1).coords().collect();
        assert!(hole.contains(&[5.0, 2.0]) && hole.contains(&[7.0, 4.0]));
    }

    #[test]
    fn triangular_mesh_area_view_faces_are_triangles() {
        let mesh = crate::triangular_mesh::TriangularMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
            [0u32, 1, 2, 0, 2, 3],
        )
        .unwrap();
        let view = AreaView::from_triangular_mesh(&mesh);
        assert_eq!(view.num_faces(), 2);
        assert_eq!(view.face(0).num_rings(), 1);
        assert_eq!(view.face(0).exterior().open_len(), 3);
        assert_eq!(view.edges().count(), 6);
    }

    #[test]
    fn flatten_recurses_nested_collections() {
        let e = CoordinateFrame::Euclidean;
        let inner = Collection2D::new([Euclidean2DGeometry::Point(Point2D::new(
            e.clone(),
            [1.0, 1.0],
        ))]);
        let outer = Euclidean2DGeometry::Collection(Collection2D::new([
            Euclidean2DGeometry::Point(Point2D::new(e.clone(), [0.0, 0.0])),
            Euclidean2DGeometry::Collection(inner),
        ]));
        let mut leaves = Vec::new();
        flatten_2d(&outer, &mut leaves);
        assert_eq!(leaves.len(), 2);
        assert!(matches!(leaves[0], Leaf2D::Point(_)));
        assert!(matches!(leaves[1], Leaf2D::Point(_)));
    }
}
