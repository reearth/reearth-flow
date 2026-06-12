//! Coordless raw building blocks.
//!
//! These are NOT geometry leaves and carry no [`Coordinate`](super::Coordinate).
//! They are the reusable vertex/index data that composite leaves (`Solid`,
//! `Csg`) are built from. A standalone mesh leaf would be one of these plus a
//! single `coord`; the same buffer embedded inside a `Solid` stays coordless,
//! and the `Solid` owns the one frame. This split is what keeps a composite
//! from carrying several (possibly disagreeing) coords.

use super::ops::write_gltf::GltfBuffer;
use super::ops::Aabb;

/// An indexed mesh: positions plus a flat face-index buffer. Coordless.
#[derive(Clone, Debug)]
pub struct RawMesh {
    pub verts: Vec<[f64; 3]>,
    pub faces: Vec<u32>,
}

impl RawMesh {
    /// Fold every vertex into a running bounding box.
    pub fn extend_aabb(&self, acc: &mut Option<Aabb>) {
        for v in &self.verts {
            Aabb::extend(acc, *v);
        }
    }

    /// Translate every vertex in place (stand-in for a real reprojection).
    pub fn translate(&mut self, d: [f64; 3]) {
        for v in &mut self.verts {
            for (c, delta) in v.iter_mut().zip(d.iter()) {
                *c += delta;
            }
        }
    }

    /// Emit one primitive into the shared sink.
    pub fn emit_gltf(&self, out: &mut GltfBuffer) {
        out.bytes.extend((self.verts.len() as u32).to_le_bytes());
        out.prim_count += 1;
    }
}

/// Coordless boolean-operation tree for `Csg`. The frame lives on the owning
/// `Csg` leaf, not on the nodes.
#[derive(Clone, Debug)]
pub enum CsgNode {
    Leaf(RawMesh),
    Union(Box<CsgNode>, Box<CsgNode>),
    Intersection(Box<CsgNode>, Box<CsgNode>),
    Difference(Box<CsgNode>, Box<CsgNode>),
}

impl CsgNode {
    pub fn extend_aabb(&self, acc: &mut Option<Aabb>) {
        match self {
            CsgNode::Leaf(m) => m.extend_aabb(acc),
            CsgNode::Union(a, b) | CsgNode::Intersection(a, b) | CsgNode::Difference(a, b) => {
                a.extend_aabb(acc);
                b.extend_aabb(acc);
            }
        }
    }
}
