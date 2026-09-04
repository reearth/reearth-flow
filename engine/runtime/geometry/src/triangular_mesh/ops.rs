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

#[cfg(feature = "new-geometry")]
impl crate::predicates::Equal for TriangularMesh2D {
    fn equal(
        &self,
        rhs: &Self,
        tolerance: crate::predicates::Tolerance,
    ) -> crate::predicates::Result<bool> {
        // 2D is the one case where every shared edge may be cancelled: the mesh
        // lies in a single plane, so its region is recoverable from its outline
        // and no crease can hide inside it.
        use crate::predicates::equal::surface_curves_2d;

        crate::predicates::require_same_frame(self.frame(), rhs.frame())?;
        Ok(surface_curves_2d(self)?.within(&surface_curves_2d(rhs)?, tolerance.distance))
    }
}

#[cfg(feature = "new-geometry")]
impl crate::predicates::Equal for TriangularMesh3D {
    fn equal(
        &self,
        rhs: &Self,
        tolerance: crate::predicates::Tolerance,
    ) -> crate::predicates::Result<bool> {
        use crate::predicates::equal::facet_curves;
        use crate::predicates::view3d::TriangleSet;

        crate::predicates::require_same_frame(self.frame(), rhs.frame())?;
        let ours = facet_curves(
            &TriangleSet::from_triangular_data(self.data()),
            tolerance.coplanarity,
        );
        let theirs = facet_curves(
            &TriangleSet::from_triangular_data(rhs.data()),
            tolerance.coplanarity,
        );
        Ok(ours.within(&theirs, tolerance.distance))
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod equal_tests {
    use super::*;
    use crate::predicates::{Equal, Tolerance};

    fn tolerance() -> Tolerance {
        Tolerance {
            distance: 1e-9,
            coplanarity: 1e-6,
        }
    }

    fn mesh(vertices: Vec<[f64; 3]>, indices: Vec<u32>) -> TriangularMesh3D {
        TriangularMesh3D::from_parts(CoordinateFrame::Euclidean, vertices, indices).unwrap()
    }

    /// The unit square in `z = 0`, corners counter-clockwise from the origin.
    fn square() -> Vec<[f64; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    /// The eight corners of the unit cube.
    fn cube_corners() -> Vec<[f64; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ]
    }

    const CUBE_FACES: [[u32; 4]; 6] = [
        [0, 1, 2, 3],
        [4, 5, 6, 7],
        [0, 1, 5, 4],
        [1, 2, 6, 5],
        [2, 3, 7, 6],
        [3, 0, 4, 7],
    ];

    /// Cut each quad along one diagonal (`flip == false`) or the other.
    fn cube(flip: bool) -> TriangularMesh3D {
        let mut indices = Vec::new();
        for [a, b, c, d] in CUBE_FACES {
            if flip {
                indices.extend([a, b, d, b, c, d]);
            } else {
                indices.extend([a, b, c, a, c, d]);
            }
        }
        mesh(cube_corners(), indices)
    }

    #[test]
    fn two_triangulations_of_one_square_are_equal() {
        let across = mesh(square(), vec![0, 1, 2, 0, 2, 3]);
        let the_other_way = mesh(square(), vec![0, 1, 3, 1, 2, 3]);

        assert!(across.equal(&the_other_way, tolerance()).unwrap());
        assert!(the_other_way.equal(&across, tolerance()).unwrap());
    }

    #[test]
    fn an_interior_vertex_added_to_a_flat_region_changes_nothing() {
        let across = mesh(square(), vec![0, 1, 2, 0, 2, 3]);
        let mut fanned = square();
        fanned.push([0.5, 0.5, 0.0]);
        let fan = mesh(fanned, vec![0, 1, 4, 1, 2, 4, 2, 3, 4, 3, 0, 4]);

        assert!(across.equal(&fan, tolerance()).unwrap());
    }

    #[test]
    fn two_triangulations_of_one_closed_cube_are_equal() {
        // The case that rules out cancelling every shared edge: a closed shell
        // has no edge with only one triangle on it, so that rule would reduce
        // both of these to nothing and call every closed body equal.
        assert!(cube(false).equal(&cube(true), tolerance()).unwrap());
    }

    #[test]
    fn a_closed_shell_does_not_reduce_to_nothing() {
        // A cube is not the same shape as a flat square, and would be if both
        // came out empty.
        let flat = mesh(square(), vec![0, 1, 2, 0, 2, 3]);
        assert!(!cube(false).equal(&flat, tolerance()).unwrap());
    }

    #[test]
    fn a_tent_is_not_equal_to_the_square_it_stands_on() {
        // Same boundary loop, different surface: the creases are what tell them
        // apart, so they have to survive the merging.
        let flat = mesh(square(), vec![0, 1, 2, 0, 2, 3]);
        let mut pitched = square();
        pitched.push([0.5, 0.5, 0.5]);
        let tent = mesh(pitched, vec![0, 1, 4, 1, 2, 4, 2, 3, 4, 3, 0, 4]);

        assert!(!flat.equal(&tent, tolerance()).unwrap());
    }

    #[test]
    fn a_cube_is_not_equal_to_one_of_another_size() {
        let bigger = cube_corners()
            .into_iter()
            .map(|[x, y, z]| [x * 2.0, y, z])
            .collect::<Vec<_>>();
        let mut indices = Vec::new();
        for [a, b, c, d] in CUBE_FACES {
            indices.extend([a, b, c, a, c, d]);
        }
        assert!(!cube(false)
            .equal(&mesh(bigger, indices), tolerance())
            .unwrap());
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
