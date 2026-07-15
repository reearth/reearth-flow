//! Flattened 3D leaf views and triangle-set extraction.
//!
//! The 3D counterpart of [`view`](super::view): [`Leaf3D`] is the
//! collection-free normal form the 3D operations dispatch over, and the
//! crate-internal `TriangleSet` re-expresses every surface-bearing leaf as one flat triangle
//! soup — a `TriangularMesh` verbatim, a `Polygon` / `PolygonMesh` / `Solid`
//! shell through the same per-face earcut used by
//! [`Triangulate`](crate::ops::Triangulate), but **borrowing** the source
//! buffers instead of consuming them.
//!
//! For planar faces the triangulation is an exact re-representation: earcut
//! only connects existing vertices, so the union of the triangles is the face.
//! Non-planar faces are triangulated through their best-fit plane (the same
//! approximation `Triangulate` makes), and a degenerate face — one whose
//! exterior cannot define a plane — contributes no triangles at all, exactly
//! as in `Triangulate`.

use std::borrow::Cow;

use crate::coordinate::CoordinateFrame;
use crate::index::IndexBuffer;
use crate::line_string::LineString3D;
use crate::ops::triangulation::{triangulate_3d, Cache};
use crate::point::Point3D;
use crate::polygon::Polygon3D;
use crate::polygon_mesh::{build_open_rings, PolygonMesh3D, PolygonMesh3DData};
use crate::solid::{Shell, Solid};
use crate::triangular_mesh::{TriangularMesh3D, TriangularMesh3DData};
use crate::Euclidean3DGeometry;

use super::view::polygon3d_rings;

/// A flattened, collection-free borrow of one 3D leaf: the normal form the 3D
/// operations dispatch over after flattening unnests collections.
#[derive(Clone, Copy, Debug)]
pub enum Leaf3D<'a> {
    Point(&'a Point3D),
    Line(&'a LineString3D),
    Polygon(&'a Polygon3D),
    PolygonMesh(&'a PolygonMesh3D),
    TriangularMesh(&'a TriangularMesh3D),
    Solid(&'a Solid),
}

impl<'a> Leaf3D<'a> {
    /// The leaf's coordinate frame.
    pub fn frame(&self) -> &'a CoordinateFrame {
        match self {
            Leaf3D::Point(p) => p.frame(),
            Leaf3D::Line(l) => l.frame(),
            Leaf3D::Polygon(p) => p.frame(),
            Leaf3D::PolygonMesh(m) => m.frame(),
            Leaf3D::TriangularMesh(m) => m.frame(),
            Leaf3D::Solid(s) => s.frame(),
        }
    }
}

/// Flatten a 3D geometry into its leaves, recursing into (possibly nested)
/// collections. A leaf kind the 3D operations do not support (`Csg`,
/// `PointCloud`) is not collected; instead its type name is recorded in
/// `unsupported` (first occurrence wins) for the caller's error.
pub(crate) fn flatten_3d<'a>(
    geometry: &'a Euclidean3DGeometry,
    out: &mut Vec<Leaf3D<'a>>,
    unsupported: &mut Option<&'static str>,
) {
    match geometry {
        Euclidean3DGeometry::Point(p) => out.push(Leaf3D::Point(p)),
        Euclidean3DGeometry::LineString(l) => out.push(Leaf3D::Line(l)),
        Euclidean3DGeometry::Polygon(p) => out.push(Leaf3D::Polygon(p)),
        Euclidean3DGeometry::PolygonMesh(m) => out.push(Leaf3D::PolygonMesh(m)),
        Euclidean3DGeometry::TriangularMesh(m) => out.push(Leaf3D::TriangularMesh(m)),
        Euclidean3DGeometry::Solid(s) => out.push(Leaf3D::Solid(s)),
        Euclidean3DGeometry::PointCloud(_) => {
            unsupported.get_or_insert("PointCloud");
        }
        Euclidean3DGeometry::Csg(_) => {
            unsupported.get_or_insert("Csg");
        }
        Euclidean3DGeometry::Collection(c) => {
            for member in c.members() {
                flatten_3d(member, out, unsupported);
            }
        }
    }
}

/// Require every leaf across both slices to share one coordinate frame.
pub(crate) fn require_common_frame_3d(a: &[Leaf3D<'_>], b: &[Leaf3D<'_>]) -> super::Result<()> {
    let mut frames = a.iter().chain(b.iter()).map(Leaf3D::frame);
    let Some(first) = frames.next() else {
        return Ok(());
    };
    for frame in frames {
        super::require_same_frame(first, frame)?;
    }
    Ok(())
}

/// A surface as a flat triangle soup: a vertex pool (borrowed from the source
/// leaf where its layout allows, owned where the rings had to be gathered) and
/// index triples into it.
pub(crate) struct TriangleSet<'a> {
    pool: Cow<'a, [[f64; 3]]>,
    tris: Vec<[u32; 3]>,
}

impl<'a> TriangleSet<'a> {
    /// The number of triangles.
    pub fn len(&self) -> usize {
        self.tris.len()
    }

    /// Whether the set has no triangles.
    pub fn is_empty(&self) -> bool {
        self.tris.is_empty()
    }

    /// The `i`-th triangle's corner coordinates.
    #[inline]
    pub fn triangle(&self, i: usize) -> [[f64; 3]; 3] {
        self.tris[i].map(|v| self.pool[v as usize])
    }

    /// The `i`-th triangle's corner indices into the pool.
    #[inline]
    pub fn indices(&self, i: usize) -> [u32; 3] {
        self.tris[i]
    }

    /// The vertex pool. May contain vertices no triangle references.
    pub fn pool(&self) -> &[[f64; 3]] {
        &self.pool
    }

    /// The triangles' corner coordinates, in order.
    pub fn triangles(&self) -> impl Iterator<Item = [[f64; 3]; 3]> + '_ {
        (0..self.len()).map(|i| self.triangle(i))
    }

    /// View a triangular mesh's data verbatim: borrowed pool, widened indices.
    pub fn from_triangular_data(data: &'a TriangularMesh3DData) -> Self {
        TriangleSet {
            pool: Cow::Borrowed(data.vertices()),
            tris: data.triangles().collect(),
        }
    }

    /// Triangulate a polygon mesh's faces without consuming them: the same
    /// per-face best-fit-plane earcut as its
    /// [`Triangulate`](crate::ops::Triangulate), reading through the CSR
    /// buffers and mapping every output corner back into the borrowed pool.
    pub fn from_polygon_mesh_data(data: &'a PolygonMesh3DData, cache: &mut Cache) -> Self {
        let (face_indices, face_offsets, interior_offsets) = data.csr_buffers();
        let indices = decode(face_indices);
        let offsets = decode(face_offsets);
        let holes_global = decode(interior_offsets);

        let earcut = &mut cache.earcut;
        let buffers = &mut cache.buffers;
        let mut tris: Vec<[u32; 3]> = Vec::new();
        let n = indices.len();
        if n != 0 {
            let n_faces = offsets.len() + 1;
            let mut start = 0usize;
            for fi in 0..n_faces {
                let end = offsets.get(fi).map_or(n, |&o| o as usize);
                build_open_rings(
                    &indices,
                    &holes_global,
                    start,
                    end,
                    &mut buffers.open_src,
                    &mut buffers.holes,
                );
                let face = &indices[start..end];
                let num_outer = buffers
                    .holes
                    .first()
                    .map_or(buffers.open_src.len(), |&h| h as usize);
                buffers.verts3.clear();
                buffers.verts3.extend(
                    buffers
                        .open_src
                        .iter()
                        .map(|&p| data.vertices()[face[p as usize] as usize]),
                );
                buffers.out.clear();
                if triangulate_3d(
                    earcut,
                    &buffers.verts3,
                    num_outer,
                    &buffers.holes,
                    &mut buffers.out,
                )
                .is_some()
                {
                    tris.extend(buffers.out.chunks_exact(3).map(|c| {
                        [
                            face[buffers.open_src[c[0] as usize] as usize],
                            face[buffers.open_src[c[1] as usize] as usize],
                            face[buffers.open_src[c[2] as usize] as usize],
                        ]
                    }));
                }
                start = end;
            }
        }

        TriangleSet {
            pool: Cow::Borrowed(data.vertices()),
            tris,
        }
    }

    /// Triangulate a polygon without consuming it: its open rings are gathered
    /// into an owned pool (exterior first, then holes) and earcut through the
    /// exterior's best-fit plane.
    pub fn from_polygon(polygon: &'a Polygon3D, cache: &mut Cache) -> Self {
        let mut pool: Vec<[f64; 3]> = Vec::new();
        let mut holes: Vec<u32> = Vec::new();
        for (r, ring) in polygon3d_rings(polygon).enumerate() {
            if r > 0 {
                holes.push(pool.len() as u32);
            }
            if r == 0 && ring.len() < 3 {
                // An exterior that cannot define a plane: no triangles.
                return TriangleSet {
                    pool: Cow::Owned(pool),
                    tris: Vec::new(),
                };
            }
            let open = if ring.len() >= 2 && ring.first() == ring.last() {
                &ring[..ring.len() - 1]
            } else {
                ring
            };
            pool.extend_from_slice(open);
        }
        let num_outer = holes.first().map_or(pool.len(), |&h| h as usize);

        let earcut = &mut cache.earcut;
        let out = &mut cache.buffers.out;
        out.clear();
        triangulate_3d(earcut, &pool, num_outer, &holes, out);
        let tris = out.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
        TriangleSet {
            pool: Cow::Owned(pool),
            tris,
        }
    }

    /// View one solid shell as its triangle set.
    pub fn from_shell(shell: &'a Shell, cache: &mut Cache) -> Self {
        match shell {
            Shell::PolygonMesh(d) => TriangleSet::from_polygon_mesh_data(d, cache),
            Shell::TriangularMesh(d) => TriangleSet::from_triangular_data(d),
        }
    }
}

/// Widen a packed index buffer into flat `u32`.
fn decode(buffer: &IndexBuffer<1>) -> Vec<u32> {
    buffer.iter_u32().map(|[i]| i).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection3D;
    use crate::coordinate::CoordinateFrame;
    use crate::polygon_mesh::PolygonMesh3D;

    fn e() -> CoordinateFrame {
        CoordinateFrame::Euclidean
    }

    #[test]
    fn flatten_recurses_and_records_unsupported() {
        let inner = Collection3D::new([Euclidean3DGeometry::Point(Point3D::new(
            e(),
            [1.0, 1.0, 1.0],
        ))]);
        let outer = Euclidean3DGeometry::Collection(Collection3D::new([
            Euclidean3DGeometry::Point(Point3D::new(e(), [0.0, 0.0, 0.0])),
            Euclidean3DGeometry::Collection(inner),
        ]));
        let mut leaves = Vec::new();
        let mut unsupported = None;
        flatten_3d(&outer, &mut leaves, &mut unsupported);
        assert_eq!(leaves.len(), 2);
        assert_eq!(unsupported, None);
    }

    #[test]
    fn polygon_triangulation_covers_the_face() {
        // A unit square in the plane z = 5, with a closing duplicate.
        let square = Polygon3D::from_rings(
            e(),
            [
                [0.0, 0.0, 5.0],
                [4.0, 0.0, 5.0],
                [4.0, 4.0, 5.0],
                [0.0, 4.0, 5.0],
                [0.0, 0.0, 5.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let mut cache = Cache::new();
        let set = TriangleSet::from_polygon(&square, &mut cache);
        assert_eq!(set.len(), 2);
        // Triangle corners are original vertices, all at z = 5.
        for t in set.triangles() {
            for p in t {
                assert_eq!(p[2], 5.0);
            }
        }
    }

    #[test]
    fn polygon_with_hole_leaves_the_hole_open() {
        let outer = [
            [0.0, 0.0, 0.0],
            [8.0, 0.0, 0.0],
            [8.0, 8.0, 0.0],
            [0.0, 8.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let hole = vec![
            [2.0, 2.0, 0.0],
            [2.0, 6.0, 0.0],
            [6.0, 6.0, 0.0],
            [6.0, 2.0, 0.0],
            [2.0, 2.0, 0.0],
        ];
        let poly = Polygon3D::from_rings(e(), outer, vec![hole]);
        let mut cache = Cache::new();
        let set = TriangleSet::from_polygon(&poly, &mut cache);
        assert!(!set.is_empty());
        use crate::predicates::kernel3d::point_in_triangle_3d;
        // The hole's center is covered by no triangle; the rim area is.
        assert!(!set
            .triangles()
            .any(|t| point_in_triangle_3d([4.0, 4.0, 0.0], t)));
        assert!(set
            .triangles()
            .any(|t| point_in_triangle_3d([1.0, 1.0, 0.0], t)));
    }

    #[test]
    fn polygon_mesh_faces_triangulate_in_place() {
        // Two unit quads sharing an edge, in the plane z = 0, rings stored
        // closed.
        let mesh = PolygonMesh3D::from_raw_parts(
            e(),
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
                [4.0, 0.0, 0.0],
                [4.0, 2.0, 0.0],
            ],
            vec![0, 1, 2, 3, 0, 1, 4, 5, 2, 1],
            vec![5],
            vec![],
        )
        .unwrap();
        let mut cache = Cache::new();
        let set = TriangleSet::from_polygon_mesh_data(mesh.data(), &mut cache);
        assert_eq!(set.len(), 4);
        // Indices reference the mesh's own pool.
        assert!(set
            .triangles()
            .flat_map(|t| t.into_iter())
            .all(|p| mesh.vertices().contains(&p)));
    }
}
