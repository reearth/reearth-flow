use super::{TriangularMesh2D, TriangularMesh3D, TriangularMesh3DData};
use crate::coordinate::{CoordinateFrame, EpsgCode};
use crate::ops::reproject::{transform_coords_2d, transform_coords_3d};
use crate::ops::{
    lift_coords, Aabb, BoundingBox, Reproject, ReprojectionCache, UnsupportedOperation,
};

use reearth_flow_common::attribute::Attributes;

use crate::ops::Split;
use crate::polygon::{Polygon2D, Polygon3D};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

impl BoundingBox for TriangularMesh2D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        Aabb::from_points_2d(self.vertices.iter().copied()).ok_or(UnsupportedOperation {
            geometry: "TriangularMesh2D",
            operation: "bounding_box",
        })
    }
}

impl BoundingBox for TriangularMesh3D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        Aabb::from_points_3d(self.data.vertices.iter().copied()).ok_or(UnsupportedOperation {
            geometry: "TriangularMesh3D",
            operation: "bounding_box",
        })
    }
}

impl TriangularMesh2D {
    /// Move the mesh out, leaving an empty husk.
    fn take(&mut self) -> Self {
        Self {
            frame: std::mem::take(&mut self.frame),
            vertices: std::mem::take(&mut self.vertices),
            z: self.z.take(),
            indices: std::mem::take(&mut self.indices),
            appearance: self.appearance.take(),
        }
    }

    /// The leaf moved out and wrapped as a [`Geometry`].
    fn take_geometry(&mut self) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(Box::new(self.take())))
    }
}

impl TriangularMesh3D {
    /// Move the mesh out, leaving an empty husk.
    fn take(&mut self) -> Self {
        Self {
            frame: std::mem::take(&mut self.frame),
            data: TriangularMesh3DData {
                vertices: std::mem::take(&mut self.data.vertices),
                indices: std::mem::take(&mut self.data.indices),
                appearance: self.data.appearance.take(),
            },
        }
    }

    /// The leaf moved out and wrapped as a [`Geometry`].
    fn take_geometry(&mut self) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(self.take())))
    }
}

impl Reproject for TriangularMesh2D {
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
        Ok(Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(
            Box::new(m),
        )))
    }
}

impl Reproject for TriangularMesh3D {
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
        Ok(Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(
            Box::new(m),
        )))
    }
}

use crate::ops::{plan_frame_step, translate_2d, translate_3d, ConvertFrame, FrameStep, Translate};

impl Translate for TriangularMesh2D {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        translate_2d(&mut self.vertices, &mut self.z, delta);
        Ok(())
    }
}

impl Translate for TriangularMesh3D {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        translate_3d(self.data.vertices_mut(), delta);
        Ok(())
    }
}

impl ConvertFrame for TriangularMesh2D {
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

impl ConvertFrame for TriangularMesh3D {
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

use crate::ops::{area_2d, emit_triangles_3d, ExtractHoles, ExtractedPart};

// A triangle mesh is an aggregate of faces, so it deaggregates like a polygon
// mesh. A triangle carries no interior ring, so every part is an outer shell.
impl ExtractHoles for TriangularMesh2D {
    fn extract_holes(
        &self,
        emit: &mut dyn FnMut(Geometry, ExtractedPart),
    ) -> Result<(), UnsupportedOperation> {
        let vertices = self.vertices();
        let frame = self.frame();
        let elevation = self.elevation();
        for [i, j, k] in self.triangles() {
            let ring = [
                vertices[i as usize],
                vertices[j as usize],
                vertices[k as usize],
                vertices[i as usize],
            ];
            emit(area_2d(frame, ring, elevation), ExtractedPart::Outershell);
        }
        Ok(())
    }
}

impl ExtractHoles for TriangularMesh3D {
    fn extract_holes(
        &self,
        emit: &mut dyn FnMut(Geometry, ExtractedPart),
    ) -> Result<(), UnsupportedOperation> {
        emit_triangles_3d(self.frame(), self.vertices(), self.triangles(), emit);
        Ok(())
    }
}

impl Split for TriangularMesh2D {
    fn split(
        &mut self,
        emit: &mut dyn FnMut(Geometry, Attributes),
    ) -> Result<(), UnsupportedOperation> {
        let vertices = self.vertices();
        let frame = self.frame();
        let elevation = self.elevation();
        for [i, j, k] in self.triangles() {
            let ring = [
                vertices[i as usize],
                vertices[j as usize],
                vertices[k as usize],
                vertices[i as usize],
            ];
            let no_holes = Vec::<Vec<[f64; 2]>>::new();
            let polygon = match elevation {
                None => Polygon2D::from_rings(frame.clone(), ring, no_holes),
                Some(elevation) => {
                    Polygon2D::from_rings_at_elevation(frame.clone(), ring, no_holes, elevation)
                }
            };
            emit(
                Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(polygon))),
                Attributes::new(),
            );
        }
        Ok(())
    }
}

impl Split for TriangularMesh3D {
    fn split(
        &mut self,
        emit: &mut dyn FnMut(Geometry, Attributes),
    ) -> Result<(), UnsupportedOperation> {
        let vertices = self.vertices();
        let frame = self.frame();
        for [i, j, k] in self.triangles() {
            let ring = [
                vertices[i as usize],
                vertices[j as usize],
                vertices[k as usize],
                vertices[i as usize],
            ];
            let polygon = Polygon3D::from_rings(frame.clone(), ring, Vec::<Vec<[f64; 3]>>::new());
            emit(
                Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(polygon))),
                Attributes::new(),
            );
        }
        Ok(())
    }
}

use crate::index::IndexBuffer;
use crate::ops::{ForceTwoDimension, ForceTwoDimensionError};

impl ForceTwoDimension for TriangularMesh2D {
    fn force_2d(&mut self) -> Result<Euclidean2DGeometry, ForceTwoDimensionError> {
        let frame = self.frame.demote_to_2d()?;
        self.z = None; // drop any 2.5D elevation; indices and appearance carry over
        Ok(Euclidean2DGeometry::TriangularMesh(Box::new(
            TriangularMesh2D {
                frame,
                vertices: std::mem::take(&mut self.vertices),
                z: None,
                indices: std::mem::replace(&mut self.indices, IndexBuffer::U8(Vec::new())),
                appearance: self.appearance.take(),
            },
        )))
    }
}

impl ForceTwoDimension for TriangularMesh3D {
    fn force_2d(&mut self) -> Result<Euclidean2DGeometry, ForceTwoDimensionError> {
        let frame = self.frame.demote_to_2d()?;
        let vertices = std::mem::take(&mut self.data.vertices)
            .into_iter()
            .map(|[x, y, _]| [x, y])
            .collect();
        Ok(Euclidean2DGeometry::TriangularMesh(Box::new(
            TriangularMesh2D {
                frame,
                vertices,
                z: None,
                indices: std::mem::replace(&mut self.data.indices, IndexBuffer::U8(Vec::new())),
                appearance: self.data.appearance.take(),
            },
        )))
    }
}

impl TriangularMesh2D {
    /// The 3D counterpart of this leaf, with every coordinate placed at the
    /// elevation the leaf lies at, or at `0.0` when it carries none.
    pub(crate) fn into_3d(self) -> TriangularMesh3D {
        TriangularMesh3D::new(
            self.frame,
            TriangularMesh3DData {
                vertices: lift_coords(self.vertices.iter(), self.z),
                indices: self.indices,
                appearance: self.appearance,
            },
        )
    }
}

use crate::ops::RemoveAppearance;

impl RemoveAppearance for TriangularMesh2D {
    fn remove_appearance(&mut self) {
        *self.appearance_mut() = None;
    }
}

impl RemoveAppearance for TriangularMesh3D {
    fn remove_appearance(&mut self) {
        *self.appearance_mut() = None;
    }
}

use crate::line_string::{LineString2D, LineString3D};
use crate::ops::coerce::{triangle_ring, unchanged, wrap_2d, wrap_3d};
use crate::ops::triangulation::Cache;
use crate::ops::{Coerce, CoercionTarget};

impl Coerce for TriangularMesh2D {
    fn coerce(
        &mut self,
        target: CoercionTarget,
        _cache: &mut Cache,
    ) -> Result<Geometry, UnsupportedOperation> {
        let vertices = self.vertices();
        let frame = self.frame();
        let elevation = self.elevation();
        match target {
            CoercionTarget::TriangularMesh => Err(unchanged::<Self>()),
            CoercionTarget::LineString => {
                let parts = self
                    .triangles()
                    .map(|triangle| {
                        let ring = triangle_ring(vertices, triangle);
                        match elevation {
                            None => Euclidean2DGeometry::LineString(LineString2D::from_coords(
                                frame.clone(),
                                ring,
                            )),
                            Some(elevation) => Euclidean2DGeometry::LineString(
                                LineString2D::from_coords_at_elevation(
                                    frame.clone(),
                                    ring,
                                    elevation,
                                ),
                            ),
                        }
                    })
                    .collect();
                wrap_2d(parts).ok_or_else(unchanged::<Self>)
            }
            CoercionTarget::Polygon => {
                let no_holes = Vec::<Vec<[f64; 2]>>::new();
                let parts =
                    self.triangles()
                        .map(|triangle| {
                            let ring = triangle_ring(vertices, triangle);
                            match elevation {
                                None => Euclidean2DGeometry::Polygon(Box::new(
                                    Polygon2D::from_rings(frame.clone(), ring, no_holes.clone()),
                                )),
                                Some(elevation) => Euclidean2DGeometry::Polygon(Box::new(
                                    Polygon2D::from_rings_at_elevation(
                                        frame.clone(),
                                        ring,
                                        no_holes.clone(),
                                        elevation,
                                    ),
                                )),
                            }
                        })
                        .collect();
                wrap_2d(parts).ok_or_else(unchanged::<Self>)
            }
        }
    }
}

impl Coerce for TriangularMesh3D {
    fn coerce(
        &mut self,
        target: CoercionTarget,
        _cache: &mut Cache,
    ) -> Result<Geometry, UnsupportedOperation> {
        let vertices = self.vertices();
        let frame = self.frame();
        match target {
            CoercionTarget::TriangularMesh => Err(unchanged::<Self>()),
            CoercionTarget::LineString => {
                let parts = self
                    .triangles()
                    .map(|triangle| {
                        let ring = triangle_ring(vertices, triangle);
                        Euclidean3DGeometry::LineString(LineString3D::from_coords(
                            frame.clone(),
                            ring,
                        ))
                    })
                    .collect();
                wrap_3d(parts).ok_or_else(unchanged::<Self>)
            }
            CoercionTarget::Polygon => {
                let parts = self
                    .triangles()
                    .map(|triangle| {
                        let ring = triangle_ring(vertices, triangle);
                        Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
                            frame.clone(),
                            ring,
                            Vec::<Vec<[f64; 3]>>::new(),
                        )))
                    })
                    .collect();
                wrap_3d(parts).ok_or_else(unchanged::<Self>)
            }
        }
    }
}

#[cfg(feature = "new-geometry")]
use crate::ops::{Footprint, FootprintError, FootprintSink};

#[cfg(feature = "new-geometry")]
impl Footprint for TriangularMesh2D {
    fn footprint(&self, sink: &mut FootprintSink<'_>) -> Result<(), FootprintError> {
        sink.enter(self.frame())?;
        let vertices = self.vertices();
        for [i, j, k] in self.triangles() {
            let ring = [
                vertices[i as usize],
                vertices[j as usize],
                vertices[k as usize],
            ];
            sink.push_face_2d(std::iter::once(&ring[..]), self.elevation());
        }
        Ok(())
    }
}

#[cfg(feature = "new-geometry")]
impl Footprint for TriangularMesh3D {
    fn footprint(&self, sink: &mut FootprintSink<'_>) -> Result<(), FootprintError> {
        sink.enter(self.frame())?;
        self.data.footprint_faces(sink);
        Ok(())
    }
}

#[cfg(feature = "new-geometry")]
mod grid_impl {
    //! `DivideByGrid` for the two triangular-mesh leaves.
    //!
    //! Unlike the polygon-mesh leaves (`polygon_mesh/ops.rs`), a triangular
    //! mesh divides its triangles directly against [`clip_to_window`] rather
    //! than detouring through a per-face `Polygon`: a clipped triangle is
    //! already the easy case (a convex 3-to-7-gon), so it is fan-triangulated
    //! straight back rather than reassembled as an n-gon face.
    //!
    //! UV lives differently here than on a `Polygon` / `PolygonMesh`: a
    //! triangular mesh's `Explicit` UV set is parallel to the *triangle
    //! corner* buffer (`3 * num_triangles()` entries, corner `c` of triangle
    //! `t` at `3*t + c`), not to a shared vertex pool -- two triangles
    //! sharing a vertex may legitimately disagree on its UV. Only the default
    //! theme's front, default-channel `Explicit` UV is threaded through
    //! [`Corner`]'s single UV channel (see [`explicit_uv`]); every other
    //! `Explicit` UV set is dropped by [`rebuild_mesh_appearance`], mirroring
    //! the rule `polygon/ops.rs`'s `rebuild_appearance` documents, but
    //! applied uniformly to the whole output mesh rather than per piece: a
    //! mesh aggregates many triangles into one `Appearance`, so there is no
    //! single piece to ask "was this one left untouched by the clip" the way
    //! a lone `Polygon` piece can. `WorldToTexture` still carries through on
    //! any theme, since it is positional and needs no threading.

    use super::{TriangularMesh2D, TriangularMesh3D};
    use crate::appearance::{
        Appearance, ChannelId, FaceBinding, Side, ThemeBinding, UvSet, UvSource,
    };
    use crate::ops::grid::{
        clip_to_window, faces_area_xy, CellCoverage, Corner, DivideByGrid, GridCell,
        GridDivideError, GridSpec,
    };
    use crate::ops::{Aabb, BoundingBox};
    use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

    /// The single scalar elevation a 2D mesh's whole surface lies at (`None`
    /// for a 3D mesh, which has no such field -- its elevation lives in every
    /// vertex's own `z`, already carried by construction). A grid clip only
    /// ever cuts in XY, so this constant is never recomputed from the
    /// divided output -- it is simply reattached to each cell's piece
    /// verbatim, the same way `polygon_mesh/ops.rs`'s `PolygonMesh2D` path
    /// does.
    trait CarryElevation {
        fn elevation_for_grid(&self) -> Option<f64>;
        fn set_elevation_for_grid(&mut self, elevation: Option<f64>);
    }

    impl CarryElevation for TriangularMesh2D {
        fn elevation_for_grid(&self) -> Option<f64> {
            self.elevation()
        }
        fn set_elevation_for_grid(&mut self, elevation: Option<f64>) {
            self.z = elevation;
        }
    }

    impl CarryElevation for TriangularMesh3D {
        fn elevation_for_grid(&self) -> Option<f64> {
            None
        }
        fn set_elevation_for_grid(&mut self, _elevation: Option<f64>) {}
    }

    /// Fan-triangulate a convex ring: vertex 0 to every non-adjacent edge.
    ///
    /// Exact here, because clipping a triangle by a rectangle always yields a
    /// convex polygon, so no fan diagonal can leave the shape.
    fn fan<const N: usize>(ring: &[Corner<N>]) -> Vec<[usize; 3]> {
        (1..ring.len().saturating_sub(1))
            .map(|i| [0, i, i + 1])
            .collect()
    }

    /// The mesh's default-theme, front-side, default-channel UV, borrowed
    /// whole when it is `Explicit` -- the one UV channel a [`Corner`] can
    /// carry through the clip. `WorldToTexture` UV is positional, not
    /// per-corner, so there is nothing to gather; [`rebuild_mesh_appearance`]
    /// carries it through unchanged instead.
    fn explicit_uv(appearance: &Option<Appearance>) -> Option<&[[f64; 2]]> {
        match appearance.as_ref()?.default_uv()? {
            UvSource::Explicit(coords) => Some(coords),
            UvSource::WorldToTexture(_) => None,
        }
    }

    /// Rebuild the appearance for one cell's output mesh.
    ///
    /// Every theme is judged as a genuine cut (there is no mesh-wide
    /// "untouched" fast path, since different source triangles in the same
    /// output mesh can each be touched differently, and the mesh needs one
    /// consistent `Appearance`): a `FaceBinding` is expanded by repeating
    /// each source triangle's material once per fan-triangle it contributed
    /// to this cell (`face_tri_counts`); an `Explicit` UV set survives only
    /// at the default theme's default slot, rebuilt from `gathered_uv`
    /// (already parallel to the output corner buffer); `WorldToTexture`
    /// carries through untouched on any theme. Dropping a front-side set
    /// drops the whole theme (`front` is mandatory); dropping a back-side set
    /// drops just the back binding. See `polygon/ops.rs`'s
    /// `rebuild_appearance` for the same rule applied per piece.
    fn rebuild_mesh_appearance(
        src: &Option<Appearance>,
        gathered_uv: Option<&[[f64; 2]]>,
        face_tri_counts: &[u32],
    ) -> Option<Appearance> {
        let app = src.as_ref()?;
        let default_theme = app.default_theme().clone();
        let (materials, themes, _) = app.clone().into_parts();

        let expand = |binding: FaceBinding| -> FaceBinding {
            match binding {
                FaceBinding::Uniform(index) => FaceBinding::Uniform(index),
                FaceBinding::PerFace(faces) => {
                    debug_assert_eq!(faces.len(), face_tri_counts.len());
                    let total: usize = face_tri_counts.iter().map(|&c| c as usize).sum();
                    let mut per_triangle = Vec::with_capacity(total);
                    for (material, &count) in faces.into_iter().zip(face_tri_counts) {
                        per_triangle.extend(std::iter::repeat_n(material, count as usize));
                    }
                    FaceBinding::PerFace(per_triangle)
                }
            }
        };

        let mut new_themes = Vec::with_capacity(themes.len());
        for theme in themes {
            let ThemeBinding {
                theme: theme_id,
                front,
                mut back,
                uv_sets,
            } = theme;
            let is_default_theme = theme_id == default_theme;
            let mut drop_front = false;
            let mut drop_back = false;

            let mut kept_uv_sets = Vec::with_capacity(uv_sets.len());
            for uv_set in uv_sets {
                let is_default_slot = is_default_theme
                    && uv_set.side == Side::Front
                    && uv_set.channel == ChannelId::default();
                match &uv_set.uv {
                    UvSource::WorldToTexture(_) => kept_uv_sets.push(uv_set),
                    UvSource::Explicit(_) if is_default_slot => match gathered_uv {
                        Some(coords) => kept_uv_sets.push(UvSet {
                            uv: UvSource::Explicit(coords.into()),
                            ..uv_set
                        }),
                        None => match uv_set.side {
                            Side::Front => drop_front = true,
                            Side::Back => drop_back = true,
                        },
                    },
                    UvSource::Explicit(_) => match uv_set.side {
                        Side::Front => drop_front = true,
                        Side::Back => drop_back = true,
                    },
                }
            }

            if drop_back {
                back = None;
                kept_uv_sets.retain(|uv| uv.side != Side::Back);
            }
            if drop_front {
                continue;
            }

            new_themes.push(ThemeBinding {
                theme: theme_id,
                front: expand(front),
                back: back.map(expand),
                uv_sets: kept_uv_sets,
            });
        }

        if new_themes.is_empty() {
            return None;
        }
        let new_default = if new_themes.iter().any(|t| t.theme == default_theme) {
            default_theme
        } else {
            new_themes[0].theme.clone()
        };
        let mut result = Appearance::from_parts(materials, new_themes, new_default);
        result.compact_materials();
        Some(result)
    }

    macro_rules! divide_tri_mesh {
        ($ty:ident, $dim:literal, $wrap:path) => {
            impl DivideByGrid for $ty {
                fn divide_by_grid(
                    &self,
                    grid: &GridSpec,
                    emit: &mut dyn FnMut(GridCell, CellCoverage, Geometry),
                ) -> Result<(), GridDivideError> {
                    let (min, max) = match self.bounding_box()? {
                        Aabb::D2 { min, max } => (min, max),
                        Aabb::D3 { min, max } => ([min[0], min[1]], [max[0], max[1]]),
                    };

                    let tris: Vec<[u32; 3]> = self.triangles().collect();
                    if tris.is_empty() {
                        return Err(GridDivideError::Empty);
                    }
                    let verts = self.vertices();
                    let src_uv = explicit_uv(self.appearance());
                    let elevation = self.elevation_for_grid();

                    let (lo, hi) = grid.cell_range(min, max);
                    let cell_area = grid.cell_size() * grid.cell_size();

                    for row in lo.row..=hi.row {
                        for col in lo.col..=hi.col {
                            let cell = GridCell { row, col };
                            let window = grid.window(cell);

                            let mut out_verts: Vec<[f64; $dim]> = Vec::new();
                            // One entry per `out_verts` vertex -- *not* yet
                            // the mesh's own per-triangle-corner convention.
                            // `out_tris` can (and typically does) reference
                            // the same `out_verts` index from more than one
                            // triangle -- shared fan spokes within one clipped
                            // face, and vertices two *different* source
                            // triangles both landed on after the clip -- and
                            // each such triangle needs its own copy of that
                            // vertex's uv, so the per-triangle-corner array
                            // actually written to the mesh is re-gathered
                            // from this, through `out_tris`, below.
                            let mut vert_uv: Vec<Option<[f64; 2]>> = Vec::new();
                            let mut out_tris: Vec<[u32; 3]> = Vec::new();
                            let mut face_tri_counts: Vec<u32> = vec![0; tris.len()];
                            let mut area = 0.0;

                            for (ti, t) in tris.iter().enumerate() {
                                let ring: Vec<Corner<$dim>> = t
                                    .iter()
                                    .enumerate()
                                    .map(|(corner, &i)| Corner {
                                        pos: verts[i as usize],
                                        uv: src_uv.map(|uv| uv[3 * ti + corner]),
                                    })
                                    .collect();
                                let clipped = clip_to_window(vec![ring], &window);
                                if clipped.is_empty() {
                                    continue;
                                }
                                area += faces_area_xy(&clipped);
                                for face in clipped {
                                    let outline = &face.rings[0];
                                    let fan_tris = fan(outline);
                                    face_tri_counts[ti] += fan_tris.len() as u32;
                                    let base = out_verts.len() as u32;
                                    out_verts.extend(outline.iter().map(|c| c.pos));
                                    vert_uv.extend(outline.iter().map(|c| c.uv));
                                    out_tris.extend(fan_tris.into_iter().map(|[a, b, c]| {
                                        [base + a as u32, base + b as u32, base + c as u32]
                                    }));
                                }
                            }

                            if out_tris.is_empty() {
                                continue;
                            }
                            // Re-gather one uv per *triangle corner* (3 per
                            // output triangle, matching the mesh's own
                            // convention), in exactly `out_tris`' order, from
                            // each referenced vertex's `vert_uv`. Missing even
                            // one (`src_uv` absent for some corner) drops the
                            // whole gather, mirroring the `Option<Vec<_>>`
                            // short-circuit `polygon/ops.rs`'s `build_one`
                            // uses for the same purpose.
                            let out_uv: Option<Vec<[f64; 2]>> = src_uv.and_then(|_| {
                                out_tris
                                    .iter()
                                    .flat_map(|t| t.iter())
                                    .map(|&i| vert_uv[i as usize])
                                    .collect::<Option<Vec<_>>>()
                            });
                            let appearance = rebuild_mesh_appearance(
                                self.appearance(),
                                out_uv.as_deref(),
                                &face_tri_counts,
                            );
                            let mut mesh = $ty::from_parts(
                                self.frame().clone(),
                                out_verts,
                                out_tris.into_iter().flatten(),
                            )
                            .map_err(|e| GridDivideError::InvalidSpec(e.to_string()))?;
                            *mesh.appearance_mut() = appearance;
                            mesh.set_elevation_for_grid(elevation);
                            emit(cell, CellCoverage::from_area(area, cell_area), $wrap(mesh));
                        }
                    }
                    Ok(())
                }
            }
        };
    }

    divide_tri_mesh!(TriangularMesh2D, 2, wrap_tri_2d);
    divide_tri_mesh!(TriangularMesh3D, 3, wrap_tri_3d);

    fn wrap_tri_2d(m: TriangularMesh2D) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(Box::new(m)))
    }

    fn wrap_tri_3d(m: TriangularMesh3D) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(m)))
    }
}

#[cfg(feature = "new-geometry")]
impl TriangularMesh3DData {
    /// Push every triangle into an entered `sink`.
    pub(crate) fn footprint_faces(&self, sink: &mut FootprintSink<'_>) {
        let vertices = self.vertices();
        for [i, j, k] in self.triangles() {
            let ring = [
                vertices[i as usize],
                vertices[j as usize],
                vertices[k as usize],
            ];
            sink.push_face_3d(std::iter::once(&ring[..]));
        }
    }
}

#[cfg(feature = "new-geometry")]
use crate::ops::area::{ring_surface_area, triangle_area_2d, triangle_area_sum_3d};
#[cfg(feature = "new-geometry")]
use crate::ops::Area;

#[cfg(feature = "new-geometry")]
impl Area for TriangularMesh2D {
    /// Each triangle's area, summed. A 2D mesh has no elevation to slope.
    fn surface_area(&self) -> Result<f64, UnsupportedOperation> {
        let vertices = self.vertices();
        Ok(self
            .triangles()
            .map(|t| {
                triangle_area_2d(
                    vertices[t[0] as usize],
                    vertices[t[1] as usize],
                    vertices[t[2] as usize],
                )
            })
            .sum())
    }
}

#[cfg(feature = "new-geometry")]
impl Area for TriangularMesh3D {
    fn surface_area(&self) -> Result<f64, UnsupportedOperation> {
        Ok(triangle_area_sum_3d(
            self.vertices(),
            self.triangles(),
            ring_surface_area,
        ))
    }
}

use crate::ops::boundary::{Boundary, ExtractBoundary};
use crate::ops::{surface_boundary_2d, surface_boundary_3d, BoundaryEdges};

fn triangle_boundary_edges(triangles: impl Iterator<Item = [u32; 3]>) -> BoundaryEdges {
    let mut edges = BoundaryEdges::new();
    for triangle in triangles {
        edges.add_triangle(triangle);
    }
    edges
}

impl ExtractBoundary for TriangularMesh2D {
    fn extract_boundary(&self) -> Result<Boundary, UnsupportedOperation> {
        Ok(surface_boundary_2d(
            self.frame(),
            self.vertices(),
            self.elevation(),
            triangle_boundary_edges(self.triangles()),
        )
        .into())
    }
}

impl ExtractBoundary for TriangularMesh3D {
    fn extract_boundary(&self) -> Result<Boundary, UnsupportedOperation> {
        Ok(surface_boundary_3d(
            self.frame(),
            self.vertices(),
            triangle_boundary_edges(self.triangles()),
        )
        .into())
    }
}

#[cfg(feature = "new-geometry")]
use crate::ops::Elevation;

#[cfg(feature = "new-geometry")]
impl Elevation for TriangularMesh2D {
    fn elevation(&self) -> Option<f64> {
        self.z
    }
}

#[cfg(feature = "new-geometry")]
impl Elevation for TriangularMesh3D {
    fn elevation(&self) -> Option<f64> {
        self.data.first_triangle_elevation()
    }
}

#[cfg(feature = "new-geometry")]
impl TriangularMesh3DData {
    /// The z of the first triangle's first vertex, which is where the mesh's
    /// traversal starts — the vertex pool's own order is unrelated.
    pub(crate) fn first_triangle_elevation(&self) -> Option<f64> {
        let [i, _, _] = self.triangles().next()?;
        Some(self.vertices()[i as usize][2])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;

    #[test]
    fn triangular_mesh2d_box() {
        let m = TriangularMesh2D::from_soup(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [3.0, 0.0], [3.0, 2.0]],
        );
        assert_eq!(
            m.bounding_box().unwrap(),
            Aabb::D2 {
                min: [0.0, 0.0],
                max: [3.0, 2.0]
            }
        );
    }

    #[test]
    fn triangular_mesh3d_box() {
        let m = TriangularMesh3D::from_soup(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [3.0, 0.0, 1.0], [3.0, 2.0, -1.0]],
        );
        assert_eq!(
            m.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, 0.0, -1.0],
                max: [3.0, 2.0, 1.0]
            }
        );
    }

    #[test]
    fn triangular_mesh3d_force_2d_keeps_triangles_and_demotes_the_frame() {
        use crate::coordinate::EpsgCode;

        let mut mesh = TriangularMesh3D::from_parts(
            CoordinateFrame::Crs(EpsgCode::new(6697)),
            vec![
                [0.0, 0.0, 9.0],
                [2.0, 0.0, 8.0],
                [2.0, 2.0, 7.0],
                [0.0, 2.0, 6.0],
            ],
            [0u32, 1, 2, 0, 2, 3],
        )
        .unwrap();
        let forced = match mesh.force_2d().unwrap() {
            Euclidean2DGeometry::TriangularMesh(m) => m,
            other => panic!("expected a 2D triangular mesh, got {other:?}"),
        };
        assert_eq!(forced.frame(), &CoordinateFrame::Crs(EpsgCode::new(6668)));
        assert_eq!(
            forced.vertices(),
            &[[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]]
        );
        assert_eq!(
            forced.triangles().collect::<Vec<_>>(),
            vec![[0, 1, 2], [0, 2, 3]]
        );
    }

    #[test]
    fn triangular_mesh2d_force_2d_clears_elevation() {
        use crate::coordinate::EpsgCode;

        let mut mesh = TriangularMesh2D::from_parts_at_elevation(
            CoordinateFrame::Crs(EpsgCode::new(6697)),
            vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0]],
            [0u32, 1, 2],
            5.0,
        )
        .unwrap();
        let forced = match mesh.force_2d().unwrap() {
            Euclidean2DGeometry::TriangularMesh(m) => m,
            other => panic!("expected a 2D triangular mesh, got {other:?}"),
        };
        assert_eq!(forced.frame(), &CoordinateFrame::Crs(EpsgCode::new(6668)));
        assert_eq!(forced.num_triangles(), 1);
        // The elevation is gone, so the box is purely 2D.
        assert_eq!(
            forced.bounding_box().unwrap(),
            Aabb::D2 {
                min: [0.0, 0.0],
                max: [2.0, 2.0]
            }
        );
    }
}
