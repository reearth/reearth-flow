//! Boolean evaluation of a [`Csg`] tree into a [`Solid`] bounded by one
//! triangulated exterior shell.
//!
//! Each boolean indexes both boundary surfaces ([`TriangleSet::rtree`]),
//! splits only the polygons near the other surface by the planes of the
//! nearby triangles (see [`split_against`]), and classifies each fragment by
//! its centroid's exact ray-crossing parity (see [`classify`]); the operation
//! keeps the matching fragments.
//!
//! Operands must share one coordinate frame (mixed frames are an error) and
//! be closed, consistently outward-wound boundaries; an open or mis-wound
//! boundary produces an arbitrary volume, not an error. Appearance does not
//! propagate to the result.

use std::collections::{HashMap, HashSet};

use rstar::{RTree, AABB};

use crate::coordinate::CoordinateFrame;
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

impl Csg {
    /// Evaluate the boolean tree into the solid volume it denotes, as a solid
    /// bounded by a single triangulated exterior shell, or `None` when the
    /// result encloses no volume.
    ///
    /// `tolerance` is the distance within which a vertex counts as lying on a
    /// cutting plane; a value at or below zero falls back to a small default.
    pub fn evaluate(&self, tolerance: f64) -> Result<Option<Solid>, Error> {
        let eps = if tolerance > 0.0 { tolerance } else { 1e-9 };
        let mut cache = Cache::default();
        let (polygons, frame) = evaluate_tree(self, eps, &mut cache)?;

        let soup = polygons.iter().flat_map(|p| p.fan()).flatten();
        let mesh = TriangularMesh3DData::from_soup(soup);
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

/// Separate the evaluated boundary into the exterior mesh and interior void
/// shells: an edge-connected component that is closed and encloses a
/// canonically negative volume is a void; everything else stays exterior.
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
    for component in components.into_values() {
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

/// The component's right-hand-rule signed volume (divergence theorem over
/// signed tetrahedra).
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
) -> Result<(Vec<Polygon>, CoordinateFrame), Error> {
    let (left, right) = match csg {
        Csg::Union(l, r) | Csg::Intersection(l, r) | Csg::Difference(l, r) => (l, r),
    };
    let (left, left_frame) = evaluate_operand(left, eps, cache)?;
    let (right, right_frame) = evaluate_operand(right, eps, cache)?;
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
    Ok((boolean(op, left, right, eps), left_frame))
}

/// Evaluate one operand into its boundary polygons and its frame.
fn evaluate_operand(
    operand: &ThreeDimensional,
    eps: f64,
    cache: &mut Cache,
) -> Result<(Vec<Polygon>, CoordinateFrame), Error> {
    match operand {
        ThreeDimensional::Solid(solid) => Ok((solid_polygons(solid, cache), solid.frame().clone())),
        ThreeDimensional::Csg(csg) => evaluate_tree(csg, eps, cache),
    }
}

/// The boundary polygons of a solid: every shell, exterior and voids,
/// triangulated, with degenerate triangles dropped.
fn solid_polygons(solid: &Solid, cache: &mut Cache) -> Vec<Polygon> {
    let mut polygons = Vec::new();
    shell_polygons(solid.exterior(), cache, &mut polygons);
    for shell in solid.interiors() {
        shell_polygons(shell, cache, &mut polygons);
    }
    polygons
}

/// Append one shell's faces as triangle polygons.
fn shell_polygons(shell: &Shell, cache: &mut Cache, out: &mut Vec<Polygon>) {
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
                vertices: vec![a, b, c],
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

/// A vertex's side of a plane, and (bitwise-or'ed) a polygon's.
const COPLANAR: u8 = 0;
const FRONT: u8 = 1;
const BACK: u8 = 2;
const SPANNING: u8 = 3;

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

    /// Classify `polygon` against this plane, pushing it to the matching output
    /// and splitting it when it spans both sides.
    fn split_polygon(
        &self,
        polygon: &Polygon,
        eps: f64,
        coplanar_front: &mut Vec<Polygon>,
        coplanar_back: &mut Vec<Polygon>,
        front: &mut Vec<Polygon>,
        back: &mut Vec<Polygon>,
    ) {
        let mut polygon_type = COPLANAR;
        let types: Vec<u8> = polygon
            .vertices
            .iter()
            .map(|&v| {
                let t = dot3(self.normal, v) - self.w;
                let ty = if t < -eps {
                    BACK
                } else if t > eps {
                    FRONT
                } else {
                    COPLANAR
                };
                polygon_type |= ty;
                ty
            })
            .collect();

        match polygon_type {
            COPLANAR => {
                if dot3(self.normal, polygon.plane.normal) > 0.0 {
                    coplanar_front.push(polygon.clone());
                } else {
                    coplanar_back.push(polygon.clone());
                }
            }
            FRONT => front.push(polygon.clone()),
            BACK => back.push(polygon.clone()),
            _ => {
                let mut f: Vec<[f64; 3]> = Vec::new();
                let mut b: Vec<[f64; 3]> = Vec::new();
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
                        let t = (self.w - dot3(self.normal, vi)) / dot3(self.normal, sub3(vj, vi));
                        let v = lerp(vi, vj, t);
                        f.push(v);
                        b.push(v);
                    }
                }
                if f.len() >= 3 {
                    front.push(Polygon {
                        vertices: f,
                        plane: polygon.plane.clone(),
                    });
                }
                if b.len() >= 3 {
                    back.push(Polygon {
                        vertices: b,
                        plane: polygon.plane.clone(),
                    });
                }
            }
        }
    }
}

/// A planar convex boundary polygon; splitting a triangle keeps its fragments
/// convex, so convexity is an invariant.
#[derive(Clone)]
struct Polygon {
    vertices: Vec<[f64; 3]>,
    plane: Plane,
}

impl Polygon {
    fn flip(&mut self) {
        self.vertices.reverse();
        self.plane.flip();
    }

    /// The polygon as a fan of triangles, each as its three corners.
    fn fan(&self) -> impl Iterator<Item = [[f64; 3]; 3]> + '_ {
        let v = &self.vertices;
        (1..v.len().saturating_sub(1)).map(move |i| [v[0], v[i], v[i + 1]])
    }

    /// The vertex average: strictly interior for a convex polygon, on its
    /// plane, so it stands in for the whole fragment once the fragment cannot
    /// cross the other surface.
    fn centroid(&self) -> [f64; 3] {
        let mut c = [0.0f64; 3];
        for v in &self.vertices {
            for k in 0..3 {
                c[k] += v[k];
            }
        }
        let n = self.vertices.len() as f64;
        c.map(|x| x / n)
    }

    /// The polygon's box, inflated by `pad` on every side.
    fn envelope(&self, pad: f64) -> AABB<[f64; 3]> {
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for v in &self.vertices {
            for k in 0..3 {
                min[k] = min[k].min(v[k]);
                max[k] = max[k].max(v[k]);
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

/// One operand surface with its triangle index: the welded triangle soup, the
/// rstar tree over its per-triangle boxes, and the pool bounds for the parity
/// probe's fast reject.
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
fn boolean(op: BoolOp, a: Vec<Polygon>, b: Vec<Polygon>, eps: f64) -> Vec<Polygon> {
    let a_data = welded(&a);
    let b_data = welded(&b);
    let a_surface = IndexedSurface::new(&a_data);
    let b_surface = IndexedSurface::new(&b_data);

    let mut out = Vec::new();

    for fragment in split_against(a, &b_surface, eps) {
        let keep = match classify(&fragment, &b_surface) {
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

    for mut fragment in split_against(b, &a_surface, eps) {
        let keep = match classify(&fragment, &a_surface) {
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
fn welded(polygons: &[Polygon]) -> TriangularMesh3DData {
    TriangularMesh3DData::from_soup(polygons.iter().flat_map(|p| p.fan()).flatten())
}

/// Split each polygon by the planes of the other surface's nearby triangles.
/// A polygon whose (inflated) box meets no triangle box passes through whole;
/// afterwards no fragment crosses the other surface, because a crossing
/// triangle's box meets the fragment's box and its plane was applied.
fn split_against(polygons: Vec<Polygon>, other: &IndexedSurface<'_>, eps: f64) -> Vec<Polygon> {
    let mut out = Vec::new();
    let mut planes: Vec<Plane> = Vec::new();
    let mut seen: HashSet<[u64; 4]> = HashSet::new();
    for polygon in polygons {
        planes.clear();
        seen.clear();
        for tri_box in other
            .tree
            .locate_in_envelope_intersecting(&polygon.envelope(eps))
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
            let mut coplanar_front = Vec::new();
            let mut coplanar_back = Vec::new();
            let mut front = Vec::new();
            let mut back = Vec::new();
            for fragment in &fragments {
                // Coplanar fragments pass through unsplit; front and back
                // fragments continue on to the remaining planes.
                plane.split_polygon(
                    fragment,
                    eps,
                    &mut coplanar_front,
                    &mut coplanar_back,
                    &mut front,
                    &mut back,
                );
            }
            front.append(&mut back);
            front.append(&mut coplanar_front);
            front.append(&mut coplanar_back);
            fragments = front;
        }
        out.extend(fragments);
    }
    out
}

/// Which side of `other`'s volume a fragment lies on, decided by its centroid:
/// exact ray-crossing parity for inside or outside, and for a centroid landing
/// exactly on the other surface, the orientation of the triangle it lands on.
fn classify(fragment: &Polygon, other: &IndexedSurface<'_>) -> FragmentSide {
    let centroid = fragment.centroid();
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

