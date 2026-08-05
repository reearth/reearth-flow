//! Boolean evaluation of a [`Csg`] tree into a [`Solid`].
//!
//! Each boolean indexes both surfaces ([`TriangleSet::rtree`]), splits only the
//! polygons near the other one (see [`split_against`]), and keeps the fragments
//! whose centroid parity matches the operation (see [`classify`]).
//!
//! Crossings are recorded against the edge they cut rather than materialised
//! into it, so both polygons on an edge (and one the plane never reached) agree
//! on the vertices along it. Only once every crossing is recorded are vertices
//! merged within the tolerance and the boundaries subdivided.
//!
//! Operands must share one frame, be in linear units (the tolerance is a
//! distance), and be closed and outward-wound; an open or mis-wound boundary
//! gives an arbitrary volume, not an error. Appearance is not propagated.

use std::collections::{HashMap, HashSet};

use rstar::{RTree, AABB};

use crate::coordinate::{CoordinateFrame, UnitKind};
use crate::error::Error;
use crate::ops::triangulation::Cache;
use crate::predicates::kernel::{cross3, dot3, sub3};
use crate::predicates::kernel3d::point_in_triangle_3d;
use crate::predicates::position3d::{pool_bounds, shell_position_bounded};
use crate::predicates::view3d::{TriBox, TriangleSet};
use crate::predicates::CoordPos;
use crate::solid::{Shell, Solid};
use crate::triangular_mesh::TriangularMesh3DData;

use super::{Csg, ThreeDimensional};

/// A corner, as an index into [`Arena::positions`].
type VertexId = u32;

/// An edge, named by its endpoints in ascending id order.
type EdgeId = (VertexId, VertexId);

/// The edge a crossing subdivides, and its parameter along that edge.
type OnEdge = (EdgeId, f64);

impl Csg {
    /// The solid the tree denotes, or `None` when it encloses no volume.
    ///
    /// `tolerance` is the distance within which a vertex counts as lying on a
    /// cutting plane and two vertices count as one; at or below zero it falls
    /// back to a small default.
    pub fn evaluate(&self, tolerance: f64) -> Result<Option<Solid>, Error> {
        let eps = if tolerance > 0.0 { tolerance } else { 1e-9 };
        let mut cache = Cache::default();
        let mut arena = Arena::default();
        let (mut polygons, frame) = evaluate_tree(self, eps, &mut cache, &mut arena)?;

        arena.merge(eps, &mut polygons);
        let mesh = arena.mesh(&polygons);
        if mesh.num_triangles() == 0 {
            return Ok(None);
        }
        let (exterior, voids) = partition_shells(mesh, &frame)?;
        Ok(Some(Solid::new(
            frame,
            exterior,
            voids.into_iter().map(Shell::from).collect(),
        )))
    }
}

/// A degree covers a different distance at every latitude, so one tolerance
/// cannot mean one distance across a geographic operand.
fn require_linear_units(frame: &CoordinateFrame) -> Result<(), Error> {
    match frame.unit_kind() {
        UnitKind::Linear => Ok(()),
        UnitKind::Angular => Err(Error::invalid_geometry(
            "CSG evaluation cannot be done in geographic coordinates: it merges vertices within \
             a distance tolerance, and a degree is not a distance. Reproject the operands to a \
             projected CRS first.",
        )),
        UnitKind::Undeterminable(reason) => Err(Error::invalid_geometry(format!(
            "CSG evaluation needs coordinates in linear units, and this frame's units could not \
             be classified: {reason}"
        ))),
    }
}

/// The vertices the boolean works over. Corners live here once and polygons
/// address them by index, so a point two polygons share is one entry rather
/// than two coordinate copies to be recognised as equal later.
#[derive(Default)]
struct Arena {
    positions: Vec<[f64; 3]>,
    /// Operand corners by exact coordinate bits, so one shared by several faces
    /// enters once.
    interned: HashMap<[u64; 3], VertexId>,
    on_edge: HashMap<VertexId, OnEdge>,
    /// The crossings on each edge, with their parameters.
    splits: HashMap<(VertexId, VertexId), Vec<(f64, VertexId)>>,
}

impl Arena {
    #[inline]
    fn position(&self, id: VertexId) -> [f64; 3] {
        self.positions[id as usize]
    }

    /// An operand corner, interned on exact coordinates.
    fn intern(&mut self, p: [f64; 3]) -> VertexId {
        let key = [p[0].to_bits(), p[1].to_bits(), p[2].to_bits()];
        if let Some(&id) = self.interned.get(&key) {
            return id;
        }
        let id = self.push(p);
        self.interned.insert(key, id);
        id
    }

    fn push(&mut self, p: [f64; 3]) -> VertexId {
        let id = self.positions.len() as VertexId;
        self.positions.push(p);
        id
    }

    /// The edge the side `u`-`v` runs along, with the parameters of `u` and `v`
    /// on it. A piece of an already-cut edge is carried by that whole edge, so
    /// every crossing anywhere along it is filed in one place: two polygons cut
    /// by the same planes in opposite orders would otherwise file under
    /// different keys and neither would see the other's. A side belonging to no
    /// such edge carries itself.
    fn carrier(&self, u: VertexId, v: VertexId) -> ((VertexId, VertexId), f64, f64) {
        let ends = |edge: (VertexId, VertexId), id: VertexId| {
            if id == edge.0 {
                Some(0.0)
            } else if id == edge.1 {
                Some(1.0)
            } else {
                None
            }
        };
        let on_u = self.on_edge.get(&u).copied();
        let on_v = self.on_edge.get(&v).copied();
        let carried = match (on_u, on_v) {
            (Some((e, tu)), Some((f, tv))) if e == f => Some((e, tu, tv)),
            (Some((e, tu)), None) => ends(e, v).map(|tv| (e, tu, tv)),
            (None, Some((e, tv))) => ends(e, u).map(|tu| (e, tu, tv)),
            _ => None,
        };
        carried.unwrap_or(if u < v {
            ((u, v), 0.0, 1.0)
        } else {
            ((v, u), 1.0, 0.0)
        })
    }

    /// The vertex where `plane` crosses the side `u`-`v`, recorded against that
    /// side's carrier. The parameter runs along the carrier's canonical
    /// direction, independent of both the traversal direction and how far the
    /// polygon had already been cut, so two polygons cut by one plane land on
    /// the same vertex rather than two a few bits apart.
    ///
    /// The position is materialised here because further splitting needs
    /// coordinates; what is deferred is cutting it into the boundaries.
    fn crossing(&mut self, u: VertexId, v: VertexId, plane: &Plane) -> VertexId {
        let (edge, _, _) = self.carrier(u, v);
        let (a, b) = (self.position(edge.0), self.position(edge.1));
        let t = (plane.w - dot3(plane.normal, a)) / dot3(plane.normal, sub3(b, a));
        if let Some(recorded) = self.splits.get(&edge) {
            if let Some(&(_, id)) = recorded
                .iter()
                .find(|(seen, _)| seen.to_bits() == t.to_bits())
            {
                return id;
            }
        }
        let id = self.push(lerp(a, b, t));
        self.on_edge.insert(id, (edge, t));
        self.splits.entry(edge).or_default().push((t, id));
        id
    }

    /// Merge vertices within `eps`, rewriting `polygons` and the crossings onto
    /// the survivors, which are then more than `eps` apart.
    fn merge(&mut self, eps: f64, polygons: &mut [Polygon]) {
        let mut grid: HashMap<[i64; 3], Vec<VertexId>> = HashMap::new();
        let mut remap: Vec<VertexId> = Vec::with_capacity(self.positions.len());
        for (i, p) in self.positions.iter().enumerate() {
            let base = cell(*p, eps);
            let mut onto = None;
            'search: for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        let key = [base[0] + dx, base[1] + dy, base[2] + dz];
                        for &j in grid.get(&key).map(Vec::as_slice).unwrap_or_default() {
                            let q = self.positions[j as usize];
                            if (0..3).all(|k| (q[k] - p[k]).abs() <= eps) {
                                onto = Some(j);
                                break 'search;
                            }
                        }
                    }
                }
            }
            match onto {
                Some(j) => remap.push(j),
                None => {
                    grid.entry(base).or_default().push(i as VertexId);
                    remap.push(i as VertexId);
                }
            }
        }

        for polygon in polygons.iter_mut() {
            for id in &mut polygon.vertices {
                *id = remap[*id as usize];
            }
            polygon.vertices.dedup();
            if polygon.vertices.len() > 1 && polygon.vertices.first() == polygon.vertices.last() {
                polygon.vertices.pop();
            }
        }

        let mut splits: HashMap<(VertexId, VertexId), Vec<(f64, VertexId)>> = HashMap::new();
        let mut on_edge: HashMap<VertexId, OnEdge> = HashMap::new();
        // In vertex order, not the map's: replay order decides which carrier a
        // merged vertex keeps, so a hashed order would differ run to run.
        let mut records: Vec<(VertexId, OnEdge)> =
            std::mem::take(&mut self.on_edge).into_iter().collect();
        records.sort_by_key(|&(id, _)| id);
        for (id, (edge, t)) in records {
            let id = remap[id as usize];
            let (u, v) = (remap[edge.0 as usize], remap[edge.1 as usize]);
            if u == v || id == u || id == v {
                continue;
            }
            // Surviving ids may reverse the pair, and a parameter measured the
            // other way runs backwards.
            let (key, t) = if u < v {
                ((u, v), t)
            } else {
                ((v, u), 1.0 - t)
            };
            splits.entry(key).or_default().push((t, id));
            on_edge.entry(id).or_insert((key, t));
        }
        for recorded in splits.values_mut() {
            recorded.sort_by(|(a, _), (b, _)| a.total_cmp(b));
            recorded.dedup_by_key(|(_, id)| *id);
        }
        self.splits = splits;
        self.on_edge = on_edge;
    }

    /// The result mesh, with every recorded crossing cut in.
    fn mesh(&self, polygons: &[Polygon]) -> TriangularMesh3DData {
        let mut soup: Vec<[f64; 3]> = Vec::new();
        let mut ring: Vec<VertexId> = Vec::new();
        for polygon in polygons {
            ring.clear();
            self.walk_boundary(&polygon.vertices, &mut ring);
            self.fan(&ring, &mut soup);
        }
        TriangularMesh3DData::from_soup(soup)
    }

    /// The boundary, subdivided by the crossings on its edges, so an edge
    /// another polygon had cut is cut here too.
    fn walk_boundary(&self, vertices: &[VertexId], out: &mut Vec<VertexId>) {
        let n = vertices.len();
        for i in 0..n {
            let (u, v) = (vertices[i], vertices[(i + 1) % n]);
            out.push(u);
            self.crossings_between(u, v, out);
        }
        out.dedup();
        if out.len() > 1 && out.first() == out.last() {
            out.pop();
        }
    }

    /// The crossings strictly between `u` and `v` on their carrier, ordered
    /// from `u` to `v`.
    fn crossings_between(&self, u: VertexId, v: VertexId, out: &mut Vec<VertexId>) {
        let (edge, tu, tv) = self.carrier(u, v);
        let Some(recorded) = self.splits.get(&edge) else {
            return;
        };
        let (low, high) = if tu <= tv { (tu, tv) } else { (tv, tu) };
        let mut between: Vec<(f64, VertexId)> = recorded
            .iter()
            .copied()
            .filter(|&(t, id)| t > low && t < high && id != u && id != v)
            .collect();
        between.sort_by(|(a, _), (b, _)| a.total_cmp(b));
        if tu > tv {
            between.reverse();
        }
        out.extend(between.into_iter().map(|(_, id)| id));
    }

    /// Fan the ring into triangles, dropping degenerate ones.
    fn fan(&self, ring: &[VertexId], soup: &mut Vec<[f64; 3]>) {
        if ring.len() < 3 {
            return;
        }
        let apex = self.sharpest_corner(ring);
        let n = ring.len();
        for i in 1..n - 1 {
            let corners = [
                self.position(ring[apex]),
                self.position(ring[(apex + i) % n]),
                self.position(ring[(apex + i + 1) % n]),
            ];
            let normal = cross3(sub3(corners[1], corners[0]), sub3(corners[2], corners[0]));
            if dot3(normal, normal) > 0.0 {
                soup.extend(corners);
            }
        }
    }

    /// The corner with the largest turn: fanning from one inside a collinear
    /// run would turn the run into slivers and lose its vertices.
    fn sharpest_corner(&self, ring: &[VertexId]) -> usize {
        let n = ring.len();
        let mut apex = 0;
        let mut sharpest = -1.0;
        for i in 0..n {
            let before = self.position(ring[(i + n - 1) % n]);
            let at = self.position(ring[i]);
            let after = self.position(ring[(i + 1) % n]);
            let turn = cross3(sub3(at, before), sub3(after, at));
            let turn = dot3(turn, turn);
            if turn > sharpest {
                sharpest = turn;
                apex = i;
            }
        }
        apex
    }
}

fn cell(p: [f64; 3], eps: f64) -> [i64; 3] {
    p.map(|x| (x / eps).floor() as i64)
}

/// Split the boundary into the exterior mesh and its void shells: a closed
/// component enclosing a canonically negative volume is a void.
fn partition_shells(
    mesh: TriangularMesh3DData,
    frame: &CoordinateFrame,
) -> Result<(TriangularMesh3DData, Vec<TriangularMesh3DData>), Error> {
    let triangles: Vec<[u32; 3]> = mesh.triangles().collect();
    let vertices = mesh.vertices();

    fn find(parent: &mut [u32], mut i: u32) -> u32 {
        while parent[i as usize] != i {
            parent[i as usize] = parent[parent[i as usize] as usize];
            i = parent[i as usize];
        }
        i
    }
    let mut parent: Vec<u32> = (0..triangles.len() as u32).collect();
    let mut first_on_edge: HashMap<(u32, u32), u32> = HashMap::new();
    for (i, t) in triangles.iter().enumerate() {
        for key in triangle_edges(t) {
            if let Some(&first) = first_on_edge.get(&key) {
                let (a, b) = (find(&mut parent, first), find(&mut parent, i as u32));
                if a != b {
                    parent[a as usize] = b;
                }
            } else {
                first_on_edge.insert(key, i as u32);
            }
        }
    }

    let mut components: HashMap<u32, Vec<usize>> = HashMap::new();
    for i in 0..triangles.len() {
        components
            .entry(find(&mut parent, i as u32))
            .or_default()
            .push(i);
    }
    if components.len() <= 1 {
        return Ok((mesh, Vec::new()));
    }

    let sign = f64::from(frame.orientation_sign()?);
    let mut voids: Vec<Vec<usize>> = Vec::new();
    let mut exterior: Vec<usize> = Vec::new();
    // In triangle order, not the map's, so shells come out the same each run.
    let mut components: Vec<Vec<usize>> = components.into_values().collect();
    components.sort_by_key(|component| component[0]);
    for component in components {
        if is_closed(&component, &triangles)
            && signed_volume(&component, &triangles, vertices) * sign < 0.0
        {
            voids.push(component);
        } else {
            exterior.extend(component);
        }
    }
    if voids.is_empty() {
        return Ok((mesh, Vec::new()));
    }
    exterior.sort_unstable();
    let rebuild = |component: &[usize]| {
        TriangularMesh3DData::from_soup(
            component
                .iter()
                .flat_map(|&i| triangles[i].map(|v| vertices[v as usize])),
        )
    };
    Ok((
        rebuild(&exterior),
        voids.iter().map(|component| rebuild(component)).collect(),
    ))
}

/// The triangle's undirected edges, each keyed low index first.
fn triangle_edges(t: &[u32; 3]) -> [(u32, u32); 3] {
    [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])].map(|(a, b)| (a.min(b), a.max(b)))
}

/// Whether every edge of the component lies on exactly two of its triangles.
fn is_closed(component: &[usize], triangles: &[[u32; 3]]) -> bool {
    let mut counts: HashMap<(u32, u32), u32> = HashMap::new();
    for &i in component {
        for key in triangle_edges(&triangles[i]) {
            *counts.entry(key).or_insert(0) += 1;
        }
    }
    counts.values().all(|&c| c == 2)
}

/// The component's right-hand-rule signed volume.
fn signed_volume(component: &[usize], triangles: &[[u32; 3]], vertices: &[[f64; 3]]) -> f64 {
    component
        .iter()
        .map(|&i| {
            let [a, b, c] = triangles[i].map(|v| vertices[v as usize]);
            dot3(a, cross3(b, c)) / 6.0
        })
        .sum()
}

/// Evaluate a tree node into its boundary polygons and their frame.
fn evaluate_tree(
    csg: &Csg,
    eps: f64,
    cache: &mut Cache,
    arena: &mut Arena,
) -> Result<(Vec<Polygon>, CoordinateFrame), Error> {
    let (left, right) = match csg {
        Csg::Union(l, r) | Csg::Intersection(l, r) | Csg::Difference(l, r) => (l, r),
    };
    let (left, left_frame) = evaluate_operand(left, eps, cache, arena)?;
    let (right, right_frame) = evaluate_operand(right, eps, cache, arena)?;
    if left_frame != right_frame {
        return Err(Error::mismatched_geometry(
            "CSG operands are expressed in different coordinate frames",
        ));
    }
    let op = match csg {
        Csg::Union(..) => BoolOp::Union,
        Csg::Intersection(..) => BoolOp::Intersection,
        Csg::Difference(..) => BoolOp::Difference,
    };
    Ok((boolean(op, left, right, arena, eps), left_frame))
}

/// Evaluate one operand into its boundary polygons and its frame.
fn evaluate_operand(
    operand: &ThreeDimensional,
    eps: f64,
    cache: &mut Cache,
    arena: &mut Arena,
) -> Result<(Vec<Polygon>, CoordinateFrame), Error> {
    match operand {
        ThreeDimensional::Solid(solid) => {
            let frame = solid.frame().clone();
            require_linear_units(&frame)?;
            Ok((solid_polygons(solid, cache, arena), frame))
        }
        ThreeDimensional::Csg(csg) => evaluate_tree(csg, eps, cache, arena),
    }
}

/// Every shell of a solid, triangulated, degenerate triangles dropped.
fn solid_polygons(solid: &Solid, cache: &mut Cache, arena: &mut Arena) -> Vec<Polygon> {
    let mut polygons = Vec::new();
    shell_polygons(solid.exterior(), cache, arena, &mut polygons);
    for shell in solid.interiors() {
        shell_polygons(shell, cache, arena, &mut polygons);
    }
    polygons
}

/// Append one shell's faces as triangle polygons.
fn shell_polygons(shell: &Shell, cache: &mut Cache, arena: &mut Arena, out: &mut Vec<Polygon>) {
    let data;
    let mesh = match shell {
        Shell::TriangularMesh(d) => d,
        Shell::PolygonMesh(d) => {
            data = d.clone().triangulate(cache).mesh;
            &data
        }
    };
    let vertices = mesh.vertices();
    for [a, b, c] in mesh.triangles() {
        let (a, b, c) = (
            vertices[a as usize],
            vertices[b as usize],
            vertices[c as usize],
        );
        if let Some(plane) = Plane::from_points(a, b, c) {
            out.push(Polygon {
                vertices: vec![arena.intern(a), arena.intern(b), arena.intern(c)],
                plane,
            });
        }
    }
}

fn lerp(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// An oriented plane `normal · x = w`; the front side is the normal side.
#[derive(Clone)]
struct Plane {
    normal: [f64; 3],
    w: f64,
}

/// A vertex's side of a plane, and or'ed together a polygon's.
const COPLANAR: u8 = 0;
const FRONT: u8 = 1;
const BACK: u8 = 2;
const SPANNING: u8 = 3;

/// Where [`Plane::split_polygon`] puts each piece.
#[derive(Default)]
struct SplitParts {
    coplanar_front: Vec<Polygon>,
    coplanar_back: Vec<Polygon>,
    front: Vec<Polygon>,
    back: Vec<Polygon>,
}

impl SplitParts {
    /// Every piece: they all carry on to the remaining planes.
    fn into_all(mut self) -> Vec<Polygon> {
        self.front.append(&mut self.back);
        self.front.append(&mut self.coplanar_front);
        self.front.append(&mut self.coplanar_back);
        self.front
    }
}

impl Plane {
    /// The plane through three points, `None` when they are (near) collinear.
    fn from_points(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Option<Plane> {
        let n = cross3(sub3(b, a), sub3(c, a));
        let len = dot3(n, n).sqrt();
        if len < 1e-12 {
            return None;
        }
        let normal = [n[0] / len, n[1] / len, n[2] / len];
        Some(Plane {
            normal,
            w: dot3(normal, a),
        })
    }

    fn flip(&mut self) {
        self.normal = [-self.normal[0], -self.normal[1], -self.normal[2]];
        self.w = -self.w;
    }

    /// Classify `polygon` against this plane, splitting it where it spans both
    /// sides. Crossings come from the arena, so they are shared with every
    /// other polygon on the same edge.
    fn split_polygon(&self, polygon: &Polygon, arena: &mut Arena, eps: f64, out: &mut SplitParts) {
        let mut polygon_type = COPLANAR;
        let mut types: Vec<u8> = Vec::with_capacity(polygon.vertices.len());
        for &id in &polygon.vertices {
            let distance = dot3(self.normal, arena.position(id)) - self.w;
            let side = if distance < -eps {
                BACK
            } else if distance > eps {
                FRONT
            } else {
                COPLANAR
            };
            polygon_type |= side;
            types.push(side);
        }

        match polygon_type {
            COPLANAR => {
                if dot3(self.normal, polygon.plane.normal) > 0.0 {
                    out.coplanar_front.push(polygon.clone());
                } else {
                    out.coplanar_back.push(polygon.clone());
                }
            }
            FRONT => out.front.push(polygon.clone()),
            BACK => out.back.push(polygon.clone()),
            _ => {
                let mut f: Vec<VertexId> = Vec::new();
                let mut b: Vec<VertexId> = Vec::new();
                let n = polygon.vertices.len();
                for i in 0..n {
                    let j = (i + 1) % n;
                    let (ti, tj) = (types[i], types[j]);
                    let (vi, vj) = (polygon.vertices[i], polygon.vertices[j]);
                    if ti != BACK {
                        f.push(vi);
                    }
                    if ti != FRONT {
                        b.push(vi);
                    }
                    if (ti | tj) == SPANNING {
                        let crossing = arena.crossing(vi, vj, self);
                        f.push(crossing);
                        b.push(crossing);
                    }
                }
                if f.len() >= 3 {
                    out.front.push(Polygon {
                        vertices: f,
                        plane: polygon.plane.clone(),
                    });
                }
                if b.len() >= 3 {
                    out.back.push(Polygon {
                        vertices: b,
                        plane: polygon.plane.clone(),
                    });
                }
            }
        }
    }
}

/// A planar convex boundary polygon. Splitting keeps fragments convex, so
/// convexity is an invariant.
#[derive(Clone)]
struct Polygon {
    vertices: Vec<VertexId>,
    plane: Plane,
}

impl Polygon {
    fn flip(&mut self) {
        self.vertices.reverse();
        self.plane.flip();
    }

    /// A fan of triangles, for the queryable surface: the crossings cut in at
    /// the end do not change the shape.
    fn triangles<'a>(&'a self, arena: &'a Arena) -> impl Iterator<Item = [[f64; 3]; 3]> + 'a {
        let v = &self.vertices;
        (1..v.len().saturating_sub(1)).map(move |i| {
            [
                arena.position(v[0]),
                arena.position(v[i]),
                arena.position(v[i + 1]),
            ]
        })
    }

    /// The corner average: interior to a convex polygon, so it stands in for
    /// the whole fragment once the fragment cannot cross the other surface.
    fn centroid(&self, arena: &Arena) -> [f64; 3] {
        let mut c = [0.0f64; 3];
        for &id in &self.vertices {
            let p = arena.position(id);
            for k in 0..3 {
                c[k] += p[k];
            }
        }
        let n = self.vertices.len() as f64;
        c.map(|x| x / n)
    }

    /// The polygon's box, inflated by `pad` on every side.
    fn envelope(&self, arena: &Arena, pad: f64) -> AABB<[f64; 3]> {
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for &id in &self.vertices {
            let p = arena.position(id);
            for k in 0..3 {
                min[k] = min[k].min(p[k]);
                max[k] = max[k].max(p[k]);
            }
        }
        AABB::from_corners(min.map(|x| x - pad), max.map(|x| x + pad))
    }
}

// --- Indexed boolean ---------------------------------------------------------

#[derive(Clone, Copy)]
enum BoolOp {
    Union,
    Intersection,
    Difference,
}

/// An operand surface with its triangle index, and the pool bounds for the
/// parity probe's fast reject.
struct IndexedSurface<'a> {
    set: TriangleSet<'a>,
    tree: RTree<TriBox>,
    bounds: ([f64; 3], [f64; 3]),
}

impl<'a> IndexedSurface<'a> {
    fn new(data: &'a TriangularMesh3DData) -> Self {
        let set = TriangleSet::from_triangular_data(data);
        let tree = set.rtree();
        let bounds = pool_bounds(set.pool());
        IndexedSurface { set, tree, bounds }
    }
}

/// Where a fragment lies relative to the other operand's volume.
enum FragmentSide {
    Inside,
    Outside,
    /// On the other boundary; whether the two surface normals agree there.
    On {
        same_normal: bool,
    },
}

/// The boundary of the boolean `op` over the volumes bounded by `a` and `b`.
fn boolean(
    op: BoolOp,
    a: Vec<Polygon>,
    b: Vec<Polygon>,
    arena: &mut Arena,
    eps: f64,
) -> Vec<Polygon> {
    let a_data = welded(&a, arena);
    let b_data = welded(&b, arena);
    let a_surface = IndexedSurface::new(&a_data);
    let b_surface = IndexedSurface::new(&b_data);

    let mut out = Vec::new();

    for fragment in split_against(a, &b_surface, arena, eps) {
        let keep = match classify(&fragment, &b_surface, arena) {
            FragmentSide::Inside => matches!(op, BoolOp::Intersection),
            FragmentSide::Outside => {
                matches!(op, BoolOp::Union | BoolOp::Difference)
            }
            // A shared boundary patch survives once, taken from `a`: where the
            // normals agree the patch bounds the union and the intersection;
            // where they oppose the volumes sit on opposite sides, so the
            // patch is interior to the union but bounds the difference.
            FragmentSide::On { same_normal } => match op {
                BoolOp::Union | BoolOp::Intersection => same_normal,
                BoolOp::Difference => !same_normal,
            },
        };
        if keep {
            out.push(fragment);
        }
    }

    for mut fragment in split_against(b, &a_surface, arena, eps) {
        let keep = match classify(&fragment, &a_surface, arena) {
            FragmentSide::Inside => {
                matches!(op, BoolOp::Intersection | BoolOp::Difference)
            }
            FragmentSide::Outside => matches!(op, BoolOp::Union),
            // Shared patches were already kept from `a`.
            FragmentSide::On { .. } => false,
        };
        if keep {
            // A `b` boundary inside `a` bounds the difference facing the
            // removed material, so it flips.
            if matches!(op, BoolOp::Difference) {
                fragment.flip();
            }
            out.push(fragment);
        }
    }

    out
}

/// The polygons' triangles welded into one mesh, the queryable form of an
/// operand surface.
fn welded(polygons: &[Polygon], arena: &Arena) -> TriangularMesh3DData {
    TriangularMesh3DData::from_soup(
        polygons
            .iter()
            .flat_map(|p| p.triangles(arena))
            .flatten()
            .collect::<Vec<_>>(),
    )
}

/// Split each polygon by the planes of the other surface's nearby triangles.
/// A polygon whose (inflated) box meets no triangle box passes through whole;
/// afterwards no fragment crosses the other surface, because a crossing
/// triangle's box meets the fragment's box and its plane was applied.
fn split_against(
    polygons: Vec<Polygon>,
    other: &IndexedSurface<'_>,
    arena: &mut Arena,
    eps: f64,
) -> Vec<Polygon> {
    let mut out = Vec::new();
    let mut planes: Vec<Plane> = Vec::new();
    let mut seen: HashSet<[u64; 4]> = HashSet::new();
    for polygon in polygons {
        planes.clear();
        seen.clear();
        for tri_box in other
            .tree
            .locate_in_envelope_intersecting(&polygon.envelope(arena, eps))
        {
            let t = other.set.triangle(tri_box.idx as usize);
            let Some(plane) = Plane::from_points(t[0], t[1], t[2]) else {
                continue;
            };
            // One split per distinct plane; coplanar neighbours share it.
            let key = [
                plane.normal[0].to_bits(),
                plane.normal[1].to_bits(),
                plane.normal[2].to_bits(),
                plane.w.to_bits(),
            ];
            if seen.insert(key) {
                planes.push(plane);
            }
        }
        if planes.is_empty() {
            out.push(polygon);
            continue;
        }
        let mut fragments = vec![polygon];
        for plane in &planes {
            let mut parts = SplitParts::default();
            for fragment in &fragments {
                // Coplanar fragments pass through unsplit; front and back
                // fragments continue on to the remaining planes.
                plane.split_polygon(fragment, arena, eps, &mut parts);
            }
            fragments = parts.into_all();
        }
        out.extend(fragments);
    }
    out
}

/// Which side of `other`'s volume a fragment lies on, decided by its centroid:
/// exact ray-crossing parity for inside or outside, and for a centroid landing
/// exactly on the other surface, the orientation of the triangle it lands on.
fn classify(fragment: &Polygon, other: &IndexedSurface<'_>, arena: &Arena) -> FragmentSide {
    let centroid = fragment.centroid(arena);
    match shell_position_bounded(centroid, &other.set, Some(&other.tree), other.bounds) {
        CoordPos::Inside => FragmentSide::Inside,
        CoordPos::Outside => FragmentSide::Outside,
        CoordPos::OnBoundary => {
            let same_normal = other
                .tree
                .locate_in_envelope_intersecting(&AABB::from_point(centroid))
                .filter(|tri_box| {
                    point_in_triangle_3d(centroid, other.set.triangle(tri_box.idx as usize))
                })
                .find_map(|tri_box| {
                    let t = other.set.triangle(tri_box.idx as usize);
                    Plane::from_points(t[0], t[1], t[2])
                        .map(|p| dot3(p.normal, fragment.plane.normal) > 0.0)
                })
                .unwrap_or(true);
            FragmentSide::On { same_normal }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::EpsgCode;
    #[cfg(feature = "new-geometry")]
    /// An axis-aligned cube as a 12-triangle shell with outward winding.
    fn cube_shell(min: [f64; 3], max: [f64; 3]) -> TriangularMesh3DData {
        let corner = |i: usize| {
            [
                if i & 1 != 0 { max[0] } else { min[0] },
                if i & 2 != 0 { max[1] } else { min[1] },
                if i & 4 != 0 { max[2] } else { min[2] },
            ]
        };
        let quads: [[usize; 4]; 6] = [
            [0, 4, 6, 2],
            [1, 3, 7, 5],
            [0, 1, 5, 4],
            [2, 6, 7, 3],
            [0, 2, 3, 1],
            [4, 5, 7, 6],
        ];
        let soup = quads.into_iter().flat_map(|[a, b, c, d]| {
            [
                corner(a),
                corner(b),
                corner(c),
                corner(a),
                corner(c),
                corner(d),
            ]
        });
        TriangularMesh3DData::from_soup(soup)
    }

    fn cube(min: [f64; 3], max: [f64; 3]) -> Solid {
        Solid::from_exterior(CoordinateFrame::Euclidean, cube_shell(min, max))
    }

    /// One shell's signed volume (divergence theorem over signed tetrahedra);
    /// positive iff the winding is outward, so a void shell comes out negative.
    fn shell_volume(shell: &Shell) -> f64 {
        let Shell::TriangularMesh(mesh) = shell else {
            panic!("expected a triangulated shell");
        };
        let vertices = mesh.vertices();
        mesh.triangles()
            .map(|[a, b, c]| {
                let (a, b, c) = (
                    vertices[a as usize],
                    vertices[b as usize],
                    vertices[c as usize],
                );
                dot3(a, cross3(b, c)) / 6.0
            })
            .sum()
    }

    /// The volume the solid encloses: the exterior shell's minus its voids'.
    fn volume(solid: &Solid) -> f64 {
        shell_volume(solid.exterior()) + solid.interiors().iter().map(shell_volume).sum::<f64>()
    }

    #[test]
    fn booleans_of_overlapping_cubes_have_the_expected_volumes() {
        // Unit cube and a copy shifted by half along x: overlap volume 0.5.
        let a = cube([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = cube([0.5, 0.0, 0.0], [1.5, 1.0, 1.0]);

        let union = Csg::union(a.clone(), b.clone())
            .evaluate(1e-9)
            .unwrap()
            .unwrap();
        assert!((volume(&union) - 1.5).abs() < 1e-9);

        let intersection = Csg::intersection(a.clone(), b.clone())
            .evaluate(1e-9)
            .unwrap()
            .unwrap();
        assert!((volume(&intersection) - 0.5).abs() < 1e-9);

        let difference = Csg::difference(a, b).evaluate(1e-9).unwrap().unwrap();
        assert!((volume(&difference) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn the_result_pools_each_position_once() {
        let a = cube([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = cube([0.5, 0.25, 0.25], [1.5, 0.75, 0.75]);
        for csg in [
            Csg::union(a.clone(), b.clone()),
            Csg::intersection(a.clone(), b.clone()),
            Csg::difference(a, b),
        ] {
            let result = csg.evaluate(1e-9).unwrap().unwrap();
            let Shell::TriangularMesh(mesh) = result.exterior() else {
                panic!("expected a triangulated shell");
            };
            let pool = mesh.vertices();
            for (i, u) in pool.iter().enumerate() {
                for v in &pool[i + 1..] {
                    let apart = (0..3).map(|k| (u[k] - v[k]).abs()).fold(0.0, f64::max);
                    assert!(apart > 1e-9, "the pool carries {u:?} twice");
                }
            }
        }
    }

    #[test]
    fn a_disjoint_intersection_encloses_no_volume() {
        let a = cube([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = cube([5.0, 5.0, 5.0], [6.0, 6.0, 6.0]);
        assert!(Csg::intersection(a, b).evaluate(1e-9).unwrap().is_none());
    }

    #[test]
    fn a_nested_tree_evaluates_recursively() {
        // (a ∪ b) − b leaves exactly the part of a outside b.
        let a = cube([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = cube([0.5, 0.0, 0.0], [1.5, 1.0, 1.0]);
        let result = Csg::difference(Csg::union(a, b.clone()), b)
            .evaluate(1e-9)
            .unwrap()
            .unwrap();
        assert!((volume(&result) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn a_difference_with_a_contained_operand_hollows_the_result() {
        let a = cube([0.0; 3], [3.0; 3]);
        let b = cube([1.0; 3], [2.0; 3]);
        let result = Csg::difference(a, b).evaluate(1e-9).unwrap().unwrap();
        #[cfg(feature = "new-geometry")]
        assert_eq!(result.interiors().len(), 1, "the removed cube is a void");
        assert!((shell_volume(result.exterior()) - 27.0).abs() < 1e-9);
        assert!(
            (shell_volume(&result.interiors()[0]) + 1.0).abs() < 1e-9,
            "the void shell winds toward the cavity"
        );
        assert!((volume(&result) - 26.0).abs() < 1e-9);
    }

    #[test]
    fn a_difference_inside_the_right_operand_encloses_no_volume() {
        let a = cube([1.0; 3], [2.0; 3]);
        let b = cube([0.0; 3], [3.0; 3]);
        assert!(Csg::difference(a, b).evaluate(1e-9).unwrap().is_none());
    }

    #[test]
    fn a_difference_of_identical_cubes_encloses_no_volume() {
        let a = cube([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = cube([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(Csg::difference(a, b).evaluate(1e-9).unwrap().is_none());
    }

    #[test]
    fn a_union_of_face_touching_cubes_drops_the_interior_membrane() {
        // Two unit cubes sharing the x = 1 face: the shared face is interior
        // to the union, and the union enclosed volume is 2.
        let a = cube([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = cube([1.0, 0.0, 0.0], [2.0, 1.0, 1.0]);
        let union = Csg::union(a, b).evaluate(1e-9).unwrap().unwrap();
        #[cfg(feature = "new-geometry")]
        assert!((volume(&union) - 2.0).abs() < 1e-9);
        let Shell::TriangularMesh(mesh) = union.exterior() else {
            panic!("expected a triangulated shell");
        };
        let vertices = mesh.vertices();
        let interior = mesh.triangles().any(|t| {
            t.iter()
                .all(|&v| (vertices[v as usize][0] - 1.0).abs() < 1e-12)
        });
        assert!(!interior, "no boundary triangle lies in the shared plane");
    }

    /// The tolerance is a distance, so degrees cannot express it: a geographic
    /// operand is refused rather than merged by an amount that means a
    /// different distance at every latitude.
    #[test]
    fn operands_in_geographic_coordinates_are_an_error() {
        let frame = CoordinateFrame::Crs(EpsgCode::new(4326));
        let a = Solid::from_exterior(frame.clone(), cube_shell([0.0; 3], [1.0; 3]));
        let b = Solid::from_exterior(frame, cube_shell([0.5, 0.0, 0.0], [1.5, 1.0, 1.0]));
        let error = Csg::union(a, b).evaluate(1e-9).unwrap_err();
        assert!(
            error.to_string().contains("geographic coordinates"),
            "expected a geographic-coordinate refusal, got: {error}"
        );
    }

    #[test]
    fn operands_in_different_frames_are_an_error() {
        let a = cube([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = Solid::from_exterior(
            CoordinateFrame::Crs(EpsgCode::new(6677)),
            cube_shell([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
        );
        assert!(Csg::union(a, b).evaluate(1e-9).is_err());
    }
}
