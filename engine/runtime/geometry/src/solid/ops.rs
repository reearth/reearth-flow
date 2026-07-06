use super::{Shell, Solid};
use crate::coordinate::{CoordinateFrame, EpsgCode};
use crate::ops::reproject::transform_coords_3d;
use crate::ops::triangulation::Cache;
use crate::ops::{
    Aabb, BoundingBox, Reproject, ReprojectionCache, Triangulate, UnsupportedOperation,
};
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
            Shell::PolygonMesh(d) => Shell::TriangularMesh(d.triangulate(cache)),
            Shell::TriangularMesh(d) => Shell::TriangularMesh(d.clone()),
        }
    }
}

impl Reproject for Solid {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        let from = self.frame.require_crs()?;
        if from != target {
            reproject_shell(&mut self.exterior, from, target, cache)?;
            for shell in &mut self.interiors {
                reproject_shell(shell, from, target, cache)?;
            }
            self.frame = CoordinateFrame::Crs(target);
        }
        Ok(())
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
