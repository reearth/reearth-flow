//! The areal sweep. A polygon is a one-face mesh, so polygons, polygon meshes
//! and triangle meshes share this routine:
//!
//! 1. clean rings (drop the closing vertex, weld near-coincident vertices,
//!    drop consecutive repeats);
//! 2. classify edges by how many faces use them (one = boundary, three or
//!    more = non-manifold);
//! 3. split into connected pieces and orient each piece consistently;
//! 4. per piece: place base and top, pick the bottom cap's winding so it faces
//!    against the sweep, then emit bottom cap, top cap (reversed) and one wall
//!    quad per boundary edge in the bottom cap's direction. That rule makes
//!    every shared edge traversed once each way, so the shell is coherent.

use std::collections::{HashMap, VecDeque};

use super::math::{centroid, dot, newell, norm, scale, sub};
use super::place::{orientation_sign, place, placement};
use super::weld::{weld, weld_tolerance};
use super::{ExtrudeDirection, ExtrudeError, ExtrudeOutput, ExtrudeSpec};
use crate::appearance::Appearance;
use crate::collection::Collection3D;
use crate::coordinate::CoordinateFrame;
use crate::polygon::Polygon3D;
use crate::polygon_mesh::{PolygonMesh3D, PolygonMesh3DData};
use crate::solid::{Shell, Solid};
use crate::Euclidean3DGeometry;

/// One face to sweep: its rings (exterior first) exactly as stored, and its
/// appearance, which only survives on a cap that keeps this ring layout.
pub(crate) struct CapFace {
    pub(crate) rings: Vec<Vec<[f64; 3]>>,
    pub(crate) appearance: Option<Appearance>,
}

/// A cleaned face: rings as open lists of welded vertex ids.
struct Face {
    rings: Vec<Vec<usize>>,
    /// The source appearance, kept only while the ring layout is unchanged.
    appearance: Option<Appearance>,
}

pub(crate) fn sweep_surface(
    frame: &CoordinateFrame,
    faces: Vec<CapFace>,
    spec: &ExtrudeSpec,
) -> Result<Euclidean3DGeometry, ExtrudeError> {
    let sign = orientation_sign(frame)?;
    let (mut faces, positions, tol) = clean(faces)?;
    let edges = edge_uses(&faces)?;
    let pieces = components(faces.len(), &edges);
    let adjacency = adjacency(faces.len(), &edges);
    let on_boundary: std::collections::HashSet<usize> = edges
        .values()
        .filter(|u| u.len() == 1)
        .map(|u| u[0].0)
        .collect();

    let mut out = Vec::with_capacity(pieces.len());
    for piece in pieces {
        if !piece.iter().any(|f| on_boundary.contains(f)) {
            return Err(ExtrudeError::InvalidSurface(
                "the surface is already closed",
            ));
        }
        orient(&mut faces, &piece, &adjacency)?;
        out.push(sweep_piece(
            frame, sign, tol, &faces, &piece, &positions, spec, &edges,
        )?);
    }
    Ok(if out.len() == 1 {
        out.pop().expect("one piece")
    } else {
        Euclidean3DGeometry::Collection(Collection3D::new(out))
    })
}

/// Cleaned faces, the welded positions their ids index, and the weld tolerance.
type Cleaned = (Vec<Face>, Vec<[f64; 3]>, f64);

/// Weld all vertices, then express every ring as open, repeat-free ids.
fn clean(faces: Vec<CapFace>) -> Result<Cleaned, ExtrudeError> {
    let all: Vec<[f64; 3]> = faces
        .iter()
        .flat_map(|f| f.rings.iter().flatten().copied())
        .collect();
    let tol = weld_tolerance(&all);
    let (ids, positions) = weld(&all, tol);
    let mut next = ids.into_iter();
    let mut cleaned = Vec::with_capacity(faces.len());
    for face in faces {
        let mut changed = false;
        let mut rings = Vec::with_capacity(face.rings.len());
        for (r, ring) in face.rings.iter().enumerate() {
            let raw: Vec<usize> = (0..ring.len())
                .map(|_| next.next().expect("one id per vertex"))
                .collect();
            let mut open: Vec<usize> = Vec::with_capacity(raw.len());
            for &id in &raw {
                if open.last() != Some(&id) {
                    open.push(id);
                }
            }
            if open.len() > 1 && open.first() == open.last() {
                open.pop();
            }
            // The layout is unchanged only when the stored ring is exactly the
            // cleaned ring closed on its first vertex: no repeats dropped, no
            // closing vertex added, same start. Rings are stored verbatim and
            // need not be closed, so a length check alone is not enough.
            changed |= raw.len() != open.len() + 1
                || raw[..open.len()] != open[..]
                || raw.last() != open.first();
            if open.len() < 3 {
                if r == 0 {
                    return Err(ExtrudeError::Degenerate(
                        "a face has fewer than three distinct vertices",
                    ));
                }
                changed = true;
                continue;
            }
            rings.push(open);
        }
        cleaned.push(Face {
            rings,
            appearance: if changed { None } else { face.appearance },
        });
    }
    Ok((cleaned, positions, tol))
}

/// Undirected edge -> every (face, directed edge) that uses it.
type EdgeUses = HashMap<(usize, usize), Vec<(usize, (usize, usize))>>;

fn edge_uses(faces: &[Face]) -> Result<EdgeUses, ExtrudeError> {
    let mut uses: EdgeUses = HashMap::new();
    for (f, face) in faces.iter().enumerate() {
        for ring in &face.rings {
            for i in 0..ring.len() {
                let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
                uses.entry((a.min(b), a.max(b)))
                    .or_default()
                    .push((f, (a, b)));
            }
        }
    }
    if uses.values().any(|u| u.len() > 2) {
        return Err(ExtrudeError::InvalidSurface(
            "an edge is shared by more than two faces",
        ));
    }
    Ok(uses)
}

/// Faces grouped into pieces connected through shared edges.
fn components(n: usize, edges: &EdgeUses) -> Vec<Vec<usize>> {
    let mut parent: Vec<usize> = (0..n).collect();
    fn root(p: &mut [usize], mut i: usize) -> usize {
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }
    for uses in edges.values() {
        if let [(f, _), (g, _)] = uses.as_slice() {
            let (rf, rg) = (root(&mut parent, *f), root(&mut parent, *g));
            parent[rf] = rg;
        }
    }
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for f in 0..n {
        let r = root(&mut parent, f);
        groups.entry(r).or_default().push(f);
    }
    let mut pieces: Vec<Vec<usize>> = groups.into_values().collect();
    pieces.sort_by_key(|p| p[0]);
    pieces
}

/// For each face, its neighbours across shared edges and whether the two
/// traverse that edge in the same direction (and so disagree in winding).
/// Built once so orientation is linear in the number of edges.
fn adjacency(n: usize, edges: &EdgeUses) -> Vec<Vec<(usize, bool)>> {
    let mut adj = vec![Vec::new(); n];
    for uses in edges.values() {
        if let [(a, da), (b, db)] = uses.as_slice() {
            adj[*a].push((*b, da == db));
            adj[*b].push((*a, da == db));
        }
    }
    adj
}

/// Make every shared edge in `piece` traversed once each way, flipping faces
/// as needed. Rejects a non-orientable piece.
fn orient(
    faces: &mut [Face],
    piece: &[usize],
    adjacency: &[Vec<(usize, bool)>],
) -> Result<(), ExtrudeError> {
    let mut flip: HashMap<usize, bool> = HashMap::from([(piece[0], false)]);
    let mut queue = VecDeque::from([piece[0]]);
    while let Some(f) = queue.pop_front() {
        for &(other, same_direction) in &adjacency[f] {
            let want = flip[&f] ^ same_direction;
            match flip.get(&other) {
                Some(&have) if have != want => {
                    return Err(ExtrudeError::InvalidSurface(
                        "the surface is not orientable",
                    ))
                }
                Some(_) => {}
                None => {
                    flip.insert(other, want);
                    queue.push_back(other);
                }
            }
        }
    }
    for (f, flipped) in flip {
        if flipped {
            let face = &mut faces[f];
            for ring in &mut face.rings {
                ring.reverse();
            }
            face.appearance = None;
        }
    }
    Ok(())
}

/// The unit front normal of a planar piece, or `InvalidSurface`.
fn planar_normal(
    faces: &[Face],
    piece: &[usize],
    positions: &[[f64; 3]],
    sign: f64,
    tol: f64,
) -> Result<[f64; 3], ExtrudeError> {
    let ring_pts = |ring: &[usize]| ring.iter().map(|&i| positions[i]).collect::<Vec<_>>();
    let sum = piece
        .iter()
        .map(|&f| newell(&ring_pts(&faces[f].rings[0])))
        .fold([0.0; 3], |acc, n| {
            [acc[0] + n[0], acc[1] + n[1], acc[2] + n[2]]
        });
    let len = norm(sum);
    if len == 0.0 {
        return Err(ExtrudeError::InvalidSurface("the surface has no area"));
    }
    let n = scale(sum, 1.0 / len);
    let pts: Vec<[f64; 3]> = piece
        .iter()
        .flat_map(|&f| faces[f].rings.iter().flatten().map(|&i| positions[i]))
        .collect();
    let c = centroid(&pts);
    let extent = pts.iter().map(|p| norm(sub(*p, c))).fold(0.0, f64::max);
    if pts
        .iter()
        .any(|p| dot(sub(*p, c), n).abs() > 1e-6 * extent.max(1.0) + tol)
    {
        return Err(ExtrudeError::InvalidSurface("the surface is not planar"));
    }
    Ok(scale(n, sign))
}

#[allow(clippy::too_many_arguments)]
fn sweep_piece(
    frame: &CoordinateFrame,
    sign: f64,
    tol: f64,
    faces: &[Face],
    piece: &[usize],
    positions: &[[f64; 3]],
    spec: &ExtrudeSpec,
    edges: &EdgeUses,
) -> Result<Euclidean3DGeometry, ExtrudeError> {
    // Local vertex table for this piece.
    let mut local: HashMap<usize, usize> = HashMap::new();
    let mut pts: Vec<[f64; 3]> = Vec::new();
    for &f in piece {
        for &id in faces[f].rings.iter().flatten() {
            local.entry(id).or_insert_with(|| {
                pts.push(positions[id]);
                pts.len() - 1
            });
        }
    }
    let normal = match spec.direction {
        ExtrudeDirection::SurfaceNormal { .. } => {
            Some(planar_normal(faces, piece, positions, sign, tol)?)
        }
        _ => None,
    };
    let (base, top) = place(&pts, &placement(frame, &spec.direction, normal)?)?;
    let at = |v: &[[f64; 3]], id: usize| v[local[&id]];

    // Which way the sweep goes, and whether each face (in its current
    // winding) faces with it. All faces must agree and none may be edge-on.
    let sweep = sub(top[0], base[0]);
    let sweep_unit = scale(sweep, 1.0 / norm(sweep));
    let mut facing = None;
    for &f in piece {
        let ring: Vec<[f64; 3]> = faces[f].rings[0].iter().map(|&i| at(&base, i)).collect();
        let n = newell(&ring);
        let len = norm(n);
        let d = if len == 0.0 {
            0.0
        } else {
            dot(n, sweep_unit) / len * sign
        };
        if d.abs() < 1e-9 || facing.is_some_and(|s: bool| s != (d > 0.0)) {
            return Err(ExtrudeError::Degenerate(
                "the surface cannot be swept along this direction",
            ));
        }
        facing = Some(d > 0.0);
    }
    // Faces that face with the sweep become the top cap as stored; otherwise
    // they are the bottom cap as stored.
    let source_is_top = facing.expect("a piece has at least one face");

    let poly = |rings: Vec<Vec<[f64; 3]>>, appearance: Option<Appearance>| {
        let mut closed = rings.into_iter().map(|mut r| {
            r.push(r[0]);
            r
        });
        let exterior = closed.next().expect("an exterior ring");
        let mut p = Polygon3D::from_rings(frame.clone(), exterior, closed.collect::<Vec<_>>());
        *p.appearance_mut() = appearance;
        p
    };

    let mut bottom = Vec::new();
    let mut top_caps = Vec::new();
    let mut walls = Vec::new();
    for &f in piece {
        let face = &faces[f];
        let stored = |v: &[[f64; 3]]| -> Vec<Vec<[f64; 3]>> {
            face.rings
                .iter()
                .map(|r| r.iter().map(|&i| at(v, i)).collect())
                .collect()
        };
        let reversed = |v: &[[f64; 3]]| -> Vec<Vec<[f64; 3]>> {
            stored(v)
                .into_iter()
                .map(|mut r| {
                    r.reverse();
                    r
                })
                .collect()
        };
        if source_is_top {
            top_caps.push(poly(stored(&top), face.appearance.clone()));
            bottom.push(poly(reversed(&base), None));
        } else {
            bottom.push(poly(stored(&base), face.appearance.clone()));
            top_caps.push(poly(reversed(&top), None));
        }
    }
    // Walls follow each boundary edge in the bottom cap's direction.
    for &f in piece {
        for ring in &faces[f].rings {
            for i in 0..ring.len() {
                let (mut a, mut b) = (ring[i], ring[(i + 1) % ring.len()]);
                if !is_boundary(edges, a, b) {
                    continue;
                }
                if source_is_top {
                    std::mem::swap(&mut a, &mut b);
                }
                walls.push(poly(
                    vec![vec![at(&base, a), at(&top, a), at(&top, b), at(&base, b)]],
                    None,
                ));
            }
        }
    }

    let assemble_err = |_| ExtrudeError::Degenerate("the result could not be assembled");
    Ok(match spec.output {
        ExtrudeOutput::Solid => {
            let all: Vec<&Polygon3D> = bottom.iter().chain(&top_caps).chain(&walls).collect();
            let data = PolygonMesh3DData::from_polygons(all);
            Euclidean3DGeometry::Solid(Box::new(Solid::from_exterior(
                frame.clone(),
                Shell::PolygonMesh(data),
            )))
        }
        ExtrudeOutput::OpenBottom => Euclidean3DGeometry::PolygonMesh(Box::new(
            PolygonMesh3D::from_polygons(frame.clone(), top_caps.iter().chain(&walls))
                .map_err(assemble_err)?,
        )),
        ExtrudeOutput::WallsOnly => Euclidean3DGeometry::PolygonMesh(Box::new(
            PolygonMesh3D::from_polygons(frame.clone(), walls.iter()).map_err(assemble_err)?,
        )),
    })
}

/// Whether the undirected edge `a`-`b` is used by exactly one face.
fn is_boundary(edges: &EdgeUses, a: usize, b: usize) -> bool {
    edges
        .get(&(a.min(b), a.max(b)))
        .is_some_and(|u| u.len() == 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::EpsgCode;
    use crate::ops::extrude::test_util::{assert_valid, up, vertical};
    use crate::ops::extrude::{extrude, BaseMode, ExtrudeDirection, ExtrudeOutput, TopMode};
    use crate::polygon::Polygon3D;
    use crate::solid::Shell;
    use crate::Geometry;

    fn square(o: [f64; 3], s: f64) -> Vec<[f64; 3]> {
        vec![
            [o[0], o[1], o[2]],
            [o[0] + s, o[1], o[2]],
            [o[0] + s, o[1] + s, o[2]],
            [o[0], o[1] + s, o[2]],
            [o[0], o[1], o[2]],
        ]
    }

    fn polygon(frame: CoordinateFrame, ext: Vec<[f64; 3]>, holes: Vec<Vec<[f64; 3]>>) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(frame, ext, holes),
        )))
    }

    fn solid_faces(g: &Geometry) -> usize {
        let Geometry::Euclidean3D(Euclidean3DGeometry::Solid(s)) = g else {
            panic!("{g:?}")
        };
        let Shell::PolygonMesh(m) = s.exterior() else {
            panic!()
        };
        m.num_faces()
    }

    #[test]
    fn a_square_becomes_a_valid_six_faced_solid() {
        let g = extrude(
            &polygon(CoordinateFrame::Euclidean, square([0.0; 3], 1.0), vec![]),
            &up(2.0),
        )
        .unwrap();
        assert_valid(&g);
        assert_eq!(solid_faces(&g), 6);
    }

    #[test]
    fn a_clockwise_square_is_still_outward() {
        let mut ring = square([0.0; 3], 1.0);
        ring.reverse();
        let g = extrude(&polygon(CoordinateFrame::Euclidean, ring, vec![]), &up(2.0)).unwrap();
        assert_valid(&g);
    }

    #[test]
    fn a_negative_height_is_still_outward() {
        let g = extrude(
            &polygon(CoordinateFrame::Euclidean, square([0.0; 3], 1.0), vec![]),
            &up(-2.0),
        )
        .unwrap();
        assert_valid(&g);
    }

    #[test]
    fn a_hole_becomes_inner_walls_not_a_lid() {
        let mut hole = square([0.25, 0.25, 0.0], 0.5);
        hole.reverse();
        let g = extrude(
            &polygon(
                CoordinateFrame::Euclidean,
                square([0.0; 3], 1.0),
                vec![hole],
            ),
            &up(1.0),
        )
        .unwrap();
        assert_valid(&g);
        // 2 caps (each with a hole) + 4 outer walls + 4 inner walls.
        assert_eq!(solid_faces(&g), 10);
    }

    #[test]
    fn every_base_and_top_combination_is_valid() {
        let sloped = vec![
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 1.0],
            [4.0, 4.0, 1.0],
            [0.0, 4.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let bases = [
            BaseMode::KeepElevation,
            BaseMode::FlattenToLowest,
            BaseMode::FixedElevation(-1.0),
        ];
        let tops = [
            TopMode::ByHeight(5.0),
            TopMode::ToElevation(9.0),
            TopMode::AboveLowest(5.0),
            TopMode::AboveHighest(5.0),
        ];
        for base in bases {
            for top in tops {
                let g = extrude(
                    &polygon(CoordinateFrame::Euclidean, sloped.clone(), vec![]),
                    &vertical(base, top),
                )
                .unwrap_or_else(|e| panic!("{base:?} {top:?}: {e}"));
                assert_valid(&g);
            }
        }
    }

    #[test]
    fn projected_magnitudes_stay_valid() {
        let g = extrude(
            &polygon(
                CoordinateFrame::Euclidean,
                square([-41_234.5, 3_912_345.6, 12.0], 10.0),
                vec![],
            ),
            &up(30.0),
        )
        .unwrap();
        assert_valid(&g);
    }

    #[test]
    fn a_lat_first_crs_stays_outward() {
        // EPSG:6668 is (lat, lon): stored winding is mirrored, the sign undoes it.
        for epsg in [6668u16, 4326] {
            let ring = square([35.68, 139.76, 0.0], 0.0001);
            let g = extrude(
                &polygon(CoordinateFrame::Crs(EpsgCode::from(epsg)), ring, vec![]),
                &up(20.0),
            )
            .unwrap();
            assert_valid(&g);
        }
    }

    #[test]
    fn repeated_and_closing_vertices_are_tolerated() {
        let ring = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let g = extrude(&polygon(CoordinateFrame::Euclidean, ring, vec![]), &up(1.0)).unwrap();
        assert_valid(&g);
        assert_eq!(solid_faces(&g), 6);
    }

    #[test]
    fn open_bottom_and_walls_only_outputs() {
        let p = polygon(CoordinateFrame::Euclidean, square([0.0; 3], 1.0), vec![]);
        for (output, faces) in [
            (ExtrudeOutput::OpenBottom, 5),
            (ExtrudeOutput::WallsOnly, 4),
        ] {
            let spec = ExtrudeSpec { output, ..up(1.0) };
            let Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(m)) =
                extrude(&p, &spec).unwrap()
            else {
                panic!("{output:?}")
            };
            assert_eq!(m.num_faces(), faces, "{output:?}");
        }
    }

    #[test]
    fn a_vertical_wall_cannot_be_swept_vertically() {
        let wall = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
        ];
        assert!(matches!(
            extrude(&polygon(CoordinateFrame::Euclidean, wall, vec![]), &up(1.0)),
            Err(ExtrudeError::Degenerate(_))
        ));
    }

    #[test]
    fn a_vertical_wall_thickens_along_its_normal() {
        let wall = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
        ];
        let spec = ExtrudeSpec {
            direction: ExtrudeDirection::SurfaceNormal { distance: 0.2 },
            output: ExtrudeOutput::Solid,
        };
        let g = extrude(&polygon(CoordinateFrame::Euclidean, wall, vec![]), &spec).unwrap();
        assert_valid(&g);
    }

    #[test]
    fn a_non_planar_face_has_no_normal_to_follow() {
        let warped = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let spec = ExtrudeSpec {
            direction: ExtrudeDirection::SurfaceNormal { distance: 1.0 },
            output: ExtrudeOutput::Solid,
        };
        assert!(matches!(
            extrude(&polygon(CoordinateFrame::Euclidean, warped, vec![]), &spec),
            Err(ExtrudeError::InvalidSurface(_))
        ));
    }

    #[test]
    fn a_2d_polygon_with_no_elevation_starts_at_zero() {
        use crate::polygon::Polygon2D;
        use crate::Euclidean2DGeometry;
        let p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]],
            std::iter::empty::<Vec<[f64; 2]>>(),
        );
        let g = extrude(
            &Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(p))),
            &up(3.0),
        )
        .unwrap();
        assert_valid(&g);
    }

    #[test]
    fn an_unknown_crs_is_an_unsupported_frame_for_an_area() {
        let g = polygon(
            CoordinateFrame::Crs(EpsgCode::from(1u16)),
            square([0.0; 3], 1.0),
            vec![],
        );
        assert!(matches!(
            extrude(&g, &up(1.0)),
            Err(ExtrudeError::UnsupportedFrame(_))
        ));
    }

    #[test]
    fn a_tangent_frame_sweeps_along_its_own_z() {
        use crate::coordinate::{BaseFrame, TangentPlane};
        let plane =
            TangentPlane::from_normal(BaseFrame::Euclidean, [0.0; 3], [1.0, 0.0, 0.0], None)
                .unwrap();
        let frame = CoordinateFrame::Tangent(Box::new(plane));
        let g = extrude(&polygon(frame, square([0.0; 3], 1.0), vec![]), &up(2.0)).unwrap();
        assert_valid(&g);
        let Geometry::Euclidean3D(Euclidean3DGeometry::Solid(s)) = &g else {
            panic!()
        };
        let Shell::PolygonMesh(m) = s.exterior() else {
            panic!()
        };
        let zmax = m
            .faces(s.frame())
            .iter()
            .flat_map(|p| p.exterior().to_vec())
            .map(|c| c[2])
            .fold(f64::MIN, f64::max);
        assert_eq!(zmax, 2.0);
    }

    #[test]
    fn a_self_intersecting_footprint_has_a_pinned_outcome() {
        // A bow-tie. Input is not validated; the result must be deterministic.
        let bowtie = vec![
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let p = polygon(CoordinateFrame::Euclidean, bowtie, vec![]);
        assert_eq!(extrude(&p, &up(1.0)), extrude(&p, &up(1.0)));
        // Its two lobes cancel: the Newell normal is zero, so the face is
        // treated as edge-on to the sweep.
        assert!(matches!(
            extrude(&p, &up(1.0)),
            Err(ExtrudeError::Degenerate(_))
        ));
    }

    fn with_appearance(ring: Vec<[f64; 3]>) -> Geometry {
        use crate::appearance::{Material, PhongMaterial, ThemeId};
        let mut p = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            ring,
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let material = Material::Phong(PhongMaterial {
            diffuse: [1.0, 0.0, 0.0],
            specular: [0.0; 3],
            emissive: [0.0; 3],
            ambient_intensity: 0.0,
            shininess: 0.0,
            transparency: 0.0,
            diffuse_map: None,
            emissive_map: None,
            normal_map: None,
        });
        p.set_appearance(ThemeId(std::sync::Arc::from("t")), material, None)
            .unwrap();
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(p)))
    }

    /// How many faces of the extruded solid carry an appearance.
    fn faces_with_appearance(g: &Geometry) -> usize {
        let Geometry::Euclidean3D(Euclidean3DGeometry::Solid(s)) = g else {
            panic!("{g:?}")
        };
        let Shell::PolygonMesh(m) = s.exterior() else {
            panic!()
        };
        // `faces()` rebuilds bare polygons, so read the mesh-wide binding.
        let Some(app) = m.appearance() else {
            return 0;
        };
        assert_eq!(app.themes().len(), 1);
        match &app.themes()[0].front {
            crate::appearance::FaceBinding::Uniform(_) => m.num_faces(),
            crate::appearance::FaceBinding::PerFace(v) => v.iter().flatten().count(),
        }
    }

    #[test]
    fn a_clean_closed_ring_keeps_its_appearance_on_one_cap() {
        let g = extrude(&with_appearance(square([0.0; 3], 1.0)), &up(1.0)).unwrap();
        assert_valid(&g);
        assert_eq!(faces_with_appearance(&g), 1);
    }

    #[test]
    fn an_open_ring_with_a_repeat_drops_its_appearance() {
        // Open (no closing vertex) with one repeat: cleaning shifts the
        // vertices after the repeat, so the source's per-vertex layout no
        // longer applies.
        let ring = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let g = extrude(&with_appearance(ring), &up(1.0)).unwrap();
        assert_valid(&g);
        assert_eq!(faces_with_appearance(&g), 0);
    }

    use crate::polygon_mesh::PolygonMesh3D as TestMesh;

    fn mesh(polys: &[Vec<[f64; 3]>]) -> Geometry {
        let ps: Vec<Polygon3D> = polys
            .iter()
            .map(|r| {
                Polygon3D::from_rings(
                    CoordinateFrame::Euclidean,
                    r.clone(),
                    Vec::<Vec<[f64; 3]>>::new(),
                )
            })
            .collect();
        Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(Box::new(
            TestMesh::from_polygons(CoordinateFrame::Euclidean, ps.iter()).unwrap(),
        )))
    }

    #[test]
    fn two_footprints_sharing_an_edge_have_no_wall_between_them() {
        let g = extrude(
            &mesh(&[square([0.0; 3], 1.0), square([1.0, 0.0, 0.0], 1.0)]),
            &up(1.0),
        )
        .unwrap();
        assert_valid(&g);
        // 2 bottom + 2 top + 6 boundary walls; the shared edge gets none.
        assert_eq!(solid_faces(&g), 10);
    }

    #[test]
    fn a_nearly_shared_edge_is_welded() {
        let mut right = square([1.0, 0.0, 0.0], 1.0);
        right[0][0] += 1e-10;
        right[4][0] += 1e-10;
        let g = extrude(&mesh(&[square([0.0; 3], 1.0), right]), &up(1.0)).unwrap();
        assert_valid(&g);
        assert_eq!(solid_faces(&g), 10);
    }

    #[test]
    fn disconnected_footprints_become_a_collection_of_solids() {
        let g = extrude(
            &mesh(&[square([0.0; 3], 1.0), square([5.0, 0.0, 0.0], 1.0)]),
            &up(1.0),
        )
        .unwrap();
        let Geometry::Euclidean3D(Euclidean3DGeometry::Collection(c)) = &g else {
            panic!("{g:?}")
        };
        assert_eq!(c.len(), 2);
        assert_valid(&g);
    }

    #[test]
    fn mixed_winding_meshes_are_oriented_consistently() {
        let mut flipped = square([1.0, 0.0, 0.0], 1.0);
        flipped.reverse();
        let g = extrude(&mesh(&[square([0.0; 3], 1.0), flipped]), &up(1.0)).unwrap();
        assert_valid(&g);
    }

    #[test]
    fn a_closed_mesh_is_an_invalid_surface() {
        // The six faces of a unit cube, outward.
        let c = |x: f64, y: f64, z: f64| [x, y, z];
        let faces = vec![
            vec![
                c(0., 0., 0.),
                c(0., 1., 0.),
                c(1., 1., 0.),
                c(1., 0., 0.),
                c(0., 0., 0.),
            ],
            vec![
                c(0., 0., 1.),
                c(1., 0., 1.),
                c(1., 1., 1.),
                c(0., 1., 1.),
                c(0., 0., 1.),
            ],
            vec![
                c(0., 0., 0.),
                c(1., 0., 0.),
                c(1., 0., 1.),
                c(0., 0., 1.),
                c(0., 0., 0.),
            ],
            vec![
                c(1., 0., 0.),
                c(1., 1., 0.),
                c(1., 1., 1.),
                c(1., 0., 1.),
                c(1., 0., 0.),
            ],
            vec![
                c(1., 1., 0.),
                c(0., 1., 0.),
                c(0., 1., 1.),
                c(1., 1., 1.),
                c(1., 1., 0.),
            ],
            vec![
                c(0., 1., 0.),
                c(0., 0., 0.),
                c(0., 0., 1.),
                c(0., 1., 1.),
                c(0., 1., 0.),
            ],
        ];
        assert!(matches!(
            extrude(&mesh(&faces), &up(1.0)),
            Err(ExtrudeError::InvalidSurface(_))
        ));
    }

    #[test]
    fn a_collection_with_one_bad_member_fails_whole() {
        let good = Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            square([0.0; 3], 1.0),
            Vec::<Vec<[f64; 3]>>::new(),
        )));
        let Geometry::Euclidean3D(bad) =
            extrude(&Geometry::Euclidean3D(good.clone()), &up(1.0)).unwrap()
        else {
            panic!()
        };
        // `bad` is a Solid, which cannot be extruded again.
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([
            good, bad,
        ])));
        assert!(matches!(
            extrude(&g, &up(1.0)),
            Err(ExtrudeError::UnsupportedType(_))
        ));
    }

    #[test]
    fn a_triangle_mesh_becomes_a_solid() {
        use crate::triangular_mesh::TriangularMesh3D;
        let tin = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.2],
                [1.0, 1.0, 0.4],
                [0.0, 1.0, 0.1],
            ],
            [0u32, 1, 2, 0, 2, 3],
        )
        .unwrap();
        let g = extrude(
            &Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(tin))),
            &vertical(BaseMode::KeepElevation, TopMode::AboveHighest(3.0)),
        )
        .unwrap();
        assert_valid(&g);
    }
}
