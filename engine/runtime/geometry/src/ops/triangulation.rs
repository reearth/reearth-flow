//! Shared earcut-based polygon triangulation primitives, plus the reusable
//! [`Cache`] that lets repeated triangulations amortize their allocations.

use earcut::Earcut;
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};

use crate::appearance::{Appearance, FaceBinding, ThemeBinding, UvSet, UvSource};

/// Reusable scratch for triangulation, threaded through
/// [`Triangulate`](crate::ops::Triangulate) so a caller tessellating many
/// features pays the allocation cost (earcut's node arenas and the index/vertex
/// scratch) once instead of per call. Construct one with [`Cache::new`] and pass
/// `&mut` it to every `triangulate` call.
#[derive(Default)]
pub struct Cache {
    pub(crate) earcut: Earcut<f64>,
    pub(crate) buffers: Buffers,
}

impl Cache {
    /// A fresh, empty cache.
    pub fn new() -> Self {
        Self::default()
    }
}

/// A triangulated mesh, plus each source polygon's flat normal and triangle
/// count, both in polygon order.
pub struct Triangulated<M> {
    pub mesh: M,
    pub polygon_normals: Vec<[f64; 3]>,
    pub polygon_tris: Vec<u32>,
}

/// The scratch buffers [`Cache`] reuses across calls; each is cleared before use.
#[derive(Default)]
pub(crate) struct Buffers {
    /// Polygon ring vertex positions into the source `coords`.
    pub(crate) positions: Vec<u32>,
    /// Hole-ring start offsets (polygon: into the ring list; mesh: into `open_src`).
    pub(crate) holes: Vec<u32>,
    /// Face-local positions of a mesh face's open-ring corners, closing duplicates
    /// dropped; parallel to the gathered `verts2` / `verts3`.
    pub(crate) open_src: Vec<u32>,
    /// One earcut output (one polygon, or one mesh face).
    pub(crate) out: Vec<u32>,
    /// Accumulated global triangle indices (mesh).
    pub(crate) tris: Vec<u32>,
    /// Per-output-corner source `face_indices` position, parallel to `tris`
    /// (mesh); the index [`retarget_uv`] re-gathers each corner's UV from.
    pub(crate) corner_src: Vec<u32>,
    /// Triangle count per source face, in face order (mesh); the per-face counts
    /// [`expand_appearance`] repeats each `PerFace` binding entry by, and a
    /// caller wanting a flat normal per output triangle repeats `face_normals`
    /// by the same counts.
    pub(crate) face_tris: Vec<u32>,
    /// Exterior ring normal per source face, in face order (mesh); `[0.0, 0.0,
    /// 1.0]` for a degenerate face, harmless since its `face_tris` count is 0.
    pub(crate) face_normals: Vec<[f64; 3]>,
    /// Decoded CSR buffers (mesh).
    pub(crate) face_indices: Vec<u32>,
    pub(crate) face_offsets: Vec<u32>,
    pub(crate) interior_offsets: Vec<u32>,
    /// Per-face gathered vertices (mesh).
    pub(crate) verts2: Vec<[f64; 2]>,
    pub(crate) verts3: Vec<[f64; 3]>,
}

/// Triangulate one planar 2D face. `verts` is the exterior ring then any hole
/// rings (each open); `holes` gives the start offset of each hole within
/// `verts`. Triangle corner indices into `verts` are written to `out`.
pub(crate) fn triangulate_2d(
    earcut: &mut Earcut<f64>,
    verts: &[[f64; 2]],
    holes: &[u32],
    out: &mut Vec<u32>,
) {
    earcut.earcut(verts.iter().copied(), holes, out);
}

/// Triangulate one planar 3D face by projecting it onto its best-fit plane
/// (fit from the first `num_outer` vertices, the exterior ring) and feeding the
/// projected points straight to earcut as an iterator — no intermediate buffer.
/// Returns the exterior ring's Newell's-method normal (already computed while
/// fitting the projection plane, and shared by every triangle the face is
/// split into), or `None` (with `out` cleared) when the exterior cannot define
/// a plane.
pub(crate) fn triangulate_3d(
    earcut: &mut Earcut<f64>,
    verts: &[[f64; 3]],
    num_outer: usize,
    holes: &[u32],
    out: &mut Vec<u32>,
) -> Option<[f64; 3]> {
    if num_outer < 3 {
        out.clear();
        return None;
    }
    let Some((projector, normal)) = Projector::fit(&verts[..num_outer]) else {
        out.clear();
        return None;
    };
    earcut.earcut(verts.iter().map(|&v| projector.project(v)), holes, out);
    Some(normal)
}

/// Triangulate one planar 3D face with a constrained Delaunay triangulation
/// (spade) instead of earcut. Same contract as [`triangulate_3d`], but the
/// interior is filled with a proper Delaunay mesh, so a collinear-subdivided
/// edge (a T-junction) does not produce the near-degenerate sliver triangles
/// earcut emits, which the geometric predicates would otherwise read as spurious
/// intersections. A pathological face whose boundary cannot be constrained (e.g.
/// a self-intersecting ring, already reported by the ring-level check) falls back
/// to earcut.
pub(crate) fn triangulate_3d_cdt(
    earcut: &mut Earcut<f64>,
    verts: &[[f64; 3]],
    num_outer: usize,
    holes: &[u32],
    out: &mut Vec<u32>,
) -> Option<[f64; 3]> {
    if num_outer < 3 {
        out.clear();
        return None;
    }
    let Some((projector, normal)) = Projector::fit(&verts[..num_outer]) else {
        out.clear();
        return None;
    };
    let pts: Vec<[f64; 2]> = verts.iter().map(|&v| projector.project(v)).collect();
    if !cdt_fill(&pts, num_outer, holes, out) {
        earcut.earcut(pts.iter().copied(), holes, out);
    }
    Some(normal)
}

/// Fill a projected planar polygon (exterior `pts[0..num_outer]`, then hole rings
/// delimited by `holes`) with a constrained Delaunay triangulation, writing the
/// interior triangles' corner indices (into `pts`) to `out`. Returns `false`,
/// leaving `out` untouched, when the boundary cannot be constrained (a non-simple
/// ring), so the caller can fall back.
fn cdt_fill(pts: &[[f64; 2]], num_outer: usize, holes: &[u32], out: &mut Vec<u32>) -> bool {
    let mut cdt: ConstrainedDelaunayTriangulation<Point2<f64>> =
        ConstrainedDelaunayTriangulation::new();
    let mut handles = Vec::with_capacity(pts.len());
    for &[x, y] in pts {
        match cdt.insert(Point2::new(x, y)) {
            Ok(h) => handles.push(h),
            Err(_) => return false,
        }
    }
    let ring = |k: usize| -> (usize, usize) {
        if k == 0 {
            (0, num_outer)
        } else {
            let start = holes[k - 1] as usize;
            let end = holes.get(k).copied().unwrap_or(pts.len() as u32) as usize;
            (start, end)
        }
    };
    for k in 0..=holes.len() {
        let (start, end) = ring(k);
        let n = end.saturating_sub(start);
        if n < 2 {
            continue;
        }
        for i in 0..n {
            let a = handles[start + i];
            let b = handles[start + (i + 1) % n];
            if a == b {
                continue;
            }
            if !cdt.can_add_constraint(a, b) {
                return false;
            }
            cdt.add_constraint(a, b);
        }
    }
    // spade vertex storage index -> polygon vertex index.
    let mut mine = vec![0u32; handles.len()];
    for (i, h) in handles.iter().enumerate() {
        mine[h.index()] = i as u32;
    }
    out.clear();
    for face in cdt.inner_faces() {
        let vs = face.vertices();
        let p = [vs[0].position(), vs[1].position(), vs[2].position()];
        let centroid = [
            (p[0].x + p[1].x + p[2].x) / 3.0,
            (p[0].y + p[1].y + p[2].y) / 3.0,
        ];
        if point_in_polygon(centroid, pts, num_outer, holes) {
            for v in vs {
                out.push(mine[v.fix().index()]);
            }
        }
    }
    true
}

/// Even-odd point-in-polygon-with-holes test on the projected rings: inside the
/// exterior ring and outside every hole.
fn point_in_polygon(p: [f64; 2], pts: &[[f64; 2]], num_outer: usize, holes: &[u32]) -> bool {
    let in_ring = |start: usize, end: usize| -> bool {
        let n = end - start;
        if n < 3 {
            return false;
        }
        let mut inside = false;
        let mut j = n - 1;
        for i in 0..n {
            let a = pts[start + i];
            let b = pts[start + j];
            if (a[1] > p[1]) != (b[1] > p[1])
                && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
            {
                inside = !inside;
            }
            j = i;
        }
        inside
    };
    if !in_ring(0, num_outer) {
        return false;
    }
    for k in 0..holes.len() {
        let start = holes[k] as usize;
        let end = holes.get(k + 1).copied().unwrap_or(pts.len() as u32) as usize;
        if in_ring(start, end) {
            return false;
        }
    }
    true
}

/// Expand a source geometry's appearance onto its triangulated mesh, consuming it.
/// `face_tris[i]` is the triangle count of source face `i`; `src_corner[j]` is the
/// source corner-buffer position output triangle-corner `j` draws from. Per-face
/// bindings are expanded (see [`expand_binding`]) and each theme's UV sets
/// re-targeted onto the triangulated corner buffer (see [`retarget_uv`]); palette
/// and themes are otherwise unchanged.
pub(crate) fn expand_appearance(
    appearance: Option<Appearance>,
    face_tris: &[u32],
    src_corner: &[u32],
) -> Option<Appearance> {
    appearance.map(|app| {
        let (materials, themes, default_theme) = app.into_parts();
        let themes = themes
            .into_iter()
            .map(|theme| ThemeBinding {
                theme: theme.theme,
                front: expand_binding(theme.front, face_tris),
                back: theme.back.map(|back| expand_binding(back, face_tris)),
                uv_sets: theme
                    .uv_sets
                    .into_iter()
                    .map(|uv| retarget_uv(uv, src_corner))
                    .collect(),
            })
            .collect();
        Appearance::from_parts(materials, themes, default_theme)
    })
}

/// Expand one source-face binding to one entry per output triangle. `Uniform` is
/// unchanged; `PerFace` repeats each source face's entry `face_tris[i]` times.
fn expand_binding(binding: FaceBinding, face_tris: &[u32]) -> FaceBinding {
    match binding {
        FaceBinding::Uniform(index) => FaceBinding::Uniform(index),
        FaceBinding::PerFace(faces) => {
            debug_assert_eq!(faces.len(), face_tris.len());
            let total = face_tris.iter().map(|&c| c as usize).sum();
            let mut per_triangle = Vec::with_capacity(total);
            for (material, &count) in faces.into_iter().zip(face_tris) {
                per_triangle.extend(std::iter::repeat_n(material, count as usize));
            }
            FaceBinding::PerFace(per_triangle)
        }
    }
}

/// Re-target one source UV set onto a triangulated corner buffer, consuming it.
///
/// `src_corner[j]` is the source corner-buffer position output triangle-corner
/// `j` draws its UV from (`positions[out[j]]` for a `Polygon`, `start + l` for a
/// `PolygonMesh` face. An `Explicit` set is re-gathered into a fresh
/// `3 * triangle_count`-long array; a `WorldToTexture` matrix is *positional*,
/// so it moves over verbatim (triangulation preserves world positions).
/// Only the `uv` payload changes; `side` / `channel` carry through.
pub(crate) fn retarget_uv(uv: UvSet, src_corner: &[u32]) -> UvSet {
    let mapped = match uv.uv {
        UvSource::Explicit(coords) => {
            UvSource::Explicit(src_corner.iter().map(|&i| coords[i as usize]).collect())
        }
        matrix @ UvSource::WorldToTexture(_) => matrix,
    };
    UvSet { uv: mapped, ..uv }
}

/// Tolerance for triangulation related algorithms.
const EPSILON: f64 = 1e-10;

/// Maps a planar 3D ring onto the xy-plane. Ported from
/// `earcut::utils3d::project3d_to_2d` so the projection can be applied lazily
/// (per point) rather than collected into a `Vec`.
enum Projector {
    /// Plane already aligned with `+z`: keep `(x, y)`.
    KeepXy,
    /// Aligned with `-z`: swap to `(y, x)`.
    FlipXy,
    /// General orientation: rotate onto the xy-plane.
    Rotate {
        m11: f64,
        m12: f64,
        m13: f64,
        m21: f64,
        m22: f64,
        m23: f64,
    },
}

impl Projector {
    /// Fit from the exterior ring `outer`; `None` if it has no usable normal.
    /// Also returns that normal, since it's already computed here and a
    /// caller triangulating a planar face wants it too.
    fn fit(outer: &[[f64; 3]]) -> Option<(Self, [f64; 3])> {
        let n @ [nx, ny, nz] = normal(outer)?;
        let dd = (nx * nx + ny * ny).sqrt();
        let projector = if dd < EPSILON {
            if nz > 0.0 {
                Projector::KeepXy
            } else {
                Projector::FlipXy
            }
        } else {
            let ax = -ny / dd;
            let ay = nx / dd;
            let theta = nz.acos();
            let (sint, cost) = (theta.sin(), theta.cos());
            let s = ax * ay * (1.0 - cost);
            Projector::Rotate {
                m11: ax * ax * (1.0 - cost) + cost,
                m12: s,
                m13: -(ay * sint),
                m21: s,
                m22: ay * ay * (1.0 - cost) + cost,
                m23: ax * sint,
            }
        };
        Some((projector, n))
    }

    #[inline]
    fn project(&self, [x, y, z]: [f64; 3]) -> [f64; 2] {
        match *self {
            Projector::KeepXy => [x, y],
            Projector::FlipXy => [y, x],
            Projector::Rotate {
                m11,
                m12,
                m13,
                m21,
                m22,
                m23,
            } => [x * m11 + y * m12 + z * m13, x * m21 + y * m22 + z * m23],
        }
    }
}

/// Newell's-method unit normal of a planar ring, following the winding by the
/// right-hand rule; `None` if degenerate (fewer than three vertices, or a
/// near-zero normal). Ported from `earcut::utils3d`.
pub(crate) fn normal(vertices: &[[f64; 3]]) -> Option<[f64; 3]> {
    let (&last, _) = vertices.split_last()?;
    if vertices.len() < 3 {
        return None;
    }
    let mut sum = [0.0f64; 3];
    let mut prev = last;
    for &[x, y, z] in vertices {
        // sum += (prev - p) x (prev + p)
        let a = [prev[0] - x, prev[1] - y, prev[2] - z];
        let b = [prev[0] + x, prev[1] + y, prev[2] + z];
        sum[0] += a[1] * b[2] - a[2] * b[1];
        sum[1] += a[2] * b[0] - a[0] * b[2];
        sum[2] += a[0] * b[1] - a[1] * b[0];
        prev = [x, y, z];
    }
    let d = (sum[0] * sum[0] + sum[1] * sum[1] + sum[2] * sum[2]).sqrt();
    if d < EPSILON {
        return None;
    }
    Some([sum[0] / d, sum[1] / d, sum[2] / d])
}

#[cfg(test)]
mod cdt_tests {
    use super::*;

    fn tri_area(c: [[f64; 3]; 3]) -> f64 {
        let u = [c[1][0] - c[0][0], c[1][1] - c[0][1], c[1][2] - c[0][2]];
        let v = [c[2][0] - c[0][0], c[2][1] - c[0][1], c[2][2] - c[0][2]];
        let cr = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        0.5 * (cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2]).sqrt()
    }

    fn total_area(verts: &[[f64; 3]], out: &[u32]) -> f64 {
        out.chunks_exact(3)
            .map(|c| {
                tri_area([
                    verts[c[0] as usize],
                    verts[c[1] as usize],
                    verts[c[2] as usize],
                ])
            })
            .sum()
    }

    #[test]
    fn cdt_avoids_sliver_on_collinear_subdivided_edge() {
        // A triangle whose base (v0 -> v3) is subdivided at v1, v2 (collinear): a
        // pentagon that is geometrically a triangle. earcut emits a near-zero-area
        // sliver among v1, v2, v3; the CDT must not. Area = 0.5 * 3 * 2 = 3.
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [1.5, 2.0, 0.0],
        ];
        let mut earcut = Earcut::<f64>::default();
        let mut out = Vec::new();
        assert!(triangulate_3d_cdt(&mut earcut, &verts, 5, &[], &mut out).is_some());
        assert!(!out.is_empty());
        for c in out.chunks_exact(3) {
            let a = tri_area([
                verts[c[0] as usize],
                verts[c[1] as usize],
                verts[c[2] as usize],
            ]);
            assert!(a > 1e-6, "CDT emitted a sliver triangle (area {a})");
        }
        assert!((total_area(&verts, &out) - 3.0).abs() < 1e-9);
    }

    #[test]
    fn cdt_fills_a_face_with_a_hole() {
        // 4x4 square with a 2x2 hole: filled area = 16 - 4 = 12.
        let verts = vec![
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 4.0, 0.0],
            [0.0, 4.0, 0.0],
            [1.0, 1.0, 0.0],
            [3.0, 1.0, 0.0],
            [3.0, 3.0, 0.0],
            [1.0, 3.0, 0.0],
        ];
        let mut earcut = Earcut::<f64>::default();
        let mut out = Vec::new();
        assert!(triangulate_3d_cdt(&mut earcut, &verts, 4, &[4], &mut out).is_some());
        assert!((total_area(&verts, &out) - 12.0).abs() < 1e-9);
    }
}
