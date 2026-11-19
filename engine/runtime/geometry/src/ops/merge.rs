//! Merging polygons and meshes into one mesh, and listing their edges.
//!
//! Faces are copied as they are, with no re-triangulation. Vertices with the
//! same coordinates (exact `f64` bits) are welded into one shared pool, and two
//! edges are equal when their endpoints are, by the same comparison.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::Range;

use crate::appearance::{
    Appearance, ChannelId, FaceBinding, Material, MaterialIndex, Side, ThemeBinding, ThemeId,
    UvSet, UvSource,
};
use crate::coordinate::CoordinateFrame;
use crate::error::Error;
use crate::index::IndexBuffer;
use crate::polygon_mesh::{PolygonMesh2D, PolygonMesh3D, bake};
use crate::triangular_mesh::{TriangularMesh2D, TriangularMesh3D};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// Whether [`merge`] takes at least one face from `geometry` on its own:
/// polygons, polygon meshes and triangular meshes, possibly in collections, all
/// in one dimension and one coordinate frame, and in 2D at one elevation.
pub fn is_mergeable(geometry: &Geometry) -> bool {
    let mut leaves = Leaves::default();
    if collect(geometry, &mut leaves).is_err() || check_leaves(&leaves).is_err() {
        return false;
    }
    leaves.d2.iter().any(|leaf| match leaf {
        Euclidean2DGeometry::Polygon(p) => !p.exterior().is_empty(),
        Euclidean2DGeometry::PolygonMesh(m) => m.csr_buffers().0.len() > 0,
        Euclidean2DGeometry::TriangularMesh(m) => m.triangles().next().is_some(),
        _ => false,
    }) || leaves.d3.iter().any(|leaf| match leaf {
        Euclidean3DGeometry::Polygon(p) => !p.exterior().is_empty(),
        Euclidean3DGeometry::PolygonMesh(m) => m.data().csr_buffers().0.len() > 0,
        Euclidean3DGeometry::TriangularMesh(m) => m.triangles().next().is_some(),
        _ => false,
    })
}

/// Merge every face of `meshes` into one mesh, also returning how many faces
/// each input added, in input order.
///
/// The result is a triangular mesh when every input is a triangular mesh, and a
/// polygon mesh otherwise. Faces keep their input order, so input `k` holds the
/// faces from the sum of the counts before it, up to that sum plus its own
/// count. Appearance is carried over: palettes concatenate, themes union, and
/// each theme's UV is rebuilt per corner. Each input is read once, so an
/// iterator can stream them. `None` when no input has a face. Errors when an
/// input holds anything but polygons and meshes, or when the inputs differ in
/// dimension, frame, or 2D elevation.
pub fn merge<G: Borrow<Geometry>>(
    meshes: impl IntoIterator<Item = G>,
) -> Result<Option<(Geometry, Vec<usize>)>, Error> {
    let mut merger = Merger::Empty;
    let mut counts = Vec::new();
    for geometry in meshes {
        let mut leaves = Leaves::default();
        collect(geometry.borrow(), &mut leaves)?;
        let before = merger.num_faces();
        merger.push(&leaves)?;
        counts.push(merger.num_faces() - before);
    }
    Ok(merger.finish()?.map(|mesh| (mesh, counts)))
}

/// An edge as the exact `f64` bits of its two ends, the smaller first. A 2D
/// end has zero bits for its third coordinate.
pub type EdgeKey = ([u64; 3], [u64; 3]);

/// The distinct edges of the faces of `geometry`, sorted, closing open rings.
/// Two geometries of one dimension share an edge when their edge lists have a
/// key in common; sharing a vertex alone does not count. Empty when `geometry`
/// is not [mergeable](is_mergeable).
pub fn edge_keys(geometry: &Geometry) -> Vec<EdgeKey> {
    let mut leaves = Leaves::default();
    if collect(geometry, &mut leaves).is_err() || check_leaves(&leaves).is_err() {
        return Vec::new();
    }
    let mut keys = Vec::new();
    let mut push = |p: [u64; 3], q: [u64; 3]| match p.cmp(&q) {
        std::cmp::Ordering::Less => keys.push((p, q)),
        std::cmp::Ordering::Greater => keys.push((q, p)),
        std::cmp::Ordering::Equal => {}
    };
    let lift = |p: [f64; 2]| [p[0].to_bits(), p[1].to_bits(), 0];
    for leaf in &leaves.d2 {
        edges_2d(leaf, &mut |p, q| push(lift(p), lift(q)));
    }
    for leaf in &leaves.d3 {
        edges_3d(leaf, &mut |p, q| {
            push(p.map(f64::to_bits), q.map(f64::to_bits))
        });
    }
    keys.sort_unstable();
    keys.dedup();
    keys
}

/// Visit every edge of a ring of coordinates, closing it when it is open.
fn ring_edges<const N: usize>(ring: &[[f64; N]], f: &mut dyn FnMut([f64; N], [f64; N])) {
    for w in ring.windows(2) {
        f(w[0], w[1]);
    }
    if let (Some(&first), Some(&last)) = (ring.first(), ring.last())
        && ring.len() > 1
        && first != last
    {
        f(last, first);
    }
}

/// Visit every edge of the rings an index buffer describes, closing open rings.
fn indexed_ring_edges<const N: usize>(
    vertices: &[[f64; N]],
    ring: impl Iterator<Item = u32> + Clone,
    f: &mut dyn FnMut([f64; N], [f64; N]),
) {
    let coords: Vec<[f64; N]> = ring.map(|i| vertices[i as usize]).collect();
    ring_edges(&coords, f);
}

/// Each face's rings of a CSR mesh as corner ranges, exterior first.
fn csr_rings(
    corners: usize,
    offsets: &IndexBuffer<1>,
    interiors: &IndexBuffer<1>,
) -> Vec<Vec<Range<usize>>> {
    let mut ends: Vec<usize> = offsets.iter_u32().map(|[o]| o as usize).collect();
    ends.push(corners);
    let holes: Vec<usize> = interiors.iter_u32().map(|[o]| o as usize).collect();
    face_rings(&ends, &holes)
}

/// Each face's rings as corner ranges, exterior first, from the end of every
/// face and the start of every hole ring.
pub(crate) fn face_rings(face_ends: &[usize], hole_starts: &[usize]) -> Vec<Vec<Range<usize>>> {
    let mut holes = hole_starts.iter().copied().peekable();
    let mut start = 0usize;
    face_ends
        .iter()
        .map(|&end| {
            let mut rings = Vec::new();
            let mut ring_start = start;
            while let Some(&hole) = holes.peek() {
                if hole >= end {
                    break;
                }
                holes.next();
                rings.push(ring_start..hole);
                ring_start = hole;
            }
            rings.push(ring_start..end);
            start = end;
            rings
        })
        .collect()
}

fn mesh_edges<const N: usize>(
    vertices: &[[f64; N]],
    indices: &IndexBuffer<1>,
    offsets: &IndexBuffer<1>,
    interiors: &IndexBuffer<1>,
    f: &mut dyn FnMut([f64; N], [f64; N]),
) {
    let idx: Vec<u32> = indices.iter_u32().map(|[i]| i).collect();
    if idx.is_empty() {
        return;
    }
    for face in csr_rings(idx.len(), offsets, interiors) {
        for ring in face {
            indexed_ring_edges(vertices, idx[ring].iter().copied(), f);
        }
    }
}

fn edges_2d(leaf: &&Euclidean2DGeometry, f: &mut dyn FnMut([f64; 2], [f64; 2])) {
    match leaf {
        Euclidean2DGeometry::Polygon(p) => {
            std::iter::once(p.exterior())
                .chain(p.interiors())
                .for_each(|ring| ring_edges(ring, f));
        }
        Euclidean2DGeometry::PolygonMesh(m) => {
            let (indices, offsets, interiors) = m.csr_buffers();
            mesh_edges(m.vertices(), indices, offsets, interiors, f);
        }
        Euclidean2DGeometry::TriangularMesh(m) => {
            for t in m.triangles() {
                indexed_ring_edges(m.vertices(), t.into_iter(), f);
            }
        }
        _ => {}
    }
}

fn edges_3d(leaf: &&Euclidean3DGeometry, f: &mut dyn FnMut([f64; 3], [f64; 3])) {
    match leaf {
        Euclidean3DGeometry::Polygon(p) => {
            std::iter::once(p.exterior())
                .chain(p.interiors())
                .for_each(|ring| ring_edges(ring, f));
        }
        Euclidean3DGeometry::PolygonMesh(m) => {
            let (indices, offsets, interiors) = m.data().csr_buffers();
            mesh_edges(m.vertices(), indices, offsets, interiors, f);
        }
        Euclidean3DGeometry::TriangularMesh(m) => {
            for t in m.triangles() {
                indexed_ring_edges(m.vertices(), t.into_iter(), f);
            }
        }
        _ => {}
    }
}

/// The polygon and mesh leaves of a geometry, by dimension.
#[derive(Default)]
struct Leaves<'a> {
    d2: Vec<&'a Euclidean2DGeometry>,
    d3: Vec<&'a Euclidean3DGeometry>,
}

fn collect<'a>(geometry: &'a Geometry, leaves: &mut Leaves<'a>) -> Result<(), Error> {
    match geometry {
        Geometry::None => Ok(()),
        Geometry::Euclidean2D(g) => collect_2d(g, &mut leaves.d2),
        Geometry::Euclidean3D(g) => collect_3d(g, &mut leaves.d3),
        Geometry::GeometryCollection(c) => c.members().iter().try_for_each(|m| collect(m, leaves)),
    }
}

fn collect_2d<'a>(
    geometry: &'a Euclidean2DGeometry,
    leaves: &mut Vec<&'a Euclidean2DGeometry>,
) -> Result<(), Error> {
    match geometry {
        Euclidean2DGeometry::Collection(c) => {
            c.members().iter().try_for_each(|m| collect_2d(m, leaves))
        }
        Euclidean2DGeometry::Polygon(_)
        | Euclidean2DGeometry::PolygonMesh(_)
        | Euclidean2DGeometry::TriangularMesh(_) => {
            leaves.push(geometry);
            Ok(())
        }
        _ => Err(unsupported()),
    }
}

fn collect_3d<'a>(
    geometry: &'a Euclidean3DGeometry,
    leaves: &mut Vec<&'a Euclidean3DGeometry>,
) -> Result<(), Error> {
    match geometry {
        Euclidean3DGeometry::Collection(c) => {
            c.members().iter().try_for_each(|m| collect_3d(m, leaves))
        }
        Euclidean3DGeometry::Polygon(_)
        | Euclidean3DGeometry::PolygonMesh(_)
        | Euclidean3DGeometry::TriangularMesh(_) => {
            leaves.push(geometry);
            Ok(())
        }
        _ => Err(unsupported()),
    }
}

fn unsupported() -> Error {
    Error::invalid_geometry("only polygons, polygon meshes and triangular meshes can be merged")
}

fn frame_2d(leaf: &Euclidean2DGeometry) -> (&CoordinateFrame, Option<f64>) {
    match leaf {
        Euclidean2DGeometry::Polygon(p) => (p.frame(), p.elevation()),
        Euclidean2DGeometry::PolygonMesh(m) => (m.frame(), m.elevation()),
        Euclidean2DGeometry::TriangularMesh(m) => (m.frame(), m.elevation()),
        _ => unreachable!("collected leaves are polygons or meshes"),
    }
}

fn frame_3d(leaf: &Euclidean3DGeometry) -> &CoordinateFrame {
    match leaf {
        Euclidean3DGeometry::Polygon(p) => p.frame(),
        Euclidean3DGeometry::PolygonMesh(m) => m.frame(),
        Euclidean3DGeometry::TriangularMesh(m) => m.frame(),
        _ => unreachable!("collected leaves are polygons or meshes"),
    }
}

/// Errors unless the leaves share one dimension, frame and 2D elevation.
fn check_leaves(leaves: &Leaves<'_>) -> Result<(), Error> {
    if !leaves.d2.is_empty() && !leaves.d3.is_empty() {
        return Err(mixed_dimensions());
    }
    let mut frames = leaves
        .d2
        .iter()
        .map(|l| frame_2d(l).0)
        .chain(leaves.d3.iter().map(|l| frame_3d(l)));
    if let Some(first) = frames.next()
        && frames.any(|f| f != first)
    {
        return Err(mixed_frames());
    }
    let mut elevations = leaves.d2.iter().map(|l| frame_2d(l).1.map(f64::to_bits));
    if let Some(first) = elevations.next()
        && elevations.any(|z| z != first)
    {
        return Err(mixed_elevations());
    }
    Ok(())
}

fn mixed_dimensions() -> Error {
    Error::invalid_geometry("cannot merge 2D and 3D geometries into one mesh")
}

fn mixed_frames() -> Error {
    Error::invalid_geometry("cannot merge geometries in different coordinate frames")
}

fn mixed_elevations() -> Error {
    Error::invalid_geometry("cannot merge 2D geometries at different elevations")
}

/// The merge in progress, fixed to one dimension by its first leaf.
enum Merger {
    Empty,
    D2 {
        builder: Builder<2>,
        /// The elevation every 2D input lies at, as bits.
        elevation: Option<u64>,
    },
    D3(Builder<3>),
}

impl Merger {
    /// The faces pushed so far.
    fn num_faces(&self) -> usize {
        match self {
            Merger::Empty => 0,
            Merger::D2 { builder, .. } => builder.face_ends.len(),
            Merger::D3(builder) => builder.face_ends.len(),
        }
    }

    fn push(&mut self, leaves: &Leaves<'_>) -> Result<(), Error> {
        check_leaves(leaves)?;
        if let Some(first) = leaves.d2.first() {
            let elevation = frame_2d(first).1.map(f64::to_bits);
            if matches!(self, Merger::Empty) {
                *self = Merger::D2 {
                    builder: Builder::default(),
                    elevation,
                };
            }
            let Merger::D2 {
                builder,
                elevation: seen,
            } = self
            else {
                return Err(mixed_dimensions());
            };
            if *seen != elevation {
                return Err(mixed_elevations());
            }
            let z = elevation.map(f64::from_bits).unwrap_or(0.0);
            let lift = |v: [f64; 2]| [v[0], v[1], z];
            for leaf in &leaves.d2 {
                builder.check_frame(frame_2d(leaf).0)?;
                match leaf {
                    Euclidean2DGeometry::Polygon(p) => builder.push_polygon(
                        std::iter::once(p.exterior()).chain(p.interiors()),
                        p.appearance().as_ref(),
                        lift,
                    )?,
                    Euclidean2DGeometry::PolygonMesh(m) => {
                        let (indices, offsets, interiors) = m.csr_buffers();
                        builder.push_polygon_mesh(
                            m.vertices(),
                            indices,
                            offsets,
                            interiors,
                            m.appearance().as_ref(),
                            lift,
                        )?
                    }
                    Euclidean2DGeometry::TriangularMesh(m) => builder.push_triangular_mesh(
                        m.vertices(),
                        m.triangles(),
                        m.appearance().as_ref(),
                        lift,
                    )?,
                    _ => unreachable!("collected leaves are polygons or meshes"),
                }
            }
        }
        if !leaves.d3.is_empty() {
            if matches!(self, Merger::Empty) {
                *self = Merger::D3(Builder::default());
            }
            let Merger::D3(builder) = self else {
                return Err(mixed_dimensions());
            };
            let lift = |v: [f64; 3]| v;
            for leaf in &leaves.d3 {
                builder.check_frame(frame_3d(leaf))?;
                match leaf {
                    Euclidean3DGeometry::Polygon(p) => builder.push_polygon(
                        std::iter::once(p.exterior()).chain(p.interiors()),
                        p.appearance().as_ref(),
                        lift,
                    )?,
                    Euclidean3DGeometry::PolygonMesh(m) => {
                        let (indices, offsets, interiors) = m.data().csr_buffers();
                        builder.push_polygon_mesh(
                            m.vertices(),
                            indices,
                            offsets,
                            interiors,
                            m.appearance().as_ref(),
                            lift,
                        )?
                    }
                    Euclidean3DGeometry::TriangularMesh(m) => builder.push_triangular_mesh(
                        m.vertices(),
                        m.triangles(),
                        m.appearance().as_ref(),
                        lift,
                    )?,
                    _ => unreachable!("collected leaves are polygons or meshes"),
                }
            }
        }
        Ok(())
    }

    fn finish(self) -> Result<Option<Geometry>, Error> {
        match self {
            Merger::Empty => Ok(None),
            Merger::D2 { builder, elevation } => {
                let z = elevation.map(f64::from_bits);
                let Some(mut parts) = builder.finish() else {
                    return Ok(None);
                };
                let geometry = if parts.all_triangular {
                    let mut mesh =
                        TriangularMesh2D::from_parts(parts.frame, parts.vertices, parts.indices)?;
                    mesh.set_elevation(z);
                    mesh.set_raw_appearance(parts.appearance.take());
                    Euclidean2DGeometry::TriangularMesh(Box::new(mesh))
                } else {
                    let mut mesh = PolygonMesh2D::from_raw_parts(
                        parts.frame,
                        parts.vertices,
                        parts.indices,
                        internal_boundaries(&parts.face_ends),
                        parts.interior_offsets,
                    )?;
                    mesh.set_elevation(z);
                    *mesh.appearance_mut() = parts.appearance.take();
                    Euclidean2DGeometry::PolygonMesh(Box::new(mesh))
                };
                Ok(Some(Geometry::Euclidean2D(geometry)))
            }
            Merger::D3(builder) => {
                let Some(mut parts) = builder.finish() else {
                    return Ok(None);
                };
                let geometry = if parts.all_triangular {
                    let mut mesh =
                        TriangularMesh3D::from_parts(parts.frame, parts.vertices, parts.indices)?;
                    mesh.set_raw_appearance(parts.appearance.take());
                    Euclidean3DGeometry::TriangularMesh(Box::new(mesh))
                } else {
                    let mut mesh = PolygonMesh3D::from_raw_parts(
                        parts.frame,
                        parts.vertices,
                        parts.indices,
                        internal_boundaries(&parts.face_ends),
                        parts.interior_offsets,
                    )?;
                    *mesh.appearance_mut() = parts.appearance.take();
                    Euclidean3DGeometry::PolygonMesh(Box::new(mesh))
                };
                Ok(Some(Geometry::Euclidean3D(geometry)))
            }
        }
    }
}

/// The face boundaries a CSR buffer stores: every face end but the last.
pub(crate) fn internal_boundaries(face_ends: &[u32]) -> Vec<u32> {
    face_ends[..face_ends.len().saturating_sub(1)].to_vec()
}

/// The finished buffers of a merge.
struct Built<const N: usize> {
    frame: CoordinateFrame,
    vertices: Vec<[f64; N]>,
    indices: Vec<u32>,
    face_ends: Vec<u32>,
    interior_offsets: Vec<u32>,
    all_triangular: bool,
    appearance: Option<Appearance>,
}

/// The merged mesh being assembled.
struct Builder<const N: usize> {
    frame: Option<CoordinateFrame>,
    vertices: Vec<[f64; N]>,
    weld: HashMap<[u64; N], u32>,
    face_indices: Vec<u32>,
    /// The end of every face in `face_indices`.
    face_ends: Vec<u32>,
    interior_offsets: Vec<u32>,
    all_triangular: bool,
    appearance: AppearanceBuilder,
}

impl<const N: usize> Default for Builder<N> {
    fn default() -> Self {
        Self {
            frame: None,
            vertices: Vec::new(),
            weld: HashMap::new(),
            face_indices: Vec::new(),
            face_ends: Vec::new(),
            interior_offsets: Vec::new(),
            all_triangular: true,
            appearance: AppearanceBuilder::default(),
        }
    }
}

/// Where one input's faces and corners landed in the merged mesh.
struct Pushed {
    faces_before: usize,
    corners_before: usize,
    faces: usize,
    /// For each merged corner of the input, in order, the input's own corner
    /// index (the UV array index it reads from).
    source_corners: Vec<usize>,
}

impl<const N: usize> Builder<N> {
    fn check_frame(&mut self, frame: &CoordinateFrame) -> Result<(), Error> {
        match &self.frame {
            None => self.frame = Some(frame.clone()),
            Some(seen) if seen != frame => return Err(mixed_frames()),
            Some(_) => {}
        }
        Ok(())
    }

    fn weld(&mut self, coord: [f64; N]) -> u32 {
        *self.weld.entry(coord.map(f64::to_bits)).or_insert_with(|| {
            self.vertices.push(coord);
            (self.vertices.len() - 1) as u32
        })
    }

    /// The merged index of input vertex `i`, welded on first use so that
    /// vertices no face uses stay out of the merged pool.
    fn weld_used(&mut self, vertices: &[[f64; N]], remap: &mut [Option<u32>], i: u32) -> u32 {
        *remap[i as usize].get_or_insert_with(|| self.weld(vertices[i as usize]))
    }

    fn start(&self) -> Pushed {
        Pushed {
            faces_before: self.face_ends.len(),
            corners_before: self.face_indices.len(),
            faces: 0,
            source_corners: Vec::new(),
        }
    }

    /// Fold the appearance of the input just pushed into the merged one.
    fn finish_input(
        &mut self,
        pushed: Pushed,
        appearance: Option<&Appearance>,
        lift: impl Fn([f64; N]) -> [f64; 3],
    ) -> Result<(), Error> {
        let Some(appearance) = appearance else {
            return Ok(());
        };
        if pushed.faces == 0 {
            return Ok(());
        }
        let position = |k: usize| {
            let corner = self.face_indices[pushed.corners_before + k];
            lift(self.vertices[corner as usize])
        };
        self.appearance.push(appearance, &pushed, position)
    }

    /// Append one polygon as one face. Each ring is stored closed; a closing
    /// corner added here reads the UV of the ring's first corner.
    fn push_polygon<'r>(
        &mut self,
        rings: impl Iterator<Item = &'r [[f64; N]]>,
        appearance: Option<&Appearance>,
        lift: impl Fn([f64; N]) -> [f64; 3],
    ) -> Result<(), Error> {
        self.all_triangular = false;
        let mut pushed = self.start();
        let mut source = 0usize;
        for (i, ring) in rings.enumerate() {
            if ring.is_empty() {
                continue;
            }
            if i > 0 && self.face_indices.len() > pushed.corners_before {
                self.interior_offsets.push(self.face_indices.len() as u32);
            } else if i > 0 {
                // A hole after an empty exterior has no face to belong to.
                source += ring.len();
                continue;
            }
            for (k, &coord) in ring.iter().enumerate() {
                let index = self.weld(coord);
                self.face_indices.push(index);
                pushed.source_corners.push(source + k);
            }
            if ring.len() >= 2 && ring.first() != ring.last() {
                let index = self.weld(ring[0]);
                self.face_indices.push(index);
                pushed.source_corners.push(source);
            }
            source += ring.len();
        }
        if self.face_indices.len() == pushed.corners_before {
            return Ok(());
        }
        self.face_ends.push(self.face_indices.len() as u32);
        pushed.faces = 1;
        self.finish_input(pushed, appearance, lift)
    }

    /// Append every face of a polygon mesh, keeping its corner order.
    fn push_polygon_mesh(
        &mut self,
        vertices: &[[f64; N]],
        indices: &IndexBuffer<1>,
        offsets: &IndexBuffer<1>,
        interiors: &IndexBuffer<1>,
        appearance: Option<&Appearance>,
        lift: impl Fn([f64; N]) -> [f64; 3],
    ) -> Result<(), Error> {
        if indices.len() == 0 {
            return Ok(());
        }
        self.all_triangular = false;
        let mut pushed = self.start();
        let mut remap = vec![None; vertices.len()];
        let base = self.face_indices.len() as u32;
        for [i] in indices.iter_u32() {
            let index = self.weld_used(vertices, &mut remap, i);
            self.face_indices.push(index);
        }
        self.face_ends
            .extend(offsets.iter_u32().map(|[o]| base + o));
        self.face_ends.push(self.face_indices.len() as u32);
        self.interior_offsets
            .extend(interiors.iter_u32().map(|[o]| base + o));
        pushed.faces = offsets.len() + 1;
        pushed.source_corners = (0..indices.len()).collect();
        self.finish_input(pushed, appearance, lift)
    }

    /// Append every triangle of a triangular mesh as a three-corner face.
    fn push_triangular_mesh(
        &mut self,
        vertices: &[[f64; N]],
        triangles: impl Iterator<Item = [u32; 3]>,
        appearance: Option<&Appearance>,
        lift: impl Fn([f64; N]) -> [f64; 3],
    ) -> Result<(), Error> {
        let mut pushed = self.start();
        let mut remap = vec![None; vertices.len()];
        for triangle in triangles {
            for i in triangle {
                let index = self.weld_used(vertices, &mut remap, i);
                self.face_indices.push(index);
            }
            self.face_ends.push(self.face_indices.len() as u32);
            pushed.faces += 1;
        }
        pushed.source_corners = (0..self.face_indices.len() - pushed.corners_before).collect();
        self.finish_input(pushed, appearance, lift)
    }

    fn finish(self) -> Option<Built<N>> {
        if self.face_ends.is_empty() {
            return None;
        }
        let appearance = self
            .appearance
            .finish(self.face_ends.len(), self.face_indices.len());
        Some(Built {
            frame: self.frame.expect("a face was pushed, so a frame was seen"),
            vertices: self.vertices,
            indices: self.face_indices,
            face_ends: self.face_ends,
            interior_offsets: self.interior_offsets,
            all_triangular: self.all_triangular,
            appearance,
        })
    }
}

/// One theme of the merged appearance, per merged face and corner. Entries for
/// faces and corners pushed before the theme first appeared are filled in when
/// it does.
struct ThemeBuilder {
    theme: ThemeId,
    front: Vec<Option<MaterialIndex>>,
    back: Option<Vec<Option<MaterialIndex>>>,
    uv_sets: Vec<(Side, ChannelId, Vec<[f64; 2]>)>,
}

/// The merged appearance, built one input at a time.
#[derive(Default)]
struct AppearanceBuilder {
    materials: Vec<Material>,
    themes: Vec<ThemeBuilder>,
    default_theme: Option<ThemeId>,
}

impl AppearanceBuilder {
    fn push(
        &mut self,
        appearance: &Appearance,
        pushed: &Pushed,
        position: impl Fn(usize) -> [f64; 3],
    ) -> Result<(), Error> {
        let offset = u32::try_from(self.materials.len())
            .map_err(|_| Error::invalid_appearance("material palette too large"))?;
        self.materials
            .extend(appearance.materials().iter().cloned());
        if self.default_theme.is_none() {
            self.default_theme = Some(appearance.default_theme().clone());
        }
        let per_face = |binding: &FaceBinding| -> Result<Vec<Option<MaterialIndex>>, Error> {
            Ok(match binding.clone().offset(offset)? {
                FaceBinding::Uniform(index) => vec![Some(index); pushed.faces],
                FaceBinding::PerFace(faces) => (0..pushed.faces)
                    .map(|f| faces.get(f).copied().flatten())
                    .collect(),
            })
        };
        for binding in appearance.themes() {
            let index = match self.themes.iter().position(|t| t.theme == binding.theme) {
                Some(index) => index,
                None => {
                    self.themes.push(ThemeBuilder {
                        theme: binding.theme.clone(),
                        front: Vec::new(),
                        back: None,
                        uv_sets: Vec::new(),
                    });
                    self.themes.len() - 1
                }
            };
            let theme = &mut self.themes[index];
            theme.front.resize(pushed.faces_before, None);
            theme.front.extend(per_face(&binding.front)?);
            if let Some(back) = &binding.back {
                let faces = theme.back.get_or_insert_with(Vec::new);
                faces.resize(pushed.faces_before, None);
                faces.extend(per_face(back)?);
            }
            for set in &binding.uv_sets {
                let uv = match theme
                    .uv_sets
                    .iter()
                    .position(|(s, c, _)| *s == set.side && *c == set.channel)
                {
                    Some(i) => &mut theme.uv_sets[i].2,
                    None => {
                        theme.uv_sets.push((set.side, set.channel, Vec::new()));
                        &mut theme.uv_sets.last_mut().expect("just pushed").2
                    }
                };
                uv.resize(pushed.corners_before, [0.0, 0.0]);
                match &set.uv {
                    UvSource::Explicit(arr) => uv.extend(
                        pushed
                            .source_corners
                            .iter()
                            .map(|&c| arr.get(c).copied().unwrap_or([0.0, 0.0])),
                    ),
                    UvSource::WorldToTexture(matrix) => uv.extend(
                        (0..pushed.source_corners.len()).map(|k| bake(matrix, position(k))),
                    ),
                }
            }
        }
        Ok(())
    }

    fn finish(self, faces: usize, corners: usize) -> Option<Appearance> {
        let default_theme = self.default_theme?;
        let themes = self
            .themes
            .into_iter()
            .map(|mut t| {
                t.front.resize(faces, None);
                ThemeBinding {
                    theme: t.theme,
                    front: FaceBinding::PerFace(t.front),
                    back: t.back.map(|mut b| {
                        b.resize(faces, None);
                        FaceBinding::PerFace(b)
                    }),
                    uv_sets: t
                        .uv_sets
                        .into_iter()
                        .map(|(side, channel, mut uv)| {
                            uv.resize(corners, [0.0, 0.0]);
                            UvSet {
                                side,
                                channel,
                                uv: UvSource::Explicit(uv.into_boxed_slice()),
                            }
                        })
                        .collect(),
                }
            })
            .collect();
        let mut appearance = Appearance::from_parts(self.materials, themes, default_theme);
        appearance.compact_materials();
        Some(appearance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::appearance::FaceBinding;
    use crate::collection::Collection3D;
    use crate::coordinate::EpsgCode;
    use crate::ops::{Aabb, BoundingBox};
    use crate::point::Point3D;
    use crate::polygon::{Polygon2D, Polygon3D};
    use crate::test_support::*;
    use pretty_assertions::assert_eq;

    const F: CoordinateFrame = CoordinateFrame::Euclidean;

    fn merge_some(geometries: &[Geometry]) -> Result<Geometry, Error> {
        Ok(merge(geometries)?.map(|(mesh, _)| mesh).unwrap_or_default())
    }

    fn tri(corners: [[f64; 3]; 3]) -> Polygon3D {
        Polygon3D::from_rings(F, corners, Vec::<Vec<[f64; 3]>>::new())
    }

    fn g3(geometry: Euclidean3DGeometry) -> Geometry {
        Geometry::Euclidean3D(geometry)
    }

    fn polygon(p: Polygon3D) -> Geometry {
        g3(Euclidean3DGeometry::Polygon(Box::new(p)))
    }

    fn trimesh(vertices: Vec<[f64; 3]>, indices: Vec<u32>) -> Geometry {
        g3(Euclidean3DGeometry::TriangularMesh(Box::new(
            TriangularMesh3D::from_parts(F, vertices, indices).unwrap(),
        )))
    }

    fn as_polygon_mesh(g: &Geometry) -> &PolygonMesh3D {
        let Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(m)) = g else {
            panic!("expected a polygon mesh, got {g:?}");
        };
        m
    }

    #[test]
    fn polygons_weld_into_one_polygon_mesh() {
        let a = tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]);
        let b = tri([[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        let merged = merge_some(&[polygon(a), polygon(b)]).unwrap();
        let mesh = as_polygon_mesh(&merged);
        assert_eq!(mesh.num_faces(), 2);
        assert_eq!(mesh.vertices().len(), 4);
        assert!(mesh.appearance().is_none());
    }

    #[test]
    fn each_input_reports_the_faces_it_added() {
        let two_triangles = trimesh(
            vec![[5., 0., 0.], [6., 0., 0.], [5., 1., 0.], [6., 1., 0.]],
            vec![0, 1, 2, 1, 3, 2],
        );
        let no_triangles = trimesh(vec![[9., 9., 9.]], Vec::new());
        let one = polygon(tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]));
        let (mesh, counts) = merge([two_triangles, no_triangles, one]).unwrap().unwrap();
        assert_eq!(counts, vec![2, 0, 1]);
        assert_eq!(as_polygon_mesh(&mesh).num_faces(), 3);
    }

    #[test]
    fn vertices_no_face_uses_are_left_out() {
        let spare = trimesh(
            vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [9., 9., 9.]],
            vec![0, 1, 2],
        );
        let faceless = trimesh(vec![[-9., -9., -9.]], Vec::new());
        let merged = merge_some(&[spare, faceless]).unwrap();
        let Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(mesh)) = &merged else {
            panic!("expected a triangular mesh, got {merged:?}");
        };
        assert_eq!(mesh.vertices().len(), 3);
        assert_eq!(
            merged.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0., 0., 0.],
                max: [1., 1., 0.]
            }
        );
    }

    #[test]
    fn a_geometry_without_faces_is_not_mergeable() {
        assert!(is_mergeable(&polygon(tri([
            [0., 0., 0.],
            [1., 0., 0.],
            [0., 1., 0.]
        ]))));
        assert!(!is_mergeable(&trimesh(vec![[0., 0., 0.]], Vec::new())));
        assert!(!is_mergeable(&Geometry::None));
        let point = g3(Euclidean3DGeometry::Point(Point3D::new(F, [0., 0., 0.])));
        assert!(!is_mergeable(&point));
    }

    #[test]
    fn no_faces_count_as_no_mesh() {
        assert_eq!(
            merge([trimesh(vec![[0., 0., 0.]], Vec::new())]).unwrap(),
            None
        );
    }

    #[test]
    fn owned_inputs_stream_in() {
        let inputs = vec![
            polygon(tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]])),
            polygon(tri([[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]])),
        ];
        let (merged, _) = merge(inputs).unwrap().unwrap();
        assert_eq!(as_polygon_mesh(&merged).num_faces(), 2);
    }

    #[test]
    fn triangular_meshes_stay_triangular() {
        let a = trimesh(
            vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            vec![0, 1, 2],
        );
        let b = trimesh(
            vec![[1., 1., 0.], [0., 1., 0.], [1., 0., 0.]],
            vec![2, 0, 1],
        );
        let merged = merge_some(&[a, b]).unwrap();
        let Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(mesh)) = &merged else {
            panic!("expected a triangular mesh, got {merged:?}");
        };
        assert_eq!(mesh.vertices().len(), 4);
        assert_eq!(
            mesh.triangles().collect::<Vec<_>>(),
            vec![[0, 1, 2], [1, 3, 2]]
        );
    }

    #[test]
    fn a_triangular_mesh_with_a_polygon_makes_a_polygon_mesh() {
        let a = trimesh(
            vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            vec![0, 1, 2],
        );
        let b = polygon(tri([[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]));
        let merged = merge_some(&[a, b]).unwrap();
        let mesh = as_polygon_mesh(&merged);
        assert_eq!(mesh.num_faces(), 2);
        assert_eq!(mesh.vertices().len(), 4);
    }

    #[test]
    fn collections_are_flattened() {
        let a =
            Euclidean3DGeometry::Polygon(Box::new(tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]])));
        let b =
            Euclidean3DGeometry::Polygon(Box::new(tri([[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]])));
        let collection = g3(Euclidean3DGeometry::Collection(Collection3D::new([a, b])));
        let merged = merge_some(&[collection]).unwrap();
        assert_eq!(as_polygon_mesh(&merged).num_faces(), 2);
    }

    #[test]
    fn a_hole_is_kept() {
        let outer = [[0., 0., 0.], [4., 0., 0.], [4., 4., 0.], [0., 4., 0.]];
        let hole = vec![[1., 1., 0.], [2., 1., 0.], [2., 2., 0.], [1., 2., 0.]];
        let with_hole = Polygon3D::from_rings(F, outer, vec![hole]);
        let merged = merge_some(&[polygon(with_hole)]).unwrap();
        let faces = as_polygon_mesh(&merged).faces();
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].interiors().count(), 1);
    }

    #[test]
    fn appearance_is_carried_per_face() {
        let mut a = tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]);
        a.set_appearance(
            theme("rgb"),
            textured(),
            Some(explicit_uv(&[[0., 0.], [1., 0.], [0., 1.]])),
        )
        .unwrap();
        let b = tri([[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        let merged = merge_some(&[polygon(a), polygon(b)]).unwrap();
        let mesh = as_polygon_mesh(&merged);
        let app = mesh.appearance().as_ref().unwrap();
        assert_eq!(app.materials().len(), 1);
        let binding = &app.themes()[0];
        assert_eq!(
            binding.front,
            FaceBinding::PerFace(vec![MaterialIndex::new(0), None])
        );
        let UvSource::Explicit(uv) = &binding.uv_sets[0].uv else {
            panic!("expected explicit UV");
        };
        // Each triangle is stored closed: four corners per face.
        assert_eq!(
            uv.to_vec(),
            vec![
                [0., 0.],
                [1., 0.],
                [0., 1.],
                [0., 0.],
                [0., 0.],
                [0., 0.],
                [0., 0.],
                [0., 0.]
            ]
        );
    }

    #[test]
    fn a_theme_first_seen_on_a_later_input_is_backfilled() {
        let a = tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]);
        let mut b = tri([[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        b.set_appearance(
            theme("rgb"),
            textured(),
            Some(explicit_uv(&[[0.5, 0.], [1., 0.], [0., 1.]])),
        )
        .unwrap();
        let merged = merge_some(&[polygon(a), polygon(b)]).unwrap();
        let app = as_polygon_mesh(&merged).appearance().clone().unwrap();
        let binding = &app.themes()[0];
        assert_eq!(
            binding.front,
            FaceBinding::PerFace(vec![None, MaterialIndex::new(0)])
        );
        let UvSource::Explicit(uv) = &binding.uv_sets[0].uv else {
            panic!("expected explicit UV");
        };
        assert_eq!(uv.len(), 8);
        assert_eq!(uv[..4].to_vec(), vec![[0., 0.]; 4]);
        assert_eq!(uv[4], [0.5, 0.]);
    }

    #[test]
    fn mesh_appearance_is_offset_into_the_merged_palette() {
        let mut a = tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]);
        a.set_appearance(theme("rgb"), phong(None), None).unwrap();
        let mut b = tri([[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        b.set_appearance(theme("rgb"), textured(), Some(uv(3)))
            .unwrap();
        let merged = merge_some(&[polygon(a), polygon(b)]).unwrap();
        let app = as_polygon_mesh(&merged).appearance().clone().unwrap();
        assert_eq!(app.materials().len(), 2);
        assert_eq!(
            app.themes()[0].front,
            FaceBinding::PerFace(vec![MaterialIndex::new(0), MaterialIndex::new(1)])
        );
    }

    #[test]
    fn mismatches_are_errors() {
        let a = polygon(tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]));
        let other_frame = polygon(Polygon3D::from_rings(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            Vec::<Vec<[f64; 3]>>::new(),
        ));
        assert!(merge_some(&[a.clone(), other_frame]).is_err());

        let flat = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                F,
                [[0., 0.], [1., 0.], [0., 1.]],
                Vec::<Vec<[f64; 2]>>::new(),
            ),
        )));
        assert!(merge_some(&[a.clone(), flat]).is_err());

        let point = g3(Euclidean3DGeometry::Point(Point3D::new(F, [0., 0., 0.])));
        assert!(!is_mergeable(&point));
        assert!(merge_some(&[a, point]).is_err());
    }

    #[test]
    fn a_geometry_mixing_frames_is_not_mergeable() {
        let a =
            Euclidean3DGeometry::Polygon(Box::new(tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]])));
        let b = Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            Vec::<Vec<[f64; 3]>>::new(),
        )));
        assert!(is_mergeable(&g3(a.clone())));
        assert!(!is_mergeable(&g3(Euclidean3DGeometry::Collection(
            Collection3D::new([a, b])
        ))));
        assert!(!is_mergeable(&Geometry::None));
    }

    #[test]
    fn two_dimensional_elevations_must_agree() {
        let at = |z: f64| {
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
                Polygon2D::from_rings_at_elevation(
                    F,
                    [[0., 0.], [1., 0.], [0., 1.]],
                    Vec::<Vec<[f64; 2]>>::new(),
                    z,
                ),
            )))
        };
        let Geometry::Euclidean2D(Euclidean2DGeometry::PolygonMesh(mesh)) =
            merge_some(&[at(5.0), at(5.0)]).unwrap()
        else {
            panic!("expected a 2D polygon mesh");
        };
        assert_eq!(mesh.elevation(), Some(5.0));
        assert!(merge_some(&[at(5.0), at(6.0)]).is_err());
    }

    #[test]
    fn nothing_to_merge_yields_none() {
        assert_eq!(merge([Geometry::None]).unwrap(), None);
        assert_eq!(merge(Vec::<Geometry>::new()).unwrap(), None);
    }

    fn share_an_edge(a: &Geometry, b: &Geometry) -> bool {
        let b = edge_keys(b);
        edge_keys(a).iter().any(|k| b.binary_search(k).is_ok())
    }

    #[test]
    fn a_shared_edge_connects() {
        let a = polygon(tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]));
        let b = polygon(tri([[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]));
        assert!(share_an_edge(&a, &b));
        assert!(share_an_edge(&b, &a));
    }

    #[test]
    fn a_shared_vertex_does_not_connect() {
        let a = polygon(tri([[0., 0., 0.], [1., 0., 0.], [1., 1., 0.]]));
        let b = polygon(tri([[1., 1., 0.], [2., 1., 0.], [2., 2., 0.]]));
        assert!(!share_an_edge(&a, &b));
    }

    #[test]
    fn an_edge_of_a_mesh_connects_to_a_polygon() {
        let mesh = trimesh(
            vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            vec![0, 1, 2],
        );
        let b = polygon(tri([[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]));
        assert!(share_an_edge(&mesh, &b));
        let apart = polygon(tri([[5., 0., 0.], [6., 0., 0.], [5., 1., 0.]]));
        assert!(!share_an_edge(&mesh, &apart));
    }

    #[test]
    fn edges_compare_exactly() {
        let a = polygon(tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]));
        let b = polygon(tri([[1., 0., 0.], [1., 1., 0.], [0., 1. + 1e-12, 0.]]));
        assert!(!share_an_edge(&a, &b));
    }

    #[test]
    fn edges_are_listed_once_each_including_the_closing_edge() {
        // Two triangles of one mesh share the edge (1,0)-(0,1).
        let mesh = trimesh(
            vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
            vec![0, 1, 2, 1, 3, 2],
        );
        let keys = edge_keys(&mesh);
        assert_eq!(keys.len(), 5);
        assert!(keys.is_sorted());
        // An open ring still has its closing edge.
        assert_eq!(
            edge_keys(&polygon(tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]))).len(),
            3
        );
    }

    #[test]
    fn an_unmergeable_geometry_has_no_edges() {
        let point = g3(Euclidean3DGeometry::Point(Point3D::new(F, [0., 0., 0.])));
        assert!(edge_keys(&point).is_empty());
    }
}
