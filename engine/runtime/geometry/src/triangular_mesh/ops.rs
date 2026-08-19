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

use crate::ops::boundary::ExtractBoundary;
use crate::ops::{surface_boundary_2d, surface_boundary_3d, BoundaryEdges};

fn triangle_boundary_edges(triangles: impl Iterator<Item = [u32; 3]>) -> BoundaryEdges {
    let mut edges = BoundaryEdges::new();
    for triangle in triangles {
        edges.add_triangle(triangle);
    }
    edges
}

impl ExtractBoundary for TriangularMesh2D {
    fn extract_boundary(&self) -> Result<Geometry, UnsupportedOperation> {
        Ok(surface_boundary_2d(
            self.frame(),
            self.vertices(),
            self.elevation(),
            triangle_boundary_edges(self.triangles()),
        ))
    }
}

impl ExtractBoundary for TriangularMesh3D {
    fn extract_boundary(&self) -> Result<Geometry, UnsupportedOperation> {
        Ok(surface_boundary_3d(
            self.frame(),
            self.vertices(),
            triangle_boundary_edges(self.triangles()),
        ))
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
