//! Shared earcut-based polygon triangulation primitives.
//!
//! `Polygon` and `PolygonMesh` both tessellate planar faces with earcut
//! (ear-clipping). A 3D face is first projected onto its best-fit plane
//! (`earcut::utils3d::project3d_to_2d`) and triangulated in 2D; because the
//! projection preserves vertex order, earcut's triangle indices map straight
//! back to the original vertices. Holes are passed to earcut as ring-start
//! offsets into the vertex list.
//!
//! Each face's vertices are the exterior ring followed by its hole rings, every
//! ring stored *open* (no closing duplicate) — earcut closes rings implicitly.

use earcut::{utils3d::project3d_to_2d, Earcut};

/// Triangulate one planar 2D face. `verts` is the exterior ring then any hole
/// rings (each open); `hole_indices` gives the start offset of each hole within
/// `verts`. Triangle corner indices into `verts` are written to `out` (flat, 3
/// per triangle). A face with fewer than three vertices yields none.
pub(crate) fn triangulate_2d(
    earcut: &mut Earcut<f64>,
    verts: &[[f64; 2]],
    hole_indices: &[u32],
    out: &mut Vec<u32>,
) {
    earcut.earcut(verts.iter().copied(), hole_indices, out);
}

/// Triangulate one planar 3D face by projecting it onto its best-fit plane,
/// which is fit from the first `num_outer` vertices (the exterior ring). `verts`
/// is exterior-then-holes (each open) and `hole_indices` the hole start offsets.
/// Triangle corner indices into `verts` are written to `out`. Returns `false`
/// (and clears `out`) when the exterior is too degenerate to define a plane.
/// `buf2d` is scratch space, reused across calls.
pub(crate) fn triangulate_3d(
    earcut: &mut Earcut<f64>,
    verts: &[[f64; 3]],
    num_outer: usize,
    hole_indices: &[u32],
    buf2d: &mut Vec<[f64; 2]>,
    out: &mut Vec<u32>,
) -> bool {
    if num_outer < 3 || !project3d_to_2d(verts, num_outer, buf2d) {
        out.clear();
        return false;
    }
    earcut.earcut(buf2d.iter().copied(), hole_indices, out);
    true
}
