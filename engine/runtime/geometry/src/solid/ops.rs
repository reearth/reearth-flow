use super::{Shell, Solid};
use crate::coordinate::{CoordinateFrame, EpsgCode};
use crate::ops::reproject::transform_coords_3d;
use crate::ops::triangulation::Cache;
use crate::ops::{
    Aabb, BoundingBox, Reproject, ReprojectionCache, Triangulate, UnsupportedOperation,
};
use crate::triangular_mesh::TriangularMesh3DData;
use crate::{Euclidean3DGeometry, Geometry};

impl BoundingBox for Solid {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        let verts = std::iter::once(&self.exterior)
            .chain(self.interiors.iter())
            .flat_map(|s| s.vertices().iter().copied());
        Aabb::from_points_3d(verts).ok_or(UnsupportedOperation {
            geometry: "Solid",
            operation: "bounding_box",
        })
    }
}

impl Triangulate for Solid {
    /// Triangulate the solid's boundary in place: each `PolygonMesh` shell is
    /// tessellated into a `TriangularMesh` shell; `TriangularMesh` shells pass
    /// through unchanged. The result is a `Solid` with the same frame and an
    /// all-triangle boundary.
    fn triangulate(&mut self, cache: &mut Cache) -> Result<Geometry, UnsupportedOperation> {
        let exterior = self.exterior.triangulated(cache);
        let interiors = self
            .interiors
            .iter_mut()
            .map(|shell| shell.triangulated(cache))
            .collect();
        let solid = Solid::new(self.frame.clone(), exterior, interiors);
        Ok(Geometry::Euclidean3D(Euclidean3DGeometry::Solid(Box::new(
            solid,
        ))))
    }
}

impl Shell {
    /// This shell with its surface triangulated: a `PolygonMesh` shell becomes a
    /// `TriangularMesh` shell (stealing its buffers); an already-`TriangularMesh`
    /// shell is cloned through unchanged.
    fn triangulated(&mut self, cache: &mut Cache) -> Shell {
        match self {
            Shell::PolygonMesh(d) => Shell::TriangularMesh(d.triangulate(cache).mesh),
            Shell::TriangularMesh(d) => Shell::TriangularMesh(d.clone()),
        }
    }
}

impl Solid {
    /// Move the solid out, leaving an empty husk.
    fn take(&mut self) -> Self {
        Self {
            frame: std::mem::take(&mut self.frame),
            exterior: std::mem::replace(
                &mut self.exterior,
                Shell::TriangularMesh(TriangularMesh3DData::empty()),
            ),
            interiors: std::mem::take(&mut self.interiors),
        }
    }

    /// The leaf moved out and wrapped as a [`Geometry`].
    fn take_geometry(&mut self) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Solid(Box::new(self.take())))
    }
}

impl Reproject for Solid {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        let from = self.frame.require_crs()?;
        let mut solid = self.take();
        if from != target {
            reproject_shell(&mut solid.exterior, from, target, cache)?;
            for shell in &mut solid.interiors {
                reproject_shell(shell, from, target, cache)?;
            }
            solid.frame = CoordinateFrame::Crs(target);
        }
        Ok(Geometry::Euclidean3D(Euclidean3DGeometry::Solid(Box::new(
            solid,
        ))))
    }
}

/// Reproject one shell's vertices from `from` to `target` (EPSG).
fn reproject_shell(
    shell: &mut Shell,
    from: EpsgCode,
    target: EpsgCode,
    cache: &mut ReprojectionCache,
) -> crate::error::Result<()> {
    let vertices = match shell {
        Shell::PolygonMesh(data) => data.vertices_mut(),
        Shell::TriangularMesh(data) => data.vertices_mut(),
    };
    transform_coords_3d(cache, from, target, vertices)
}

use crate::ops::{plan_frame_step, translate_3d, ConvertFrame, FrameStep, Translate};

impl Translate for Solid {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        for shell in std::iter::once(&mut self.exterior).chain(self.interiors.iter_mut()) {
            let vertices = match shell {
                Shell::PolygonMesh(data) => data.vertices_mut(),
                Shell::TriangularMesh(data) => data.vertices_mut(),
            };
            translate_3d(vertices, delta);
        }
        Ok(())
    }
}

impl ConvertFrame for Solid {
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

// A solid is a volume; flattening its boundary to 2D has no single well-defined
// result, so it has no 2D counterpart.
crate::unsupported!(Solid: ForceTwoDimension);

use crate::ops::{
    emit_face_3d, emit_triangles_3d, CountHoles, ExtractHoles, ExtractedPart, RemoveAppearance,
};

impl CountHoles for Solid {
    /// The holes in the boundary faces of every shell. The void shells
    /// themselves are hollow volumes rather than face boundaries, so they are
    /// not counted; only the rings inside their faces are. A triangle-mesh shell
    /// carries no rings and contributes nothing.
    fn count_holes(&self) -> usize {
        std::iter::once(&self.exterior)
            .chain(self.interiors.iter())
            .map(|shell| match shell {
                Shell::PolygonMesh(data) => data.num_holes(),
                Shell::TriangularMesh(_) => 0,
            })
            .sum()
    }
}

impl ExtractHoles for Solid {
    /// Take apart the boundary faces of every shell. Matching [`CountHoles`], a
    /// void shell is not itself a hole — it is a hollow volume — so it is not
    /// emitted as one; its faces are taken apart like the exterior's.
    fn extract_holes(
        &self,
        emit: &mut dyn FnMut(Geometry, ExtractedPart),
    ) -> Result<(), UnsupportedOperation> {
        let frame = self.frame();
        for shell in std::iter::once(&self.exterior).chain(self.interiors.iter()) {
            match shell {
                Shell::PolygonMesh(data) => data.for_each_face_polygon(frame, |face| {
                    emit_face_3d(&face, emit);
                }),
                Shell::TriangularMesh(data) => {
                    emit_triangles_3d(frame, data.vertices(), data.triangles(), emit)
                }
            }
        }
        Ok(())
    }
}

impl RemoveAppearance for Solid {
    fn remove_appearance(&mut self) {
        for shell in std::iter::once(&mut self.exterior).chain(self.interiors.iter_mut()) {
            match shell {
                Shell::PolygonMesh(data) => data.remove_appearance(),
                Shell::TriangularMesh(data) => data.remove_appearance(),
            }
        }
    }
}

use crate::ops::coerce::{push_face_lines_3d, triangle_ring, unchanged, wrap_3d};
use crate::ops::{Coerce, CoercionTarget};
use crate::polygon::Polygon3D;

impl Solid {
    /// Invoke `f` once per boundary face of the solid, the exterior shell's
    /// faces first, then each void shell's. A triangle-mesh shell contributes
    /// one closed triangle per face.
    fn for_each_boundary_face(&self, mut f: impl FnMut(Polygon3D)) {
        let frame = self.frame();
        for shell in std::iter::once(&self.exterior).chain(self.interiors.iter()) {
            match shell {
                Shell::PolygonMesh(data) => data.for_each_face_polygon(frame, &mut f),
                Shell::TriangularMesh(data) => {
                    let vertices = data.vertices();
                    for triangle in data.triangles() {
                        f(Polygon3D::from_rings(
                            frame.clone(),
                            triangle_ring(vertices, triangle),
                            Vec::<Vec<[f64; 3]>>::new(),
                        ));
                    }
                }
            }
        }
    }
}

impl Coerce for Solid {
    fn coerce(
        &mut self,
        target: CoercionTarget,
        cache: &mut Cache,
    ) -> Result<Geometry, UnsupportedOperation> {
        match target {
            // `Triangulate` on a volume yields a volume with a triangulated
            // boundary, not a bare mesh — the result stays a `Solid`.
            CoercionTarget::TriangularMesh => self.triangulate(cache),
            CoercionTarget::Polygon => {
                let mut faces = Vec::new();
                self.for_each_boundary_face(|face| {
                    faces.push(Euclidean3DGeometry::Polygon(Box::new(face)))
                });
                wrap_3d(faces).ok_or_else(unchanged::<Self>)
            }
            CoercionTarget::LineString => {
                let mut lines = Vec::new();
                self.for_each_boundary_face(|face| push_face_lines_3d(&face, &mut lines));
                wrap_3d(lines).ok_or_else(unchanged::<Self>)
            }
        }
    }
}

#[cfg(feature = "new-geometry")]
use crate::ops::{Footprint, FootprintError, FootprintSink};

#[cfg(feature = "new-geometry")]
impl Footprint for Solid {
    /// Push the faces of every shell, voids included.
    fn footprint(&self, sink: &mut FootprintSink<'_>) -> Result<(), FootprintError> {
        sink.enter(self.frame())?;
        for shell in std::iter::once(&self.exterior).chain(self.interiors.iter()) {
            match shell {
                Shell::PolygonMesh(data) => data.footprint_faces(sink),
                Shell::TriangularMesh(data) => data.footprint_faces(sink),
            }
        }
        Ok(())
    }
}

use crate::ops::boundary::ExtractBoundary;
use crate::polygon_mesh::PolygonMesh3D;
use crate::triangular_mesh::TriangularMesh3D;

// The shells the volume already carries, paired with the frame they are
// expressed in. Nothing is re-triangulated and appearance stays on them.
//
// Whether the shells close is not asserted here: taking the boundary of the
// result answers that, and is empty exactly when they do.
impl ExtractBoundary for Solid {
    fn extract_boundary(&self) -> Result<Geometry, UnsupportedOperation> {
        let frame = self.frame();
        let shells = std::iter::once(&self.exterior)
            .chain(self.interiors.iter())
            .map(|shell| match shell {
                Shell::PolygonMesh(data) => Euclidean3DGeometry::PolygonMesh(Box::new(
                    PolygonMesh3D::new(frame.clone(), data.clone()),
                )),
                Shell::TriangularMesh(data) => Euclidean3DGeometry::TriangularMesh(Box::new(
                    TriangularMesh3D::new(frame.clone(), data.clone()),
                )),
            })
            .collect();
        Ok(wrap_3d(shells).unwrap_or(Geometry::None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::solid::Shell;
    use crate::triangular_mesh::TriangularMesh3DData;

    fn shell(verts: Vec<[f64; 3]>) -> TriangularMesh3DData {
        TriangularMesh3DData::from_parts(verts, [0u32, 1, 2]).unwrap()
    }

    #[test]
    fn solid_box_spans_exterior_shell() {
        let s = Solid::from_exterior(
            CoordinateFrame::Euclidean,
            shell(vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 3.0]]),
        );
        assert_eq!(
            s.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, 0.0, 0.0],
                max: [2.0, 2.0, 3.0]
            }
        );
    }

    #[test]
    fn solid_box_includes_interior_shells() {
        let s = Solid::new(
            CoordinateFrame::Euclidean,
            shell(vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]]),
            vec![Shell::from(shell(vec![
                [5.0, 5.0, 5.0],
                [6.0, 5.0, 5.0],
                [5.0, 6.0, 5.0],
            ]))],
        );
        assert_eq!(
            s.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, 0.0, 0.0],
                max: [6.0, 6.0, 5.0]
            }
        );
    }

    #[test]
    fn solid_triangulation_yields_a_solid_with_triangulated_shells() {
        use crate::polygon_mesh::PolygonMesh3DData;
        use crate::triangular_mesh::TriangularMesh3D;

        // Exterior: a quad polygon-mesh shell -> becomes a 2-triangle mesh shell.
        let quad = PolygonMesh3DData::from_parts(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
            ],
            vec![vec![0u32, 1, 2, 3]],
        )
        .unwrap();
        // Interior void: already a triangle-mesh shell -> passes through unchanged.
        let void = shell(vec![[5.0, 5.0, 5.0], [6.0, 5.0, 5.0], [5.0, 6.0, 5.0]]);
        let mut solid = Solid::new(CoordinateFrame::Euclidean, quad, vec![Shell::from(void)]);

        let out = match solid.triangulate(&mut Cache::new()).unwrap() {
            // The output is a Solid, not a bare mesh.
            Geometry::Euclidean3D(Euclidean3DGeometry::Solid(s)) => s,
            other => panic!("expected a solid, got {other:?}"),
        };
        // The polygon-mesh exterior is now a 2-triangle triangular-mesh shell.
        match &out.exterior {
            Shell::TriangularMesh(d) => {
                let tris = TriangularMesh3D::new(CoordinateFrame::Euclidean, d.clone());
                assert_eq!(tris.num_triangles(), 2);
            }
            Shell::PolygonMesh(_) => panic!("exterior polygon-mesh shell should be triangulated"),
        }
        // The already-triangular interior shell stays a triangular mesh.
        assert_eq!(out.interiors.len(), 1);
        assert!(matches!(out.interiors[0], Shell::TriangularMesh(_)));
    }
}
