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
use super::weld::{weld, weld_tolerance, Tolerance};
use super::{ExtrudeDirection, ExtrudeError, ExtrudeOutput, ExtrudeSpec};
use crate::appearance::Appearance;
use crate::collection::Collection3D;
use crate::coordinate::{CoordinateFrame, UnitKind};
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
    let plan_view = !matches!(frame.unit_kind(), UnitKind::Linear);
    let (mut faces, positions, tol) = clean(faces, plan_view)?;
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
type Cleaned = (Vec<Face>, Vec<[f64; 3]>, Tolerance);

/// Weld all vertices, then express every ring as open, repeat-free ids. A
/// hole wound the same way as its exterior is reversed, so its walls face
/// into the hole rather than into the material.
///
/// With `plan_view` (a frame not in linear units, where x and y are degrees
/// and z metres) only the vertical component of each ring's normal decides
/// agreement: on a slightly draped footprint the horizontal components
/// (degrees times metres) swamp the vertical one (degrees squared) and would
/// follow the tilt rather than the winding. Such frames only sweep
/// vertically, so the plan-view winding is the one that matters.
fn clean(faces: Vec<CapFace>, plan_view: bool) -> Result<Cleaned, ExtrudeError> {
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
        if let Some((exterior, holes)) = rings.split_first_mut() {
            let normal =
                |ring: &[usize]| newell(&ring.iter().map(|&i| positions[i]).collect::<Vec<_>>());
            let agree = |a: [f64; 3], b: [f64; 3]| {
                if plan_view {
                    a[2] * b[2] > 0.0
                } else {
                    dot(a, b) > 0.0
                }
            };
            let n = normal(exterior);
            for hole in holes {
                if agree(normal(hole), n) {
                    hole.reverse();
                    changed = true;
                }
            }
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
    tol: Tolerance,
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
        .any(|p| dot(sub(*p, c), n).abs() > 1e-6 * extent.max(1.0) + tol.xy)
    {
        return Err(ExtrudeError::InvalidSurface("the surface is not planar"));
    }
    Ok(scale(n, sign))
}

#[allow(clippy::too_many_arguments)]
fn sweep_piece(
    frame: &CoordinateFrame,
    sign: f64,
    tol: Tolerance,
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

    // Welded ids are pairwise apart, so an edge whose ends now coincide was
    // collapsed by placement (a flattened base or a flat top over a vertical
    // step). Its wall and cap edge would have no length.
    for &f in piece {
        for ring in &faces[f].rings {
            for i in 0..ring.len() {
                let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
                if tol.coincide(at(&base, a), at(&base, b))
                    || tol.coincide(at(&top, a), at(&top, b))
                {
                    return Err(ExtrudeError::Degenerate(
                        "placement collapses an edge of the surface",
                    ));
                }
            }
        }
    }

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
    // A boundary edge parallel to the sweep (a vertical step in a non-planar
    // face swept vertically) would get a wall with no area.
    for &f in piece {
        for ring in &faces[f].rings {
            for i in 0..ring.len() {
                let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
                if !is_boundary(edges, a, b) {
                    continue;
                }
                let e = sub(at(&base, b), at(&base, a));
                let across = sub(e, scale(sweep_unit, dot(e, sweep_unit)));
                if norm(across) <= tol.xy {
                    return Err(ExtrudeError::Degenerate(
                        "an edge of the surface is parallel to the extrusion",
                    ));
                }
            }
        }
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
        // Coordinates are stored in each CRS's declared axis order. EPSG:6668
        // (JGD2011) and EPSG:4326 (WGS 84) both declare (lat, lon), so the
        // stored winding is mirrored and the frame's sign undoes it. EPSG:3857
        // is (easting, northing) in metres, a positive-sign control.
        for (epsg, origin, size) in [
            (6668u16, [35.68, 139.76, 0.0], 0.0001),
            (4326, [35.68, 139.76, 0.0], 0.0001),
            (3857, [15_557_700.0, 4_257_400.0, 0.0], 10.0),
        ] {
            for ring in [square(origin, size), {
                let mut r = square(origin, size);
                r.reverse();
                r
            }] {
                let g = extrude(
                    &polygon(CoordinateFrame::Crs(EpsgCode::from(epsg)), ring, vec![]),
                    &up(20.0),
                )
                .unwrap();
                assert_valid(&g);
            }
        }
    }

    #[test]
    fn a_northing_first_projected_crs_sweeps_its_front_normal_up() {
        // EPSG:6677 (JGD2011 plane IX) stores (northing, easting). A ring that
        // is counter-clockwise in canonical (east, north) order is clockwise
        // as stored; its front normal is still up, so a positive surface
        // normal distance raises it.
        let (n0, e0) = (-35_000.0, -6_000.0);
        let canonical_ccw = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let mut ring: Vec<[f64; 3]> = canonical_ccw
            .iter()
            .map(|[e, n]| [n0 + n, e0 + e, 0.0])
            .collect();
        ring.push(ring[0]);
        let spec = ExtrudeSpec {
            direction: ExtrudeDirection::SurfaceNormal { distance: 5.0 },
            output: ExtrudeOutput::Solid,
        };
        let g = extrude(
            &polygon(CoordinateFrame::Crs(EpsgCode::from(6677u16)), ring, vec![]),
            &spec,
        )
        .unwrap();
        assert_valid(&g);
        assert_eq!(z_range(&g), (0.0, 5.0));
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
    fn a_shared_edge_with_jittered_z_at_projected_magnitudes_is_welded() {
        // The right square's shared vertices sit 1e-7 m higher than the left
        // square's: well inside the 0.5 mm horizontal tolerance at x ~ 5e5,
        // so the vertical tolerance must not be the one keeping them apart.
        let (x, y, z) = (500_000.0, 3_900_000.0, 10.0);
        let left = square([x, y, z], 10.0);
        let mut right = square([x + 10.0, y, z], 10.0);
        for i in [0, 3, 4] {
            right[i][2] += 1e-7;
        }
        let g = extrude(&mesh(&[left, right]), &up(5.0)).unwrap();
        assert_valid(&g);
        // One solid: 2 bottom + 2 top + 6 boundary walls, no interior wall.
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

    /// The z extent of `g` (every vertex, every member).
    fn z_range(g: &Geometry) -> (f64, f64) {
        use crate::ops::BoundingBox;
        let (min, max) = g.bounding_box().unwrap().corners_3d();
        (min[2], max[2])
    }

    /// The z extent of each member of a 3D collection.
    fn member_z_ranges(g: &Geometry) -> Vec<(f64, f64)> {
        let Geometry::Euclidean3D(Euclidean3DGeometry::Collection(c)) = g else {
            panic!("{g:?}")
        };
        c.members()
            .iter()
            .map(|m| z_range(&Geometry::Euclidean3D(m.clone())))
            .collect()
    }

    #[test]
    fn a_hole_wound_like_its_exterior_still_gets_outward_walls() {
        // Same winding as the exterior: input that is mis-wound but common.
        let hole = square([0.25, 0.25, 0.0], 0.5);
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
        assert_eq!(solid_faces(&g), 10);
    }

    fn polygon_2d(ring: Vec<[f64; 2]>) -> Geometry {
        use crate::polygon::Polygon2D;
        use crate::Euclidean2DGeometry;
        Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                CoordinateFrame::Euclidean,
                ring,
                std::iter::empty::<Vec<[f64; 2]>>(),
            ),
        )))
    }

    #[test]
    fn a_2d_surface_normal_sweeps_up_whatever_the_winding() {
        let ccw = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]];
        let mut cw = ccw.clone();
        cw.reverse();
        let spec = ExtrudeSpec {
            direction: ExtrudeDirection::SurfaceNormal { distance: 5.0 },
            output: ExtrudeOutput::Solid,
        };
        for ring in [ccw, cw] {
            let g = extrude(&polygon_2d(ring.clone()), &spec).unwrap();
            assert_valid(&g);
            assert_eq!(z_range(&g), (0.0, 5.0), "{ring:?}");
            // The same rule holds for a 2D member of a geometry collection.
            let c = Geometry::GeometryCollection(crate::GeometryCollection::new([polygon_2d(
                ring.clone(),
            )]));
            let g = extrude(&c, &spec).unwrap();
            assert_valid(&g);
            assert_eq!(z_range(&g), (0.0, 5.0), "collection {ring:?}");
        }
    }

    #[test]
    fn a_2d_surface_normal_still_needs_linear_units() {
        use crate::polygon::Polygon2D;
        use crate::Euclidean2DGeometry;
        let p = Polygon2D::from_rings(
            CoordinateFrame::Crs(EpsgCode::from(4326u16)),
            [
                [35.0, 139.0],
                [35.0, 139.1],
                [35.1, 139.1],
                [35.1, 139.0],
                [35.0, 139.0],
            ],
            std::iter::empty::<Vec<[f64; 2]>>(),
        );
        let spec = ExtrudeSpec {
            direction: ExtrudeDirection::SurfaceNormal { distance: 5.0 },
            output: ExtrudeOutput::Solid,
        };
        assert!(matches!(
            extrude(
                &Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(p))),
                &spec
            ),
            Err(ExtrudeError::UnsupportedFrame(_))
        ));
    }

    #[test]
    fn flat_modes_use_the_whole_features_extremes_across_mesh_pieces() {
        let feature = mesh(&[square([0.0; 3], 1.0), square([5.0, 0.0, 7.0], 1.0)]);
        let g = extrude(
            &feature,
            &vertical(BaseMode::FlattenToLowest, TopMode::AboveHighest(10.0)),
        )
        .unwrap();
        assert_valid(&g);
        assert_eq!(member_z_ranges(&g), vec![(0.0, 10.0), (0.0, 10.0)]);

        // A kept base still rises to one roof above the feature's highest vertex.
        let g = extrude(
            &feature,
            &vertical(BaseMode::KeepElevation, TopMode::AboveHighest(10.0)),
        )
        .unwrap();
        assert_valid(&g);
        assert_eq!(member_z_ranges(&g), vec![(0.0, 17.0), (7.0, 17.0)]);

        // And above the feature's lowest one.
        let g = extrude(
            &feature,
            &vertical(BaseMode::KeepElevation, TopMode::AboveLowest(10.0)),
        )
        .unwrap();
        assert_eq!(member_z_ranges(&g), vec![(0.0, 10.0), (7.0, 10.0)]);
    }

    #[test]
    fn flat_modes_use_the_whole_features_extremes_across_collection_members() {
        let member = |o: [f64; 3]| {
            Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
                CoordinateFrame::Euclidean,
                square(o, 1.0),
                Vec::<Vec<[f64; 3]>>::new(),
            )))
        };
        let feature = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([
            member([0.0; 3]),
            member([5.0, 0.0, 7.0]),
        ])));
        let g = extrude(
            &feature,
            &vertical(BaseMode::FlattenToLowest, TopMode::AboveHighest(10.0)),
        )
        .unwrap();
        assert_valid(&g);
        assert_eq!(member_z_ranges(&g), vec![(0.0, 10.0), (0.0, 10.0)]);
    }

    #[test]
    fn a_vertex_notch_in_degrees_survives_a_high_elevation() {
        // Lat/lon-like magnitudes at 3000 m: a 5 cm (~4.5e-7 degree) notch
        // must not be welded away because z is large.
        let (x, y, z, s, d) = (139.76, 35.68, 3000.0, 0.0001, 4.5e-7);
        let ring = vec![
            [x, y, z],
            [x + d, y + d, z],
            [x + s, y, z],
            [x + s, y + s, z],
            [x, y + s, z],
            [x, y, z],
        ];
        let g = extrude(
            &polygon(CoordinateFrame::Euclidean, ring, vec![]),
            &up(10.0),
        )
        .unwrap();
        // 2 caps + 5 walls: the notch vertex is kept.
        assert_eq!(solid_faces(&g), 7);
        assert_valid(&g);
    }

    #[test]
    fn a_vertical_step_swept_vertically_is_degenerate() {
        let ring = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
        ];
        // Kept elevations do not collapse the step, but its wall would have
        // no area.
        let p = polygon(CoordinateFrame::Euclidean, ring, vec![]);
        assert!(matches!(
            extrude(&p, &up(5.0)),
            Err(ExtrudeError::Degenerate(_))
        ));
    }

    /// A square in (lat, lon) degrees whose z rises `tilt` metres across it
    /// along the first axis, as a draped footprint would.
    fn tilted_square(o: [f64; 3], s: f64, tilt: f64) -> Vec<[f64; 3]> {
        square(o, s)
            .into_iter()
            .map(|[x, y, z]| [x, y, z + tilt * (x - o[0]) / s])
            .collect()
    }

    /// The plan-view (n_z) sign of the exterior and of each hole of the solid's
    /// bottom cap: the holed face with the lowest mean z.
    fn bottom_cap_plan_signs(g: &Geometry) -> (f64, Vec<f64>) {
        let Geometry::Euclidean3D(Euclidean3DGeometry::Solid(s)) = g else {
            panic!("{g:?}")
        };
        let Shell::PolygonMesh(m) = s.exterior() else {
            panic!()
        };
        let mean_z = |p: &Polygon3D| {
            let e = p.exterior();
            e.iter().map(|c| c[2]).sum::<f64>() / e.len() as f64
        };
        let bottom = m
            .faces(s.frame())
            .into_iter()
            .filter(|p| p.interiors().next().is_some())
            .min_by(|a, b| mean_z(a).total_cmp(&mean_z(b)))
            .expect("a holed cap");
        let n_z = |ring: &[[f64; 3]]| newell(ring)[2].signum();
        (
            n_z(bottom.exterior()),
            bottom.interiors().map(n_z).collect(),
        )
    }

    /// `g`, a solid stored in EPSG:6668 (lat, lon) degrees near `o`, copied
    /// into a metric east-north-up frame (about 1 m per 1e-5 degree), and
    /// fully validated there: an independent check, in a frame whose axes
    /// share a unit, alongside validating the native-frame output (whose hole
    /// check judges plan-view winding in degrees).
    fn assert_valid_in_metres(g: &Geometry, o: [f64; 3]) {
        let Geometry::Euclidean3D(Euclidean3DGeometry::Solid(s)) = g else {
            panic!("{g:?}")
        };
        let Shell::PolygonMesh(m) = s.exterior() else {
            panic!()
        };
        let metric = |ring: &[[f64; 3]]| -> Vec<[f64; 3]> {
            ring.iter()
                .map(|&[lat, lon, z]| [(lon - o[1]) * 91_000.0, (lat - o[0]) * 111_000.0, z])
                .collect()
        };
        let faces: Vec<Polygon3D> = m
            .faces(s.frame())
            .iter()
            .map(|p| {
                Polygon3D::from_rings(
                    CoordinateFrame::Euclidean,
                    metric(p.exterior()),
                    p.interiors().map(metric).collect::<Vec<_>>(),
                )
            })
            .collect();
        let data = PolygonMesh3DData::from_polygons(faces.iter().collect::<Vec<_>>());
        assert_valid(&Geometry::Euclidean3D(Euclidean3DGeometry::Solid(
            Box::new(Solid::from_exterior(
                CoordinateFrame::Euclidean,
                Shell::PolygonMesh(data),
            )),
        )));
    }

    #[test]
    fn a_draped_hole_in_degrees_keeps_its_plan_view_winding() {
        // EPSG:6668 is (lat, lon) degrees with z in metres. A hole wound
        // opposite its exterior in plan view, on a footprint draped 1 cm one
        // way while the hole tilts 1 cm the other: the raw 3D normals are
        // dominated by the tilt (deg*m against deg^2), so they must not decide
        // the hole's winding.
        let (o, s) = ([35.68, 139.76, 0.0], 0.0001);
        let ext = tilted_square(o, s, 0.01);
        let mut hole = tilted_square([o[0] + s / 4.0, o[1] + s / 4.0, 0.0], s / 2.0, -0.01);
        hole.reverse();
        let frame = CoordinateFrame::Crs(EpsgCode::from(6668u16));
        let g = extrude(&polygon(frame, ext, vec![hole]), &up(20.0)).unwrap();
        assert_valid(&g);
        assert_valid_in_metres(&g, o);
        let (ext_sign, hole_signs) = bottom_cap_plan_signs(&g);
        assert_eq!(hole_signs, vec![-ext_sign]);
        assert_eq!(solid_faces(&g), 10);
    }

    #[test]
    fn a_draped_same_wound_hole_in_degrees_is_rewound() {
        let (o, s) = ([35.68, 139.76, 0.0], 0.0001);
        let ext = tilted_square(o, s, 0.01);
        let hole = tilted_square([o[0] + s / 4.0, o[1] + s / 4.0, 0.0], s / 2.0, -0.01);
        let frame = CoordinateFrame::Crs(EpsgCode::from(6668u16));
        let g = extrude(&polygon(frame, ext, vec![hole]), &up(20.0)).unwrap();
        assert_valid(&g);
        assert_valid_in_metres(&g, o);
        let (ext_sign, hole_signs) = bottom_cap_plan_signs(&g);
        assert_eq!(hole_signs, vec![-ext_sign]);
        assert_eq!(solid_faces(&g), 10);
    }

    #[test]
    fn flattening_a_vertical_step_is_degenerate() {
        // Two consecutive vertices share x and y but not z: flattening
        // collapses the edge between them.
        let ring = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
        ];
        let p = polygon(CoordinateFrame::Euclidean, ring, vec![]);
        for top in [TopMode::ByHeight(5.0), TopMode::ToElevation(9.0)] {
            assert!(
                matches!(
                    extrude(&p, &vertical(BaseMode::FlattenToLowest, top)),
                    Err(ExtrudeError::Degenerate(_))
                ),
                "{top:?}"
            );
        }
    }
}
