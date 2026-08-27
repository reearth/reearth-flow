//! Clipping a set of rings against one axis-aligned half-plane.
//!
//! The rectangle clip is four of these in sequence. A half-plane is the easy
//! case of polygon clipping: every crossing of the boundary lies on one line, so
//! reconnecting the severed pieces is a 1D sort along that line rather than a
//! general planar arrangement. That is what lets this split a concave ring into
//! several rings, which Sutherland-Hodgman cannot do.

// Nothing outside this module's own tests calls these yet: the rectangle clip
// that wires `Edge::MaxX` / `MinY` / `MaxY` and `clip_rings_halfplane` into the
// grid-division op lands in a later task. Drop this once that caller exists.
#![allow(dead_code)]

/// One polygon corner: its position, and the per-corner UV it carries when the
/// host geometry has `Explicit` texture coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Corner<const N: usize> {
    pub pos: [f64; N],
    pub uv: Option<[f64; 2]>,
}

/// One side of the cell, as a half-plane keeping the inside.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Edge {
    /// Keep `x >= v`.
    MinX(f64),
    /// Keep `x <= v`.
    MaxX(f64),
    /// Keep `y >= v`.
    MinY(f64),
    /// Keep `y <= v`.
    MaxY(f64),
}

impl Edge {
    /// The axis this edge cuts on: 0 for x, 1 for y.
    fn axis(self) -> usize {
        match self {
            Edge::MinX(_) | Edge::MaxX(_) => 0,
            Edge::MinY(_) | Edge::MaxY(_) => 1,
        }
    }

    /// The coordinate value of the cut line.
    fn value(self) -> f64 {
        match self {
            Edge::MinX(v) | Edge::MaxX(v) | Edge::MinY(v) | Edge::MaxY(v) => v,
        }
    }

    /// Whether a point lies on the kept side. Points exactly on the line count
    /// as inside, so a face flush against a cell edge is not eroded.
    fn contains<const N: usize>(self, p: &[f64; N]) -> bool {
        let c = p[self.axis()];
        match self {
            Edge::MinX(v) | Edge::MinY(v) => c >= v,
            Edge::MaxX(v) | Edge::MaxY(v) => c <= v,
        }
    }

    /// The other in-plane axis, the one crossings are sorted along.
    fn sort_axis(self) -> usize {
        1 - self.axis()
    }
}

/// The point where segment `a -> b` meets the cut line.
///
/// The clipped axis is assigned the line's value **exactly**; only the other
/// axes interpolate. Exactness here is what makes a full cell detectable without
/// a tolerance, and what makes neighbouring cells share edges bit for bit.
fn intersect<const N: usize>(a: &Corner<N>, b: &Corner<N>, edge: Edge) -> Corner<N> {
    let axis = edge.axis();
    let denom = b.pos[axis] - a.pos[axis];
    let t = if denom == 0.0 {
        0.5
    } else {
        (edge.value() - a.pos[axis]) / denom
    };

    let mut pos = [0.0; N];
    for ((p, &pa), &pb) in pos.iter_mut().zip(a.pos.iter()).zip(b.pos.iter()) {
        *p = pa + t * (pb - pa);
    }
    pos[axis] = edge.value();

    // Same `t` as the position, or UV and geometry drift apart.
    let uv = match (a.uv, b.uv) {
        (Some(ua), Some(ub)) => Some([ua[0] + t * (ub[0] - ua[0]), ua[1] + t * (ub[1] - ua[1])]),
        _ => None,
    };

    Corner { pos, uv }
}

/// An open chain left after a ring was severed by the cut line, tagged with
/// where it starts and ends on that line.
struct Chain<const N: usize> {
    corners: Vec<Corner<N>>,
    /// Position along the cut line where the chain enters the kept side.
    enter: f64,
    /// Position along the cut line where it leaves.
    exit: f64,
}

/// Clip every ring against `edge`, reconnecting severed pieces along the cut
/// line. Rings fully inside pass through untouched; rings fully outside vanish.
///
/// Winding is preserved: a ring is never re-wound, so exteriors stay exteriors
/// and holes stay holes for the caller's later classification.
pub(crate) fn clip_rings_halfplane<const N: usize>(
    rings: Vec<Vec<Corner<N>>>,
    edge: Edge,
) -> Vec<Vec<Corner<N>>> {
    let mut closed: Vec<Vec<Corner<N>>> = Vec::new();
    let mut chains: Vec<Chain<N>> = Vec::new();

    for ring in rings {
        if ring.len() < 3 {
            continue;
        }
        let all_in = ring.iter().all(|c| edge.contains(&c.pos));
        if all_in {
            closed.push(ring);
            continue;
        }
        if ring.iter().all(|c| !edge.contains(&c.pos)) {
            continue;
        }
        collect_chains(&ring, edge, &mut chains);
    }

    stitch(chains, edge, &mut closed);
    closed
}

/// Walk one ring, emitting each maximal run of kept corners as an open chain
/// whose ends sit exactly on the cut line.
fn collect_chains<const N: usize>(ring: &[Corner<N>], edge: Edge, out: &mut Vec<Chain<N>>) {
    let n = ring.len();
    let sort_axis = edge.sort_axis();
    let mut current: Vec<Corner<N>> = Vec::new();
    let mut pending: Vec<Vec<Corner<N>>> = Vec::new();

    for i in 0..n {
        let a = &ring[i];
        let b = &ring[(i + 1) % n];
        let a_in = edge.contains(&a.pos);
        let b_in = edge.contains(&b.pos);

        if a_in {
            current.push(*a);
        }
        if a_in != b_in {
            let cut = intersect(a, b, edge);
            current.push(cut);
            if a_in {
                // Leaving: this chain is finished.
                pending.push(std::mem::take(&mut current));
            } else {
                // Entering: start a fresh chain at the cut.
                current = vec![cut];
            }
        }
    }

    // The walk started mid-chain if the ring's first corner was inside; that
    // trailing run belongs to the front of the first chain.
    if !current.is_empty() {
        if let Some(first) = pending.first_mut() {
            let mut joined = current;
            joined.extend_from_slice(first);
            *first = joined;
        } else {
            pending.push(current);
        }
    }

    for corners in pending {
        if corners.len() < 2 {
            continue;
        }
        let enter = corners[0].pos[sort_axis];
        let exit = corners[corners.len() - 1].pos[sort_axis];
        out.push(Chain {
            corners,
            enter,
            exit,
        });
    }
}

/// Reconnect open chains into closed rings by walking along the cut line.
///
/// Each chain leaves the kept side at `exit` and some chain re-enters at the
/// next crossing along the line. Sorting the chains by where they leave and
/// pairing each with the nearest unused entry closes every ring, and a concave
/// ring that the cut severed in two closes as two rings rather than one with a
/// degenerate bridge.
fn stitch<const N: usize>(mut chains: Vec<Chain<N>>, edge: Edge, out: &mut Vec<Vec<Corner<N>>>) {
    if chains.is_empty() {
        return;
    }

    // Order of travel along the cut line is fixed per edge so the pairing is
    // deterministic and matches the ring's own winding direction. Walking the
    // cut line with the kept half-plane's interior on the left (matching a
    // CCW ring's own convention) means: MinX (interior +x) descends y, MaxX
    // (interior -x) ascends y, MinY (interior +y) ascends x, MaxY (interior
    // -y) descends x.
    let ascending = matches!(edge, Edge::MaxX(_) | Edge::MinY(_));
    chains.sort_by(|a, b| {
        let (x, y) = (a.exit, b.exit);
        if ascending {
            x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            y.partial_cmp(&x).unwrap_or(std::cmp::Ordering::Equal)
        }
    });

    let mut used = vec![false; chains.len()];
    for start in 0..chains.len() {
        if used[start] {
            continue;
        }
        let mut ring: Vec<Corner<N>> = Vec::new();
        let mut idx = start;
        loop {
            used[idx] = true;
            ring.extend_from_slice(&chains[idx].corners);
            let leaving = chains[idx].exit;
            match next_chain(&chains, &used, leaving, ascending, start) {
                Some(next) if next != start => idx = next,
                _ => break,
            }
        }
        if ring.len() >= 3 {
            out.push(ring);
        }
    }
}

/// The chain that re-enters the kept side nearest after `leaving`, travelling in
/// the given direction along the cut line. Falls back to `start` to close the
/// ring when nothing further along is free.
fn next_chain<const N: usize>(
    chains: &[Chain<N>],
    used: &[bool],
    leaving: f64,
    ascending: bool,
    start: usize,
) -> Option<usize> {
    let mut best: Option<(f64, usize)> = None;
    for (i, chain) in chains.iter().enumerate() {
        if used[i] && i != start {
            continue;
        }
        let delta = if ascending {
            chain.enter - leaving
        } else {
            leaving - chain.enter
        };
        if delta < 0.0 {
            continue;
        }
        if best.is_none_or(|(d, _)| delta < d) {
            best = Some((delta, i));
        }
    }
    best.map(|(_, i)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c2(x: f64, y: f64) -> Corner<2> {
        Corner {
            pos: [x, y],
            uv: None,
        }
    }

    fn c3(x: f64, y: f64, z: f64) -> Corner<3> {
        Corner {
            pos: [x, y, z],
            uv: None,
        }
    }

    fn positions<const N: usize>(rings: &[Vec<Corner<N>>]) -> Vec<Vec<[f64; N]>> {
        rings
            .iter()
            .map(|r| r.iter().map(|c| c.pos).collect())
            .collect()
    }

    #[test]
    fn ring_entirely_inside_is_untouched() {
        let ring = vec![c2(1.0, 1.0), c2(3.0, 1.0), c2(3.0, 3.0), c2(1.0, 3.0)];
        let out = clip_rings_halfplane(vec![ring.clone()], Edge::MinX(0.0));
        assert_eq!(positions(&out), positions(&[ring]));
    }

    #[test]
    fn ring_entirely_outside_disappears() {
        let ring = vec![c2(-3.0, 1.0), c2(-1.0, 1.0), c2(-1.0, 3.0), c2(-3.0, 3.0)];
        let out = clip_rings_halfplane(vec![ring], Edge::MinX(0.0));
        assert!(out.is_empty());
    }

    #[test]
    fn cut_vertex_lands_exactly_on_the_cut_line() {
        // Straddles x = 0. The two new vertices must be exactly 0.0, not 1e-17.
        let ring = vec![c2(-1.0, 0.0), c2(1.0, 0.0), c2(1.0, 2.0), c2(-1.0, 2.0)];
        let out = clip_rings_halfplane(vec![ring], Edge::MinX(0.0));
        assert_eq!(out.len(), 1);
        for corner in &out[0] {
            assert!(corner.pos[0] >= 0.0);
        }
        assert!(out[0].iter().any(|c| c.pos[0] == 0.0));
    }

    #[test]
    fn z_interpolates_linearly_at_the_cut() {
        // Edge from (-1, 0, 10) to (1, 0, 20) crosses x = 0 at t = 0.5, so z = 15.
        let ring = vec![
            c3(-1.0, 0.0, 10.0),
            c3(1.0, 0.0, 20.0),
            c3(1.0, 2.0, 20.0),
            c3(-1.0, 2.0, 10.0),
        ];
        let out = clip_rings_halfplane(vec![ring], Edge::MinX(0.0));
        assert_eq!(out.len(), 1);
        let cut: Vec<_> = out[0].iter().filter(|c| c.pos[0] == 0.0).collect();
        assert_eq!(cut.len(), 2);
        for c in cut {
            assert!((c.pos[2] - 15.0).abs() < 1e-12, "z was {}", c.pos[2]);
        }
    }

    #[test]
    fn uv_interpolates_with_the_same_parameter_as_z() {
        let ring = vec![
            Corner {
                pos: [-1.0, 0.0, 10.0],
                uv: Some([0.0, 0.0]),
            },
            Corner {
                pos: [1.0, 0.0, 20.0],
                uv: Some([1.0, 0.0]),
            },
            Corner {
                pos: [1.0, 2.0, 20.0],
                uv: Some([1.0, 1.0]),
            },
            Corner {
                pos: [-1.0, 2.0, 10.0],
                uv: Some([0.0, 1.0]),
            },
        ];
        let out = clip_rings_halfplane(vec![ring], Edge::MinX(0.0));
        let cut: Vec<_> = out[0].iter().filter(|c| c.pos[0] == 0.0).collect();
        assert_eq!(cut.len(), 2);
        // t = 0.5 on both crossing edges, so u = 0.5 on both.
        for c in cut {
            let uv = c.uv.expect("uv must survive the clip");
            assert!((uv[0] - 0.5).abs() < 1e-12, "u was {}", uv[0]);
        }
    }

    #[test]
    fn concave_ring_split_by_the_cut_yields_two_rings() {
        // A "U" opening to -x. Clipping at x >= 0 severs the bridge, leaving
        // the two prongs as separate rings. Sutherland-Hodgman would return one
        // ring joined by a degenerate edge; this must return two.
        let ring = vec![
            c2(2.0, 0.0),
            c2(2.0, 1.0),
            c2(-2.0, 1.0),
            c2(-2.0, 2.0),
            c2(2.0, 2.0),
            c2(2.0, 3.0),
            c2(-3.0, 3.0),
            c2(-3.0, 0.0),
        ];
        let out = clip_rings_halfplane(vec![ring], Edge::MinX(0.0));
        assert_eq!(out.len(), 2, "concave ring must split into two rings");
    }
}
