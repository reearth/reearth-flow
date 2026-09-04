//! Geometric equality: whether two geometries occupy the same space, to a
//! tolerance.
//!
//! Two geometries occupy the same space when neither strays further than the
//! tolerance from the other: every point of one has a point of the other within
//! that distance, and the other way round. That is the Hausdorff distance
//! between the two point sets, so a shape stays itself under a re-wound ring, a
//! different starting vertex, or an extra vertex sitting on an edge.
//!
//! Unlike the neighbouring predicates, which are free functions that match the
//! operand pair internally, this one is a trait: what "the same space" means is
//! genuinely leaf-specific — a face weighs its exterior against exteriors and
//! its holes against holes, a mesh first has to decide which of its edges are
//! real — so each leaf answers for itself. That rules out `#[enum_dispatch]`,
//! which dispatches on the receiver only and has no way to match `rhs` in
//! lockstep, so every enum level dispatches by hand instead.
//!
//! A collection is answered for only when it denotes exactly one geometry;
//! nesting is descended to reach it. Two or more members are refused rather
//! than guessed at — see [`denoted`] for why neither reading of such a
//! collection is faithful here.

use std::collections::HashMap;

use super::{PredicateError, Result};
use crate::ops::{Boundary, ExtractBoundary};
use crate::predicates::view3d::TriangleSet;
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

use rstar::{PointDistance, RTree, RTreeObject, AABB};

/// Number of pieces above which a point set gets its own spatial index; below
/// it, scanning the pieces costs less than building and walking a tree.
const INDEX_THRESHOLD: usize = 64;

/// How many sub-segments one piece may be split into while deciding whether it
/// stays within the tolerance. The refinement below halves a sub-segment only
/// where neither exact test settles it, so this is reached only by a piece that
/// hugs the tolerance along its whole length.
const REFINEMENT_BUDGET: usize = 4096;

/// How much slack the comparison allows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tolerance {
    /// Greatest distance, in the units the coordinates are expressed in, that
    /// two geometries may stray from one another and still occupy the same
    /// space. Zero admits only coordinates that coincide exactly, which leaves
    /// no room for rounding; prefer a small positive distance.
    pub distance: f64,
    /// Greatest angle, in radians, between two adjacent faces still counted as
    /// lying in one flat facet. This is what lets a mesh be compared
    /// independently of how it was cut into triangles.
    pub coplanarity: f64,
}

/// Whether two geometries occupy the same space.
///
/// Reflexive and symmetric, but **not transitive** above a zero distance: `a`
/// may reach `b` and `b` reach `c` with `a` and `c` further apart than the
/// tolerance. A caller wanting an equivalence — one identifier per shape —
/// must take the transitive closure itself.
pub trait Equal {
    fn equal(&self, rhs: &Self, tolerance: Tolerance) -> Result<bool>;
}

// The boxed enum variants (`Box<Polygon3D>`, `Box<Solid>`, …) need no blanket
// impl: the hand-written arms call the leaf method directly, and deref coercion
// reaches through the box on both the receiver and `rhs`.

/// What a geometry denotes once single-member collections have been descended.
pub(crate) enum Denoted<'a> {
    /// Nothing at all: an absent geometry, or a collection with no members.
    Nothing,
    TwoD(&'a Euclidean2DGeometry),
    ThreeD(&'a Euclidean3DGeometry),
}

/// Descend single-member collections to the one geometry this denotes.
///
/// A collection of two or more members is refused. The two readings of what
/// such a collection means disagree, and on a point set built from boundary
/// curves neither is faithful: taking the union dissolves a face's exterior and
/// its holes into one bag of rings, which is what makes a face equal to its
/// ring-inverted twin, while pairing the members off makes a curve split across
/// two members a different shape from the same curve given whole. A collection
/// denoting exactly one geometry carries no such ambiguity — both readings
/// agree there — so that much is answered.
pub(crate) fn denoted(geometry: &Geometry) -> Result<Denoted<'_>> {
    match geometry {
        Geometry::None => Ok(Denoted::Nothing),
        Geometry::Euclidean2D(g) => Ok(match single_leaf_2d(g)? {
            None => Denoted::Nothing,
            Some(leaf) => Denoted::TwoD(leaf),
        }),
        Geometry::Euclidean3D(g) => Ok(match single_leaf_3d(g)? {
            None => Denoted::Nothing,
            Some(leaf) => Denoted::ThreeD(leaf),
        }),
        Geometry::GeometryCollection(c) => denoted_members(c.members()),
    }
}

/// The one geometry a heterogeneous collection's members denote.
pub(crate) fn denoted_members(members: &[Geometry]) -> Result<Denoted<'_>> {
    match members {
        [] => Ok(Denoted::Nothing),
        [only] => denoted(only),
        _ => Err(PredicateError::Unsupported {
            geometry: "GeometryCollection",
        }),
    }
}

/// Descend single-member collections to the one 2D leaf this denotes.
pub(crate) fn single_leaf_2d(
    geometry: &Euclidean2DGeometry,
) -> Result<Option<&Euclidean2DGeometry>> {
    match geometry {
        Euclidean2DGeometry::Collection(c) => single_of_members_2d(c.members()),
        leaf => Ok(Some(leaf)),
    }
}

/// The one 2D leaf a collection's members denote.
pub(crate) fn single_of_members_2d(
    members: &[Euclidean2DGeometry],
) -> Result<Option<&Euclidean2DGeometry>> {
    match members {
        [] => Ok(None),
        [only] => single_leaf_2d(only),
        _ => Err(PredicateError::Unsupported {
            geometry: "Collection2D",
        }),
    }
}

/// Descend single-member collections to the one 3D leaf this denotes.
pub(crate) fn single_leaf_3d(
    geometry: &Euclidean3DGeometry,
) -> Result<Option<&Euclidean3DGeometry>> {
    match geometry {
        Euclidean3DGeometry::Collection(c) => single_of_members_3d(c.members()),
        leaf => Ok(Some(leaf)),
    }
}

/// The one 3D leaf a collection's members denote.
pub(crate) fn single_of_members_3d(
    members: &[Euclidean3DGeometry],
) -> Result<Option<&Euclidean3DGeometry>> {
    match members {
        [] => Ok(None),
        [only] => single_leaf_3d(only),
        _ => Err(PredicateError::Unsupported {
            geometry: "Collection3D",
        }),
    }
}

/// Whether two bags pair off one-to-one under `matches`.
///
/// Greedy first-fit is not enough. Above a zero distance the relation is not
/// transitive, so a greedy choice can consume the only partner another member
/// had, and report a difference where some pairing would have succeeded. This
/// takes a maximum bipartite matching instead.
pub(crate) fn pair_off<T, U>(
    left: &[T],
    right: &[U],
    matches: impl Fn(&T, &U) -> Result<bool>,
) -> Result<bool> {
    // Bags of different sizes cannot pair off. That is an answer, not a refusal.
    if left.len() != right.len() {
        return Ok(false);
    }
    // Each comparison can be a full Hausdorff test, so every pair is weighed
    // once, up front, and the matching runs over the answers.
    let mut adjacency: Vec<Vec<bool>> = Vec::with_capacity(left.len());
    for a in left {
        let mut row = Vec::with_capacity(right.len());
        for b in right {
            row.push(matches(a, b)?);
        }
        adjacency.push(row);
    }
    let mut owner = vec![usize::MAX; right.len()];
    Ok((0..left.len()).all(|member| {
        let mut seen = vec![false; right.len()];
        augment(member, &adjacency, &mut owner, &mut seen)
    }))
}

/// Find a partner for `member`, displacing earlier pairings along the way.
fn augment(member: usize, adjacency: &[Vec<bool>], owner: &mut [usize], seen: &mut [bool]) -> bool {
    for candidate in 0..owner.len() {
        if seen[candidate] || !adjacency[member][candidate] {
            continue;
        }
        seen[candidate] = true;
        if owner[candidate] == usize::MAX || augment(owner[candidate], adjacency, owner, seen) {
            owner[candidate] = member;
            return true;
        }
    }
    false
}

/// A point set expressed as the straight pieces it is the union of: the
/// segments of every curve, plus each isolated position as a piece of zero
/// length.
///
/// This is what a leaf reduces itself to before comparing. Reducing to *which*
/// curves is the leaf's decision — a face gives one ring at a time, a mesh
/// gives the edges that survive its facet merging.
#[derive(Debug, Clone)]
pub(crate) struct Curves {
    pieces: Vec<Piece>,
    min: [f64; 3],
    max: [f64; 3],
    /// Built only for a set large enough for the scan to cost more than the
    /// tree; see [`INDEX_THRESHOLD`].
    index: Option<RTree<Piece>>,
}

/// One straight piece of a point set. A position is the degenerate case where
/// both ends coincide.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Piece {
    a: [f64; 3],
    b: [f64; 3],
}

impl RTreeObject for Piece {
    type Envelope = AABB<[f64; 3]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(self.a, self.b)
    }
}

impl PointDistance for Piece {
    fn distance_2(&self, point: &[f64; 3]) -> f64 {
        piece_distance_2(*point, self)
    }
}

impl Curves {
    pub(crate) fn new() -> Self {
        Self {
            pieces: Vec::new(),
            min: [f64::INFINITY; 3],
            max: [f64::NEG_INFINITY; 3],
            index: None,
        }
    }

    /// The closed curve one ring traces. A ring stored open is closed here: a
    /// ring is a loop whether or not the closing vertex was written down.
    pub(crate) fn from_ring(ring: &[[f64; 3]]) -> Self {
        let mut curves = Self::new();
        curves.push_ring(ring);
        curves.finish()
    }

    pub(crate) fn push_ring(&mut self, ring: &[[f64; 3]]) {
        self.push_chain(ring.iter().copied());
        if let (Some(&first), Some(&last)) = (ring.first(), ring.last()) {
            if first != last {
                self.push_piece(last, first);
            }
        }
    }

    /// Push the segments between consecutive coordinates of a chain. A chain of
    /// one coordinate contributes that position.
    pub(crate) fn push_chain(&mut self, coords: impl IntoIterator<Item = [f64; 3]>) {
        let mut previous: Option<[f64; 3]> = None;
        for coord in coords {
            match previous {
                None => self.push_piece(coord, coord),
                Some(previous) => self.push_piece(previous, coord),
            }
            previous = Some(coord);
        }
    }

    pub(crate) fn push_piece(&mut self, a: [f64; 3], b: [f64; 3]) {
        // A chain's first coordinate enters as a position and is covered again
        // by the piece that follows it, so drop the position once it is.
        if let Some(last) = self.pieces.last() {
            if last.a == last.b && last.a == a {
                self.pieces.pop();
            }
        }
        for axis in 0..3 {
            self.min[axis] = self.min[axis].min(a[axis]).min(b[axis]);
            self.max[axis] = self.max[axis].max(a[axis]).max(b[axis]);
        }
        self.pieces.push(Piece { a, b });
    }

    /// Index the set if it is big enough to want indexing. Call once, after the
    /// last piece is in.
    pub(crate) fn finish(mut self) -> Self {
        if self.pieces.len() > INDEX_THRESHOLD {
            self.index = Some(RTree::bulk_load(self.pieces.clone()));
        }
        self
    }

    /// Whether the two sets occupy the same space: neither strays further than
    /// `distance` from the other.
    pub(crate) fn within(&self, other: &Self, distance: f64) -> bool {
        self.covers(other, distance) && other.covers(self, distance)
    }

    /// Whether every point of `other` lies within `distance` of this set.
    fn covers(&self, other: &Self, distance: f64) -> bool {
        if self.pieces.is_empty() != other.pieces.is_empty() {
            return false;
        }
        other
            .pieces
            .iter()
            .all(|piece| self.covers_segment(piece.a, piece.b, distance))
    }

    /// Whether every point of the straight segment from `a` to `b` lies within
    /// `distance` of this set.
    ///
    /// Two exact tests settle a segment without looking inside it, and a segment
    /// neither settles is halved and retried. The distance to a set is
    /// 1-Lipschitz, which bounds how far it can climb between the two ends; and
    /// a piece is convex, so one piece holding both ends within the tolerance
    /// holds everything between them too.
    fn covers_segment(&self, a: [f64; 3], b: [f64; 3], distance: f64) -> bool {
        let mut pending = vec![(a, b)];
        let mut budget = REFINEMENT_BUDGET;
        while let Some((p, q)) = pending.pop() {
            let dp = self.distance(p);
            let dq = self.distance(q);
            if dp > distance || dq > distance {
                return false;
            }
            if (dp + dq + span(p, q)) / 2.0 <= distance {
                continue;
            }
            if self.holds_both(p, q, distance) {
                continue;
            }
            // Out of refinement: the ends are within the tolerance and the rest
            // of this sub-segment goes undecided rather than failing the set.
            if budget == 0 {
                continue;
            }
            budget -= 1;
            let mid = [
                (p[0] + q[0]) / 2.0,
                (p[1] + q[1]) / 2.0,
                (p[2] + q[2]) / 2.0,
            ];
            pending.push((p, mid));
            pending.push((mid, q));
        }
        true
    }

    /// Distance from `point` to the nearest piece.
    fn distance(&self, point: [f64; 3]) -> f64 {
        let squared = match &self.index {
            Some(index) => index
                .nearest_neighbor(&point)
                .map(|piece| piece_distance_2(point, piece))
                .unwrap_or(f64::INFINITY),
            None => self
                .pieces
                .iter()
                .map(|piece| piece_distance_2(point, piece))
                .fold(f64::INFINITY, f64::min),
        };
        squared.sqrt()
    }

    /// Whether one piece alone holds both `p` and `q` within `distance`.
    fn holds_both(&self, p: [f64; 3], q: [f64; 3], distance: f64) -> bool {
        let limit = distance * distance;
        let holds = |piece: &Piece| piece_distance_2(q, piece) <= limit;
        match &self.index {
            Some(index) => index.locate_within_distance(p, limit).any(holds),
            None => self
                .pieces
                .iter()
                .filter(|piece| piece_distance_2(p, piece) <= limit)
                .any(holds),
        }
    }
}

/// Face edges accumulated so the ones that merely cut a flat region can be
/// dropped.
///
/// Which edges to drop is the whole question. Dropping none leaves the answer at
/// the mercy of how a flat region happened to be cut up — two triangulations of
/// one square would compare as different shapes. Dropping every edge two faces
/// share goes too far the other way: a closed shell has no such edge left over
/// and would reduce to nothing at all, and a flat square would come out equal to
/// a tent pitched over it. Dropping only the coplanar ones keeps every crease and
/// every boundary, which is what carries the shape, and discards only the cuts.
pub(crate) struct FacetEdges {
    faces_on_edge: HashMap<[u32; 2], Vec<usize>>,
    normals: Vec<[f64; 3]>,
}

impl FacetEdges {
    pub(crate) fn new() -> Self {
        Self {
            faces_on_edge: HashMap::new(),
            normals: Vec::new(),
        }
    }

    /// Add one face by its welded corner indices and its unit normal. A face
    /// with no normal has no area, so it is no surface and is left out; that can
    /// only turn a shared edge into a boundary one, which is the cautious way to
    /// be wrong.
    pub(crate) fn push_face(&mut self, corners: &[u32], normal: Option<[f64; 3]>) {
        let Some(normal) = normal else {
            return;
        };
        let face = self.normals.len();
        self.normals.push(normal);
        for pair in corners.windows(2) {
            self.add_edge(pair[0], pair[1], face);
        }
        // A ring stored open still closes; one stored closed adds nothing here.
        if let (Some(&first), Some(&last)) = (corners.first(), corners.last()) {
            if first != last {
                self.add_edge(last, first, face);
            }
        }
    }

    fn add_edge(&mut self, from: u32, to: u32, face: usize) {
        if from == to {
            return;
        }
        let edge = if from <= to { [from, to] } else { [to, from] };
        self.faces_on_edge.entry(edge).or_default().push(face);
    }

    /// The curves the surviving edges trace, in the coordinates `position`
    /// gives for each welded index.
    pub(crate) fn into_curves(
        self,
        position: impl Fn(u32) -> [f64; 3],
        coplanarity: f64,
    ) -> Curves {
        let limit = coplanarity.cos();
        let mut curves = Curves::new();
        for (edge, faces) in self.faces_on_edge {
            // Exactly two faces lying in one plane: the edge is an artefact of
            // how the surface was cut up, not a feature of the shape. Anything
            // else — a boundary edge, a crease, a non-manifold junction — is kept.
            if let [one, other] = faces[..] {
                // Unsigned: two faces wound against one another still lie flat.
                if cosine(self.normals[one], self.normals[other]).abs() >= limit {
                    continue;
                }
            }
            curves.push_piece(position(edge[0]), position(edge[1]));
        }
        curves.finish()
    }
}

/// The curves a triangle set's flat facets are bounded by.
///
/// Every surface-bearing 3D leaf reaches this through
/// [`TriangleSet`](crate::predicates::view3d::TriangleSet), so a mesh, a face
/// and a solid's shell are all read the same way.
pub(crate) fn facet_curves(triangles: &TriangleSet<'_>, coplanarity: f64) -> Curves {
    let pool = triangles.pool();
    let welded = weld(pool);
    let mut edges = FacetEdges::new();
    for i in 0..triangles.len() {
        let corners = triangles.indices(i).map(|index| welded[index as usize]);
        let normal = unit_normal(
            pool[corners[0] as usize],
            pool[corners[1] as usize],
            pool[corners[2] as usize],
        );
        edges.push_face(&corners, normal);
    }
    edges.into_curves(|index| pool[index as usize], coplanarity)
}

/// Lift a 2D coordinate to the elevation its leaf sits at. A leaf without one
/// lies at zero.
pub(crate) fn lift([x, y]: [f64; 2], elevation: Option<f64>) -> [f64; 3] {
    [x, y, elevation.unwrap_or(0.0)]
}

/// A leaf's concrete type name, for reporting a refusal.
pub(crate) trait NameOf {
    fn name_of(&self) -> &'static str;
}

impl NameOf for Euclidean2DGeometry {
    fn name_of(&self) -> &'static str {
        match self {
            Euclidean2DGeometry::Point(_) => "Point2D",
            Euclidean2DGeometry::LineString(_) => "LineString2D",
            Euclidean2DGeometry::Polygon(_) => "Polygon2D",
            Euclidean2DGeometry::PolygonMesh(_) => "PolygonMesh2D",
            Euclidean2DGeometry::TriangularMesh(_) => "TriangularMesh2D",
            Euclidean2DGeometry::Collection(_) => "Collection2D",
        }
    }
}

impl NameOf for Euclidean3DGeometry {
    fn name_of(&self) -> &'static str {
        match self {
            Euclidean3DGeometry::Point(_) => "Point3D",
            Euclidean3DGeometry::PointCloud(_) => "PointCloud",
            Euclidean3DGeometry::LineString(_) => "LineString3D",
            Euclidean3DGeometry::Polygon(_) => "Polygon3D",
            Euclidean3DGeometry::PolygonMesh(_) => "PolygonMesh3D",
            Euclidean3DGeometry::TriangularMesh(_) => "TriangularMesh3D",
            Euclidean3DGeometry::Solid(_) => "Solid",
            Euclidean3DGeometry::Csg(_) => "Csg",
            Euclidean3DGeometry::Collection(_) => "Collection3D",
        }
    }
}

/// The curves one 2D chain traces, at the elevation its leaf sits at.
pub(crate) fn chain_curves_2d(coords: &[[f64; 2]], elevation: Option<f64>) -> Curves {
    let mut curves = Curves::new();
    curves.push_chain(coords.iter().map(|&c| lift(c, elevation)));
    curves.finish()
}

/// The closed curve one 2D ring traces, at the elevation its leaf sits at.
pub(crate) fn ring_curves_2d(ring: &[[f64; 2]], elevation: Option<f64>) -> Curves {
    let lifted: Vec<[f64; 3]> = ring.iter().map(|&c| lift(c, elevation)).collect();
    Curves::from_ring(&lifted)
}

/// The curves bounding a 2D surface, through [`ExtractBoundary`].
///
/// In the plane every edge two faces share may be cancelled: a 2D surface has no
/// creases, so a shared edge is always a cut. That is the rule `ExtractBoundary`
/// already applies, which is why a flat mesh needs none of the coplanarity
/// weighing its 3D counterpart does.
pub(crate) fn surface_curves_2d(surface: &impl ExtractBoundary) -> Result<Curves> {
    let boundary = surface
        .extract_boundary()
        .map_err(|e| PredicateError::Unsupported {
            geometry: e.geometry,
        })?;
    let mut curves = Curves::new();
    if let Boundary::Bounded(geometry) = boundary {
        gather_curves(&geometry, &mut curves)?;
    }
    Ok(curves.finish())
}

/// Read every curve and position out of a geometry built only from them, as the
/// boundary of a surface is.
fn gather_curves(geometry: &Geometry, curves: &mut Curves) -> Result<()> {
    fn from_2d(g: &Euclidean2DGeometry, curves: &mut Curves) -> Result<()> {
        match g {
            Euclidean2DGeometry::Point(p) => {
                let position = lift(p.position(), None);
                curves.push_piece(position, position);
                Ok(())
            }
            Euclidean2DGeometry::LineString(l) => {
                curves.push_chain(l.coords().iter().map(|&c| lift(c, l.elevation())));
                Ok(())
            }
            Euclidean2DGeometry::Collection(c) => {
                c.members().iter().try_for_each(|m| from_2d(m, curves))
            }
            other => Err(PredicateError::Unsupported {
                geometry: other.name_of(),
            }),
        }
    }
    fn from_3d(g: &Euclidean3DGeometry, curves: &mut Curves) -> Result<()> {
        match g {
            Euclidean3DGeometry::Point(p) => {
                curves.push_piece(p.position(), p.position());
                Ok(())
            }
            Euclidean3DGeometry::LineString(l) => {
                curves.push_chain(l.coords().iter().copied());
                Ok(())
            }
            Euclidean3DGeometry::Collection(c) => {
                c.members().iter().try_for_each(|m| from_3d(m, curves))
            }
            other => Err(PredicateError::Unsupported {
                geometry: other.name_of(),
            }),
        }
    }
    match geometry {
        Geometry::None => Ok(()),
        Geometry::Euclidean2D(g) => from_2d(g, curves),
        Geometry::Euclidean3D(g) => from_3d(g, curves),
        Geometry::GeometryCollection(c) => c
            .members()
            .iter()
            .try_for_each(|m| gather_curves(m, curves)),
    }
}

/// Squared distance from a point to the nearest point of a piece.
fn piece_distance_2(point: [f64; 3], piece: &Piece) -> f64 {
    let (a, b) = (piece.a, piece.b);
    let along = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let length_2 = along[0] * along[0] + along[1] * along[1] + along[2] * along[2];
    let to_point = [point[0] - a[0], point[1] - a[1], point[2] - a[2]];
    let projection = if length_2 <= 0.0 {
        0.0
    } else {
        ((to_point[0] * along[0] + to_point[1] * along[1] + to_point[2] * along[2]) / length_2)
            .clamp(0.0, 1.0)
    };
    let offset = [
        to_point[0] - projection * along[0],
        to_point[1] - projection * along[1],
        to_point[2] - projection * along[2],
    ];
    offset[0] * offset[0] + offset[1] * offset[1] + offset[2] * offset[2]
}

fn span(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Map each vertex index to the first index carrying the same position, so that
/// faces meeting at a coordinate but written with separate indices still count
/// as neighbours.
///
/// Welding is exact. A near-miss between two positions of *one* mesh is a defect
/// in that mesh, not a difference between two of them, and repairing it is not
/// this operation's job.
pub(crate) fn weld(vertices: &[[f64; 3]]) -> Vec<u32> {
    fn bits(value: f64) -> u64 {
        // `-0.0` and `0.0` are one position written two ways.
        let value = if value == 0.0 { 0.0 } else { value };
        value.to_bits()
    }
    let mut first: HashMap<[u64; 3], u32> = HashMap::new();
    vertices
        .iter()
        .enumerate()
        .map(|(index, position)| *first.entry(position.map(bits)).or_insert(index as u32))
        .collect()
}

/// The unit normal of a triangle, or `None` when it has no area to have one.
pub(crate) fn unit_normal(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Option<[f64; 3]> {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let normal = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    (length > 0.0).then(|| [normal[0] / length, normal[1] / length, normal[2] / length])
}

/// Cosine of the angle between two unit vectors.
pub(crate) fn cosine(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
