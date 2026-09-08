//! Geometric equality: whether two geometries occupy the same space, to a
//! tolerance.
//!
//! Two geometries occupy the same space when neither strays further than the
//! tolerance from the other: every point of one has a point of the other within
//! that distance, and the other way round. That is the Hausdorff distance
//! between the two point sets, so a shape stays itself under a re-wound ring, a
//! different starting vertex, or an extra vertex sitting on an edge.

use super::{PredicateError, Result};
use crate::collection::{Collection2D, Collection3D};
use crate::csg::Csg;
use crate::ops::{Boundary, ExtractBoundary};
use crate::point_cloud::PointCloud;
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

/// Whether two geometries occupy the same space.
pub trait Equal {
    /// Whether `self` and `rhs` occupy the same space, to `tolerance`.
    fn equal(&self, rhs: &Self, tolerance: f64) -> Result<bool> {
        let _ = (rhs, tolerance);
        Err(PredicateError::Unsupported {
            geometry: core::any::type_name::<Self>(),
        })
    }
}

crate::unsupported!(Csg: self::Equal);
crate::unsupported!(PointCloud: self::Equal);
crate::unsupported!(Collection2D: self::Equal);
crate::unsupported!(Collection3D: self::Equal);
crate::unsupported!(crate::GeometryCollection: self::Equal);

// The boxed enum variants (`Box<Polygon3D>`, `Box<Solid>`, etc.) need no blanket
// impl. The hand-written arms call the leaf method directly, and deref coercion
// reaches through the box on both the receiver and `rhs`.

/// Whether [`Equal`] is defined for `geometry`, without comparing it to
/// anything.
///
/// An absent geometry is comparable by convention.
pub fn is_comparable(geometry: &Geometry) -> bool {
    match geometry {
        Geometry::None => true,
        Geometry::Euclidean2D(g) => matches!(
            g,
            Euclidean2DGeometry::Point(_)
                | Euclidean2DGeometry::LineString(_)
                | Euclidean2DGeometry::Polygon(_)
                | Euclidean2DGeometry::PolygonMesh(_)
                | Euclidean2DGeometry::TriangularMesh(_)
        ),
        // The 3D meshes and `Solid` are missing on purpose: their comparison is
        // unwritten, and the leaf impls panic rather than refuse, so a caller
        // that cannot afford that has to be told here instead.
        Geometry::Euclidean3D(g) => matches!(
            g,
            Euclidean3DGeometry::Point(_)
                | Euclidean3DGeometry::LineString(_)
                | Euclidean3DGeometry::Polygon(_)
        ),
        Geometry::GeometryCollection(_) => false,
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

/// One face reduced to the curves its rings trace, exterior kept apart from
/// holes.
///
/// Weighing all of a face's rings together as one bag would make a face equal to
/// its ring-inverted twin — the invalid face whose exterior is the other's hole
/// — because the two trace the very same curves.
pub(crate) struct FaceCurves {
    exterior: Curves,
    holes: Vec<Curves>,
}

impl FaceCurves {
    pub(crate) fn new<'a>(
        exterior: &[[f64; 3]],
        holes: impl IntoIterator<Item = &'a [[f64; 3]]>,
    ) -> Self {
        Self {
            exterior: Curves::from_ring(exterior),
            holes: holes.into_iter().map(Curves::from_ring).collect(),
        }
    }

    /// Whether the two faces occupy the same space.
    pub(crate) fn within(&self, other: &Self, distance: f64) -> Result<bool> {
        if !self.exterior.may_reach(&other.exterior, distance)
            || !self.exterior.within(&other.exterior, distance)
        {
            return Ok(false);
        }
        // The supporting planes need no separate test: exteriors that stay
        // within `distance` of one another already pin the planes together.
        pair_off(&self.holes, &other.holes, |a, b| {
            Ok(a.may_reach(b, distance) && a.within(b, distance))
        })
    }
}

/// A point set expressed as the straight pieces it is the union of: the
/// segments of every curve, plus each isolated position as a piece of zero
/// length.
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

    /// Whether the two sets are near enough to be worth weighing: their boxes
    /// come within `distance` of one another. A cheap reject before the real
    /// test, which matters when faces are paired off and every pair would
    /// otherwise be measured.
    ///
    /// Only ever a reject: it has to admit every pair [`within`](Self::within)
    /// would accept. An empty set has no box — its bounds are still the
    /// infinities they were seeded with — so the box test would reject two
    /// empty sets, which occupy the same nothing and do stay within any
    /// distance of one another.
    pub(crate) fn may_reach(&self, other: &Self, distance: f64) -> bool {
        if self.pieces.is_empty() || other.pieces.is_empty() {
            return self.pieces.is_empty() == other.pieces.is_empty();
        }
        (0..3).all(|axis| {
            self.min[axis] - distance <= other.max[axis]
                && other.min[axis] - distance <= self.max[axis]
        })
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

/// Lift a 2D coordinate to the elevation its leaf sits at. A leaf without one
/// lies at zero.
pub(crate) fn lift([x, y]: [f64; 2], elevation: Option<f64>) -> [f64; 3] {
    [x, y, elevation.unwrap_or(0.0)]
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
                geometry: other.type_name(),
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
                geometry: other.type_name(),
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

#[cfg(test)]
mod tests {
    use super::*;

    const SQUARE: [[f64; 3]; 5] = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0],
    ];

    #[test]
    fn an_empty_set_reaches_an_empty_set_and_nothing_else() {
        // `may_reach` only ever rejects, so it has to admit every pair `within`
        // accepts. An empty set has no box, which is the case the box test
        // cannot decide on its own.
        let empty = Curves::new().finish();
        let square = Curves::from_ring(&SQUARE);

        assert!(empty.may_reach(&empty, 0.0));
        assert!(empty.within(&empty, 0.0));

        assert!(!empty.may_reach(&square, 1e9));
        assert!(!square.may_reach(&empty, 1e9));
        assert!(!empty.within(&square, 1e9));
    }

    #[test]
    fn faces_pair_off_their_empty_holes() {
        // Reached through `FaceCurves` rather than `Polygon3D`, whose
        // constructor discards an empty interior ring before it gets this far.
        let hole: &[[f64; 3]] = &[];
        let one = FaceCurves::new(&SQUARE, [hole]);
        let other = FaceCurves::new(&SQUARE, [hole]);

        assert!(one.within(&other, 1e-9).unwrap());
    }

    #[test]
    fn a_face_with_an_empty_hole_is_not_one_with_a_real_hole() {
        let hole: &[[f64; 3]] = &[];
        let inner: &[[f64; 3]] = &[
            [0.2, 0.2, 0.0],
            [0.4, 0.2, 0.0],
            [0.4, 0.4, 0.0],
            [0.2, 0.2, 0.0],
        ];
        let empty_hole = FaceCurves::new(&SQUARE, [hole]);
        let real_hole = FaceCurves::new(&SQUARE, [inner]);

        assert!(!empty_hole.within(&real_hole, 1e-9).unwrap());
        assert!(!real_hole.within(&empty_hole, 1e-9).unwrap());
    }
}
