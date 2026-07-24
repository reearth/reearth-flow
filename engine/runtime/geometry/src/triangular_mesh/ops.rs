use super::{TriangularMesh2D, TriangularMesh3D};
use crate::coordinate::{CoordinateFrame, EpsgCode};
use crate::ops::reproject::{transform_coords_2d, transform_coords_3d};
use crate::ops::{Aabb, BoundingBox, Reproject, ReprojectionCache, UnsupportedOperation};

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

impl Reproject for TriangularMesh2D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        let from = self.frame.require_crs()?;
        if from != target {
            transform_coords_2d(
                cache,
                from,
                target,
                &mut self.vertices,
                self.z.as_deref_mut(),
            )?;
            self.frame = CoordinateFrame::Crs(target);
        }
        Ok(())
    }
}

impl Reproject for TriangularMesh3D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        let from = self.frame.require_crs()?;
        if from != target {
            transform_coords_3d(cache, from, target, self.data.vertices_mut())?;
            self.frame = CoordinateFrame::Crs(target);
        }
        Ok(())
    }
}

use crate::ops::{plan_frame_step, translate_2d, translate_3d, ConvertFrame, FrameStep, Translate};

impl Translate for TriangularMesh2D {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        translate_2d(&mut self.vertices, self.z.as_deref_mut(), delta);
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
    ) -> crate::error::Result<()> {
        match plan_frame_step(&self.frame, target, base_point)? {
            FrameStep::Noop => Ok(()),
            FrameStep::Reproject(to) => self.reproject(to, cache),
            FrameStep::Translate(offset, frame) => {
                self.translate(offset)?;
                self.frame = frame;
                Ok(())
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
    ) -> crate::error::Result<()> {
        match plan_frame_step(&self.frame, target, base_point)? {
            FrameStep::Noop => Ok(()),
            FrameStep::Reproject(to) => self.reproject(to, cache),
            FrameStep::Translate(offset, frame) => {
                self.translate(offset)?;
                self.frame = frame;
                Ok(())
            }
        }
    }
}

impl Split for TriangularMesh2D {
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
            let polygon = Polygon2D::from_rings(frame.clone(), ring, Vec::<Vec<[f64; 2]>>::new());
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
}
