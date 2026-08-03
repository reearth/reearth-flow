//! Boolean evaluation of a [`Csg`] tree.
//!
//! Evaluation resolves the unevaluated boolean tree into the concrete volume
//! it denotes, as a solid bounded by one triangulated exterior shell. The
//! boolean itself is a BSP-tree mesh boolean over the operands' triangulated
//! boundary shells.
//!
//! Operand constraints:
//!
//! - Every operand solid must be expressed in the same coordinate frame;
//!   mixed frames are an error.
//! - Every operand boundary must be closed and consistently oriented, with
//!   triangle winding counter-clockwise seen from outside the enclosed
//!   material (interior void shells therefore wind toward the void). An open
//!   or mis-wound boundary produces an arbitrary volume, not an error.
//! - Operand appearance does not propagate to the result.

use crate::coordinate::CoordinateFrame;
use crate::error::Error;
use crate::ops::triangulation::Cache;
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
        Ok(Some(Solid::from_exterior(frame, mesh)))
    }
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
    let polygons = match csg {
        Csg::Union(..) => bsp_union(left, right, eps),
        Csg::Intersection(..) => bsp_intersection(left, right, eps),
        Csg::Difference(..) => bsp_difference(left, right, eps),
    };
    Ok((polygons, left_frame))
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

/// The boundary polygons of a solid: every shell — exterior and voids —
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

// --- BSP boolean -------------------------------------------------------------
//
// The classic BSP CSG (the csg.js algorithm): each operand becomes a BSP tree
// of its boundary polygons; clipping one tree's polygons against the other
// removes the parts inside (or outside, after inversion) the other volume, and
// the surviving boundaries are recombined per operation.

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
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
        let n = cross(sub(b, a), sub(c, a));
        let len = dot(n, n).sqrt();
        if len < 1e-12 {
            return None;
        }
        let normal = [n[0] / len, n[1] / len, n[2] / len];
        Some(Plane {
            normal,
            w: dot(normal, a),
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
                let t = dot(self.normal, v) - self.w;
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
                if dot(self.normal, polygon.plane.normal) > 0.0 {
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
                        let t = (self.w - dot(self.normal, vi)) / dot(self.normal, sub(vj, vi));
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
}

/// One BSP node: a splitting plane, the polygons lying on it, and subtrees for
/// each side.
#[derive(Default)]
struct Node {
    plane: Option<Plane>,
    front: Option<Box<Node>>,
    back: Option<Box<Node>>,
    polygons: Vec<Polygon>,
}

impl Node {
    fn new(polygons: Vec<Polygon>, eps: f64) -> Node {
        let mut node = Node::default();
        node.build(polygons, eps);
        node
    }

    /// Swap the solid and empty half-spaces of the whole tree.
    fn invert(&mut self) {
        for polygon in &mut self.polygons {
            polygon.flip();
        }
        if let Some(plane) = &mut self.plane {
            plane.flip();
        }
        if let Some(front) = &mut self.front {
            front.invert();
        }
        if let Some(back) = &mut self.back {
            back.invert();
        }
        std::mem::swap(&mut self.front, &mut self.back);
    }

    /// The parts of `polygons` outside this tree's volume.
    fn clip_polygons(&self, polygons: Vec<Polygon>, eps: f64) -> Vec<Polygon> {
        let Some(plane) = &self.plane else {
            return polygons;
        };
        let mut front = Vec::new();
        let mut back = Vec::new();
        let mut coplanar_front = Vec::new();
        let mut coplanar_back = Vec::new();
        for polygon in &polygons {
            plane.split_polygon(
                polygon,
                eps,
                &mut coplanar_front,
                &mut coplanar_back,
                &mut front,
                &mut back,
            );
        }
        front.append(&mut coplanar_front);
        back.append(&mut coplanar_back);
        let mut front = match &self.front {
            Some(node) => node.clip_polygons(front, eps),
            None => front,
        };
        let back = match &self.back {
            Some(node) => node.clip_polygons(back, eps),
            None => Vec::new(),
        };
        front.extend(back);
        front
    }

    /// Remove the parts of this tree's polygons inside `bsp`'s volume.
    fn clip_to(&mut self, bsp: &Node, eps: f64) {
        self.polygons = bsp.clip_polygons(std::mem::take(&mut self.polygons), eps);
        if let Some(front) = &mut self.front {
            front.clip_to(bsp, eps);
        }
        if let Some(back) = &mut self.back {
            back.clip_to(bsp, eps);
        }
    }

    /// Every polygon in the tree.
    fn all_polygons(&self) -> Vec<Polygon> {
        let mut out = self.polygons.clone();
        if let Some(front) = &self.front {
            out.extend(front.all_polygons());
        }
        if let Some(back) = &self.back {
            out.extend(back.all_polygons());
        }
        out
    }

    /// Insert `polygons` into the tree, extending it where they span new space.
    fn build(&mut self, polygons: Vec<Polygon>, eps: f64) {
        if polygons.is_empty() {
            return;
        }
        if self.plane.is_none() {
            self.plane = Some(polygons[0].plane.clone());
        }
        let plane = self.plane.clone().expect("plane was just set");
        let mut front = Vec::new();
        let mut back = Vec::new();
        let mut coplanar_front = Vec::new();
        let mut coplanar_back = Vec::new();
        for polygon in &polygons {
            plane.split_polygon(
                polygon,
                eps,
                &mut coplanar_front,
                &mut coplanar_back,
                &mut front,
                &mut back,
            );
        }
        self.polygons.append(&mut coplanar_front);
        self.polygons.append(&mut coplanar_back);
        if !front.is_empty() {
            self.front.get_or_insert_default().build(front, eps);
        }
        if !back.is_empty() {
            self.back.get_or_insert_default().build(back, eps);
        }
    }
}

/// The boundary of `a ∪ b`.
fn bsp_union(a: Vec<Polygon>, b: Vec<Polygon>, eps: f64) -> Vec<Polygon> {
    let mut a = Node::new(a, eps);
    let mut b = Node::new(b, eps);
    a.clip_to(&b, eps);
    b.clip_to(&a, eps);
    b.invert();
    b.clip_to(&a, eps);
    b.invert();
    a.build(b.all_polygons(), eps);
    a.all_polygons()
}

/// The boundary of `a ∩ b`.
fn bsp_intersection(a: Vec<Polygon>, b: Vec<Polygon>, eps: f64) -> Vec<Polygon> {
    let mut a = Node::new(a, eps);
    let mut b = Node::new(b, eps);
    a.invert();
    b.clip_to(&a, eps);
    b.invert();
    a.clip_to(&b, eps);
    b.clip_to(&a, eps);
    a.build(b.all_polygons(), eps);
    a.invert();
    a.all_polygons()
}

/// The boundary of `a − b`.
fn bsp_difference(a: Vec<Polygon>, b: Vec<Polygon>, eps: f64) -> Vec<Polygon> {
    let mut a = Node::new(a, eps);
    let mut b = Node::new(b, eps);
    a.invert();
    a.clip_to(&b, eps);
    b.clip_to(&a, eps);
    b.invert();
    b.clip_to(&a, eps);
    b.invert();
    a.build(b.all_polygons(), eps);
    a.invert();
    a.all_polygons()
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

    /// The signed volume enclosed by the solid's exterior shell (divergence
    /// theorem over signed tetrahedra); positive iff the winding is outward.
    fn volume(solid: &Solid) -> f64 {
        let Shell::TriangularMesh(mesh) = solid.exterior() else {
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
                dot(a, cross(b, c)) / 6.0
            })
            .sum()
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
    fn operands_in_different_frames_are_an_error() {
        let a = cube([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = Solid::from_exterior(
            CoordinateFrame::Crs(EpsgCode::new(6677)),
            cube_shell([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
        );
        assert!(Csg::union(a, b).evaluate(1e-9).is_err());
    }
}
