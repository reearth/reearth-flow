//! Shared fixtures for the 3D predicate tests.

use crate::coordinate::CoordinateFrame;
use crate::solid::{Shell, Solid};
use crate::triangular_mesh::TriangularMesh3DData;
use crate::{Euclidean3DGeometry, Geometry};

pub(crate) fn e() -> CoordinateFrame {
    CoordinateFrame::Euclidean
}

pub(crate) fn g3(g: Euclidean3DGeometry) -> Geometry {
    Geometry::Euclidean3D(g)
}

pub(crate) fn solid_geometry(solid: Solid) -> Euclidean3DGeometry {
    Euclidean3DGeometry::Solid(Box::new(solid))
}

/// A closed triangle shell over the axis-aligned box `[min, min + size]`:
/// 8 corners, 12 triangles.
pub(crate) fn box_shell(min: [f64; 3], size: [f64; 3]) -> TriangularMesh3DData {
    let corners: Vec<[f64; 3]> = (0..8u32)
        .map(|i| {
            [
                min[0] + if i & 1 != 0 { size[0] } else { 0.0 },
                min[1] + if i & 2 != 0 { size[1] } else { 0.0 },
                min[2] + if i & 4 != 0 { size[2] } else { 0.0 },
            ]
        })
        .collect();
    #[rustfmt::skip]
    const TRIS: [u32; 36] = [
        0, 1, 3,  0, 3, 2, // z = min
        4, 7, 5,  4, 6, 7, // z = max
        0, 4, 5,  0, 5, 1, // y = min
        2, 3, 7,  2, 7, 6, // y = max
        0, 2, 6,  0, 6, 4, // x = min
        1, 5, 7,  1, 7, 3, // x = max
    ];
    TriangularMesh3DData::from_parts(corners, TRIS).unwrap()
}

/// A solid axis-aligned box.
pub(crate) fn box_solid(min: [f64; 3], size: [f64; 3]) -> Solid {
    Solid::from_exterior(e(), Shell::TriangularMesh(box_shell(min, size)))
}

/// A solid box with a hollow box-shaped void.
pub(crate) fn box_solid_with_void(
    min: [f64; 3],
    size: [f64; 3],
    void_min: [f64; 3],
    void_size: [f64; 3],
) -> Solid {
    Solid::new(
        e(),
        Shell::TriangularMesh(box_shell(min, size)),
        vec![Shell::TriangularMesh(box_shell(void_min, void_size))],
    )
}

/// A closed triangle shell over the unit-scaled box `[0, w]^3`, each of the six
/// faces subdivided into a `w x w` grid: `12 * w^2` triangles. Seam vertices are
/// duplicated per face, which is immaterial to ray-crossing parity. For sizing
/// the point-in-solid benchmark.
#[cfg(test)]
pub(crate) fn subdivided_box_shell(w: usize) -> TriangularMesh3DData {
    let s = w as f64;
    // (origin, du, dv) per face; winding is irrelevant to parity.
    let faces: [([f64; 3], [f64; 3], [f64; 3]); 6] = [
        ([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]),
        ([0.0, 0.0, s], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        ([0.0, s, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
        ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),
        ([s, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]),
    ];
    let mut verts: Vec<[f64; 3]> = Vec::new();
    let mut tris: Vec<u32> = Vec::new();
    let stride = (w + 1) as u32;
    for (origin, du, dv) in faces {
        let base = verts.len() as u32;
        for i in 0..=w {
            for j in 0..=w {
                let (fi, fj) = (i as f64, j as f64);
                verts.push([
                    origin[0] + du[0] * fi + dv[0] * fj,
                    origin[1] + du[1] * fi + dv[1] * fj,
                    origin[2] + du[2] * fi + dv[2] * fj,
                ]);
            }
        }
        let at = |i: usize, j: usize| base + i as u32 * stride + j as u32;
        for i in 0..w {
            for j in 0..w {
                tris.extend([
                    at(i, j),
                    at(i + 1, j),
                    at(i + 1, j + 1),
                    at(i, j),
                    at(i + 1, j + 1),
                    at(i, j + 1),
                ]);
            }
        }
    }
    TriangularMesh3DData::from_parts(verts, tris).unwrap()
}

/// A solid over [`subdivided_box_shell`].
#[cfg(test)]
pub(crate) fn subdivided_box_solid(w: usize) -> Solid {
    Solid::from_exterior(e(), Shell::TriangularMesh(subdivided_box_shell(w)))
}

/// A solid tetrahedron over the four vertices.
pub(crate) fn tetra_solid(v: [[f64; 3]; 4]) -> Solid {
    let shell =
        TriangularMesh3DData::from_parts(v.to_vec(), [0u32, 1, 2, 0, 1, 3, 0, 2, 3, 1, 2, 3])
            .unwrap();
    Solid::from_exterior(e(), Shell::TriangularMesh(shell))
}

/// A tiny deterministic splitmix64 generator for property tests.
pub(crate) struct Rng(pub u64);

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A uniform integer in `[lo, hi]` (inclusive).
    pub fn int(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next_u64() % (hi - lo + 1) as u64) as i64
    }

    /// A point with integer coordinates in `[lo, hi]^3`.
    pub fn grid_point(&mut self, lo: i64, hi: i64) -> [f64; 3] {
        [
            self.int(lo, hi) as f64,
            self.int(lo, hi) as f64,
            self.int(lo, hi) as f64,
        ]
    }
}
