use super::{PolygonMesh2D, PolygonMesh3D, PolygonMesh3DData};
use crate::coordinate::{CoordinateFrame, EpsgCode};
use crate::index::IndexBuffer;
use crate::ops::reproject::{transform_coords_2d, transform_coords_3d};
use crate::ops::triangulation::{
    expand_appearance, triangulate_2d, triangulate_3d, Cache, Triangulated,
};
use crate::ops::{
    lift_coords, Aabb, BoundingBox, Reproject, ReprojectionCache, Triangulate, UnsupportedOperation,
};
use crate::triangular_mesh::{TriangularMesh2D, TriangularMesh3D, TriangularMesh3DData};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

use reearth_flow_common::attribute::Attributes;

use crate::ops::Split;

impl BoundingBox for PolygonMesh2D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        Aabb::from_points_2d(self.vertices.iter().copied()).ok_or(UnsupportedOperation {
            geometry: "PolygonMesh2D",
            operation: "bounding_box",
        })
    }
}

impl BoundingBox for PolygonMesh3D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        Aabb::from_points_3d(self.data.vertices.iter().copied()).ok_or(UnsupportedOperation {
            geometry: "PolygonMesh3D",
            operation: "bounding_box",
        })
    }
}

impl PolygonMesh2D {
    /// Move the mesh out, leaving an empty husk.
    fn take(&mut self) -> Self {
        Self {
            frame: std::mem::take(&mut self.frame),
            vertices: std::mem::take(&mut self.vertices),
            z: self.z.take(),
            face_indices: std::mem::take(&mut self.face_indices),
            face_offsets: std::mem::take(&mut self.face_offsets),
            interior_offsets: std::mem::take(&mut self.interior_offsets),
            appearance: self.appearance.take(),
        }
    }

    /// The leaf moved out and wrapped as a [`Geometry`].
    fn take_geometry(&mut self) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::PolygonMesh(Box::new(self.take())))
    }
}

impl PolygonMesh3D {
    /// Move the mesh out, leaving an empty husk.
    fn take(&mut self) -> Self {
        Self {
            frame: std::mem::take(&mut self.frame),
            data: PolygonMesh3DData {
                vertices: std::mem::take(&mut self.data.vertices),
                face_indices: std::mem::take(&mut self.data.face_indices),
                face_offsets: std::mem::take(&mut self.data.face_offsets),
                interior_offsets: std::mem::take(&mut self.data.interior_offsets),
                appearance: self.data.appearance.take(),
            },
        }
    }

    /// The leaf moved out and wrapped as a [`Geometry`].
    fn take_geometry(&mut self) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(Box::new(self.take())))
    }
}

impl Reproject for PolygonMesh2D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        let from = self.frame.require_crs()?;
        if from == target {
            return Ok(self.take_geometry());
        }
        if self.z.is_some() {
            return self.take().into_3d().reproject(target, cache);
        }
        let mut m = self.take();
        transform_coords_2d(cache, from, target, &mut m.vertices)?;
        m.frame = CoordinateFrame::Crs(target);
        Ok(Geometry::Euclidean2D(Euclidean2DGeometry::PolygonMesh(
            Box::new(m),
        )))
    }
}

impl Reproject for PolygonMesh3D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        let from = self.frame.require_crs()?;
        let mut m = self.take();
        if from != target {
            transform_coords_3d(cache, from, target, m.data.vertices_mut())?;
            m.frame = CoordinateFrame::Crs(target);
        }
        Ok(Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(
            Box::new(m),
        )))
    }
}

use crate::ops::{plan_frame_step, translate_2d, translate_3d, ConvertFrame, FrameStep, Translate};

impl Translate for PolygonMesh2D {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        translate_2d(&mut self.vertices, &mut self.z, delta);
        Ok(())
    }
}

impl Translate for PolygonMesh3D {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        translate_3d(self.data.vertices_mut(), delta);
        Ok(())
    }
}

impl ConvertFrame for PolygonMesh2D {
    fn convert_frame(
        &mut self,
        target: &CoordinateFrame,
        base_point: Option<[f64; 3]>,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        match plan_frame_step(&self.frame, target, base_point)? {
            FrameStep::Noop => Ok(self.take_geometry()),
            FrameStep::Reproject(to) => self.reproject(to, cache),
            FrameStep::Translate(offset, frame) => {
                self.translate(offset)?;
                self.frame = frame;
                Ok(self.take_geometry())
            }
        }
    }
}

impl ConvertFrame for PolygonMesh3D {
    fn convert_frame(
        &mut self,
        target: &CoordinateFrame,
        base_point: Option<[f64; 3]>,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        match plan_frame_step(&self.frame, target, base_point)? {
            FrameStep::Noop => Ok(self.take_geometry()),
            FrameStep::Reproject(to) => self.reproject(to, cache),
            FrameStep::Translate(offset, frame) => {
                self.translate(offset)?;
                self.frame = frame;
                Ok(self.take_geometry())
            }
        }
    }
}

impl Triangulate for PolygonMesh2D {
    fn triangulate(&mut self, cache: &mut Cache) -> Result<Geometry, UnsupportedOperation> {
        let Cache { earcut, buffers } = cache;
        decode_into(&self.face_indices, &mut buffers.face_indices);
        decode_into(&self.face_offsets, &mut buffers.face_offsets);
        decode_into(&self.interior_offsets, &mut buffers.interior_offsets);

        buffers.tris.clear();
        buffers.corner_src.clear();
        buffers.face_tris.clear();
        buffers.tris.reserve(3 * buffers.face_indices.len());

        let n = buffers.face_indices.len();
        if n != 0 {
            let n_faces = buffers.face_offsets.len() + 1;
            let mut start = 0usize;
            for fi in 0..n_faces {
                let end = buffers.face_offsets.get(fi).map_or(n, |&o| o as usize);
                build_open_rings(
                    &buffers.face_indices,
                    &buffers.interior_offsets,
                    start,
                    end,
                    &mut buffers.open_src,
                    &mut buffers.holes,
                );
                let face = &buffers.face_indices[start..end];
                buffers.verts2.clear();
                // SAFETY: face indices are validated `< vertices.len()` at construction.
                buffers
                    .verts2
                    .extend(buffers.open_src.iter().map(|&p| unsafe {
                        *self
                            .vertices
                            .get_unchecked(*face.get_unchecked(p as usize) as usize)
                    }));
                // earcut clears `out` itself, but reset explicitly so the per-face
                // count below stays correct without relying on that internal.
                buffers.out.clear();
                triangulate_2d(earcut, &buffers.verts2, &buffers.holes, &mut buffers.out);
                // Map face-local corner indices back to the shared vertex pool.
                // SAFETY: each earcut index is `< open_src.len()`.
                buffers.tris.extend(buffers.out.iter().map(|&l| unsafe {
                    *face.get_unchecked(*buffers.open_src.get_unchecked(l as usize) as usize)
                }));
                buffers.corner_src.extend(
                    buffers
                        .out
                        .iter()
                        .map(|&l| start as u32 + buffers.open_src[l as usize]),
                );
                buffers.face_tris.push((buffers.out.len() / 3) as u32);
                start = end;
            }
        }

        let appearance = expand_appearance(
            std::mem::take(&mut self.appearance),
            &buffers.face_tris,
            &buffers.corner_src,
        );
        let triangle_count = buffers.tris.len() / 3;
        // `tris` index the existing pool (each `< vertices.len()`) in triples.
        // SAFETY: every index is `< vertices.len()`; count is a multiple of 3.
        let mut mesh = unsafe {
            match self.z.take() {
                Some(elevation) => TriangularMesh2D::from_parts_at_elevation_unchecked(
                    self.frame.clone(),
                    std::mem::take(&mut self.vertices),
                    triangle_count,
                    buffers.tris.iter().copied(),
                    elevation,
                ),
                None => TriangularMesh2D::from_parts_unchecked(
                    self.frame.clone(),
                    std::mem::take(&mut self.vertices),
                    triangle_count,
                    buffers.tris.iter().copied(),
                ),
            }
        };
        mesh.set_raw_appearance(appearance);
        Ok(Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(
            Box::new(mesh),
        )))
    }
}

impl PolygonMesh3DData {
    /// Triangulate every polygon into coordinate-free triangle-mesh data,
    /// **stealing** this mesh's vertex pool, appearance and UV (see
    /// [`Triangulate`]). Also returns each source polygon's flat normal
    /// (Newell's method over that polygon's own exterior ring, not
    /// recomputed per triangle) and how many triangles that polygon was
    /// split into, in polygon order — plain, transient values, not stored on
    /// any mesh type. A caller wanting the normal for output triangle `t`
    /// sums the triangle counts until it passes `t` and reads the matching
    /// normal entry, rather than this pre-expanding to one entry per
    /// triangle itself.
    pub(crate) fn triangulate(&mut self, cache: &mut Cache) -> Triangulated<TriangularMesh3DData> {
        let Cache { earcut, buffers } = cache;
        decode_into(&self.face_indices, &mut buffers.face_indices);
        decode_into(&self.face_offsets, &mut buffers.face_offsets);
        decode_into(&self.interior_offsets, &mut buffers.interior_offsets);

        buffers.tris.clear();
        buffers.corner_src.clear();
        buffers.face_tris.clear();
        buffers.face_normals.clear();
        buffers.tris.reserve(3 * buffers.face_indices.len());

        let n = buffers.face_indices.len();
        if n != 0 {
            let n_faces = buffers.face_offsets.len() + 1;
            let mut start = 0usize;
            for fi in 0..n_faces {
                let end = buffers.face_offsets.get(fi).map_or(n, |&o| o as usize);
                build_open_rings(
                    &buffers.face_indices,
                    &buffers.interior_offsets,
                    start,
                    end,
                    &mut buffers.open_src,
                    &mut buffers.holes,
                );
                let face = &buffers.face_indices[start..end];
                let num_outer = buffers
                    .holes
                    .first()
                    .map_or(buffers.open_src.len(), |&h| h as usize);
                buffers.verts3.clear();
                // SAFETY: face indices are validated `< vertices.len()` at construction.
                buffers
                    .verts3
                    .extend(buffers.open_src.iter().map(|&p| unsafe {
                        *self
                            .vertices
                            .get_unchecked(*face.get_unchecked(p as usize) as usize)
                    }));
                // `triangulate_3d` clears `out` on its degenerate paths and earcut
                // clears it on success; reset explicitly so the per-face count
                // below holds without relying on either internal.
                buffers.out.clear();
                let face_normal = triangulate_3d(
                    earcut,
                    &buffers.verts3,
                    num_outer,
                    &buffers.holes,
                    &mut buffers.out,
                );
                if face_normal.is_some() {
                    // SAFETY: each earcut index is `< open_src.len()`.
                    buffers.tris.extend(buffers.out.iter().map(|&l| unsafe {
                        *face.get_unchecked(*buffers.open_src.get_unchecked(l as usize) as usize)
                    }));
                    buffers.corner_src.extend(
                        buffers
                            .out
                            .iter()
                            .map(|&l| start as u32 + buffers.open_src[l as usize]),
                    );
                }
                // Outside the `if`: a degenerate face records 0 triangles (its
                // `out` is empty) and a placeholder normal that's never repeated.
                buffers.face_tris.push((buffers.out.len() / 3) as u32);
                buffers
                    .face_normals
                    .push(face_normal.unwrap_or([0.0, 0.0, 1.0]));
                start = end;
            }
        }

        let appearance = expand_appearance(
            std::mem::take(&mut self.appearance),
            &buffers.face_tris,
            &buffers.corner_src,
        );
        // SAFETY: `tris` index `self.vertices` (`< len`); count is a multiple of 3.
        let mut data = unsafe {
            TriangularMesh3DData::from_parts_unchecked(
                std::mem::take(&mut self.vertices),
                buffers.tris.len() / 3,
                buffers.tris.iter().copied(),
            )
        };
        data.set_raw_appearance(appearance);
        Triangulated {
            mesh: data,
            polygon_normals: std::mem::take(&mut buffers.face_normals),
            polygon_tris: std::mem::take(&mut buffers.face_tris),
        }
    }
}

impl Triangulate for PolygonMesh3D {
    fn triangulate(&mut self, cache: &mut Cache) -> Result<Geometry, UnsupportedOperation> {
        let result = self.data.triangulate(cache);
        let mesh = TriangularMesh3D::new(self.frame.clone(), result.mesh);
        Ok(Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(
            Box::new(mesh),
        )))
    }
}

impl PolygonMesh3D {
    /// As [`Triangulate::triangulate`], but also returns each source
    /// polygon's flat normal and its output triangle count, in polygon order
    /// (see [`PolygonMesh3DData::triangulate`]), for a caller that wants to
    /// attach flat normals as a mesh attribute without this crate's geometry
    /// types ever carrying a normal field themselves.
    ///
    /// Each normal is the polygon's canonical outward normal (see
    /// [`crate::coordinate`]). Errors when the frame's orientation sign cannot
    /// be determined (e.g. an unknown or non-axis-aligned CRS).
    pub fn triangulate_with_normals(
        &mut self,
        cache: &mut Cache,
    ) -> crate::error::Result<Triangulated<TriangularMesh3D>> {
        let sign = self.frame.orientation_sign()? as f64;
        let result = self.data.triangulate(cache);
        let polygon_normals = result
            .polygon_normals
            .into_iter()
            .map(|[x, y, z]| [x * sign, y * sign, z * sign])
            .collect();
        Ok(Triangulated {
            mesh: TriangularMesh3D::new(self.frame.clone(), result.mesh),
            polygon_normals,
            polygon_tris: result.polygon_tris,
        })
    }
}

/// Decode a `<1>`-stride index buffer into the reused `out` buffer as flat
/// `u32`, widening each entry.
fn decode_into(buf: &IndexBuffer<1>, out: &mut Vec<u32>) {
    out.clear();
    match buf {
        IndexBuffer::U8(v) => out.extend(v.iter().map(|&[i]| i as u32)),
        IndexBuffer::U16(v) => out.extend(v.iter().map(|&[i]| i as u32)),
        IndexBuffer::U32(v) => out.extend(v.iter().map(|&[i]| i)),
    }
}

/// Build the open-ring layout of the face spanning `[start, end)` of
/// `face_indices` for triangulation. Fills `open_src` with the face-local position
/// of each corner (each ring's closing duplicate dropped) and `holes` with the
/// start offset of each hole ring into `open_src`.
pub(crate) fn build_open_rings(
    face_indices: &[u32],
    interior_offsets: &[u32],
    start: usize,
    end: usize,
    open_src: &mut Vec<u32>,
    holes: &mut Vec<u32>,
) {
    open_src.clear();
    holes.clear();
    let mut ring_start = start;
    for &offset in interior_offsets {
        let boundary = offset as usize;
        if boundary <= start || boundary >= end {
            continue;
        }
        push_open_ring(face_indices, start, ring_start, boundary, open_src);
        holes.push(open_src.len() as u32);
        ring_start = boundary;
    }
    push_open_ring(face_indices, start, ring_start, end, open_src);
}

/// Append the face-local positions of ring `[ring_start, ring_end)` to `open_src`,
/// dropping the ring's closing vertex when it duplicates the first.
fn push_open_ring(
    face_indices: &[u32],
    start: usize,
    ring_start: usize,
    ring_end: usize,
    open_src: &mut Vec<u32>,
) {
    let open_end =
        if ring_end - ring_start >= 2 && face_indices[ring_start] == face_indices[ring_end - 1] {
            ring_end - 1
        } else {
            ring_end
        };
    open_src.extend((ring_start..open_end).map(|p| (p - start) as u32));
}

impl Split for PolygonMesh2D {
    fn split(
        &mut self,
        emit: &mut dyn FnMut(Geometry, Attributes),
    ) -> Result<(), UnsupportedOperation> {
        self.for_each_face_polygon(|polygon| {
            emit(
                Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(polygon))),
                Attributes::new(),
            );
        });
        Ok(())
    }
}

impl Split for PolygonMesh3D {
    fn split(
        &mut self,
        emit: &mut dyn FnMut(Geometry, Attributes),
    ) -> Result<(), UnsupportedOperation> {
        self.for_each_face_polygon(|polygon| {
            emit(
                Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(polygon))),
                Attributes::new(),
            );
        });
        Ok(())
    }
}

use crate::ops::{ForceTwoDimension, ForceTwoDimensionError};

impl ForceTwoDimension for PolygonMesh2D {
    fn force_2d(&mut self) -> Result<Euclidean2DGeometry, ForceTwoDimensionError> {
        let frame = self.frame.demote_to_2d()?;
        self.z = None; // drop any 2.5D elevation; topology and appearance carry over
        Ok(Euclidean2DGeometry::PolygonMesh(Box::new(PolygonMesh2D {
            frame,
            vertices: std::mem::take(&mut self.vertices),
            z: None,
            face_indices: std::mem::replace(&mut self.face_indices, IndexBuffer::U8(Vec::new())),
            face_offsets: std::mem::replace(&mut self.face_offsets, IndexBuffer::U8(Vec::new())),
            interior_offsets: std::mem::replace(
                &mut self.interior_offsets,
                IndexBuffer::U8(Vec::new()),
            ),
            appearance: self.appearance.take(),
        })))
    }
}

impl ForceTwoDimension for PolygonMesh3D {
    fn force_2d(&mut self) -> Result<Euclidean2DGeometry, ForceTwoDimensionError> {
        let frame = self.frame.demote_to_2d()?;
        let vertices = std::mem::take(&mut self.data.vertices)
            .into_iter()
            .map(|[x, y, _]| [x, y])
            .collect();
        Ok(Euclidean2DGeometry::PolygonMesh(Box::new(PolygonMesh2D {
            frame,
            vertices,
            z: None,
            face_indices: std::mem::replace(
                &mut self.data.face_indices,
                IndexBuffer::U8(Vec::new()),
            ),
            face_offsets: std::mem::replace(
                &mut self.data.face_offsets,
                IndexBuffer::U8(Vec::new()),
            ),
            interior_offsets: std::mem::replace(
                &mut self.data.interior_offsets,
                IndexBuffer::U8(Vec::new()),
            ),
            appearance: self.data.appearance.take(),
        })))
    }
}

impl PolygonMesh2D {
    /// The 3D counterpart of this leaf, with every coordinate placed at the
    /// elevation the leaf lies at, or at `0.0` when it carries none.
    pub(crate) fn into_3d(self) -> PolygonMesh3D {
        PolygonMesh3D::new(
            self.frame,
            PolygonMesh3DData {
                vertices: lift_coords(self.vertices.iter(), self.z),
                face_indices: self.face_indices,
                face_offsets: self.face_offsets,
                interior_offsets: self.interior_offsets,
                appearance: self.appearance,
            },
        )
    }
}

use crate::ops::{
    emit_face_2d, emit_face_3d, CountHoles, ExtractHoles, ExtractedPart, RemoveAppearance,
};

// `interior_offsets` already spans every face, so the mesh-wide count is its
// length — no per-face walk, and no risk of counting a ring twice.
impl CountHoles for PolygonMesh2D {
    fn count_holes(&self) -> usize {
        self.interior_offsets.len()
    }
}

impl CountHoles for PolygonMesh3D {
    fn count_holes(&self) -> usize {
        self.data().num_holes()
    }
}

// A mesh is an aggregate of faces, so it deaggregates: every face contributes its
// own outer shell, and its holes come out alongside.
impl ExtractHoles for PolygonMesh2D {
    fn extract_holes(
        &self,
        emit: &mut dyn FnMut(Geometry, ExtractedPart),
    ) -> Result<(), UnsupportedOperation> {
        self.for_each_face_polygon(|face| {
            emit_face_2d(&face, emit);
        });
        Ok(())
    }
}

impl ExtractHoles for PolygonMesh3D {
    fn extract_holes(
        &self,
        emit: &mut dyn FnMut(Geometry, ExtractedPart),
    ) -> Result<(), UnsupportedOperation> {
        self.for_each_face_polygon(|face| {
            emit_face_3d(&face, emit);
        });
        Ok(())
    }
}

impl RemoveAppearance for PolygonMesh2D {
    fn remove_appearance(&mut self) {
        *self.appearance_mut() = None;
    }
}

impl RemoveAppearance for PolygonMesh3D {
    fn remove_appearance(&mut self) {
        *self.appearance_mut() = None;
    }
}

use crate::ops::coerce::{push_face_lines_2d, push_face_lines_3d, unchanged, wrap_2d, wrap_3d};
use crate::ops::{Coerce, CoercionTarget};

impl Coerce for PolygonMesh2D {
    fn coerce(
        &mut self,
        target: CoercionTarget,
        cache: &mut Cache,
    ) -> Result<Geometry, UnsupportedOperation> {
        match target {
            CoercionTarget::TriangularMesh => self.triangulate(cache),
            CoercionTarget::Polygon => {
                let mut faces = Vec::new();
                self.for_each_face_polygon(|face| {
                    faces.push(Euclidean2DGeometry::Polygon(Box::new(face)))
                });
                wrap_2d(faces).ok_or_else(unchanged::<Self>)
            }
            CoercionTarget::LineString => {
                let mut lines = Vec::new();
                self.for_each_face_polygon(|face| push_face_lines_2d(&face, &mut lines));
                wrap_2d(lines).ok_or_else(unchanged::<Self>)
            }
        }
    }
}

impl Coerce for PolygonMesh3D {
    fn coerce(
        &mut self,
        target: CoercionTarget,
        cache: &mut Cache,
    ) -> Result<Geometry, UnsupportedOperation> {
        match target {
            CoercionTarget::TriangularMesh => self.triangulate(cache),
            CoercionTarget::Polygon => {
                let mut faces = Vec::new();
                self.for_each_face_polygon(|face| {
                    faces.push(Euclidean3DGeometry::Polygon(Box::new(face)))
                });
                wrap_3d(faces).ok_or_else(unchanged::<Self>)
            }
            CoercionTarget::LineString => {
                let mut lines = Vec::new();
                self.for_each_face_polygon(|face| push_face_lines_3d(&face, &mut lines));
                wrap_3d(lines).ok_or_else(unchanged::<Self>)
            }
        }
    }
}

#[cfg(feature = "new-geometry")]
use crate::ops::{Footprint, FootprintError, FootprintSink};

#[cfg(feature = "new-geometry")]
impl Footprint for PolygonMesh2D {
    fn footprint(&self, sink: &mut FootprintSink<'_>) -> Result<(), FootprintError> {
        sink.enter(self.frame())?;
        let (face_indices, face_offsets, interior_offsets) = self.csr_buffers();
        super::faces::for_each_face_coords(
            self.vertices(),
            face_indices,
            face_offsets,
            interior_offsets,
            |rings| sink.push_face_2d(rings.iter().map(Vec::as_slice), self.elevation()),
        );
        Ok(())
    }
}

#[cfg(feature = "new-geometry")]
impl Footprint for PolygonMesh3D {
    fn footprint(&self, sink: &mut FootprintSink<'_>) -> Result<(), FootprintError> {
        sink.enter(self.frame())?;
        self.data().footprint_faces(sink);
        Ok(())
    }
}

#[cfg(feature = "new-geometry")]
mod grid_impl {
    //! `DivideByGrid` for the two polygon-mesh leaves.
    //!
    //! Rather than clip a mesh's CSR face data directly, this decomposes the
    //! mesh into its faces as standalone [`Polygon2D`] / [`Polygon3D`] values
    //! -- each carrying its own *slice* of the mesh's appearance (see
    //! [`face_appearance`]), unlike
    //! [`PolygonMesh2D::for_each_face_polygon`], which explicitly does not --
    //! divides each through the already-reviewed [`Polygon::divide_by_grid`]
    //! (which owns the clip and the per-piece appearance rebuild described in
    //! `polygon/ops.rs`), and welds the pieces that land in the same cell
    //! back into one mesh per cell.
    //!
    //! This calls each face's own `divide_by_grid` once with the *whole*
    //! grid (not a fresh single-cell `GridSpec` per cell): each face's own
    //! bounding box already limits it to the cells it can possibly touch, so
    //! nothing is re-derived per cell, and every cell a face lands in is
    //! collected in one pass. Buckets are keyed `(row, col)` in a `BTreeMap`,
    //! which sorts row-major for free, matching the row-major emission every
    //! other leaf promises.
    //!
    //! Coverage is judged once per cell over the *union* of every face's
    //! piece landing there (`Polygon::area_xy`, summed), not per source face:
    //! two faces that only together fill a cell must report `Full`, which a
    //! per-face check cannot see (this is `B1` in the spec; see
    //! `mesh_whose_faces_together_fill_a_cell_reports_full`).

    use std::collections::BTreeMap;

    use super::{PolygonMesh2D, PolygonMesh3D, PolygonMesh3DData};
    use crate::appearance::{
        Appearance, FaceBinding, MaterialIndex, Side, ThemeBinding, UvSet, UvSource,
    };
    use crate::coordinate::CoordinateFrame;
    use crate::ops::grid::{CellCoverage, DivideByGrid, GridCell, GridDivideError, GridSpec};
    use crate::polygon::{Polygon2D, Polygon3D};
    use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

    /// Slice one mesh face's own appearance out of the mesh-wide appearance:
    /// the same material palette (indices don't move -- this is a slice, not
    /// a merge), each theme's binding reduced to this one face's material
    /// (the whole theme is dropped if this face is unbound for it, since a
    /// `Polygon`'s `front` binding is mandatory), and each `Explicit` UV set
    /// sliced to this face's own `corners` range -- the same span its rings
    /// occupy in the per-face polygon this attaches to, so the UV stays
    /// parallel exactly as `Polygon::set_appearance` requires.
    /// `WorldToTexture` needs no slicing at all: it is positional.
    fn face_appearance(
        mesh_appearance: &Option<Appearance>,
        face_index: usize,
        corners: std::ops::Range<usize>,
    ) -> Option<Appearance> {
        let app = mesh_appearance.as_ref()?;

        let single = |binding: &FaceBinding| -> Option<MaterialIndex> {
            match binding {
                FaceBinding::Uniform(idx) => Some(*idx),
                FaceBinding::PerFace(v) => v.get(face_index).copied().flatten(),
            }
        };

        let mut themes = Vec::new();
        for theme in app.themes() {
            let Some(front) = single(&theme.front) else {
                // This face is unbound for this theme's front; `front` is
                // mandatory on a single-face polygon, so the theme cannot be
                // represented for this face at all.
                continue;
            };
            let back = theme.back.as_ref().and_then(single);
            let uv_sets = theme
                .uv_sets
                .iter()
                .filter(|uv| uv.side == Side::Front || back.is_some())
                .map(|uv| UvSet {
                    side: uv.side,
                    channel: uv.channel,
                    uv: match &uv.uv {
                        UvSource::Explicit(arr) => {
                            UvSource::Explicit(arr[corners.clone()].to_vec().into_boxed_slice())
                        }
                        UvSource::WorldToTexture(m) => UvSource::WorldToTexture(*m),
                    },
                })
                .collect();
            themes.push(ThemeBinding {
                theme: theme.theme.clone(),
                front: FaceBinding::Uniform(front),
                back: back.map(FaceBinding::Uniform),
                uv_sets,
            });
        }

        if themes.is_empty() {
            return None;
        }
        let default_theme = if themes.iter().any(|t| t.theme == *app.default_theme()) {
            app.default_theme().clone()
        } else {
            themes[0].theme.clone()
        };
        let mut result = Appearance::from_parts(app.materials().to_vec(), themes, default_theme);
        result.compact_materials();
        Some(result)
    }

    /// Every face of a 2D mesh as a standalone `Polygon2D`, each carrying its
    /// own slice of the mesh's appearance.
    fn faces_2d(mesh: &PolygonMesh2D) -> Vec<Polygon2D> {
        let (face_indices, face_offsets, interior_offsets) = mesh.csr_buffers();
        let frame = mesh.frame();
        let mut out = Vec::new();
        let mut face_index = 0usize;
        let mut cursor = 0usize;
        super::super::faces::for_each_face_coords(
            mesh.vertices(),
            face_indices,
            face_offsets,
            interior_offsets,
            |rings: &[Vec<[f64; 2]>]| {
                let count: usize = rings.iter().map(Vec::len).sum();
                let exterior = rings.first().cloned().unwrap_or_default();
                let interiors: Vec<Vec<[f64; 2]>> = rings.iter().skip(1).cloned().collect();
                let mut poly = Polygon2D::from_rings(frame.clone(), exterior, interiors);
                *poly.appearance_mut() =
                    face_appearance(mesh.appearance(), face_index, cursor..cursor + count);
                out.push(poly);
                face_index += 1;
                cursor += count;
            },
        );
        out
    }

    /// As [`faces_2d`], for a 3D mesh's coordinate-free data.
    fn faces_3d(data: &PolygonMesh3DData, frame: &CoordinateFrame) -> Vec<Polygon3D> {
        let (face_indices, face_offsets, interior_offsets) = data.csr_buffers();
        let mut out = Vec::new();
        let mut face_index = 0usize;
        let mut cursor = 0usize;
        super::super::faces::for_each_face_coords(
            data.vertices(),
            face_indices,
            face_offsets,
            interior_offsets,
            |rings: &[Vec<[f64; 3]>]| {
                let count: usize = rings.iter().map(Vec::len).sum();
                let exterior = rings.first().cloned().unwrap_or_default();
                let interiors: Vec<Vec<[f64; 3]>> = rings.iter().skip(1).cloned().collect();
                let mut poly = Polygon3D::from_rings(frame.clone(), exterior, interiors);
                *poly.appearance_mut() =
                    face_appearance(&data.appearance, face_index, cursor..cursor + count);
                out.push(poly);
                face_index += 1;
                cursor += count;
            },
        );
        out
    }

    /// Unwrap what a `Polygon2D::divide_by_grid` emitted for one cell (a
    /// single `Polygon`, or a `Collection` of several) into `out`.
    fn collect_faces_2d(geom: Geometry, out: &mut Vec<Polygon2D>) {
        match geom {
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(p)) => out.push(*p),
            Geometry::Euclidean2D(Euclidean2DGeometry::Collection(coll)) => {
                for m in coll.members() {
                    if let Euclidean2DGeometry::Polygon(p) = m {
                        out.push((**p).clone());
                    }
                }
            }
            _ => {}
        }
    }

    /// As [`collect_faces_2d`], for the 3D leaf.
    fn collect_faces_3d(geom: Geometry, out: &mut Vec<Polygon3D>) {
        match geom {
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(p)) => out.push(*p),
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(coll)) => {
                for m in coll.members() {
                    if let Euclidean3DGeometry::Polygon(p) = m {
                        out.push((**p).clone());
                    }
                }
            }
            _ => {}
        }
    }

    /// Divide every face against `grid`, bucketing the pieces by the cell
    /// they land in (row-major, via the `(row, col)` key order). A face whose
    /// own `divide_by_grid` reports [`GridDivideError::Empty`] (a degenerate
    /// face with no area) is skipped rather than failing the whole mesh;
    /// any other error propagates.
    fn bucket<P>(
        faces: &[impl DivideByGrid],
        grid: &GridSpec,
        collect_one: impl Fn(Geometry, &mut Vec<P>),
    ) -> Result<BTreeMap<(i64, i64), Vec<P>>, GridDivideError> {
        let mut buckets: BTreeMap<(i64, i64), Vec<P>> = BTreeMap::new();
        for face in faces {
            let result = face.divide_by_grid(grid, &mut |cell, _coverage, geom| {
                collect_one(geom, buckets.entry((cell.row, cell.col)).or_default());
            });
            match result {
                Ok(()) | Err(GridDivideError::Empty) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(buckets)
    }

    /// Whether every ring of `a` and `b` (exterior then holes, in order) is
    /// positionally identical.
    fn rings_eq_2d(a: &Polygon2D, b: &Polygon2D) -> bool {
        a.exterior() == b.exterior() && a.interiors().eq(b.interiors())
    }

    /// As [`rings_eq_2d`], for the 3D leaf.
    fn rings_eq_3d(a: &Polygon3D, b: &Polygon3D) -> bool {
        a.exterior() == b.exterior() && a.interiors().eq(b.interiors())
    }

    /// Whether `buckets` shows the division touched nothing: every face
    /// landed in the *same* single cell, contributing exactly one piece each
    /// (no split, no drop), each positionally identical to its source face.
    ///
    /// This is the mesh-level analogue of `polygon/ops.rs`'s
    /// `corner_layout_unchanged`, needed for the same reason: re-welding
    /// through [`PolygonMesh3D::from_polygons`] is not a lossless round trip
    /// even when every piece is individually untouched -- the weld rebuilds
    /// the material palette from scratch (no dedup against the original) and
    /// *bakes* any `WorldToTexture` UV into `Explicit` (a welded mesh's faces
    /// cannot share one matrix). A mesh this check confirms unchanged must
    /// bypass the weld entirely and hand back the source verbatim, or a
    /// "genuine cut" reduction gets applied to a mesh nothing ever cut.
    ///
    /// Bucket completeness (one piece per face, not fewer) rules out a face
    /// silently dropped as degenerate; a face actually split by any grid line
    /// would necessarily also land a piece in a neighbouring cell, so a
    /// single surviving bucket already rules out a split -- the per-piece
    /// position check is kept anyway as a direct, observable confirmation
    /// rather than relying on that argument alone.
    fn mesh_unchanged<P>(
        buckets: &BTreeMap<(i64, i64), Vec<P>>,
        faces: &[P],
        eq: impl Fn(&P, &P) -> bool,
    ) -> Option<(i64, i64)> {
        if buckets.len() != 1 {
            return None;
        }
        let (&key, pieces) = buckets.iter().next().expect("checked len == 1");
        let unchanged =
            pieces.len() == faces.len() && faces.iter().zip(pieces.iter()).all(|(f, p)| eq(f, p));
        unchanged.then_some(key)
    }

    impl DivideByGrid for PolygonMesh2D {
        fn divide_by_grid(
            &self,
            grid: &GridSpec,
            emit: &mut dyn FnMut(GridCell, CellCoverage, Geometry),
        ) -> Result<(), GridDivideError> {
            if self.num_faces() == 0 {
                return Err(GridDivideError::Empty);
            }
            let faces = faces_2d(self);
            let buckets = bucket(&faces, grid, collect_faces_2d)?;
            let cell_area = grid.cell_size() * grid.cell_size();

            if let Some((row, col)) = mesh_unchanged(&buckets, &faces, rings_eq_2d) {
                // Nothing was cut: hand back the source mesh verbatim rather
                // than re-welding it (see `mesh_unchanged`'s doc comment for
                // why the weld is not a no-op here even when every piece is).
                let area: f64 = buckets[&(row, col)].iter().map(Polygon2D::area_xy).sum();
                emit(
                    GridCell { row, col },
                    CellCoverage::from_area(area, cell_area),
                    Geometry::Euclidean2D(Euclidean2DGeometry::PolygonMesh(Box::new(self.clone()))),
                );
                return Ok(());
            }

            for ((row, col), pieces) in buckets {
                if pieces.is_empty() {
                    continue;
                }
                let area: f64 = pieces.iter().map(Polygon2D::area_xy).sum();
                let coverage = CellCoverage::from_area(area, cell_area);

                // Weld through the 3D constructor (`PolygonMesh2D` has none
                // of its own): elevation plays no part in a grid clip (it
                // only cuts in XY), so the pieces' own elevation is
                // discarded and the *mesh's* single scalar elevation is
                // reattached directly below, verbatim.
                let pieces_3d: Vec<Polygon3D> =
                    pieces.into_iter().map(Polygon2D::into_3d).collect();
                let welded = PolygonMesh3D::from_polygons(self.frame().clone(), pieces_3d.iter())
                    .map_err(|e| GridDivideError::InvalidSpec(e.to_string()))?;
                let PolygonMesh3DData {
                    vertices,
                    face_indices,
                    face_offsets,
                    interior_offsets,
                    appearance,
                } = welded.into_data();
                let mesh = PolygonMesh2D {
                    frame: self.frame().clone(),
                    vertices: vertices.into_iter().map(|[x, y, _]| [x, y]).collect(),
                    z: self.elevation(),
                    face_indices,
                    face_offsets,
                    interior_offsets,
                    appearance,
                };
                emit(
                    GridCell { row, col },
                    coverage,
                    Geometry::Euclidean2D(Euclidean2DGeometry::PolygonMesh(Box::new(mesh))),
                );
            }
            Ok(())
        }
    }

    impl DivideByGrid for PolygonMesh3D {
        fn divide_by_grid(
            &self,
            grid: &GridSpec,
            emit: &mut dyn FnMut(GridCell, CellCoverage, Geometry),
        ) -> Result<(), GridDivideError> {
            if self.num_faces() == 0 {
                return Err(GridDivideError::Empty);
            }
            let faces = faces_3d(self.data(), self.frame());
            let buckets = bucket(&faces, grid, collect_faces_3d)?;
            let cell_area = grid.cell_size() * grid.cell_size();

            if let Some((row, col)) = mesh_unchanged(&buckets, &faces, rings_eq_3d) {
                let area: f64 = buckets[&(row, col)].iter().map(Polygon3D::area_xy).sum();
                emit(
                    GridCell { row, col },
                    CellCoverage::from_area(area, cell_area),
                    Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(Box::new(self.clone()))),
                );
                return Ok(());
            }

            for ((row, col), pieces) in buckets {
                if pieces.is_empty() {
                    continue;
                }
                let area: f64 = pieces.iter().map(Polygon3D::area_xy).sum();
                let coverage = CellCoverage::from_area(area, cell_area);
                let mesh = PolygonMesh3D::from_polygons(self.frame().clone(), pieces.iter())
                    .map_err(|e| GridDivideError::InvalidSpec(e.to_string()))?;
                emit(
                    GridCell { row, col },
                    coverage,
                    Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(Box::new(mesh))),
                );
            }
            Ok(())
        }
    }
}

#[cfg(feature = "new-geometry")]
impl PolygonMesh3DData {
    /// Push every face into an entered `sink`.
    pub(crate) fn footprint_faces(&self, sink: &mut FootprintSink<'_>) {
        let (face_indices, face_offsets, interior_offsets) = self.csr_buffers();
        super::faces::for_each_face_coords(
            self.vertices(),
            face_indices,
            face_offsets,
            interior_offsets,
            |rings| sink.push_face_3d(rings.iter().map(Vec::as_slice)),
        );
    }
}

#[cfg(feature = "new-geometry")]
use crate::ops::area::polygon_3d_surface_area;
#[cfg(feature = "new-geometry")]
use crate::ops::Area;

#[cfg(feature = "new-geometry")]
impl Area for PolygonMesh2D {
    /// Each face's planar area, summed. A 2D mesh has no elevation to slope.
    fn surface_area(&self) -> Result<f64, UnsupportedOperation> {
        let mut total = 0.0;
        self.for_each_face_polygon(|face| total += face.area());
        Ok(total)
    }
}

#[cfg(feature = "new-geometry")]
impl Area for PolygonMesh3D {
    fn surface_area(&self) -> Result<f64, UnsupportedOperation> {
        let mut total = 0.0;
        self.for_each_face_polygon(|face| total += polygon_3d_surface_area(&face));
        Ok(total)
    }
}

use crate::ops::boundary::{Boundary, ExtractBoundary};
use crate::ops::{surface_boundary_2d, surface_boundary_3d, BoundaryEdges};

fn csr_boundary_edges(
    face_indices: &IndexBuffer<1>,
    face_offsets: &IndexBuffer<1>,
    interior_offsets: &IndexBuffer<1>,
) -> BoundaryEdges {
    let mut edges = BoundaryEdges::new();
    super::faces::for_each_ring(face_indices, face_offsets, interior_offsets, |ring, _| {
        edges.add_ring(ring)
    });
    edges
}

// A hole ring no neighbouring face fills is walked once, like any outer edge, so
// it bounds the surface too.
impl ExtractBoundary for PolygonMesh2D {
    fn extract_boundary(&self) -> Result<Boundary, UnsupportedOperation> {
        let (face_indices, face_offsets, interior_offsets) = self.csr_buffers();
        Ok(surface_boundary_2d(
            self.frame(),
            self.vertices(),
            self.elevation(),
            csr_boundary_edges(face_indices, face_offsets, interior_offsets),
        )
        .into())
    }
}

impl ExtractBoundary for PolygonMesh3D {
    fn extract_boundary(&self) -> Result<Boundary, UnsupportedOperation> {
        let (face_indices, face_offsets, interior_offsets) = self.data().csr_buffers();
        Ok(surface_boundary_3d(
            self.frame(),
            self.vertices(),
            csr_boundary_edges(face_indices, face_offsets, interior_offsets),
        )
        .into())
    }
}

#[cfg(feature = "new-geometry")]
use crate::ops::Elevation;

#[cfg(feature = "new-geometry")]
impl Elevation for PolygonMesh2D {
    fn elevation(&self) -> Option<f64> {
        self.z
    }
}

#[cfg(feature = "new-geometry")]
impl Elevation for PolygonMesh3D {
    fn elevation(&self) -> Option<f64> {
        self.data().first_face_elevation()
    }
}

#[cfg(feature = "new-geometry")]
impl PolygonMesh3DData {
    /// The z of the first face's first exterior vertex. The CSR index buffer
    /// begins with that face's exterior ring, so this is where the mesh's
    /// traversal starts — the vertex pool's own order is unrelated.
    pub(crate) fn first_face_elevation(&self) -> Option<f64> {
        let (face_indices, _, _) = self.csr_buffers();
        let [i] = face_indices.iter_u32().next()?;
        Some(self.vertices()[i as usize][2])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::{CoordinateFrame, EpsgCode};

    #[test]
    fn polygon_mesh2d_box_spans_vertex_pool() {
        let m = PolygonMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [3.0, 0.0], [3.0, 2.0]],
            vec![vec![0u32, 1, 2]],
        )
        .unwrap();
        assert_eq!(
            m.bounding_box().unwrap(),
            Aabb::D2 {
                min: [0.0, 0.0],
                max: [3.0, 2.0]
            }
        );
    }

    #[test]
    fn polygon_mesh3d_box_spans_vertex_pool() {
        let m = PolygonMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 1.0], [3.0, 2.0, -1.0]],
            vec![vec![0u32, 1, 2]],
        )
        .unwrap();
        assert_eq!(
            m.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, 0.0, -1.0],
                max: [3.0, 2.0, 1.0]
            }
        );
    }

    #[test]
    fn polygon_mesh2d_triangulates_every_face() {
        // Two quads sharing edge 1-2; each quad -> 2 triangles.
        let vertices = vec![
            [0.0, 0.0],
            [2.0, 0.0],
            [2.0, 2.0],
            [0.0, 2.0],
            [4.0, 0.0],
            [4.0, 2.0],
        ];
        let mut mesh = PolygonMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vertices,
            vec![vec![0u32, 1, 2, 3], vec![1, 4, 5, 2]],
        )
        .unwrap();
        let mesh_box = mesh.bounding_box().unwrap();
        let g = mesh.triangulate(&mut Cache::new()).unwrap();
        let tm = match &g {
            Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(m)) => m,
            _ => panic!("expected a 2D triangular mesh"),
        };
        assert_eq!(tm.num_triangles(), 4);
        assert_eq!(g.bounding_box().unwrap(), mesh_box);
    }

    #[test]
    fn polygon_mesh3d_triangulates_quad_in_plane() {
        let mut mesh = PolygonMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
            ],
            vec![vec![0u32, 1, 2, 3]],
        )
        .unwrap();
        let mesh_box = mesh.bounding_box().unwrap();
        let g = mesh.triangulate(&mut Cache::new()).unwrap();
        let tm = match &g {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => m,
            _ => panic!("expected a 3D triangular mesh"),
        };
        assert_eq!(tm.num_triangles(), 2);
        assert_eq!(g.bounding_box().unwrap(), mesh_box);
    }

    #[test]
    fn triangulate_with_normals_orients_normal_by_frame_sign() {
        // A quad wound CCW in the z = 0 plane: its right-hand-rule normal is +Z.
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 2.0, 0.0],
            [0.0, 2.0, 0.0],
        ];
        let build = |frame| {
            PolygonMesh3D::from_parts(frame, vertices.clone(), vec![vec![0u32, 1, 2, 3]]).unwrap()
        };

        // A right-handed (Euclidean) frame keeps the raw +Z as the outward normal.
        let mut mesh = build(CoordinateFrame::Euclidean);
        assert_eq!(
            mesh.triangulate_with_normals(&mut Cache::new())
                .unwrap()
                .polygon_normals,
            vec![[0.0, 0.0, 1.0]]
        );

        // EPSG:6697 is lat-first (orientation sign -1), so the same winding is
        // canonically the opposite orientation: the normal comes out flipped,
        // without the caller reprojecting into a right-handed frame first.
        let mut mesh = build(CoordinateFrame::Crs(EpsgCode::new(6697)));
        assert_eq!(
            mesh.triangulate_with_normals(&mut Cache::new())
                .unwrap()
                .polygon_normals,
            vec![[0.0, 0.0, -1.0]]
        );
    }

    #[test]
    fn polygon_mesh3d_triangulates_face_with_hole() {
        // One face: a square exterior with a square hole, given via raw CSR
        // (interior_offsets marks where the hole ring starts in face_indices).
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 4.0, 0.0],
            [0.0, 4.0, 0.0],
            [1.0, 1.0, 0.0],
            [3.0, 1.0, 0.0],
            [3.0, 3.0, 0.0],
            [1.0, 3.0, 0.0],
        ];
        let mut mesh = PolygonMesh3D::from_raw_parts(
            CoordinateFrame::Euclidean,
            vertices,
            vec![0, 1, 2, 3, 4, 5, 6, 7], // face_indices: exterior then hole
            vec![],                       // one face
            vec![4],                      // hole ring starts at index 4
        )
        .unwrap();
        let g = mesh.triangulate(&mut Cache::new()).unwrap();
        let tm = match &g {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => m,
            _ => panic!("expected a 3D triangular mesh"),
        };
        // A square ring with a square hole tessellates into 8 triangles.
        assert_eq!(tm.num_triangles(), 8);
    }

    #[test]
    fn polygon_mesh3d_triangulates_holed_polygon_with_closed_rings() {
        // Built via `from_polygons`, so the exterior and hole rings are stored
        // closed; triangulation must drop each closing vertex before earcut.
        use crate::polygon::Polygon3D;

        let exterior = [
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 4.0, 0.0],
            [0.0, 4.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let hole = vec![
            [1.0, 1.0, 0.0],
            [3.0, 1.0, 0.0],
            [3.0, 3.0, 0.0],
            [1.0, 3.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let p = Polygon3D::from_rings(CoordinateFrame::Euclidean, exterior, vec![hole]);
        let mut mesh = PolygonMesh3D::from_polygons(CoordinateFrame::Euclidean, [&p]).unwrap();
        let g = mesh.triangulate(&mut Cache::new()).unwrap();
        let tm = match &g {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => m,
            _ => panic!("expected a 3D triangular mesh"),
        };
        assert_eq!(tm.num_triangles(), 8);
    }

    #[test]
    fn triangulation_expands_per_face_binding() {
        use crate::appearance::{FaceBinding, MaterialIndex};
        use crate::polygon::Polygon3D;
        use crate::test_support::{textured, theme, uv};

        let ring_a = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let ring_b = [
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [3.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
            [2.0, 0.0, 0.0],
        ];
        let mut a = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            ring_a,
            Vec::<Vec<[f64; 3]>>::new(),
        );
        a.set_appearance(theme("rgb"), textured(), Some(uv(5)))
            .unwrap();
        let mut b = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            ring_b,
            Vec::<Vec<[f64; 3]>>::new(),
        );
        b.set_appearance(theme("rgb"), textured(), Some(uv(5)))
            .unwrap();

        let mut mesh = PolygonMesh3D::from_polygons(CoordinateFrame::Euclidean, [&a, &b]).unwrap();
        assert!(matches!(
            mesh.appearance().as_ref().unwrap().themes()[0].front,
            FaceBinding::PerFace(_)
        ));

        let g = mesh.triangulate(&mut Cache::new()).unwrap();
        let tm = match &g {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => m,
            _ => panic!("expected a 3D triangular mesh"),
        };
        // Each quad -> 2 triangles; the per-face binding expands to one entry per triangle.
        assert_eq!(tm.num_triangles(), 4);
        let FaceBinding::PerFace(front) = &tm.appearance().as_ref().unwrap().themes()[0].front
        else {
            panic!("expected PerFace");
        };
        assert_eq!(
            front,
            &[
                MaterialIndex::new(0),
                MaterialIndex::new(0),
                MaterialIndex::new(1),
                MaterialIndex::new(1),
            ]
        );
    }

    #[test]
    fn triangulation_regathers_uv_per_corner_for_uniform_mesh() {
        use crate::appearance::{
            Appearance, FaceBinding, MaterialIndex, Side, ThemeBinding, UvSet, UvSource,
        };
        use crate::test_support::{textured, theme};

        // A uniform-bound mesh is only reachable by direct construction (welding
        // always yields PerFace).
        let mut data = PolygonMesh3DData::from_parts(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
            ],
            vec![vec![0u32, 1, 2, 3]],
        )
        .unwrap();
        let src = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        data.appearance = Some(Appearance::from_parts(
            vec![textured()],
            vec![ThemeBinding {
                theme: theme("rgb"),
                front: FaceBinding::Uniform(MaterialIndex::new(0).unwrap()),
                back: None,
                uv_sets: vec![UvSet {
                    side: Side::Front,
                    channel: Default::default(),
                    uv: UvSource::Explicit(Box::new(src)),
                }],
            }],
            theme("rgb"),
        ));

        let result = data.triangulate(&mut Cache::new());
        let mesh = TriangularMesh3D::new(CoordinateFrame::Euclidean, result.mesh);
        assert_eq!(mesh.num_triangles(), 2);

        let app = mesh.appearance().as_ref().unwrap();
        let UvSource::Explicit(out_uv) = &app.themes()[0].uv_sets[0].uv else {
            panic!("expected an explicit output UV set");
        };
        assert_eq!(out_uv.len(), 6);
        assert!(out_uv.iter().all(|uv| src.contains(uv)));
        assert!(matches!(
            mesh.appearance().as_ref().unwrap().themes()[0].front,
            FaceBinding::Uniform(_)
        ));
    }

    #[test]
    fn polygon_mesh3d_force_2d_keeps_topology_and_demotes_the_frame() {
        // One square face with one square hole, so all three CSR buffers carry
        // content that must land back in the matching field.
        let mut mesh = PolygonMesh3D::from_raw_parts(
            CoordinateFrame::Crs(EpsgCode::new(6697)),
            vec![
                [0.0, 0.0, 9.0],
                [4.0, 0.0, 9.0],
                [4.0, 4.0, 9.0],
                [0.0, 4.0, 9.0],
                [1.0, 1.0, 9.0],
                [3.0, 1.0, 9.0],
                [3.0, 3.0, 9.0],
                [1.0, 3.0, 9.0],
            ],
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![],
            vec![4],
        )
        .unwrap();
        let forced = match mesh.force_2d().unwrap() {
            Euclidean2DGeometry::PolygonMesh(m) => m,
            other => panic!("expected a 2D polygon mesh, got {other:?}"),
        };
        assert_eq!(forced.frame(), &CoordinateFrame::Crs(EpsgCode::new(6668)));
        assert_eq!(forced.vertices()[0], [0.0, 0.0]);
        assert_eq!(forced.vertices()[4], [1.0, 1.0]);
        assert_eq!(forced.num_faces(), 1);
        let mut faces = Vec::new();
        forced.for_each_face_polygon(|p| faces.push(p));
        assert_eq!(faces[0].exterior().len(), 4);
        assert_eq!(faces[0].interiors().count(), 1);
    }

    #[test]
    fn polygon_mesh2d_force_2d_clears_elevation() {
        let mut mesh = PolygonMesh2D::from_parts_at_elevation(
            CoordinateFrame::Crs(EpsgCode::new(6697)),
            vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0]],
            vec![vec![0u32, 1, 2]],
            5.0,
        )
        .unwrap();
        let forced = match mesh.force_2d().unwrap() {
            Euclidean2DGeometry::PolygonMesh(m) => m,
            other => panic!("expected a 2D polygon mesh, got {other:?}"),
        };
        assert_eq!(forced.frame(), &CoordinateFrame::Crs(EpsgCode::new(6668)));
        assert_eq!(forced.num_faces(), 1);
        assert_eq!(
            forced.bounding_box().unwrap(),
            Aabb::D2 {
                min: [0.0, 0.0],
                max: [2.0, 2.0]
            }
        );
    }
}
