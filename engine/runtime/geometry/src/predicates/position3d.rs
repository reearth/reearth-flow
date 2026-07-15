//! Point position relative to 3D geometry, including **point-in-solid**.
//!
//! [`point_position_3d`] classifies a coordinate against any 3D geometry as
//! [`Inside`](CoordPos::Inside) / [`OnBoundary`](CoordPos::OnBoundary) /
//! [`Outside`](CoordPos::Outside), with the per-leaf semantics graded by
//! codimension:
//!
//! - **Point** — the interior is the point itself (as in 2D).
//! - **LineString** — the chain minus its endpoints is interior, open-chain
//!   endpoints are boundary (as in 2D, counted mod 2 across collection
//!   members).
//! - **Polygon / meshes** — a surface embedded in 3D has no volumetric
//!   interior: every point of it classifies as `OnBoundary`.
//! - **Solid** — the genuinely volumetric case, which 2D has no analog of and
//!   the legacy geometry never implemented: on any shell is `OnBoundary`,
//!   enclosed by the exterior shell and outside every interior (void) shell is
//!   `Inside`.
//! - **Collections** — the union: the highest member classification wins.
//!
//! Point-in-solid is decided by **exact ray-crossing parity**: a probe segment
//! is cast from the coordinate to a point outside the shell's bounding box and
//! the strict crossings through shell triangles are counted, every sign coming
//! from robust [`orient3d`]. A probe that grazes the shell — hitting an edge,
//! a vertex, or a triangle's plane tangentially, where naive parity counters
//! silently double-count — is *detected exactly* (some orientation sign is
//! zero) and retried with a different probe target instead of being fudged
//! with an epsilon. Shells are assumed closed and non-self-intersecting (the
//! [`Validate`](crate::validation_next) contract); parity over an open shell
//! is meaningless.

use crate::ops::triangulation::Cache;
use crate::solid::Solid;
use crate::Euclidean3DGeometry;

use super::kernel::{orient3d, CoordPos, Orientation};
use super::kernel3d::{point_in_triangle_3d, point_on_segment_3d};
use super::view3d::{flatten_3d, require_common_frame_3d, Leaf3D, TriangleSet};
use super::{PredicateError, Result};

/// Position of a coordinate relative to a 3D geometry, treating collections as
/// point-set unions of their members. The coordinate is taken to be in the
/// geometry's own frame; the geometry's leaves must agree on one
/// ([`MixedFrames`](PredicateError::MixedFrames) otherwise), and a `Csg` or
/// `PointCloud` leaf is
/// [`Unsupported`](PredicateError::Unsupported).
pub fn point_position_3d(coord: [f64; 3], geometry: &Euclidean3DGeometry) -> Result<CoordPos> {
    let mut leaves = Vec::new();
    let mut unsupported = None;
    flatten_3d(geometry, &mut leaves, &mut unsupported);
    if let Some(name) = unsupported {
        return Err(PredicateError::Unsupported { geometry: name });
    }
    require_common_frame_3d(&leaves, &[])?;

    let mut cache = Cache::new();
    let mut best = CoordPos::Outside;
    let mut boundary_endpoints = 0usize;
    let mut on_any_line = false;
    for leaf in &leaves {
        let pos = match leaf {
            Leaf3D::Point(p) => {
                if coord == p.position() {
                    CoordPos::Inside
                } else {
                    CoordPos::Outside
                }
            }
            Leaf3D::Line(l) => {
                let coords = l.coords();
                if on_chain_3d(coord, coords) {
                    on_any_line = true;
                }
                let closed = coords.len() >= 2 && coords.first() == coords.last();
                if !closed && coords.len() >= 2 {
                    if coord == coords[0] {
                        boundary_endpoints += 1;
                    }
                    if coord == coords[coords.len() - 1] {
                        boundary_endpoints += 1;
                    }
                }
                CoordPos::Outside // combined below, mod 2 across members
            }
            Leaf3D::Polygon(p) => {
                surface_position(coord, &TriangleSet::from_polygon(p, &mut cache))
            }
            Leaf3D::PolygonMesh(m) => surface_position(
                coord,
                &TriangleSet::from_polygon_mesh_data(m.data(), &mut cache),
            ),
            Leaf3D::TriangularMesh(m) => {
                surface_position(coord, &TriangleSet::from_triangular_data(m.data()))
            }
            Leaf3D::Solid(s) => solid_position(coord, s, &mut cache),
        };
        best = max_pos(best, pos);
        if best == CoordPos::Inside {
            return Ok(CoordPos::Inside);
        }
    }

    let line_pos = if !on_any_line {
        CoordPos::Outside
    } else if boundary_endpoints % 2 == 1 {
        CoordPos::OnBoundary
    } else {
        CoordPos::Inside
    };
    Ok(max_pos(best, line_pos))
}

/// Position relative to one solid: on any shell is boundary; inside the
/// exterior shell and outside every void shell is inside.
pub(crate) fn solid_position(coord: [f64; 3], solid: &Solid, cache: &mut Cache) -> CoordPos {
    let exterior = TriangleSet::from_shell(solid.exterior(), cache);
    let voids: Vec<TriangleSet<'_>> = solid
        .interiors()
        .iter()
        .map(|shell| TriangleSet::from_shell(shell, cache))
        .collect();
    solid_position_sets(coord, &exterior, voids.iter())
}

/// [`solid_position`] over already-extracted shell triangle sets, for callers
/// that hold them (the 3D `intersects` keeps them for its boundary tests).
pub(crate) fn solid_position_sets<'a, 'b>(
    coord: [f64; 3],
    exterior: &TriangleSet<'a>,
    voids: impl Iterator<Item = &'b TriangleSet<'a>>,
) -> CoordPos
where
    'a: 'b,
{
    match shell_position(coord, exterior) {
        CoordPos::OnBoundary => return CoordPos::OnBoundary,
        CoordPos::Outside => return CoordPos::Outside,
        CoordPos::Inside => {}
    }
    for void in voids {
        match shell_position(coord, void) {
            CoordPos::OnBoundary => return CoordPos::OnBoundary,
            CoordPos::Inside => return CoordPos::Outside, // in a hollow
            CoordPos::Outside => {}
        }
    }
    CoordPos::Inside
}

/// On-surface position: `OnBoundary` anywhere on the triangle set, else
/// `Outside` (a surface embedded in 3D has no volumetric interior).
pub(crate) fn surface_position(coord: [f64; 3], set: &TriangleSet<'_>) -> CoordPos {
    if set.triangles().any(|t| point_in_triangle_3d(coord, t)) {
        CoordPos::OnBoundary
    } else {
        CoordPos::Outside
    }
}

/// Position relative to one closed shell, by exact ray-crossing parity.
pub(crate) fn shell_position(coord: [f64; 3], shell: &TriangleSet<'_>) -> CoordPos {
    if shell.is_empty() {
        return CoordPos::Outside;
    }
    let (min, max) = pool_bounds(shell.pool());
    if (0..3).any(|k| coord[k] < min[k] || coord[k] > max[k]) {
        return CoordPos::Outside;
    }
    if shell.triangles().any(|t| point_in_triangle_3d(coord, t)) {
        return CoordPos::OnBoundary;
    }

    'attempt: for attempt in 0..MAX_PROBE_ATTEMPTS {
        let target = probe_target(min, max, attempt);
        let mut crossings = 0usize;
        for t in shell.triangles() {
            match probe_crossing(coord, target, t) {
                ProbeCrossing::Miss => {}
                ProbeCrossing::Cross => crossings += 1,
                ProbeCrossing::Degenerate => continue 'attempt,
            }
        }
        return if crossings % 2 == 1 {
            CoordPos::Inside
        } else {
            CoordPos::Outside
        };
    }
    // Every probe grazed the shell — possible only for wildly degenerate
    // input. Degrade to the boundary answer rather than guess a side.
    CoordPos::OnBoundary
}

/// How the probe segment relates to one shell triangle.
enum ProbeCrossing {
    /// No shared point.
    Miss,
    /// One strict crossing through the triangle's interior.
    Cross,
    /// A graze (edge, vertex, or coplanar contact): the parity of this probe
    /// direction is unreliable — retry with another.
    Degenerate,
}

/// Exact classification of the probe segment `[p, target]` against a shell
/// triangle. `p` is known to be off the shell and `target` outside its
/// bounding box.
fn probe_crossing(p: [f64; 3], target: [f64; 3], t: [[f64; 3]; 3]) -> ProbeCrossing {
    let side_p = orient3d(t[0], t[1], t[2], p);
    let side_q = orient3d(t[0], t[1], t[2], target);
    match (side_p, side_q) {
        // A degenerate triangle contributes no plane crossing; parity ignores
        // it exactly as its zero-area face bounds no volume.
        _ if side_p == Orientation::Collinear && side_q == Orientation::Collinear => {
            // Both endpoints in the triangle's plane: either the triangle is
            // degenerate or the probe runs inside the plane; both are grazes
            // unless the probe misses the triangle's bounding box entirely.
            if probe_meets_triangle_in_plane(p, target, t) {
                ProbeCrossing::Degenerate
            } else {
                ProbeCrossing::Miss
            }
        }
        // The probe touches the plane only at an endpoint: `p` itself is off
        // the triangle (pre-checked) and `target` is outside the shell's box,
        // so a touch at either endpoint cannot be on the triangle.
        (Orientation::Collinear, _) | (_, Orientation::Collinear) => ProbeCrossing::Miss,
        (a, b) if a == b => ProbeCrossing::Miss,
        // A strict straddle: classify by the three edge orientations.
        _ => {
            let u = orient3d(p, target, t[0], t[1]);
            let v = orient3d(p, target, t[1], t[2]);
            let w = orient3d(p, target, t[2], t[0]);
            if u == Orientation::Collinear
                || v == Orientation::Collinear
                || w == Orientation::Collinear
            {
                // The crossing would sit exactly on a triangle edge or vertex,
                // shared with a neighboring face: unreliable to count.
                return ProbeCrossing::Degenerate;
            }
            if u == v && v == w {
                ProbeCrossing::Cross
            } else {
                ProbeCrossing::Miss
            }
        }
    }
}

/// Whether a coplanar probe segment shares a point with the triangle (used
/// only to decide graze vs. clean miss when both probe endpoints land in a
/// triangle's plane).
fn probe_meets_triangle_in_plane(p: [f64; 3], q: [f64; 3], t: [[f64; 3]; 3]) -> bool {
    super::kernel3d::segment_intersects_triangle_3d(p, q, t)
}

const MAX_PROBE_ATTEMPTS: u32 = 32;

/// The probe target for one attempt: a point strictly outside the shell's
/// bounding box, varied across attempts (corner choice from the attempt's low
/// bits, offsets jittered by a splitmix64 stream) so that consecutive
/// attempts take genuinely different directions.
fn probe_target(min: [f64; 3], max: [f64; 3], attempt: u32) -> [f64; 3] {
    let extent = (0..3).map(|k| max[k] - min[k]).fold(1.0f64, f64::max);
    let mut state = 0x9E37_79B9_7F4A_7C15u64.wrapping_mul(u64::from(attempt) + 1);
    let mut coord = [0.0f64; 3];
    for (k, c) in coord.iter_mut().enumerate() {
        state = splitmix64(state);
        // Offset in [1, 2) times the extent, beyond the box on this axis.
        let offset = (1.0 + unit_f64(state)) * extent;
        *c = if attempt >> k & 1 == 0 {
            max[k] + offset
        } else {
            min[k] - offset
        };
    }
    coord
}

/// One step of the splitmix64 generator.
fn splitmix64(state: u64) -> u64 {
    let mut z = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Map 64 random bits to a float in `[0, 1)`.
fn unit_f64(bits: u64) -> f64 {
    (bits >> 11) as f64 / (1u64 << 53) as f64
}

/// The bounding box of a vertex pool (which is non-empty for a non-empty
/// triangle set).
fn pool_bounds(pool: &[[f64; 3]]) -> ([f64; 3], [f64; 3]) {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for p in pool {
        for k in 0..3 {
            min[k] = min[k].min(p[k]);
            max[k] = max[k].max(p[k]);
        }
    }
    (min, max)
}

/// Whether the coordinate lies anywhere on the 3D chain.
pub(crate) fn on_chain_3d(coord: [f64; 3], coords: &[[f64; 3]]) -> bool {
    match coords {
        [] => false,
        [single] => coord == *single,
        _ => coords
            .windows(2)
            .any(|e| point_on_segment_3d(coord, e[0], e[1])),
    }
}

/// The higher of two classifications (`Inside` > `OnBoundary` > `Outside`).
fn max_pos(a: CoordPos, b: CoordPos) -> CoordPos {
    fn rank(p: CoordPos) -> u8 {
        match p {
            CoordPos::Outside => 0,
            CoordPos::OnBoundary => 1,
            CoordPos::Inside => 2,
        }
    }
    if rank(a) >= rank(b) {
        a
    } else {
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection3D;
    use crate::coordinate::{CoordinateFrame, EpsgCode};
    use crate::line_string::LineString3D;
    use crate::point::Point3D;
    use crate::polygon::Polygon3D;
    use crate::predicates::test3d::{
        box_solid, box_solid_with_void, e, solid_geometry, tetra_solid, Rng,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn box_solid_positions() {
        let s = solid_geometry(box_solid([0.0, 0.0, 0.0], [4.0, 4.0, 4.0]));
        assert_eq!(point_position_3d([2.0, 2.0, 2.0], &s), Ok(CoordPos::Inside));
        assert_eq!(
            point_position_3d([5.0, 2.0, 2.0], &s),
            Ok(CoordPos::Outside)
        );
        // Face interior, edge, and vertex are all boundary.
        assert_eq!(
            point_position_3d([0.0, 1.0, 2.0], &s),
            Ok(CoordPos::OnBoundary)
        );
        assert_eq!(
            point_position_3d([0.0, 0.0, 2.0], &s),
            Ok(CoordPos::OnBoundary)
        );
        assert_eq!(
            point_position_3d([4.0, 4.0, 4.0], &s),
            Ok(CoordPos::OnBoundary)
        );
        // On a face's internal triangulation diagonal: still boundary.
        assert_eq!(
            point_position_3d([0.0, 2.0, 2.0], &s),
            Ok(CoordPos::OnBoundary)
        );
        // Just inside a corner (probes graze many vertices; parity must
        // survive the retries).
        assert_eq!(point_position_3d([1.0, 1.0, 1.0], &s), Ok(CoordPos::Inside));
    }

    #[test]
    fn hollow_solid_positions() {
        let s = solid_geometry(box_solid_with_void(
            [0.0, 0.0, 0.0],
            [6.0, 6.0, 6.0],
            [2.0, 2.0, 2.0],
            [2.0, 2.0, 2.0],
        ));
        // In the wall between the shells.
        assert_eq!(point_position_3d([1.0, 1.0, 1.0], &s), Ok(CoordPos::Inside));
        // In the hollow.
        assert_eq!(
            point_position_3d([3.0, 3.0, 3.0], &s),
            Ok(CoordPos::Outside)
        );
        // On the void shell.
        assert_eq!(
            point_position_3d([2.0, 3.0, 3.0], &s),
            Ok(CoordPos::OnBoundary)
        );
    }

    #[test]
    fn surface_leaves_are_all_boundary() {
        let square = Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
            e(),
            [
                [0.0, 0.0, 1.0],
                [4.0, 0.0, 1.0],
                [4.0, 4.0, 1.0],
                [0.0, 4.0, 1.0],
                [0.0, 0.0, 1.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        )));
        assert_eq!(
            point_position_3d([2.0, 2.0, 1.0], &square),
            Ok(CoordPos::OnBoundary)
        );
        assert_eq!(
            point_position_3d([0.0, 0.0, 1.0], &square),
            Ok(CoordPos::OnBoundary)
        );
        assert_eq!(
            point_position_3d([2.0, 2.0, 2.0], &square),
            Ok(CoordPos::Outside)
        );
    }

    #[test]
    fn line_and_point_leaves_mirror_2d_semantics() {
        let l = Euclidean3DGeometry::LineString(LineString3D::from_coords(
            e(),
            [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [4.0, 4.0, 4.0]],
        ));
        assert_eq!(point_position_3d([2.0, 0.0, 0.0], &l), Ok(CoordPos::Inside));
        assert_eq!(point_position_3d([4.0, 0.0, 0.0], &l), Ok(CoordPos::Inside)); // mid vertex
        assert_eq!(
            point_position_3d([0.0, 0.0, 0.0], &l),
            Ok(CoordPos::OnBoundary)
        );
        assert_eq!(
            point_position_3d([1.0, 1.0, 1.0], &l),
            Ok(CoordPos::Outside)
        );

        let p = Euclidean3DGeometry::Point(Point3D::new(e(), [1.0, 2.0, 3.0]));
        assert_eq!(point_position_3d([1.0, 2.0, 3.0], &p), Ok(CoordPos::Inside));
        assert_eq!(
            point_position_3d([1.0, 2.0, 3.5], &p),
            Ok(CoordPos::Outside)
        );

        // Two chains joined at an endpoint make it interior (mod-2 union).
        let a = Euclidean3DGeometry::LineString(LineString3D::from_coords(
            e(),
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
        ));
        let b = Euclidean3DGeometry::LineString(LineString3D::from_coords(
            e(),
            [[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
        ));
        let c = Euclidean3DGeometry::Collection(Collection3D::new([a, b]));
        assert_eq!(point_position_3d([1.0, 0.0, 0.0], &c), Ok(CoordPos::Inside));
        assert_eq!(
            point_position_3d([0.0, 0.0, 0.0], &c),
            Ok(CoordPos::OnBoundary)
        );
    }

    #[test]
    fn unsupported_and_mixed_frame_errors() {
        use crate::csg::{Csg, ThreeDimensional};
        let solid = || Box::new(box_solid([0.0; 3], [1.0; 3]));
        let csg = Euclidean3DGeometry::Csg(Csg::Union(
            Box::new(ThreeDimensional::Solid(solid())),
            Box::new(ThreeDimensional::Solid(solid())),
        ));
        assert_eq!(
            point_position_3d([0.0; 3], &csg),
            Err(PredicateError::Unsupported { geometry: "Csg" })
        );

        let mixed = Euclidean3DGeometry::Collection(Collection3D::new([
            Euclidean3DGeometry::Point(Point3D::new(e(), [0.0; 3])),
            Euclidean3DGeometry::Point(Point3D::new(
                CoordinateFrame::Crs(EpsgCode::new(4979)),
                [0.0; 3],
            )),
        ]));
        assert_eq!(
            point_position_3d([0.0; 3], &mixed),
            Err(PredicateError::MixedFrames)
        );

        // An empty collection is outside everything.
        let empty = Euclidean3DGeometry::Collection(Collection3D::new([]));
        assert_eq!(point_position_3d([0.0; 3], &empty), Ok(CoordPos::Outside));
    }

    /// Half-space oracle for a (non-degenerate) tetrahedron.
    fn tetra_oracle(v: [[f64; 3]; 4], p: [f64; 3]) -> CoordPos {
        let faces = [(0, 1, 2, 3), (0, 1, 3, 2), (0, 2, 3, 1), (1, 2, 3, 0)];
        let mut on_boundary = false;
        for (a, b, c, d) in faces {
            let side_p = orient3d(v[a], v[b], v[c], p);
            let side_d = orient3d(v[a], v[b], v[c], v[d]);
            if side_p == Orientation::Collinear {
                on_boundary = true;
            } else if side_p != side_d {
                return CoordPos::Outside;
            }
        }
        if on_boundary {
            CoordPos::OnBoundary
        } else {
            CoordPos::Inside
        }
    }

    #[test]
    fn tetra_parity_matches_half_space_oracle() {
        let mut rng = Rng(20260716);
        let mut tested = 0usize;
        for _ in 0..150 {
            let v = [
                rng.grid_point(-3, 3),
                rng.grid_point(-3, 3),
                rng.grid_point(-3, 3),
                rng.grid_point(-3, 3),
            ];
            if orient3d(v[0], v[1], v[2], v[3]) == Orientation::Collinear {
                continue; // degenerate tetrahedron
            }
            let solid = solid_geometry(tetra_solid(v));

            // Vertices and edge midpoints land exactly on the boundary;
            // integer grid points exercise faces, edges, and both sides.
            let mut queries: Vec<[f64; 3]> = v.to_vec();
            for i in 0..4 {
                for j in (i + 1)..4 {
                    queries.push([
                        (v[i][0] + v[j][0]) / 2.0,
                        (v[i][1] + v[j][1]) / 2.0,
                        (v[i][2] + v[j][2]) / 2.0,
                    ]);
                }
            }
            for _ in 0..20 {
                queries.push(rng.grid_point(-4, 4));
            }

            for q in queries {
                assert_eq!(
                    point_position_3d(q, &solid),
                    Ok(tetra_oracle(v, q)),
                    "tetra {v:?}, query {q:?}"
                );
                tested += 1;
            }
        }
        assert!(tested > 3000, "the sweep should exercise many cases");
    }
}
